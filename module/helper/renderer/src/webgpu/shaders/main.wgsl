// Canonical opaque PBR shader (WGSL).
//
// Port of the direct-lighting slice of `../../webgl/shaders/main.vert` +
// `main.frag`: Burley diffuse + GGX specular with Schlick Fresnel, point /
// direct / spot lights, metallic-roughness material with optional base-color
// and metallic-roughness textures. Slice scope — not yet ported: IBL,
// skinning, morph targets, tangent-space normal mapping, emission target,
// WBOIT targets. This file is the canonical source per ADR-001 §5; the
// WebGL2 override is generated from it at build time by gpu_hal's
// `webgl_build` kit — see `build.rs`.
//
// Feature toggles are runtime flags (`MaterialUniform.flags`) instead of the
// GLSL `#define` scheme: WGSL has no preprocessor, and unused texture slots
// are bound to 1x1 dummies so one pipeline serves every material.

const PI : f32 = 3.141592653589793;
const RECIPROCAL_PI : f32 = 0.3183098861837907;

const MAX_POINT_LIGHTS : u32 = 8u;
const MAX_DIRECT_LIGHTS : u32 = 8u;
const MAX_SPOT_LIGHTS : u32 = 8u;

const FLAG_USE_BASE_COLOR_TEXTURE : u32 = 1u;
const FLAG_USE_MR_TEXTURE : u32 = 2u;
const FLAG_USE_ALPHA_CUTOFF : u32 = 4u;

struct CameraUniform
{
  view_matrix : mat4x4f,
  projection_matrix : mat4x4f,
  // xyz — world-space camera position; w — exposure (applied as exp2).
  position_exposure : vec4f,
}

struct ModelUniform
{
  world_matrix : mat4x4f,
  // mat3x3f in uniform space: three vec4-aligned columns.
  normal_matrix : mat3x3f,
}

struct MaterialUniform
{
  base_color_factor : vec4f,
  metallic_factor : f32,
  roughness_factor : f32,
  alpha_cutoff : f32,
  flags : u32,
}

struct PointLight
{
  // xyz — position; w — range.
  position_range : vec4f,
  // xyz — color; w — strength.
  color_strength : vec4f,
}

struct DirectLight
{
  // xyz — direction; w — strength.
  direction_strength : vec4f,
  // xyz — color; w — unused.
  color : vec4f,
}

struct SpotLight
{
  // xyz — position; w — range.
  position_range : vec4f,
  // xyz — direction (unit, cpu-normalized); w — strength.
  direction_strength : vec4f,
  // xyz — color; w — inner cone angle (radians).
  color_inner : vec4f,
  // x — outer cone angle (radians); yzw — unused.
  outer : vec4f,
}

struct LightsUniform
{
  // x — point count; y — direct count; z — spot count; w — unused.
  counts : vec4u,
  point : array< PointLight, MAX_POINT_LIGHTS >,
  direct : array< DirectLight, MAX_DIRECT_LIGHTS >,
  spot : array< SpotLight, MAX_SPOT_LIGHTS >,
}

@group( 0 ) @binding( 0 ) var< uniform > camera : CameraUniform;
@group( 0 ) @binding( 1 ) var< uniform > lights : LightsUniform;

@group( 1 ) @binding( 0 ) var< uniform > material : MaterialUniform;
@group( 1 ) @binding( 1 ) var base_color_texture : texture_2d< f32 >;
@group( 1 ) @binding( 2 ) var base_color_sampler : sampler;
@group( 1 ) @binding( 3 ) var metallic_roughness_texture : texture_2d< f32 >;
@group( 1 ) @binding( 4 ) var metallic_roughness_sampler : sampler;

@group( 2 ) @binding( 0 ) var< uniform > model : ModelUniform;

struct VertexInput
{
  @location( 0 ) position : vec3f,
  @location( 1 ) normal : vec3f,
  @location( 2 ) uv_0 : vec2f,
  @location( 3 ) color_0 : vec4f,
}

struct VertexOutput
{
  @builtin( position ) clip_position : vec4f,
  @location( 0 ) world_position : vec3f,
  @location( 1 ) normal : vec3f,
  @location( 2 ) uv_0 : vec2f,
  @location( 3 ) color_0 : vec4f,
}

@vertex
fn vs_main( in : VertexInput ) -> VertexOutput
{
  var out : VertexOutput;

  let world_position = model.world_matrix * vec4f( in.position, 1.0 );
  out.world_position = world_position.xyz;
  out.normal = normalize( model.normal_matrix * in.normal );
  out.uv_0 = in.uv_0;
  out.color_0 = in.color_0;
  out.clip_position = camera.projection_matrix * camera.view_matrix * world_position;

  return out;
}

struct PhysicalMaterial
{
  diffuse_color : vec3f,
  metallness : f32,
  roughness : f32,
  f0 : vec3f,
  f90 : vec3f,
}

struct ReflectedLight
{
  indirect_diffuse : vec3f,
  indirect_specular : vec3f,
  direct_diffuse : vec3f,
  direct_specular : vec3f,
}

fn pow2( x : f32 ) -> f32
{
  return x * x;
}

fn srgb_to_linear( color : vec3f ) -> vec3f
{
  let more = pow( color * 0.9478672986 + vec3f( 0.0521327014 ), vec3f( 2.4 ) );
  let less = color * 0.0773993808;

  return select( more, less, color <= vec3f( 0.04045 ) );
}

// Schlick's Fresnel with the Spherical Gaussian power approximation
// (Karis, s2013 course notes) — matches the GLSL original bit for bit.
fn f_schlick( f0 : vec3f, f90 : vec3f, dot_v_h : f32 ) -> vec3f
{
  let fresnel = exp2( ( -5.55473 * dot_v_h - 6.98316 ) * dot_v_h );
  return f0 + ( f90 - f0 ) * fresnel;
}

fn fd_burley( alpha : f32, dot_n_v : f32, dot_n_l : f32, dot_l_h : f32 ) -> vec3f
{
  let f90 = vec3f( 0.5 + 2.0 * alpha * pow2( dot_l_h ) );
  let light_scatter = f_schlick( vec3f( 1.0 ), f90, dot_n_l );
  let view_scatter = f_schlick( vec3f( 1.0 ), f90, dot_n_v );
  return view_scatter * light_scatter * RECIPROCAL_PI;
}

fn v_ggx_smith_correlated( alpha : f32, dot_n_l : f32, dot_n_v : f32 ) -> f32
{
  let a2 = pow2( alpha );
  let gv = dot_n_l * sqrt( a2 + ( 1.0 - a2 ) * pow2( dot_n_v ) );
  let gl = dot_n_v * sqrt( a2 + ( 1.0 - a2 ) * pow2( dot_n_l ) );
  return 0.5 / max( gv + gl, 1e-6 );
}

fn d_ggx( alpha : f32, dot_n_h : f32 ) -> f32
{
  let a2 = pow2( alpha );
  let denom = pow2( dot_n_h ) * ( a2 - 1.0 ) + 1.0;
  return RECIPROCAL_PI * a2 / pow2( denom );
}

fn apply_light_contribution
(
  light_dir : vec3f,
  view_dir : vec3f,
  normal : vec3f,
  mat : PhysicalMaterial,
  light_color : vec3f,
  light_intensity : f32,
  reflected : ptr< function, ReflectedLight >
)
{
  let alpha = pow2( mat.roughness );
  let half_dir = normalize( light_dir + view_dir );

  let dot_n_l = clamp( dot( normal, light_dir ), 0.0, 1.0 );
  let dot_n_v = clamp( dot( normal, view_dir ), 0.0, 1.0 );
  let dot_n_h = clamp( dot( normal, half_dir ), 0.0, 1.0 );
  let dot_v_h = clamp( dot( view_dir, half_dir ), 0.0, 1.0 );
  let dot_l_h = clamp( dot( light_dir, half_dir ), 0.0, 1.0 );

  let fs = f_schlick( mat.f0, mat.f90, dot_v_h );
  let fd = fd_burley( alpha, dot_n_v, dot_n_l, dot_l_h );
  let v = v_ggx_smith_correlated( alpha, dot_n_l, dot_n_v );
  let d = d_ggx( alpha, dot_n_h );

  let irradiance = light_color * light_intensity * dot_n_l;
  let diffuse_color = mat.diffuse_color * irradiance;
  let specular_color = d * v * irradiance;

  ( *reflected ).direct_diffuse += ( 1.0 - fs ) * fd * diffuse_color;
  ( *reflected ).direct_specular += fs * specular_color;
}

fn compute_lights
(
  world_position : vec3f,
  view_dir : vec3f,
  normal : vec3f,
  mat : PhysicalMaterial,
  reflected : ptr< function, ReflectedLight >
)
{
  let point_count = min( lights.counts.x, MAX_POINT_LIGHTS );
  for( var i : u32 = 0u; i < point_count; i++ )
  {
    let light = lights.point[ i ];
    var light_dir = light.position_range.xyz - world_position;
    let distance = length( light_dir );
    light_dir = normalize( light_dir );

    if( dot( normal, light_dir ) <= 0.0 ) { continue; }

    let range = light.position_range.w;
    var attenuation = pow2( clamp( 1.0 - distance / range, 0.0, 1.0 ) ) / ( distance * distance + 1.0 );
    attenuation *= light.color_strength.w;

    apply_light_contribution( light_dir, view_dir, normal, mat, light.color_strength.xyz, attenuation, reflected );
  }

  let direct_count = min( lights.counts.y, MAX_DIRECT_LIGHTS );
  for( var i : u32 = 0u; i < direct_count; i++ )
  {
    let light = lights.direct[ i ];
    let light_dir = light.direction_strength.xyz;

    if( dot( normal, light_dir ) <= 0.0 ) { continue; }

    apply_light_contribution( light_dir, view_dir, normal, mat, light.color.xyz, light.direction_strength.w, reflected );
  }

  let spot_count = min( lights.counts.z, MAX_SPOT_LIGHTS );
  for( var i : u32 = 0u; i < spot_count; i++ )
  {
    let light = lights.spot[ i ];
    var light_dir = light.position_range.xyz - world_position;
    let distance = length( light_dir );
    light_dir = normalize( light_dir );

    if( dot( normal, light_dir ) <= 0.0 ) { continue; }

    let range = light.position_range.w;
    var attenuation = pow2( clamp( 1.0 - distance / range, 0.0, 1.0 ) ) / ( distance * distance + 1.0 );

    let angle = acos( dot( -light_dir, light.direction_strength.xyz ) );
    let angular_attenuation = smoothstep( light.outer.x, light.color_inner.w, angle );
    attenuation *= angular_attenuation * light.direction_strength.w;

    apply_light_contribution( light_dir, view_dir, normal, mat, light.color_inner.xyz, attenuation, reflected );
  }
}

@fragment
fn fs_main( in : VertexOutput, @builtin( front_facing ) front_facing : bool ) -> @location( 0 ) vec4f
{
  var mat : PhysicalMaterial;
  var reflected = ReflectedLight( vec3f( 0.0 ), vec3f( 0.0 ), vec3f( 0.0 ), vec3f( 0.0 ) );

  var alpha = material.base_color_factor.a;

  mat.metallness = material.metallic_factor;
  mat.roughness = material.roughness_factor;
  mat.diffuse_color = material.base_color_factor.rgb;

  if( ( material.flags & FLAG_USE_BASE_COLOR_TEXTURE ) != 0u )
  {
    let base_color = textureSample( base_color_texture, base_color_sampler, in.uv_0 );
    mat.diffuse_color *= srgb_to_linear( base_color.rgb );
    alpha *= base_color.a;
  }

  if( ( material.flags & FLAG_USE_MR_TEXTURE ) != 0u )
  {
    // Roughness — G channel; metalness — B channel (glTF convention).
    let mr_sample = textureSample( metallic_roughness_texture, metallic_roughness_sampler, in.uv_0 );
    mat.metallness *= mr_sample.b;
    mat.roughness *= mr_sample.g;
  }

  if( ( material.flags & FLAG_USE_ALPHA_CUTOFF ) != 0u )
  {
    if( alpha < material.alpha_cutoff )
    {
      discard;
    }
  }

  // 0.04 — reflectance of glass, the glTF dielectric baseline.
  mat.f0 = mix( vec3f( 0.04 ), mat.diffuse_color, mat.metallness );
  mat.f90 = vec3f( 1.0 );
  mat.diffuse_color *= 1.0 - mat.metallness;

  var normal = normalize( in.normal );
  if( !front_facing )
  {
    normal = -normal;
  }

  // Geometric specular anti-aliasing (Tokuyoshi & Kaplanyan 2019), as in the
  // GLSL original: widen roughness where screen-space normal variance is high.
  let d_n_dx = dpdx( normal );
  let d_n_dy = dpdy( normal );
  let geometric_variance = dot( d_n_dx, d_n_dx ) + dot( d_n_dy, d_n_dy );
  mat.roughness = sqrt( clamp( pow2( mat.roughness ) + 0.5 * geometric_variance, 0.0, 1.0 ) );
  mat.roughness = max( mat.roughness, 0.0525 );

  let view_dir = normalize( camera.position_exposure.xyz - in.world_position );

  compute_lights( in.world_position, view_dir, normal, mat, &reflected );

  // Non-IBL ambient fallback, matching the GLSL `#else` branch.
  reflected.indirect_diffuse += 0.1 * mat.diffuse_color;

  var color = reflected.indirect_diffuse
    + reflected.indirect_specular
    + reflected.direct_diffuse
    + reflected.direct_specular;

  color *= exp2( camera.position_exposure.w );

  // Opaque pass marks covered pixels with alpha = 1; the cleared background
  // stays at alpha = 0 so the tone mapping pass leaves it untouched.
  return vec4f( color, 1.0 );
}

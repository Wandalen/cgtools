#version 300 es

// GLSL 300 es twin of `main.wgsl` `fs_main`, consumed by the gpu_hal WebGL2
// backend. `main.wgsl` is the canonical source ( ADR-001 §5 ) — edit it first
// and mirror changes here. Uniform block names follow the HAL binding-name
// convention `ub_{group}_{binding}`; sampler uniforms are named after their
// texture entries `tex_{group}_{binding}` ( the paired HAL sampler entries
// bind sampler objects to the same texture units and have no GLSL identifier ).

precision highp float;
precision highp int;

const float PI = 3.141592653589793;
const float RECIPROCAL_PI = 0.3183098861837907;

const uint MAX_POINT_LIGHTS = 8u;
const uint MAX_DIRECT_LIGHTS = 8u;
const uint MAX_SPOT_LIGHTS = 8u;

const uint FLAG_USE_BASE_COLOR_TEXTURE = 1u;
const uint FLAG_USE_MR_TEXTURE = 2u;
const uint FLAG_USE_ALPHA_CUTOFF = 4u;

struct PointLight
{
  // xyz — position; w — range.
  vec4 position_range;
  // xyz — color; w — strength.
  vec4 color_strength;
};

struct DirectLight
{
  // xyz — direction; w — strength.
  vec4 direction_strength;
  // xyz — color; w — unused.
  vec4 color;
};

struct SpotLight
{
  // xyz — position; w — range.
  vec4 position_range;
  // xyz — direction ( unit, cpu-normalized ); w — strength.
  vec4 direction_strength;
  // xyz — color; w — inner cone angle ( radians ).
  vec4 color_inner;
  // x — outer cone angle ( radians ); yzw — unused.
  vec4 outer;
};

layout( std140 ) uniform ub_0_0
{
  mat4 view_matrix;
  mat4 projection_matrix;
  // xyz — world-space camera position; w — exposure ( applied as exp2 ).
  vec4 position_exposure;
} camera;

layout( std140 ) uniform ub_0_1
{
  // x — point count; y — direct count; z — spot count; w — unused.
  uvec4 counts;
  // Array sizes mirror the MAX_*_LIGHTS constants.
  PointLight point[ 8 ];
  DirectLight direct[ 8 ];
  SpotLight spot[ 8 ];
} lights;

layout( std140 ) uniform ub_1_0
{
  vec4 base_color_factor;
  float metallic_factor;
  float roughness_factor;
  float alpha_cutoff;
  uint flags;
} material;

// Base color ( sRGB-encoded content ).
uniform highp sampler2D tex_1_1;
// Metallic-roughness ( G — roughness, B — metalness ).
uniform highp sampler2D tex_1_3;

in vec3 v_world_position;
in vec3 v_normal;
in vec2 v_uv_0;
in vec4 v_color_0;

layout( location = 0 ) out vec4 frag_color;

struct PhysicalMaterial
{
  vec3 diffuse_color;
  float metallness;
  float roughness;
  vec3 f0;
  vec3 f90;
};

struct ReflectedLight
{
  vec3 indirect_diffuse;
  vec3 indirect_specular;
  vec3 direct_diffuse;
  vec3 direct_specular;
};

float pow2( float x )
{
  return x * x;
}

vec3 srgb_to_linear( vec3 color )
{
  vec3 more = pow( color * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) );
  vec3 less = color * 0.0773993808;

  return mix( more, less, lessThanEqual( color, vec3( 0.04045 ) ) );
}

// Schlick's Fresnel with the Spherical Gaussian power approximation
// (Karis, s2013 course notes) — matches the WGSL original bit for bit.
vec3 f_schlick( vec3 f0, vec3 f90, float dot_v_h )
{
  float fresnel = exp2( ( -5.55473 * dot_v_h - 6.98316 ) * dot_v_h );
  return f0 + ( f90 - f0 ) * fresnel;
}

vec3 fd_burley( float alpha, float dot_n_v, float dot_n_l, float dot_l_h )
{
  vec3 f90 = vec3( 0.5 + 2.0 * alpha * pow2( dot_l_h ) );
  vec3 light_scatter = f_schlick( vec3( 1.0 ), f90, dot_n_l );
  vec3 view_scatter = f_schlick( vec3( 1.0 ), f90, dot_n_v );
  return view_scatter * light_scatter * RECIPROCAL_PI;
}

float v_ggx_smith_correlated( float alpha, float dot_n_l, float dot_n_v )
{
  float a2 = pow2( alpha );
  float gv = dot_n_l * sqrt( a2 + ( 1.0 - a2 ) * pow2( dot_n_v ) );
  float gl = dot_n_v * sqrt( a2 + ( 1.0 - a2 ) * pow2( dot_n_l ) );
  return 0.5 / max( gv + gl, 1e-6 );
}

float d_ggx( float alpha, float dot_n_h )
{
  float a2 = pow2( alpha );
  float denom = pow2( dot_n_h ) * ( a2 - 1.0 ) + 1.0;
  return RECIPROCAL_PI * a2 / pow2( denom );
}

void apply_light_contribution
(
  vec3 light_dir,
  vec3 view_dir,
  vec3 normal,
  PhysicalMaterial mat,
  vec3 light_color,
  float light_intensity,
  inout ReflectedLight reflected
)
{
  float alpha = pow2( mat.roughness );
  vec3 half_dir = normalize( light_dir + view_dir );

  float dot_n_l = clamp( dot( normal, light_dir ), 0.0, 1.0 );
  float dot_n_v = clamp( dot( normal, view_dir ), 0.0, 1.0 );
  float dot_n_h = clamp( dot( normal, half_dir ), 0.0, 1.0 );
  float dot_v_h = clamp( dot( view_dir, half_dir ), 0.0, 1.0 );
  float dot_l_h = clamp( dot( light_dir, half_dir ), 0.0, 1.0 );

  vec3 fs = f_schlick( mat.f0, mat.f90, dot_v_h );
  vec3 fd = fd_burley( alpha, dot_n_v, dot_n_l, dot_l_h );
  float v = v_ggx_smith_correlated( alpha, dot_n_l, dot_n_v );
  float d = d_ggx( alpha, dot_n_h );

  vec3 irradiance = light_color * light_intensity * dot_n_l;
  vec3 diffuse_color = mat.diffuse_color * irradiance;
  vec3 specular_color = d * v * irradiance;

  reflected.direct_diffuse += ( 1.0 - fs ) * fd * diffuse_color;
  reflected.direct_specular += fs * specular_color;
}

void compute_lights
(
  vec3 world_position,
  vec3 view_dir,
  vec3 normal,
  PhysicalMaterial mat,
  inout ReflectedLight reflected
)
{
  uint point_count = min( lights.counts.x, MAX_POINT_LIGHTS );
  for( uint i = 0u; i < point_count; i++ )
  {
    PointLight light = lights.point[ i ];
    vec3 light_dir = light.position_range.xyz - world_position;
    float distance = length( light_dir );
    light_dir = normalize( light_dir );

    if( dot( normal, light_dir ) <= 0.0 ) { continue; }

    float range = light.position_range.w;
    float attenuation = pow2( clamp( 1.0 - distance / range, 0.0, 1.0 ) ) / ( distance * distance + 1.0 );
    attenuation *= light.color_strength.w;

    apply_light_contribution( light_dir, view_dir, normal, mat, light.color_strength.xyz, attenuation, reflected );
  }

  uint direct_count = min( lights.counts.y, MAX_DIRECT_LIGHTS );
  for( uint i = 0u; i < direct_count; i++ )
  {
    DirectLight light = lights.direct[ i ];
    vec3 light_dir = light.direction_strength.xyz;

    if( dot( normal, light_dir ) <= 0.0 ) { continue; }

    apply_light_contribution( light_dir, view_dir, normal, mat, light.color.xyz, light.direction_strength.w, reflected );
  }

  uint spot_count = min( lights.counts.z, MAX_SPOT_LIGHTS );
  for( uint i = 0u; i < spot_count; i++ )
  {
    SpotLight light = lights.spot[ i ];
    vec3 light_dir = light.position_range.xyz - world_position;
    float distance = length( light_dir );
    light_dir = normalize( light_dir );

    if( dot( normal, light_dir ) <= 0.0 ) { continue; }

    float range = light.position_range.w;
    float attenuation = pow2( clamp( 1.0 - distance / range, 0.0, 1.0 ) ) / ( distance * distance + 1.0 );

    float angle = acos( dot( -light_dir, light.direction_strength.xyz ) );
    float angular_attenuation = smoothstep( light.outer.x, light.color_inner.w, angle );
    attenuation *= angular_attenuation * light.direction_strength.w;

    apply_light_contribution( light_dir, view_dir, normal, mat, light.color_inner.xyz, attenuation, reflected );
  }
}

void main()
{
  PhysicalMaterial mat;
  ReflectedLight reflected = ReflectedLight( vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ), vec3( 0.0 ) );

  float alpha = material.base_color_factor.a;

  mat.metallness = material.metallic_factor;
  mat.roughness = material.roughness_factor;
  mat.diffuse_color = material.base_color_factor.rgb;

  if( ( material.flags & FLAG_USE_BASE_COLOR_TEXTURE ) != 0u )
  {
    vec4 base_color = texture( tex_1_1, v_uv_0 );
    mat.diffuse_color *= srgb_to_linear( base_color.rgb );
    alpha *= base_color.a;
  }

  if( ( material.flags & FLAG_USE_MR_TEXTURE ) != 0u )
  {
    // Roughness — G channel; metalness — B channel ( glTF convention ).
    vec4 mr_sample = texture( tex_1_3, v_uv_0 );
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
  mat.f0 = mix( vec3( 0.04 ), mat.diffuse_color, mat.metallness );
  mat.f90 = vec3( 1.0 );
  mat.diffuse_color *= 1.0 - mat.metallness;

  vec3 normal = normalize( v_normal );
  if( !gl_FrontFacing )
  {
    normal = -normal;
  }

  // Geometric specular anti-aliasing (Tokuyoshi & Kaplanyan 2019), as in the
  // WGSL original: widen roughness where screen-space normal variance is high.
  vec3 d_n_dx = dFdx( normal );
  vec3 d_n_dy = dFdy( normal );
  float geometric_variance = dot( d_n_dx, d_n_dx ) + dot( d_n_dy, d_n_dy );
  mat.roughness = sqrt( clamp( pow2( mat.roughness ) + 0.5 * geometric_variance, 0.0, 1.0 ) );
  mat.roughness = max( mat.roughness, 0.0525 );

  vec3 view_dir = normalize( camera.position_exposure.xyz - v_world_position );

  compute_lights( v_world_position, view_dir, normal, mat, reflected );

  // Non-IBL ambient fallback, matching the WGSL original.
  reflected.indirect_diffuse += 0.1 * mat.diffuse_color;

  vec3 color = reflected.indirect_diffuse
    + reflected.indirect_specular
    + reflected.direct_diffuse
    + reflected.direct_specular;

  color *= exp2( camera.position_exposure.w );

  // Opaque pass marks covered pixels with alpha = 1; the cleared background
  // stays at alpha = 0 so the tone mapping pass leaves it untouched.
  frag_color = vec4( color, 1.0 );
}

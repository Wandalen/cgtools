struct VSInput
{
  @location( 0 ) mesh_position : vec3f,
  @location( 1 ) light_position : vec3f,
  @location( 2 ) light_color : vec3f,
  @location( 3 ) light_power : f32,
}

struct VSOutput
{
  @builtin( position ) pos : vec4f,
  @location( 0 ) light_position : vec3f,
  @location( 1 ) light_color : vec3f,
  @location( 2 ) light_power : f32,
  @location( 3 ) light_radius : f32
}

struct Uniforms
{
  view_matrix : mat4x4< f32 >,
  projection_matrix : mat4x4< f32 >,
  camera_pos : vec3f,
  time : f32,
  elapsed_time : f32
}

@group( 0 ) @binding( 0 ) var< uniform > uniforms : Uniforms;

@group( 1 ) @binding( 0 ) var albedo_texture : texture_2d< f32 >;
@group( 1 ) @binding( 1 ) var pos_texture : texture_2d< f32 >;
@group( 1 ) @binding( 2 ) var normal_texture : texture_2d< f32 >;
@group( 1 ) @binding( 3 ) var depth_texture : texture_depth_2d;

@vertex
fn vs_main( in : VSInput ) -> VSOutput
{
  var out : VSOutput;

  let radius = 15.0;

  let view_pos = uniforms.view_matrix * vec4( in.mesh_position * radius + in.light_position, 1.0 );
  var clip_pos = uniforms.projection_matrix * view_pos;

  out.pos = clip_pos;
  out.light_position = in.light_position;
  out.light_color = in.light_color;
  out.light_power = in.light_power;
  out.light_radius = radius;
  return out;
}

@fragment
fn fs_main( in : VSOutput ) -> @location( 0 ) vec4f
{
  let uv = vec2< u32 >( floor( in.pos.xy ) );
  let depth = textureLoad( depth_texture, uv, 0 );

  if depth >= 1.0 { discard; }

  let albedo = textureLoad( albedo_texture, uv, 0 ).rgb;
  let position = textureLoad( pos_texture, uv, 0 ).rgb;
  let normal = normalize( textureLoad( normal_texture, uv, 0 ).rgb );

  let view_dir = normalize( uniforms.camera_pos - position );
  var color = vec3f( 0.0 );

  let dist = length( in.light_position - position );
  let norm_dist = dist / in.light_radius;

  if norm_dist > 1.0 { discard; }

  let light_dir = normalize( in.light_position - position );
  let attenuation = exp( -norm_dist * 5.0 );

  let NdotL = saturate( dot( normal, light_dir ) );
  let half = normalize( view_dir + light_dir );
  let phong_value = pow( saturate( dot( half, normal ) ), 64.0 );

  // Bling-Phong model
  color += ( albedo * NdotL + phong_value ) * in.light_color * in.light_power * attenuation; 

  return vec4f( color, 1.0 );
}
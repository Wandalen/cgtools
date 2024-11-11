struct Uniforms
{
  view_matrix : mat4x4< f32 >,
  projection_matrix : mat4x4< f32 >,
  camera_pos : vec3f
}

@group( 0 ) @binding( 0 ) var< uniform > uniforms : Uniforms;
@group( 1 ) @binding( 0 ) var albedo_texture : texture_2d< f32 >;
@group( 1 ) @binding( 1 ) var pos_texture : texture_2d< f32 >;
@group( 1 ) @binding( 2 ) var normal_texture : texture_2d< f32 >;
@group( 1 ) @binding( 3 ) var depth_texture : texture_depth_2d;

@vertex
fn vs_main( @builtin( vertex_index ) id : u32 ) -> @builtin( position ) vec4f
{
  var positions = array< vec2f, 4 >
  (
    vec2f( -1.0, -1.0 ),
    vec2f( 1.0, -1.0 ),
    vec2f( -1.0, 1.0 ),
    vec2f( 1.0, 1.0 )
  );
  return vec4f( positions[ id ], 0.0, 1.0 );
}

struct FSOutput
{
  @location( 0 ) albedo : vec4f,
  @location( 1 ) position : vec4f,
  @location( 2 ) normal : vec4f
}

@fragment
fn fs_main( @builtin( position ) coords : vec4f ) -> @location( 0 ) vec4f
{
  let uv = vec2< u32 >( floor( coords.xy ) );
  let depth = textureLoad( depth_texture, uv, 0 );

  if depth >= 1.0 { discard; }

  let albedo = textureLoad( albedo_texture, uv, 0 ).rgb;
  let position = textureLoad( pos_texture, uv, 0 ).rgb;
  let normal = textureLoad( normal_texture, uv, 0 ).rgb;

  let view_dir = normalize( uniforms.camera_pos - position );
  let light_dir = normalize( vec3f( 1.0 ) );
  let light_color = vec3f( 1.0 );

  let NdotL = saturate( dot( normal, light_dir ) ) + 0.00001;
  let half = normalize( view_dir + light_dir );
  let phong_value = pow( saturate( dot( half, normal ) ), 64.0 );

  let ambient_light = vec3f( 1.0 );
  let ambient_scale = 0.1;

  let color = ambient_light * ambient_scale + ( albedo + light_color * phong_value / NdotL ) * NdotL;

  return vec4f( color, 1.0 );
}
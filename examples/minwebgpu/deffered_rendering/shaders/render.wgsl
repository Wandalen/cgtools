struct Uniforms
{
  view_matrix : mat4x4< f32 >,
  projection_matrix : mat4x4< f32 >,
  camera_pos : vec3f,
  time : f32,
  elapsed_time : f32
}

struct Light
{
  color : vec3f,
  power : f32,
  position : vec3f,
  direction : f32
}

@group( 0 ) @binding( 0 ) var< uniform > uniforms : Uniforms;
@group( 0 ) @binding( 1 ) var< storage, read > lights : array< Light >;

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

  const BACKGROUND_LIGHT : vec3f = vec3f( 1.0 );

  if depth >= 1.0 { return vec4f( BACKGROUND_LIGHT, 1.0 ); }

  let albedo = textureLoad( albedo_texture, uv, 0 ).rgb;
  let position = textureLoad( pos_texture, uv, 0 ).rgb;
  let normal = normalize( textureLoad( normal_texture, uv, 0 ).rgb );

  let view_dir = normalize( uniforms.camera_pos - position );
  var color = BACKGROUND_LIGHT * 0.25;
  for( var i : u32 = 0; i < arrayLength( &lights ); i += u32( 1 ) )
  {
    let light = lights[ i ];

    let dist = length( light.position - position );
    let radius = 16.0;
    let norm_dist = dist / radius;

    if norm_dist > 1.0 { continue; }

    let light_dir = normalize( light.position - position );
    let attenuation = exp( -norm_dist * 5.0 );

    let NdotL = saturate( dot( normal, light_dir ) );
    let half = normalize( view_dir + light_dir );
    let phong_value = pow( saturate( dot( half, normal ) ), 64.0 );

    // Bling-Phong model
    color += ( albedo * NdotL + phong_value ) * light.color * light.power * attenuation; 
  }

  return vec4f( color, 1.0 );
}
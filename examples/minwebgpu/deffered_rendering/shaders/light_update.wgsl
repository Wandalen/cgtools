struct Light
{
  color : vec3f,
  power : f32,
  position : vec3f,
  direction : f32
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
@group( 0 ) @binding( 1 ) var< storage, read_write > lights : array< Light >;

fn rot( angle : f32 ) -> mat3x3< f32 >
{
  let s = sin( angle );
  let c = cos( angle );
  return mat3x3< f32 >
  ( 
    c,   0.0, -s, 
    0.0, 1.0, 0.0,
    s,   0.0, c
  );
}

@compute @workgroup_size( 64 )
fn update_light( @builtin( global_invocation_id ) id : vec3u )
{
  if id.x >= arrayLength( &lights ) { return; }

  var light = lights[ id.x ];
  
  let rotation = rot( uniforms.elapsed_time * 20.0 * light.direction / max( length( light.position), 0.000001 ) );
  light.position = rotation * light.position;
  lights[ id.x ] = light;
}
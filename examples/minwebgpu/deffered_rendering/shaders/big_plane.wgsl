
struct VSOutput
{
  @builtin( position ) clip_pos  : vec4f,
  @location( 0 ) position : vec3f,
  @location( 1 ) albedo : vec4f,
  @location( 2 ) normal : vec3f,
  @location( 3 ) uv : vec2f
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

@vertex
fn vs_main( @builtin( vertex_index) id : u32 ) -> VSOutput
{
  var positions = array< vec2f, 6 >
  (
    vec2f( -1.0, -1.0 ),
    vec2f(  1.0,  1.0 ),
    vec2f( -1.0,  1.0 ),
    vec2f( -1.0, -1.0 ),
    vec2f(  1.0, -1.0 ),
    vec2f(  1.0,  1.0 ),
  );

  let position = vec3f( positions[ id ], 0.0 ) * 100.0 - vec3f( 0.0, 0.0, 5.0 );
  let pos = position.xzy;

  var output : VSOutput;
  output.position = pos;
  output.albedo = vec4( 1.0, 1.0, 1.0, 1.0 );
  output.normal = vec3f( 0.0, 1.0, 0.0 );
  output.uv = vec2( 0.0 );
  output.clip_pos = uniforms.projection_matrix * uniforms.view_matrix * vec4f( pos, 1.0 );
  return output;
}
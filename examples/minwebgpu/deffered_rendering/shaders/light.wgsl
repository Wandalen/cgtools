struct VSInput
{
  @builtin( vertex_index ) id : u32,
  @location( 0 ) position : vec3f,
  @location( 1 ) color : vec3f
}

struct VSOutput
{
  @builtin( position ) pos : vec4f,
  @location( 0 ) color : vec3f
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
fn vs_main( in : VSInput ) -> VSOutput
{
  var out : VSOutput;

  var offsets = array< vec3f, 14 >
  (
    vec3f( -1.0,  1.0,  1.0 ),    // Front-top-left
    vec3f(  1.0,  1.0,  1.0 ),    // Front-top-right
    vec3f( -1.0, -1.0,  1.0 ),    // Front-bottom-left
    vec3f(  1.0, -1.0,  1.0 ),    // Front-bottom-right
    vec3f(  1.0, -1.0, -1.0 ),    // Back-bottom-right
    vec3f(  1.0,  1.0,  1.0 ),    // Front-top-right
    vec3f(  1.0,  1.0, -1.0 ),    // Back-top-right
    vec3f( -1.0,  1.0,  1.0 ),    // Front-top-left
    vec3f( -1.0,  1.0, -1.0 ),    // Back-top-left
    vec3f( -1.0, -1.0,  1.0 ),    // Front-bottom-left
    vec3f( -1.0, -1.0, -1.0 ),    // Back-bottom-left
    vec3f(  1.0, -1.0, -1.0 ),    // Back-bottom-right
    vec3f( -1.0,  1.0, -1.0 ),    // Back-top-left
    vec3f(  1.0,  1.0, -1.0 )     // Back-top-right
  );

  let view_pos = uniforms.view_matrix * vec4( in.position + offsets[ in.id ] * 0.2, 1.0 );
  var clip_pos = uniforms.projection_matrix * view_pos;

  out.pos = clip_pos;
  out.color = in.color;
  return out;
}

@fragment
fn fs_main( in : VSOutput ) -> @location( 0 ) vec4f
{
  var color = vec4f( in.color, 1.0 );
  return color;
}
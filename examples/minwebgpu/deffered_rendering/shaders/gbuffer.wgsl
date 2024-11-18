
struct VSInput
{
  @location( 0 ) position : vec3f,
  @location( 1 ) normal : vec3f,
  @location( 2 ) uv : vec2f
}

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
  time : f32
}

@group( 0 ) @binding( 0 ) var< uniform > uniforms : Uniforms;

@vertex
fn vs_main( input : VSInput ) -> VSOutput
{
  var output : VSOutput;
  output.position = input.position;
  output.albedo = vec4( 1.0, 1.0, 1.0, 1.0 );
  output.normal = input.normal;
  output.uv = input.uv;
  output.clip_pos = uniforms.projection_matrix * uniforms.view_matrix * vec4f( ( input.position ) * 15.0, 1.0 );
  return output;
}

struct FSOutput
{
  @location( 0 ) albedo : vec4f,
  @location( 1 ) position : vec4f,
  @location( 2 ) normal : vec4f
}

@fragment
fn fs_main( input : VSOutput ) -> FSOutput
{
  var output : FSOutput;
  output.albedo = input.albedo;
  output.position = vec4f( input.position, 1.0 );
  output.normal = vec4f( normalize( input.normal ), 1.0 );
  return output;
}
struct VSInput
{
  @builtin( vertex_index ) id : u32,
  @location( 0 ) position : vec3f,
  @location( 1 ) color : vec3f,
  @location( 2 ) direction : f32
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
  time : f32
}

@group( 0 ) @binding( 0 ) var< uniform > uniforms : Uniforms;

fn rot( angle : f32 ) -> mat2x2< f32 >
{
  let s = sin( angle );
  let c = cos( angle );
  return mat2x2< f32 >( c, s, -s, c);
}

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

  let rot = rot( uniforms.time * 20.0 * in.direction / max( length( in.position), 0.000001 ) );
  let rot_pos = rot * in.position.xz;
  let pos = vec3f( rot_pos.x, in.position.y, rot_pos.y ) + offsets[ in.id ] * 0.2;

  let view_pos = uniforms.view_matrix * vec4( pos, 1.0 );
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
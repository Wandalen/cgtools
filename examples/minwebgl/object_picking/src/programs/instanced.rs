use minwebgl as gl;
use gl::GL;
use web_sys::
{
  WebGlProgram,
  WebGlUniformLocation,
};

pub struct Instanced
{
  pub program : WebGlProgram,
  pub projection_view_location : Option< WebGlUniformLocation >,
}

impl Instanced
{
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "../shaders/instanced.vert" );
    let fragment_shader = include_str!( "../shaders/instanced.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( &gl )
    .unwrap();
    let projection_view_location = gl.get_uniform_location( &program, "u_projection_view" );

    Self
    {
      program,
      projection_view_location,
    }
  }
}

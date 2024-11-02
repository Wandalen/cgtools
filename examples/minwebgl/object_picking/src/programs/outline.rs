use minwebgl as gl;
use gl::GL;
use web_sys::
{
  WebGlProgram,
  WebGlUniformLocation,
};

pub struct Outline
{
  pub program : WebGlProgram,
  pub mvp_location : Option< WebGlUniformLocation >,
}

impl Outline
{
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "../shaders/outline.vert" );
    let fragment_shader = include_str!( "../shaders/outline.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( &gl )
    .unwrap();
    let mvp_location = gl.get_uniform_location( &program, "u_mvp" );

    Self
    {
      program,
      mvp_location,
    }
  }
}

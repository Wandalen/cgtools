use minwebgl as gl;
use gl::GL;
use web_sys::WebGlProgram;

pub struct Rasterize
{
  pub program : WebGlProgram,
}

impl Rasterize
{
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "../shaders/rasterize.vert" );
    let fragment_shader = include_str!( "../shaders/rasterize.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( &gl )
    .unwrap();

    Self
    {
      program,
    }
  }
}

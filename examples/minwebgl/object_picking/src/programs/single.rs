use minwebgl as gl;
use gl::GL;
use web_sys::
{
  WebGlProgram,
  WebGlUniformLocation,
};

pub struct Single
{
  pub program : WebGlProgram,
  pub model : Option< WebGlUniformLocation >,
  pub norm_mat : Option< WebGlUniformLocation >,
  pub projection_view : Option< WebGlUniformLocation >,
  pub id : Option< WebGlUniformLocation >,
}

impl Single
{
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "../shaders/single.vert" );
    let fragment_shader = include_str!( "../shaders/single.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( &gl )
    .unwrap();
    let model = gl.get_uniform_location( &program, "u_model" );
    let norm_mat = gl.get_uniform_location( &program, "u_norm_mat" );
    let projection_view = gl.get_uniform_location( &program, "u_projection_view" );
    let id = gl.get_uniform_location( &program, "u_id" );

    Self
    {
      program,
      model,
      norm_mat,
      projection_view,
      id,
    }
  }
}

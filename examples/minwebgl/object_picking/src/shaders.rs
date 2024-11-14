use minwebgl as gl;
use gl::GL;
use web_sys::
{
  WebGlProgram,
  WebGlUniformLocation,
};

pub struct ObjectShader
{
  pub program : WebGlProgram,
  pub model : Option< WebGlUniformLocation >,
  pub projection_view : Option< WebGlUniformLocation >,
}

impl ObjectShader
{
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "shaders/object.vert" );
    let fragment_shader = include_str!( "shaders/object.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( &gl )
    .unwrap();
    let model = gl.get_uniform_location( &program, "u_model" );
    let projection_view = gl.get_uniform_location( &program, "u_projection_view" );

    Self
    {
      program,
      model,
      projection_view,
    }
  }
}

pub struct OutlineShader
{
  pub program : WebGlProgram,
  pub mvp : Option< WebGlUniformLocation >,
}

impl OutlineShader
{
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "shaders/outline.vert" );
    let fragment_shader = include_str!( "shaders/outline.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( &gl )
    .unwrap();
    let mvp = gl.get_uniform_location( &program, "u_mvp" );

    Self
    {
      program,
      mvp,
    }
  }
}

pub struct IdShader
{
  pub program : WebGlProgram,
  pub mvp : Option< WebGlUniformLocation >,
  pub id : Option< WebGlUniformLocation >,
}

impl IdShader
{
  pub fn new( gl : &GL ) -> Self
  {
    let vertex_shader = include_str!( "shaders/id.vert" );
    let fragment_shader = include_str!( "shaders/id.frag" );
    let program = gl::ProgramFromSources::new( vertex_shader, fragment_shader )
    .compile_and_link( &gl )
    .unwrap();
    let mvp = gl.get_uniform_location( &program, "u_mvp" );
    let id = gl.get_uniform_location( &program, "u_id" );

    Self
    {
      program,
      mvp,
      id,
    }
  }
}

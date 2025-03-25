use minwebgl as gl;
use gl::{ WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject, GL };

/// A shader program used for rendering hexagons.
/// This shader handles the transformation matrix (MVP) and the color of the hexagons.
pub struct HexShader
{
  program : WebGlProgram,
  mvp_location : WebGlUniformLocation,
  color_location : WebGlUniformLocation,
}

impl HexShader
{
  pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    let vert = include_str!( "shaders/main.vert" );
    let frag = include_str!( "shaders/main.frag" );
    let program = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl )?;
    let mvp_location = gl.get_uniform_location( &program, "MVP" ).unwrap();
    let color_location = gl.get_uniform_location( &program, "color" ).unwrap();

    Ok
    (
      HexShader
      {
        program,
        mvp_location,
        color_location,
      }
    )
  }

  pub fn draw
  (
    &self,
    gl : &GL,
    mode : u32,
    geometry : &Geometry,
    mvp : &[ f32 ],
    color : [ f32; 4 ]
  )
  -> Result< (), gl::WebglError >
  {
    gl.bind_vertex_array( Some( &geometry.vao ) );
    gl.use_program( Some( &self.program ) );
    gl::uniform::matrix_upload( gl, Some( self.mvp_location.clone() ), mvp, true )?;
    gl::uniform::upload( gl, Some( self.color_location.clone() ), color.as_slice() )?;
    gl.draw_arrays( mode, 0, geometry.count );

    Ok( () )
  }
}

/// Represents the vertices geometry, including its vertex array object (VAO)
/// and the number of vertices.
pub struct Geometry
{
  pub vao : WebGlVertexArrayObject,
  pub count : i32
}

/// Generates the geometry for a hexagon's outline as a series of lines.
///
/// # Parameters
/// - `gl`: The WebGL context.
///
/// # Returns
/// A `Geometry` instance containing the hexagon's vertex array object (VAO) and vertex count.
///
/// # Errors
/// Returns a `WebglError` if there is an issue creating or uploading the geometry data.
pub fn geometry2d( gl : &GL, positions : &[ f32 ] ) -> Result< Geometry, gl::WebglError >
{
  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, positions, gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 0 ).offset( 0 ).attribute_pointer( &gl, 0, &position_buffer )?;

  Ok( Geometry { vao, count : positions.len() as i32 / 2 } )
}

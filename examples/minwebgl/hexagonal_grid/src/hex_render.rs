use minwebgl as gl;
use gl::{ WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject, GL };

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

/// Generates line mesh geometry in the manner of LINE LOOP for a hexagon.
/// The hexagon is flat top and has an outer circle radius of 1.
///
/// # Returns
/// A `Vec<f32>` containing the x and y coordinates of the hexagon's outline.
pub fn hex_line_loop_mesh() -> Vec< f32 >
{
  let points = vertices_points();
  let mut positions = vec![];
  for point in points
  {
    positions.push( point.0 );
    positions.push( point.1 );
  }

  positions
}

/// Generates triangle mesh geometry in the manner of TRIANGLE FAN for a hexagon.
/// The hexagon is flat top and has an outer circle radius of 1.
/// # Returns
/// A `Vec<f32>` containing the x and y coordinates of the triangles.
pub fn hex_triangle_fan_mesh() -> Vec< f32 >
{
  let points = vertices_points();
  let mut positions = vec![];

  let hex_center = ( 0.0, 0.0 );
  positions.push( hex_center.0 );
  positions.push( hex_center.1 );

  for point in points
  {
    positions.push( point.0 );
    positions.push( point.1 );
  }

  let point = points.first().unwrap();
  positions.push( point.0 );
  positions.push( point.1 );

  positions
}

/// Generates the six corner points of a flat top hexagon, with outer circle radius of 1.
///
/// # Returns
/// An array of six `(f32, f32)` tuples representing the x and y coordinates of the hexagon's corners.
pub fn vertices_points() -> [ ( f32, f32 ); 6 ]
{
  let mut points : [ ( f32, f32 ); 6 ] = Default::default();
  for i in 0..6
  {
    let angle = 60 * i;
    let angle = ( angle as f32 ).to_radians();
    points[ i ] = ( angle.cos(), angle.sin() )
  }
  points
}

/// Represents the vertices geometry, including its vertex array object (VAO)
/// and the number of vertices.
pub struct Geometry
{
  pub vao : WebGlVertexArrayObject,
  pub count : i32
}

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

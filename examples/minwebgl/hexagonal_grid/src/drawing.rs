use minwebgl as gl;
use gl::{ WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject, GL };

pub fn hex_geometry( gl : &GL ) -> Result< Geometry, gl::WebglError >
{
  let positions = hex_positions();
  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 0 ).offset( 0 ).attribute_pointer( &gl, 0, &position_buffer )?;

  Ok( Geometry { vao, count : positions.len() as i32 } )
}

pub fn hex_positions() -> Vec< f32 >
{
  let hex_point = hex_points();
  let mut positions = vec![];
  for w in hex_point.windows( 2 )
  {
    let point1 = w[ 0 ];
    let point2 = w[ 1 ];
    positions.push( point1.0 );
    positions.push( point1.1 );
    positions.push( point2.0 );
    positions.push( point2.1 );
  }
  // connect last and first points into a line
  let last_point = hex_point.last().unwrap();
  let first_point = hex_point.first().unwrap();
  positions.push( last_point.0 );
  positions.push( last_point.1 );
  positions.push( first_point.0 );
  positions.push( first_point.1 );

  positions
}

pub fn hex_points() -> [ ( f32, f32 ); 6 ]
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

pub struct Geometry
{
  pub vao : WebGlVertexArrayObject,
  pub count : i32
}

pub struct LineShader
{
  program : WebGlProgram,
  mvp_location : WebGlUniformLocation,
  color_location : WebGlUniformLocation,
}

impl LineShader
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
      LineShader
      {
        program,
        mvp_location,
        color_location,
      }
    )
  }

  pub fn draw( &self, gl : &GL, geometry : &Geometry, mvp : &[ f32 ], color : [ f32; 4 ] ) -> Result< (), gl::WebglError >
  {
    gl.bind_vertex_array( Some( &geometry.vao ) );
    gl.use_program( Some( &self.program ) );
    gl::uniform::matrix_upload( gl, Some( self.mvp_location.clone() ), mvp, true )?;
    gl::uniform::upload( gl, Some( self.color_location.clone() ), color.as_slice() )?;
    gl.draw_arrays( gl::LINES, 0, geometry.count );

    Ok( () )
  }
}

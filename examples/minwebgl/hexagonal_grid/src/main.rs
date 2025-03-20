use minwebgl as gl;
use gl::{ WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject, GL };

fn main() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  let ortho = cgmath::ortho( -10.0f32, 10.0, -8.0, 8.0, 0.0, 1.0 );

  let geometry = hex_geometry( &gl )?;
  let hex_shader = HexShader::new( &gl )?;

  gl.clear_color( 0.9, 0.9, 0.9, 1.0 );
  gl.clear( gl::COLOR_BUFFER_BIT );

  let size = 0.5;
  let spacing = 3.0f32.sqrt() * size;
  let total_width = ( 9.0 + 0.5 ) * spacing;
  let total_height = 9.0 * ( 1.5 * size );

  for y in 0..10
  {
    for x in 0..10
    {
      let odd_r = spacing / 2.0 * ( y & 1 ) as f32;
      let x = x as f32;
      let x = odd_r + x * spacing;
      let y = -y as f32;
      let y = y * 1.5 * size;

      let position = cgmath::vec3( x - total_width * 0.5, y + total_height * 0.5, 0.0 );
      let translation = cgmath::Matrix4::from_translation( position );
      let rotation = cgmath::Matrix4::from_angle_z( cgmath::Deg( 30.0 ) );
      let scale = cgmath::Matrix4::from_scale( size );
      let mvp = ortho * translation * rotation * scale;
      let mvp : &[ f32; 16 ] = mvp.as_ref();
      hex_shader.draw_hex( &geometry, mvp )?;
    }
  }

  Ok( () )
}

fn hex_geometry( gl : &GL ) -> Result< Geometry, gl::WebglError >
{
  let positions = hex_positions();
  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl )?;
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 0 ).offset( 0 ).attribute_pointer( &gl, 0, &position_buffer )?;

  Ok( Geometry { vao, count : positions.len() as i32 } )
}

fn hex_positions() -> Vec< f32 >
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

fn hex_points() -> [ ( f32, f32 ); 6 ]
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

pub struct HexShader
{
  program : WebGlProgram,
  mvp_location : WebGlUniformLocation,
  gl : GL,
}

impl HexShader
{
  pub fn new( gl : &GL ) -> Result< Self, gl::WebglError >
  {
    let vert = include_str!( "shaders/main.vert" );
    let frag = include_str!( "shaders/main.frag" );
    let program = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl )?;
    let mvp_location = gl.get_uniform_location( &program, "MVP" ).unwrap();

    Ok
    (
      HexShader
      {
        program,
        mvp_location,
        gl : gl.clone()
      }
    )
  }

  pub fn draw_hex( &self, geometry : &Geometry, mvp : &[ f32 ] ) -> Result< (), gl::WebglError >
  {
    self.gl.bind_vertex_array( Some( &geometry.vao ) );
    self.gl.use_program( Some( &self.program ) );
    gl::uniform::matrix_upload( &self.gl, Some( self.mvp_location.clone() ), mvp, true )?;
    self.gl.draw_arrays( gl::LINES, 0, geometry.count );

    Ok( () )
  }
}

pub struct AxialCoordinate
{
  pub q : i32,
  pub r : i32,
}

pub enum TopType
{
  PointyTop,
  FlatTop,
}

pub enum LayoutType
{
  OddShift,
  EvenShift,
}

pub struct HexGrid< T >
{
  data : Vec< Vec< Option< T > > >,
  size : f32,
  top_type : TopType,
  layout_type : LayoutType,
}

impl< T > HexGrid< T >
{
  pub fn new( len : usize, count : usize, top_type : TopType, layout_type : LayoutType, size : f32 ) -> Self
  {
    let mut data = vec![];
    for _ in 0..count
    {
      let mut v = vec![];
      for _ in 0..len
      {
        v.push( None );
      }

      data.push( v );
    }

    Self
    {
      data,
      size,
      top_type,
      layout_type,
    }
  }

  pub fn insert( &mut self )
  {
    todo!()
  }
}

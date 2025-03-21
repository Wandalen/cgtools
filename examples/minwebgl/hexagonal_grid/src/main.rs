use minwebgl as gl;
use gl::{ WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject, GL };

fn main() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;

  let ortho = cgmath::ortho( -10.0f32, 10.0, -8.0, 8.0, 0.0, 1.0 );

  let geometry = hex_geometry( &gl )?;
  let hex_shader = LineShader::new( &gl )?;

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
      hex_shader.draw( &gl, &geometry, mvp, [ 0.1, 0.1, 0.1, 1.0 ] )?;
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

#[ derive( Debug, Clone, Copy, Hash, PartialEq, Eq ) ]
pub struct Axial
{
  pub q : i32,
  pub r : i32,
}

impl From< Offset > for Axial
{
  fn from( value: Offset ) -> Self
  {
    let ( q, r ) = match value.layout
    {
      Layout::HorizontalOddShift =>
      {
        let q = value.column - ( value.row - value.row & 1 ) / 2;
        let r = value.row;
        ( q, r )
      },
      Layout::HorizontalEvenShift =>
      {
        let q = value.column - ( value.row + value.row & 1 ) / 2;
        let r = value.row;
        ( q, r )
      },
      Layout::VerticalOddShift =>
      {
        let q = value.column;
        let r = value.row - ( value.column - value.column & 1 ) / 2;
        ( q, r )
      },
      Layout::VerticalEvenShift =>
      {
        let q = value.column;
        let r = value.row - ( value.column + value.column & 1 ) / 2;
        ( q, r )
      },
    };

    Self
    {
      q,
      r,
    }
  }
}

pub struct Offset
{
  pub row : i32,
  pub column : i32,
  pub layout : Layout,
}

pub enum Layout
{
  HorizontalOddShift,
  HorizontalEvenShift,
  VerticalOddShift,
  VerticalEvenShift,
}

pub struct GridMap< T >
{
  data : rustc_hash::FxHashMap< Axial, T >
}

impl< T > GridMap< T >
{
  pub fn new() -> Self
  {
    Self { data : Default::default() }
  }

  pub fn insert< C : Into< Axial > >( &mut self, coord : C, val : T ) -> Option< T >
  {
    let axial : Axial = coord.into();
    self.data.insert( axial, val )
  }

  pub fn remove< C : Into< Axial > >( &mut self, coord : C ) -> Option< T >
  {
    let axial : Axial = coord.into();
    self.data.remove( &axial )
  }

  pub fn iter( &self ) -> std::collections::hash_map::Iter< Axial, T >
  {
    self.data.iter()
  }
}

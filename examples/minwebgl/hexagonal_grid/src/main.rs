// use std::marker::PhantomData;
use minwebgl as gl;

fn main()
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make().expect( "Can't retrieve GL context" );

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

  let position_buffer = gl::buffer::create( &gl ).unwrap();
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl ).unwrap();
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 0 ).offset( 0 ).attribute_pointer( &gl, 0, &position_buffer ).unwrap();

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let program = gl::ProgramFromSources::new(vert, frag).compile_and_link( &gl ).unwrap();
  gl.use_program( Some( &program ));
  gl.draw_arrays( gl::LINES, 0, positions.len() as i32 );
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

struct AxialCoordinate
{
  q : i32,
  r : i32,
}

enum TopType
{
  PointyTop,
  FlatTop,
}

enum LayoutType
{
  OddShift,
  EvenShift,
}

struct HexGrid
{
  size : f32,
  top_type : TopType,
  layout_type : LayoutType,
  len : usize,
  count : usize,
}

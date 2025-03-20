use minwebgl as gl;

fn main()
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make().expect( "Can't retrieve GL context" );

  let positions = hex_positions();
  let position_buffer = gl::buffer::create( &gl ).unwrap();
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), gl::STATIC_DRAW );

  let vao = gl::vao::create( &gl ).unwrap();
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 0 ).offset( 0 ).attribute_pointer( &gl, 0, &position_buffer ).unwrap();

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let program = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl ).unwrap();
  gl.use_program( Some( &program ) );

  let ortho = cgmath::ortho( -10.0f32, 10.0, -8.0, 8.0, 0.0, 1.0 );
  let ortho : &[ f32; 16 ] = ortho.as_ref();
  let mvp_location = gl.get_uniform_location( &program, "MVP" ).unwrap();
  gl::uniform::matrix_upload( &gl, Some(mvp_location), ortho.as_slice(), true ).unwrap();

  gl.clear_color( 0.9, 0.9, 0.9, 1.0 );
  gl.clear( gl::COLOR_BUFFER_BIT );

  gl.draw_arrays( gl::LINES, 0, positions.len() as i32 );
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

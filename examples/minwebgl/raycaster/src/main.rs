use minwebgl as gl;
use gl::GL;

fn main()
{
  gl::browser::setup( Default::default() );
  run();
}

const SCREEN_WIDTH : u32 = 1024;
const SCREEN_HEIGHT : u32 = 512;
const MAP_SIDE_LEN : usize = 8;
const MAP : [ u8; MAP_SIDE_LEN * MAP_SIDE_LEN ] =
[
  1, 1, 1, 1, 1, 1, 1, 1,
  1, 0, 0, 0, 0, 0, 0, 1,
  1, 1, 0, 0, 0, 0, 0, 1,
  1, 0, 0, 0, 0, 0, 0, 1,
  1, 0, 1, 0, 0, 0, 0, 1,
  1, 0, 1, 0, 0, 0, 0, 1,
  1, 0, 1, 0, 0, 0, 0, 1,
  1, 1, 1, 1, 1, 1, 1, 1,
];

fn run()
{
  let gl = gl::context::retrieve_or_make().unwrap();
  gl.clear_color( 0.3, 0.3, 0.3, 1.0 );

  let v = include_str!( "shaders/point.vert" );
  let f = include_str!( "shaders/point.frag" );
  let program = gl::ProgramFromSources::new( v, f ).compile_and_link( &gl ).unwrap();
  gl.use_program( Some( &program) );

  let loop_ = move | _t |
  {
    gl.clear( GL::COLOR_BUFFER_BIT );

    draw_map( &gl );

    gl.vertex_attrib2f( 0, 0.0, 0.0 );
    gl.vertex_attrib1f( 1, 8.0 );
    gl.vertex_attrib3f( 2, 1.0, 0.5, 0.0 );
    gl.draw_arrays( GL::POINTS, 0, 1 );

    true
  };
  gl::exec_loop::run( loop_ );
}

fn draw_map( gl : &GL )
{
  for ( i, item ) in MAP.iter().enumerate()
  {
    let col = ( i % MAP_SIDE_LEN ) as f32;
    let row = ( i / MAP_SIDE_LEN ) as f32;
    let color = if *item == 1
    {
      [ 1.0, 1.0, 1.0 ]
    }
    else
    {
      [ 0.0, 0.0, 0.0 ]
    };
    let size = 64.0;
    let pos = coord2screen( &[ -512.0 + size * ( col + 0.5 ), 256.0 - size * ( row + 0.5 ) ] );
    gl.vertex_attrib2fv_with_f32_array( 0, &pos );
    gl.vertex_attrib1f( 1, size - 1.0 );
    gl.vertex_attrib3fv_with_f32_array( 2, &color );
    gl.draw_arrays( GL::POINTS, 0, 1 );
  }
}

fn coord2screen( coord : &[ f32; 2 ] ) -> [ f32; 2 ]
{
  const WIDTH : f32 = SCREEN_WIDTH as f32;
  const HEIGHT : f32 = SCREEN_HEIGHT as f32;

  [ coord[ 0 ] / ( WIDTH / 2.0 ), coord[ 1 ] / ( HEIGHT / 2.0 ) ]
}

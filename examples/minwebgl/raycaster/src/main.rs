mod controls;

use std::f32::consts;
use controls::Controls;
use minwebgl as gl;
use gl::GL;

fn main()
{
  gl::browser::setup( Default::default() );
  run();
}

const PI2 : f32 = consts::PI * 2.0;

const WORLD_WIDTH : u32 = 1024;
const WORLD_HEIGHT : u32 = 512;
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

  let point = include_str!( "shaders/point.vert" );
  let line = include_str!( "shaders/line.vert" );
  let fragment = include_str!( "shaders/main.frag" );
  let point_shader = gl::ProgramFromSources::new( point, fragment ).compile_and_link( &gl ).unwrap();
  let line_shader = gl::ProgramFromSources::new( line, fragment ).compile_and_link( &gl ).unwrap();

  let controls = Controls::setup();
  let mut player_pos = [ -80.0, 30.0 ];
  let mut angle = 0.0;
  let mut last_time = 0.0;

  let loop_ = move | time |
  {
    let time = ( time / 1000.0 ) as f32;
    let delta_time = time - last_time;
    last_time = time;

    angle += controls.borrow().as_vec()[ 1 ] * delta_time * 2.5;
    if angle < 0.0
    {
      angle = PI2 + angle % PI2;
    }
    if angle > PI2
    {
      angle %= PI2;
    }
    let dir = player_direction( angle );

    player_pos[ 0 ] += dir[ 0 ] * delta_time * controls.borrow().as_vec()[ 0 ] * 70.0;
    player_pos[ 1 ] += dir[ 1 ] * delta_time * controls.borrow().as_vec()[ 0 ] * 70.0;

    let line_start = world2screen( &player_pos );
    let line_end = world2screen( &[ player_pos[ 0 ] + dir[ 0 ] * 20.0, player_pos[ 1 ] + dir[ 1 ] * 20.0 ] );

    gl.clear( GL::COLOR_BUFFER_BIT );

    gl.use_program( Some( &point_shader ) );
    draw_map( &gl );
    draw_player( &gl, &world2screen( &player_pos ) );

    gl.use_program( Some( &line_shader ) );
    gl.vertex_attrib2fv_with_f32_array( 0, &line_start );
    gl.vertex_attrib2fv_with_f32_array( 1, &line_end );
    gl.vertex_attrib3f( 2, 1.0, 0.5, 0.0 );
    gl.draw_arrays( GL::LINES, 0, 2 );

    true
  };
  gl::exec_loop::run( loop_ );
}

fn player_direction( angle : f32 ) -> [ f32; 2 ]
{
  [
    angle.cos(),
    angle.sin(),
  ]
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
    let pos = world2screen( &[ -512.0 + size * ( col + 0.5 ), 256.0 - size * ( row + 0.5 ) ] );
    gl.vertex_attrib2fv_with_f32_array( 0, &pos );
    gl.vertex_attrib1f( 1, size - 1.0 );
    gl.vertex_attrib3fv_with_f32_array( 2, &color );
    gl.draw_arrays( GL::POINTS, 0, 1 );
  }
}

fn draw_player( gl : &GL, player_pos : &[ f32; 2 ] )
{
  gl.vertex_attrib2fv_with_f32_array( 0, player_pos );
  gl.vertex_attrib1f( 1, 8.0 );
  gl.vertex_attrib3f( 2, 1.0, 0.5, 0.0 );
  gl.draw_arrays( GL::POINTS, 0, 1 );
}

fn world2screen( coord : &[ f32; 2 ] ) -> [ f32; 2 ]
{
  const WIDTH : f32 = WORLD_WIDTH as f32;
  const HEIGHT : f32 = WORLD_HEIGHT as f32;

  [ coord[ 0 ] / ( WIDTH / 2.0 ), coord[ 1 ] / ( HEIGHT / 2.0 ) ]
}

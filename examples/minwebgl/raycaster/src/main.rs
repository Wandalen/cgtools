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

const WORLD_WIDTH : f32     = 1024.0;
const WORLD_HEIGHT : f32    = 512.0;
const CELL_SIZE : f32       = 64.0;

const MAP_SIDE : usize = 8;
const MAP : [ u8; MAP_SIDE * MAP_SIDE ] =
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

  let point         = include_str!( "shaders/point.vert" );
  let line          = include_str!( "shaders/line.vert" );
  let fragment      = include_str!( "shaders/main.frag" );
  let point_shader  = gl::ProgramFromSources::new( point, fragment ).compile_and_link( &gl ).unwrap();
  let line_shader   = gl::ProgramFromSources::new( line, fragment ).compile_and_link( &gl ).unwrap();

  let controls = Controls::setup();
  let rotation_velocity     = 2.5;
  let move_velocity         = 1.0;
  let mut player_pos        = [ 2.5, 2.5 ];
  let mut angle             = 0.0;
  let mut last_time         = 0.0;

  let loop_ = move | time |
  {
    let time = ( time / 1000.0 ) as f32;
    let delta_time = time - last_time;
    last_time = time;

    angle += rotation_velocity * delta_time * controls.borrow().as_vec()[ 1 ];

    // wrap angle between 0 and 2PI
    if angle < 0.0
    {
      angle = PI2 + angle % PI2;
    }
    if angle > PI2
    {
      angle %= PI2;
    }

    // direction where player is facing
    let dir = direction( angle );

    // update player position
    player_pos[ 0 ] += move_velocity * dir[ 0 ] * delta_time * controls.borrow().as_vec()[ 0 ];
    player_pos[ 1 ] += move_velocity * -dir[ 1 ] * delta_time * controls.borrow().as_vec()[ 0 ];

    let end = cast_rays( &player_pos, angle );
    // gl::info!( "{end:?}" );

    gl.clear( GL::COLOR_BUFFER_BIT );

    gl.use_program( Some( &point_shader ) );

    // draw map
    for ( i, item ) in MAP.iter().enumerate()
    {
      let col = ( i % MAP_SIDE ) as f32;
      let row = ( i / MAP_SIDE ) as f32;

      let color = if *item == 1
      {
        [ 1.0, 1.0, 1.0 ]
      }
      else
      {
        [ 0.0, 0.0, 0.0 ]
      };

      let posx = ( -512.0 + CELL_SIZE * ( col + 0.5 ) ) / 512.0;
      let posy = ( 256.0 - CELL_SIZE * ( row + 0.5 ) ) / 256.0;

      gl.vertex_attrib2fv_with_f32_array( 0, &[ posx, posy ] );
      gl.vertex_attrib1f( 1, CELL_SIZE - 1.0 );
      gl.vertex_attrib3fv_with_f32_array( 2, &color );
      gl.draw_arrays( GL::POINTS, 0, 1 );
    }

    // draw player
    // transform player pos to screen space
    let posx = player_pos[ 0 ] / MAP_SIDE as f32 - 1.0;
    let posy = 1.0 - player_pos[ 1 ] / MAP_SIDE as f32 * 2.0;
    let player_pos = [ posx, posy ];
    gl.vertex_attrib2fv_with_f32_array( 0, &player_pos );
    gl.vertex_attrib1f( 1, 8.0 );
    gl.vertex_attrib3f( 2, 1.0, 0.5, 0.0 );
    gl.draw_arrays( GL::POINTS, 0, 1 );

    // draw direction line
    // determine start and end of line in screen space
    let line_start = player_pos;
    let line_end = [ end[ 0 ] / MAP_SIDE as f32 - 1.0, 1.0 - end[ 1 ] / MAP_SIDE as f32 * 2.0 ];
    gl.use_program( Some( &line_shader ) );
    gl.vertex_attrib2fv_with_f32_array( 0, &line_start );
    gl.vertex_attrib2fv_with_f32_array( 1, &line_end );
    gl.vertex_attrib3f( 2, 1.0, 0.5, 0.0 );
    gl.draw_arrays( GL::LINES, 0, 2 );


    true
  };

  gl::exec_loop::run( loop_ );
}

fn direction( angle : f32 ) -> [ f32; 2 ]
{
  [
    angle.cos(),
    angle.sin(),
  ]
}

fn cast_rays( start : &[ f32; 2 ], angle : f32 ) -> [ f32; 2 ]
{
  let direction = direction( angle );
  let scale_x = ( 1.0 + ( direction[ 1 ] / direction[ 0 ] ).powi( 2 ) ).sqrt();
  let scale_y = ( 1.0 + ( direction[ 0 ] / direction[ 1 ] ).powi( 2 ) ).sqrt();
  let mut accum_x = 0.0;
  let mut accum_y = 0.0;

  let step_x = if direction[ 0 ] < 0.0
  {
    accum_x += start[ 0 ].fract() * scale_x;
    -1.0
  }
  else
  {
    accum_x += ( 1.0 - start[ 0 ].fract() ) * scale_x;
    1.0
  };
  let step_y = if direction[ 1 ] < 0.0
  {
    accum_y += ( 1.0 - start[ 1 ].fract() ) * scale_y;
    1.0
  }
  else
  {
    accum_y += start[ 1 ].fract() * scale_y;
    -1.0
  };

  for _ in 0..24
  {
    if accum_x < accum_y
    {
      let pos = [ start[ 0 ] + direction[ 0 ] * accum_x, start[ 1 ] + -direction[ 1 ] * accum_x ];
      let col = pos[ 0 ].trunc() as usize;
      let row = pos[ 1 ].trunc() as usize;
      let id = row * MAP_SIDE + col;
      if MAP[ id ] == 0
      {
        break;
      }
    }
    else
    {
      let pos = [ start[ 0 ] + direction[ 0 ] * accum_y, start[ 1 ] + -direction[ 1 ] * accum_y ];
      let col = pos[ 0 ].trunc() as usize;
      let row = pos[ 1 ].trunc() as usize;
      let id = row * MAP_SIDE + col;
    }
  }
  todo!()
  // end
  // let rays_count = 1;
  // for _ in 0..rays_count
  // {

  // }
}

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

const PI2 : f32 = consts::PI * 2.;
const CELL_SIZE : f32 = 64.;

const MAP_SIDE : usize = 8;
const MAP : [ u8; MAP_SIDE * MAP_SIDE ] =
[
  1, 1, 1, 1, 1, 1, 1, 1,
  1, 0, 0, 0, 0, 0, 0, 1,
  1, 1, 0, 0, 0, 0, 0, 1,
  1, 0, 0, 0, 0, 1, 0, 1,
  1, 0, 1, 0, 0, 1, 0, 1,
  1, 0, 1, 0, 0, 1, 0, 1,
  1, 0, 0, 0, 0, 0, 0, 1,
  1, 1, 1, 1, 1, 1, 1, 1,
];

fn run()
{
  let gl = gl::context::retrieve_or_make().unwrap();
  gl.clear_color( 0.3, 0.3, 0.3, 1. );

  let point         = include_str!( "shaders/point.vert" );
  let line          = include_str!( "shaders/line.vert" );
  let slice         = include_str!( "shaders/slice.vert" );
  let fragment      = include_str!( "shaders/main.frag" );
  let point_shader  = gl::ProgramFromSources::new( point, fragment ).compile_and_link( &gl ).unwrap();
  let line_shader   = gl::ProgramFromSources::new( line, fragment ).compile_and_link( &gl ).unwrap();
  let slice_shader  = gl::ProgramFromSources::new( slice, fragment ).compile_and_link( &gl ).unwrap();

  let controls = Controls::setup();
  let rotation_velocity = 2.5;
  let move_velocity = 1.3;
  let mut player_pos = [ 3., 3. ];
  let mut angle = 0.;
  let ray_count = 120;
  let fov = 60.;
  let mut last_time = 0.;

  gl.clear( GL::COLOR_BUFFER_BIT );

  let loop_ = move | time |
  {
    let time = ( time / 1000. ) as f32;
    let delta_time = time - last_time;
    last_time = time;

    angle += rotation_velocity * delta_time * controls.borrow().as_vec()[ 1 ];
    angle = wrap_angle( angle );

    let move_dir = controls.borrow().as_vec()[ 0 ];

    let move_dir = match move_dir
    {
      1.0 =>
      {
        let RayCollision { len, .. } = cast_ray( &player_pos, angle );
        if len > 0.1 { 1.0 } else { 0.0 }
      }
      -1.0 =>
      {
        let angle = wrap_angle( consts::PI + angle );
        let RayCollision { len, .. } = cast_ray( &player_pos, angle );
        if len > 0.2 { -1.0 } else { 0.0 }
      }
      _ => 0.0
    };

    let dir = direction( angle );
    player_pos[ 0 ] += move_velocity * dir[ 0 ] * delta_time * move_dir;
    player_pos[ 1 ] += move_velocity * dir[ 1 ] * delta_time * move_dir;

    gl.clear( GL::COLOR_BUFFER_BIT );

    gl.use_program( Some( &point_shader ) );

    // draw map
    for ( i, item ) in MAP.iter().enumerate()
    {
      let col = ( i % MAP_SIDE ) as f32;
      let row = ( i / MAP_SIDE ) as f32;

      let color = if *item == 1
      {
        [ 1., 1., 1. ]
      }
      else
      {
        [ 0., 0., 0. ]
      };

      let posx = ( -512. + CELL_SIZE * ( col + 0.5 ) ) / 512.;
      let posy = ( 256. - CELL_SIZE * ( row + 0.5 ) ) / 256.;

      gl.vertex_attrib2fv_with_f32_array( 0, &[ posx, posy ] );
      gl.vertex_attrib1f( 1, CELL_SIZE - 1. );
      gl.vertex_attrib3fv_with_f32_array( 2, &color );
      gl.draw_arrays( GL::POINTS, 0, 1 );
    }

    // draw player
    // transform player pos to screen space
    let posx = player_pos[ 0 ] / MAP_SIDE as f32 - 1.;
    let posy = 1. - player_pos[ 1 ] / MAP_SIDE as f32 * 2.;
    let player_pos_screen_space = [ posx, posy ];
    gl.vertex_attrib2fv_with_f32_array( 0, &player_pos_screen_space );
    gl.vertex_attrib1f( 1, 8. );
    gl.vertex_attrib3f( 2, 1., 0.5, 0. );
    gl.draw_arrays( GL::POINTS, 0, 1 );

    for i in 0..ray_count
    {
      let ray_angle = angle + ( i as f32 * fov / ( ray_count - 1 ) as f32 ).to_radians() - ( fov / 2. ).to_radians();
      let ray_angle = wrap_angle( ray_angle );
      let RayCollision { pos, len, .. } = cast_ray( &player_pos, ray_angle );
      let len = len * ( ray_angle - angle ).cos();
      let end = pos;
      let line_start = player_pos_screen_space;
      let line_end =
      [
        end[ 0 ] / MAP_SIDE as f32 - 1.,
        1. - end[ 1 ] / MAP_SIDE as f32 * 2.
      ];

      gl.use_program( Some( &line_shader ) );
      gl.vertex_attrib2fv_with_f32_array( 0, &line_start );
      gl.vertex_attrib2fv_with_f32_array( 1, &line_end );
      gl.vertex_attrib3f( 2, 0.75, 0.75, 0. );
      gl.draw_arrays( GL::LINES, 0, 2 );

      let color = [ 0.8, 0.7, 0.6 ];

      gl.use_program( Some( &slice_shader ) );
      gl.vertex_attrib1f( 0, 1. / len );
      gl.vertex_attrib1f( 1, 1. / ray_count as f32 );
      gl.vertex_attrib1f( 2, i as f32 );
      gl.vertex_attrib3fv_with_f32_array( 3, &color );
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
    }

    true
  };

  gl::exec_loop::run( loop_ );
}


// wrap angle between 0 and 2PI
fn wrap_angle( val : f32 ) -> f32
{
  match val < 0.0
  {
    true => PI2 + val % PI2,
    false => val % PI2
  }
}

fn direction( angle : f32 ) -> [ f32; 2 ]
{
  [
    angle.cos(),
    -angle.sin(),
  ]
}

fn cast_ray( start : &[ f32; 2 ], angle : f32 ) -> RayCollision
{
  // positive y direction goes down the map, so y component is actually inverted
  let direction = direction( angle );

  // length of the vector if step along x and y axes respectively by 1 unit
  let length_x = ( 1.0 + ( direction[ 1 ] / direction[ 0 ] ).powi( 2 ) ).sqrt();
  let length_y = ( 1.0 + ( direction[ 0 ] / direction[ 1 ] ).powi( 2 ) ).sqrt();

  // accumulating length of vector
  let mut accum_x = if direction[ 0 ] < 0.0
  {
    start[ 0 ].fract() * length_x
  }
  else
  {
    ( 1.0 - start[ 0 ].fract() ) * length_x
  };
  let mut accum_y = if direction[ 1 ] < 0.0
  {
    start[ 1 ].fract() * length_y
  }
  else
  {
    ( 1.0 - start[ 1 ].fract() ) * length_y
  };

  let step_x = if direction[ 0 ] < 0.0 { -1 } else { 1 };
  let step_y = if direction[ 1 ] < 0.0 { -1 } else { 1 };
  let mut col = start[ 0 ] as i32;
  let mut row = start[ 1 ] as i32;

  loop
  {
    let ( intersect_pos, len ) = match accum_x < accum_y
    {
      true =>
      {
        let intersect_pos =
        [
          start[ 0 ] + direction[ 0 ] * accum_x,
          start[ 1 ] + direction[ 1 ] * accum_x
        ];
        let len = accum_x;
        accum_x += length_x;
        col += step_x;

        ( intersect_pos, len )
      }
      false =>
      {
        let intersect_pos =
        [
          start[ 0 ] + direction[ 0 ] * accum_y,
          start[ 1 ] + direction[ 1 ] * accum_y
        ];
        let len = accum_y;
        accum_y += length_y;
        row += step_y;

        ( intersect_pos, len )
      }
    };

    // dont go out of bounds
    if row < 0 || col < 0
    || row as usize >= MAP_SIDE
    || col as usize >= MAP_SIDE
    {
      break RayCollision { len, pos: intersect_pos, _tile_index: None };
    }

    // map check
    let row = row as usize;
    let col = col as usize;
    let index = row * MAP_SIDE + col;

    if MAP[ index ] == 1
    {
      break RayCollision { len, pos: intersect_pos, _tile_index: Some( index ) };
    }
  }
}

struct RayCollision
{
  len : f32,
  pos : [ f32; 2 ],
  _tile_index : Option< usize >,
}

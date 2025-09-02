#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_sign_loss ) ]

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

// screen width in pixels
const WIDTH : f32 = 1024.0;
// screen height in pixels
const HEIGHT : f32 = 512.0;
// size of a tile in pixels
const CELL_SIZE : f32 = 64.;

const PI2 : f32 = consts::PI * 2.;

const MAP_SIDE : usize = 8;
// 1 means wall, 0 means empty
const MAP : [ u8; MAP_SIDE * MAP_SIDE ] =
[
  // x positive →
  1, 1, 1, 1, 1, 1, 1, 1, // y positive
  1, 0, 0, 0, 0, 0, 0, 1, // ↓
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

  // qqq : purpose of each shader? what is drawn by what?
  // qqq : more documentatiion overall please

  // shader for drawing points
  let point_shader  = gl::ProgramFromSources::new( point, fragment ).compile_and_link( &gl ).unwrap();
  // shader for drawing lines
  let line_shader   = gl::ProgramFromSources::new( line, fragment ).compile_and_link( &gl ).unwrap();
  // shader for drawing vertical slices, that are basically scaled quads
  let slice_shader  = gl::ProgramFromSources::new( slice, fragment ).compile_and_link( &gl ).unwrap();

  let controls = Controls::setup();
  let rotation_velocity = 2.5;
  let move_velocity = 1.3;
  let mut player_pos = [ 3., 3. ];
  let mut angle = 0.2;
  // amount of rays casted
  // should be even
  let ray_count = 40;
  // field of view
  let fov = 60.;
  let mut last_time = 0.;

  let map_vao = map_vao( &gl );
  let mut rays = Vec::with_capacity( ray_count );

  let loop_ = move | time |
  {
    let time = ( time / 1000. ) as f32;
    let delta_time = time - last_time;
    last_time = time;

    // rotate based on pressed keys
    // if left key pressed then rotation is counter-clockwise
    // if right - then clockwise
    // if none is pressed then rotation is 0
    angle += rotation_velocity * delta_time * controls.borrow().rotation_direction();
    angle = wrap_angle( angle );

    // 1 is forward, -1 is backward
    let move_dir = controls.borrow().move_direction();

    // assure that player doesn't go beyond walls
    // restrict movement depending on how close is an obstacle
    // for both forward movement and backward movement
    let move_dir = match move_dir
    {
      1.0 =>
      {
        // throw ray forward and check distance to an obstacle
        let RayCollision { len, .. } = cast_ray( &player_pos, angle );
        // if an obstacle it too close then the movement is 0
        if len > 0.1 { 1.0 } else { 0.0 }
      }
      -1.0 =>
      {
        // thow ray backward and check distance to an obstacle
        let angle = wrap_angle( consts::PI + angle );
        let RayCollision { len, .. } = cast_ray( &player_pos, angle );
        if len > 0.1 { -1.0 } else { 0.0 }
      }
      _ => 0.0
    };

    // this is the direction vector where the player is facing
    let dir = direction( angle );
    player_pos[ 0 ] += move_velocity * dir[ 0 ] * delta_time * move_dir;
    player_pos[ 1 ] += move_velocity * dir[ 1 ] * delta_time * move_dir;

    // calculate player position in screen space
    // player position is constrained by the map
    // which is 8x8 tiles so palyer position is somewhere
    // inside this grid. we normalize player position
    // with map size len which is 8 and then move x coordinate
    // to left so it is on the left half of the screen
    let posx = player_pos[ 0 ] / MAP_SIDE as f32 - 1.;
    // y coodinate should be flipped because map's y positive
    // direction is downwards
    let posy = 1. - player_pos[ 1 ] / MAP_SIDE as f32 * 2.;
    let player_pos_screen_space = [ posx, posy ];

    // do raycasting
    rays.clear();
    for i in 0..ray_count
    {
      // this calculates a ray angle for every ray in field of view

      // step by which ray angle is increased
      let step = fov / ( ray_count - 1 ) as f32;
      // angle for current ray
      let ray_angle = ( i as f32 * step ).to_radians();
      // adjust ray angle to player angle and shift by half of the field of view
      let ray_angle = angle + ray_angle - ( fov / 2. ).to_radians();
      let ray_angle = wrap_angle( ray_angle );
      let RayCollision { pos, len } = cast_ray( &player_pos, ray_angle );

      // adjust len to remove fish-eye effect
      let len = len * ( ray_angle - angle ).cos();
      let line_start = player_pos_screen_space;
      // same as player position, this is converted to
      // screen space and shifted to left half of the screen
      let line_end =
      [
        pos[ 0 ] / MAP_SIDE as f32 - 1.,
        1. - pos[ 1 ] / MAP_SIDE as f32 * 2.
      ];
      rays.push( ( line_start, line_end, len ) );
    }

    gl.clear( GL::COLOR_BUFFER_BIT );

    gl.use_program( Some( &point_shader ) );

    // draw the map
    gl.bind_vertex_array( Some( &map_vao ) );
    gl.vertex_attrib1f( 1, CELL_SIZE - 1. );
    gl.draw_arrays( GL::POINTS, 0, MAP.len() as i32 );
    gl.bind_vertex_array( None );

    // draw player on the map
    // just draws a point of some color
    gl.vertex_attrib2fv_with_f32_array( 0, &player_pos_screen_space );
    gl.vertex_attrib1f( 1, 8. );
    gl.vertex_attrib3f( 2, 1., 0.5, 0. );
    gl.draw_arrays( GL::POINTS, 0, 1 );

    for ( i, ( start, end, len ) ) in rays.iter().enumerate()
    {
      // draw rays on the map
      // just draws a line of some color
      gl.use_program( Some( &line_shader ) );
      gl.vertex_attrib2fv_with_f32_array( 0, start );
      gl.vertex_attrib2fv_with_f32_array( 1, end );
      gl.vertex_attrib3f( 2, 0.75, 0.75, 0. );
      gl.draw_arrays( GL::LINES, 0, 2 );

      // draw geometry
      gl.use_program( Some( &slice_shader ) );
      // every ray corresponds to a vertical slice
      // on the screen, so every slice is scaled by
      // the ray length
      gl.vertex_attrib1f( 0, 1. / len );
      // amount of rays determines slice width
      gl.vertex_attrib1f( 1, 1. / ray_count as f32 );
      // index determines horizontal position of the slice
      gl.vertex_attrib1f( 2, i as f32 );
      // slice's color
      gl.vertex_attrib3fv_with_f32_array( 3, &[ 0.8, 0.7, 0.6 ] );
      // slice is just a quad
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
    }

    true
  };

  gl::exec_loop::run( loop_ );
}

fn map_vao( gl : &GL ) -> gl::WebGlVertexArrayObject
{
  // bakes tile data into vao to draw entire map
  // with one draw call
  let mut data = Vec::new();
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

    // screen-space coordinates of a tile
    // shifted to the left part of the screen
    let posx = ( -WIDTH / 2. + CELL_SIZE * ( col + 0.5 ) ) / ( WIDTH / 2. );
    let posy = ( HEIGHT / 2. - CELL_SIZE * ( row + 0.5 ) ) / ( HEIGHT / 2. );

    data.push( posx );
    data.push( posy );
    data.push( color[ 0 ] );
    data.push( color[ 1 ] );
    data.push( color[ 2 ] );
  }

  let buf = gl::buffer::create( gl ).unwrap();
  gl::upload( gl, &buf, data.as_slice(), GL::STATIC_DRAW );

  let vao = gl::vao::create( gl ).unwrap();
  gl.bind_vertex_array( Some( &vao ) );

  gl::BufferDescriptor::new::< [ f32; 2 ] >()
  .offset( 0 )
  .stride( 5 )
  .attribute_pointer( gl, 0, &buf )
  .unwrap();
  gl::BufferDescriptor::new::< [ f32; 3 ] >()
  .offset( 2 )
  .stride( 5 )
  .attribute_pointer( gl, 2, &buf )
  .unwrap();

  gl.bind_vertex_array( None );

  vao
}

// algorithm explanation - https://www.youtube.com/watch?v=NbSee-XM7WA&t=1574s&ab_channel=javidx9
fn cast_ray( start : &[ f32; 2 ], angle : f32 ) -> RayCollision
{
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
    let ( intersect_pos, len ) = if accum_x < accum_y
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
    else
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
    };

    // dont go out of bounds
    if row < 0 || col < 0
    || row as usize >= MAP_SIDE
    || col as usize >= MAP_SIDE
    {
      break RayCollision { len, pos : intersect_pos };
    }

    // map check
    let row = row as usize;
    let col = col as usize;
    let index = row * MAP_SIDE + col;

    if MAP[ index ] == 1
    {
      break RayCollision { len, pos : intersect_pos };
    }
  }
}

fn direction( angle : f32 ) -> [ f32; 2 ]
{
  // here's y component is inverted because y axis positive direction is downwards on the map
  [
    angle.cos(),
    -angle.sin(),
  ]
}

// wrap angle between 0 and 2PI
fn wrap_angle( val : f32 ) -> f32
{
  if val < 0.0
  {
    PI2 + val % PI2
  }
  else
  {
    val % PI2
  }
}

struct RayCollision
{
  len : f32,
  pos : [ f32; 2 ],
}

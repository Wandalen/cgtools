//! Raycaster example — casts rays against scene geometry for collision and interaction with WebGL2.

mod controls;
mod sim;

use controls::Controls;
use minwebgl as gl;
use gl::GL;
use sim::{ MAP, MAP_SIDE, RayCollision, angle_wrap, frame_dt_clamp, move_dir_resolve, player_step, ray_cast };

/// One map tile's position/color record — `stride( 5 )` in `map_vao` covers all 5 `f32`
/// fields, matching this struct's own ( `repr( C )`, no padding ) byte layout.
#[ repr( C ) ]
#[ derive( Debug, Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
struct Vertex
{
  position : [ f32; 2 ],
  color : [ f32; 3 ],
}

impl mingl::Attribute for Vertex
{
  fn describe() -> Vec< mingl::VertexAttribute >
  {
    vec!
    [
      mingl::VertexAttribute::new( 0, mingl::VectorDataType::new( mingl::DataType::F32, 2, 1 ), 0 ),
      mingl::VertexAttribute::new( 2, mingl::VectorDataType::new( mingl::DataType::F32, 3, 1 ), 2 ),
    ]
  }
}

fn main()
{
  gl::browser::setup( gl::browser::Config::default() );
  app_run();
}

// screen width in pixels
const WIDTH : f32 = 1024.0;
// screen height in pixels
const HEIGHT : f32 = 512.0;
// size of a tile in pixels
const CELL_SIZE : f32 = 64.;

fn app_run()
{
  let gl = gl::context::retrieve_or_make().unwrap();
  gl.clear_color( 0.3, 0.3, 0.3, 1. );

  let point         = include_str!( "shaders/point.vert" );
  let line          = include_str!( "shaders/line.vert" );
  let slice         = include_str!( "shaders/slice.vert" );
  let fragment      = include_str!( "shaders/main.frag" );

  // All three share the same flat-color fragment shader and differ only in
  // vertex layout/topology, matching what each draws below: point_shader
  // draws the minimap tile dots and player dot (GL::POINTS), line_shader
  // draws each ray on the minimap (GL::LINES), slice_shader draws the
  // pseudo-3D wall column per ray — the actual raycast view (GL::TRIANGLE_STRIP).

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
    controls.borrow_mut().state_update();

    let time = ( time / 1000. ) as f32;
    // Fix(BUG-522): clamp the real per-frame delta via `sim::frame_dt_clamp` before it's used
    // to step rotation or position.
    // Root cause: `time` is `requestAnimationFrame`'s raw timestamp (`mingl::web::exec_loop::
    // run` applies no smoothing), so a stalled frame (tab backgrounded, GC pause) reports an
    // arbitrarily large `delta_time`; `move_dir_resolve` below only proves a wall is farther
    // than `WALL_CLEARANCE` away *at the start of the frame*, so an unclamped `delta_time` let
    // `player_step` move the player past that clearance and tunnel straight through a wall.
    // Pitfall: raising `move_velocity` or lowering `WALL_CLEARANCE`/`MAX_DT` without re-checking
    // `MAX_DT * move_velocity < WALL_CLEARANCE` (see `sim::MAX_DT`'s doc comment) would silently
    // reopen this — the invariant isn't enforced by the type system, only by the constants.
    let delta_time = frame_dt_clamp( time - last_time );
    last_time = time;

    // rotate based on pressed keys
    // if left key pressed then rotation is counter-clockwise
    // if right - then clockwise
    // if none is pressed then rotation is 0
    angle += rotation_velocity * delta_time * controls.borrow().rotation_direction();
    angle = angle_wrap( angle );

    // 1 is forward, -1 is backward
    let move_dir = controls.borrow().move_direction();

    // assure that player doesn't go beyond walls; restrict movement depending on how close is
    // an obstacle for both forward and backward movement — see `sim::move_dir_resolve`.
    let move_dir = move_dir_resolve( player_pos, angle, move_dir );

    player_pos = player_step( player_pos, angle, move_velocity, delta_time, move_dir );

    // calculate player position in screen space
    // player position is constrained by the map
    // which is 8x8 tiles so palyer position is somewhere
    // inside this grid. we normalize player position
    // with map size len which is 8 and then move x coordinate
    // to left so it is on the left half of the screen
    let pos_x = player_pos[ 0 ] / MAP_SIDE as f32 - 1.;
    // y coodinate should be flipped because map's y positive
    // direction is downwards
    let pos_y = 1. - player_pos[ 1 ] / MAP_SIDE as f32 * 2.;
    let player_pos_screen_space = [ pos_x, pos_y ];

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
      let ray_angle = angle_wrap( ray_angle );
      let RayCollision { pos, len } = ray_cast( player_pos, ray_angle );

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
    let pos_x = ( -WIDTH / 2. + CELL_SIZE * ( col + 0.5 ) ) / ( WIDTH / 2. );
    let pos_y = ( HEIGHT / 2. - CELL_SIZE * ( row + 0.5 ) ) / ( HEIGHT / 2. );

    data.push( Vertex { position : [ pos_x, pos_y ], color } );
  }

  let buf = gl::buffer::create( gl ).unwrap();
  // Fully qualified because `minwebgl`'s `buffer` and `index` layers both
  // expose an `upload` fn, making the crate-root glob-imported `gl::upload`
  // ambiguous (E0659). This buffer is used as a vertex attribute source via
  // `attribute_pointer` below, so `buffer::upload` (ARRAY_BUFFER) is correct.
  gl::buffer::upload( gl, &buf, data.as_slice(), GL::STATIC_DRAW );

  let vao = gl::vao::create( gl ).unwrap();
  gl.bind_vertex_array( Some( &vao ) );

  let vertex_layout = mingl::VertexBufferLayout::from_attribute::< Vertex >( 5 );
  gl::vertex_buffer_layout_bind( gl, &buf, &vertex_layout ).unwrap();

  gl.bind_vertex_array( None );

  vao
}

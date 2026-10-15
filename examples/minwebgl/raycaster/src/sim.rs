//! Pure ( no WebGL / no browser ) raycasting and movement simulation logic, kept in its own
//! module — rather than inline in `main.rs`'s render-loop closure — specifically so it can be
//! exercised from `tests/` without a browser or GPU context. `main.rs` is the only caller;
//! `tests/wall_tunnel_test.rs` is the only other one, included via `#[ path ]` since this crate
//! has no `[lib]` target ( see that test file's own doc comment for why ).

use std::f32::consts;

pub const MAP_SIDE : usize = 8;
// 1 means wall, 0 means empty
pub const MAP : [ u8; MAP_SIDE * MAP_SIDE ] =
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

const PI2 : f32 = consts::PI * 2.;

/// Minimum ray length ( arena units ) `move_dir_resolve` requires before it treats a wall as
/// "too close" and blocks movement in that direction entirely.
pub const WALL_CLEARANCE : f32 = 0.1;

/// Real per-frame delta time — `main.rs`'s `time` parameter comes straight from
/// `requestAnimationFrame`'s raw timestamp ( `mingl::web::exec_loop::run`, no smoothing or
/// clamping of its own ) — is clamped to this ceiling before it reaches `player_step`, so a
/// stall ( tab backgrounded, GC pause, slow frame ) produces one slow-motion frame instead of a
/// large position jump. `move_dir_resolve` only proves a wall is farther than `WALL_CLEARANCE`
/// units away *at the start of the frame*; it does not limit how far `player_step` actually
/// moves the player once movement is allowed, so an unclamped `dt` can cover more ground than
/// `WALL_CLEARANCE` in one step and tunnel straight through a wall ( walls are one tile, i.e.
/// `1.0` unit, thick ). Chosen so `MAX_DT * move_velocity < WALL_CLEARANCE` holds for `main.rs`'s
/// own `move_velocity` ( `1.3` ): `0.05 * 1.3 == 0.065 < 0.1`, so a clamped step can never exceed
/// the clearance that was just proven to exist. See `tests/wall_tunnel_test.rs`'s
/// `bug_reproducer` for the failure this prevents, and `flecs_bouncing_circles`' own `MAX_DT`
/// for the same guard against the same failure mode in a different example.
pub const MAX_DT : f32 = 0.05;

/// Clamps a raw per-frame delta time ( `main.rs`'s `time - last_time`, straight from
/// `requestAnimationFrame` ) to `MAX_DT`. The one function whose output actually reaches
/// `player_step`'s `dt` parameter in `main.rs` — see `MAX_DT`'s doc comment for the safety
/// invariant this enforces.
pub fn frame_dt_clamp( raw_dt : f32 ) -> f32
{
  raw_dt.min( MAX_DT )
}

pub struct RayCollision
{
  pub len : f32,
  pub pos : [ f32; 2 ],
}

// algorithm explanation - https://www.youtube.com/watch?v=NbSee-XM7WA&t=1574s&ab_channel=javidx9
pub fn ray_cast( start : [ f32; 2 ], angle : f32 ) -> RayCollision
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

pub fn direction( angle : f32 ) -> [ f32; 2 ]
{
  // here's y component is inverted because y axis positive direction is downwards on the map
  [
    angle.cos(),
    -angle.sin(),
  ]
}

// wrap angle between 0 and 2PI
pub fn angle_wrap( val : f32 ) -> f32
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

/// Resolves the forward ( `1.0` ) / backward ( `-1.0` ) / none ( `0.0` ) movement actually
/// applied this frame: casts a clearance ray from `pos` in the requested direction and blocks
/// the step entirely if a wall is within `WALL_CLEARANCE` units. Does **not** limit the *size*
/// of the step once allowed — pairing this with an unclamped `dt` in `player_step` is what
/// `bug_reproducer(BUG-522)` in `tests/wall_tunnel_test.rs` demonstrates; see `MAX_DT`'s doc
/// comment for why clamping `dt` before it reaches `player_step` is what actually closes it.
pub fn move_dir_resolve( pos : [ f32; 2 ], angle : f32, raw_move_dir : f32 ) -> f32
{
  match raw_move_dir
  {
    1.0 =>
    {
      // throw ray forward and check distance to an obstacle
      let RayCollision { len, .. } = ray_cast( pos, angle );
      // if an obstacle it too close then the movement is 0
      if len > WALL_CLEARANCE { 1.0 } else { 0.0 }
    }
    -1.0 =>
    {
      // throw ray backward and check distance to an obstacle
      let angle = angle_wrap( consts::PI + angle );
      let RayCollision { len, .. } = ray_cast( pos, angle );
      if len > WALL_CLEARANCE { -1.0 } else { 0.0 }
    }
    _ => 0.0
  }
}

/// Advances `pos` by one frame's movement: `move_velocity` along the facing direction, scaled
/// by `dt` and `move_dir` ( see `move_dir_resolve` ). `dt` must already be clamped by the
/// caller — see `MAX_DT` — or the step can tunnel through a wall `move_dir_resolve` just
/// proved was merely `> WALL_CLEARANCE` away, not arbitrarily far away.
pub fn player_step( pos : [ f32; 2 ], angle : f32, move_velocity : f32, dt : f32, move_dir : f32 ) -> [ f32; 2 ]
{
  let dir = direction( angle );
  [
    pos[ 0 ] + move_velocity * dir[ 0 ] * dt * move_dir,
    pos[ 1 ] + move_velocity * dir[ 1 ] * dt * move_dir,
  ]
}

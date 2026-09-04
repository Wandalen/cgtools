//! Regression tests for `raycaster`'s pure simulation logic ( `src/sim.rs` ), extracted from
//! `src/main.rs`'s render-loop closure specifically so it can run natively -- no browser/GPU
//! context required. `raycaster` is a binary-only crate ( no `[lib]` target ), so this file
//! includes `sim.rs` directly via `#[ path ]` rather than depending on the crate as a library --
//! the same pattern `examples/minwebgl/hexagonal_grid/tests/basic.rs` uses for its own
//! binary-only crate. `sim.rs` declares no nested `mod`s of its own, so this inclusion is
//! unambiguous.

#[ path = "../src/sim.rs" ]
mod sim;

use sim::{ MAP, MAP_SIDE, MAX_DT, RayCollision, WALL_CLEARANCE, frame_dt_clamp, move_dir_resolve, player_step, ray_cast };

/// Is the arena position `pos` inside a wall cell ( or out of bounds, treated the same way
/// `ray_cast`'s own bounds check does )?
fn is_wall( pos : [ f32; 2 ] ) -> bool
{
  let col = pos[ 0 ] as usize;
  let row = pos[ 1 ] as usize;
  if row >= MAP_SIDE || col >= MAP_SIDE
  {
    return true;
  }
  MAP[ row * MAP_SIDE + col ] == 1
}

/// ## Root Cause
/// `main.rs`'s render loop computed `delta_time` directly from `requestAnimationFrame`'s raw
/// timestamp with no ceiling ( `mingl::web::exec_loop::run` applies no smoothing of its own ), so
/// a single stalled frame ( tab backgrounded, GC pause, slow first frame ) could report an
/// arbitrarily large `delta_time`. `move_dir_resolve` casts one clearance ray and permits
/// movement whenever the wall is farther than `WALL_CLEARANCE` ( `0.1` ) units away -- a fixed
/// distance check entirely independent of how far the upcoming step will actually move the
/// player. `player_step` then multiplies that unclamped `delta_time` straight into the position
/// update, so a large enough `delta_time` overshoots clean through a wall the clearance check had
/// just certified as "far enough away."
///
/// ## Why Not Caught
/// Every manual playtest runs at a steady ~60fps, where `delta_time` never exceeds a few
/// milliseconds and the failure mode never triggers -- nothing in the render loop or its
/// dependencies asserted an upper bound on `delta_time` before this fix, and no test exercised a
/// stalled/backgrounded-tab frame.
///
/// ## Fix Applied
/// Added `sim::frame_dt_clamp`, called on every frame's raw `time - last_time` in `main.rs`
/// before it reaches rotation or `player_step`, ceiling it to `sim::MAX_DT` ( `0.05` ). Chosen so
/// `MAX_DT * move_velocity < WALL_CLEARANCE` ( `0.05 * 1.3 == 0.065 < 0.1` ) always holds for
/// `main.rs`'s own `move_velocity`, so a clamped step can never exceed the clearance
/// `move_dir_resolve` already proved exists.
///
/// ## Prevention
/// This test drives the exact numeric scenario a stalled frame produces -- a player well clear of
/// a wall ( `move_dir_resolve` permits movement ), paired with a large raw `dt` -- first
/// unclamped ( proving the tunnel is real ), then through `frame_dt_clamp` ( proving the fix
/// closes it ). A future change to `MAX_DT`, `move_velocity`, or `WALL_CLEARANCE` that reopens
/// the `MAX_DT * move_velocity < WALL_CLEARANCE` invariant fails this test immediately.
///
/// ## Pitfall
/// `move_dir_resolve`'s clearance check only proves a wall is farther than `WALL_CLEARANCE` away
/// *before* the step -- it says nothing about the step's own size. Any future movement code that
/// consumes raw, unclamped frame time the same way will reintroduce this exact failure mode under
/// the same conditions ( stalled frame + any nonzero clearance check ), regardless of how small
/// `WALL_CLEARANCE` is set.
#[ test ]
#[ allow
(
  clippy::float_cmp,
  reason = "move_dir and clamped_dt are exact pass-throughs -- a hardcoded literal returned \
            unmodified by move_dir_resolve's match arms, and f32::min selecting one of its two \
            inputs unchanged -- with no arithmetic applied, so exact equality is the correct \
            check, not an approximation"
) ]
fn bug_reproducer_bug_522_stalled_frame_tunnels_through_wall()
{
  // Player well clear of the wall at row 3 col 5 ( `MAP[ 29 ] == 1` ) -- facing due east,
  // `move_dir_resolve` must permit movement since the ray hits the wall's near face 1.5 units
  // away, far past `WALL_CLEARANCE`.
  let pos = [ 3.5_f32, 3.5_f32 ];
  let angle = 0.0_f32; // facing +x ( east ), matches `sim::direction`'s convention
  let move_velocity = 1.3_f32; // main.rs's own constant
  assert_eq!( MAP[ 3 * MAP_SIDE + 5 ], 1, "sanity: row 3 col 5 must be a wall for this scenario" );
  assert!( !is_wall( pos ), "sanity: player start position must not itself be inside a wall" );

  // Direct sanity check on the underlying `ray_cast` this scenario depends on -- epsilon-bounded
  // since, unlike `move_dir`/`clamped_dt` below, `pos`/`len` are genuine arithmetic results
  // ( `start + direction * accum` ), not a pass-through of an unmodified input.
  let RayCollision { pos : hit_pos, len : hit_len } = ray_cast( pos, angle );
  assert!( ( hit_len - 1.5 ).abs() < 1e-4, "sanity: ray_cast len should be ~1.5, got {hit_len}" );
  assert!
  (
    ( hit_pos[ 0 ] - 5.0 ).abs() < 1e-4 && ( hit_pos[ 1 ] - 3.5 ).abs() < 1e-4,
    "sanity: ray_cast should hit the wall's near face at [ 5.0, 3.5 ], got {hit_pos:?}"
  );

  let move_dir = move_dir_resolve( pos, angle, 1.0 );
  assert_eq!( move_dir, 1.0, "sanity: 1.5 units of clearance must permit forward movement" );

  // A stalled frame ( tab backgrounded, GC pause ) -- 1.6s between frames, ~100x a steady 60fps
  // frame's ~0.0167s.
  let raw_dt = 1.6_f32;

  // Pre-fix behavior: an unclamped dt fed straight into `player_step` tunnels through the wall.
  // This documents *why* the fix is needed and always holds, independent of `frame_dt_clamp`'s
  // own state -- it calls `player_step` directly, bypassing the clamp entirely.
  let unclamped_pos = player_step( pos, angle, move_velocity, raw_dt, move_dir );
  assert!
  (
    is_wall( unclamped_pos ),
    "MRE invariant broken: an unclamped dt of {raw_dt} from {pos:?} was expected to land inside \
     the wall at {unclamped_pos:?} -- if this no longer tunnels, the scenario itself ( map \
     layout, start position, dt ) needs updating, not the fix"
  );

  // Post-fix behavior: `frame_dt_clamp` ceilings the same raw dt to `MAX_DT` before it reaches
  // `player_step` -- this is the actual call sequence `main.rs`'s render loop now uses, and the
  // part of this test that actually regresses if the fix is reverted or weakened.
  let clamped_dt = frame_dt_clamp( raw_dt );
  assert_eq!( clamped_dt, MAX_DT, "sanity: a dt this large must be clamped down to MAX_DT" );

  let clamped_pos = player_step( pos, angle, move_velocity, clamped_dt, move_dir );
  assert!
  (
    !is_wall( clamped_pos ),
    "BUG-522 regression: clamped dt still landed inside a wall at {clamped_pos:?} -- the \
     MAX_DT * move_velocity < WALL_CLEARANCE invariant no longer holds"
  );

  let step_distance = ( clamped_pos[ 0 ] - pos[ 0 ] ).hypot( clamped_pos[ 1 ] - pos[ 1 ] );
  assert!
  (
    step_distance < WALL_CLEARANCE,
    "BUG-522 regression: a single clamped-dt step moved {step_distance} units, which is not \
     less than WALL_CLEARANCE ( {WALL_CLEARANCE} ) -- move_dir_resolve's clearance check no \
     longer covers the step player_step actually takes"
  );
}

/// Directly guards the constant relationship `sim::MAX_DT`'s doc comment states as the reason the
/// fix is sound, independent of any specific map/position scenario -- if a future change to
/// either constant violates it, this fails immediately rather than relying solely on the
/// scenario-specific reproducer above to notice.
#[ test ]
fn max_dt_times_move_velocity_stays_under_wall_clearance()
{
  let move_velocity = 1.3_f32; // main.rs's own constant
  assert!
  (
    MAX_DT * move_velocity < WALL_CLEARANCE,
    "MAX_DT ( {MAX_DT} ) * move_velocity ( {move_velocity} ) == {} must stay < WALL_CLEARANCE \
     ( {WALL_CLEARANCE} ), or a single clamped-dt step can tunnel through a wall move_dir_resolve \
     just certified as \"far enough away\" ( BUG-522 )",
    MAX_DT * move_velocity
  );
}

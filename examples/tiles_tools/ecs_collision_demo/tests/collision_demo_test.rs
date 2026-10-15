//! Integration test for the `ecs_collision_demo` binary's own console
//! output. This crate is bin-only (no lib target), so per the project's
//! bug-fixing workflow this test runs the compiled binary as a subprocess
//! and inspects its stdout, rather than unit-testing private `main.rs`
//! functions directly.

use std::process::Command;

/// ## Root Cause
/// `main()` captured the player's `Position` once, immediately after
/// `entities_spawn`, and passed that single stale snapshot to every
/// "of player" / "to player" spatial query in `spatial_queries_run` --
/// even though `collisions_detect_and_resolve` (called in between) moves
/// entities, including the player itself, via
/// `CollisionSystem::collisions_resolve`. Separately, `nearest_entity_find`
/// is a pure "nearest entity to a coordinate" primitive with no
/// self-exclusion parameter, so calling it with the player's own (live)
/// position always trivially returns the player itself at distance 0 -- a
/// vacuous, uninformative result for a section titled "Nearest Entity
/// Search" whose entire point is to find another entity relative to the
/// player.
///
/// ## Why Not Caught
/// The demo had no test coverage at all prior to this bug (bin-only crate,
/// nothing exercised `main()`'s own composition logic). Every individual
/// spatial-query primitive it calls (`circle_query`, `by_team_query`,
/// `nearest_entity_find`) is correct in isolation against whatever position
/// it's given -- the defect is purely in what `main.rs` passes those
/// primitives, which a purely-library-level test can never see.
///
/// ## Fix Applied
/// `entities_spawn` now returns the player's `hecs::Entity` handle instead
/// of a one-time `Position` snapshot. `spatial_queries_run` re-fetches the
/// player's live position from that handle at the top of the function (so
/// circle/team queries always reflect post-resolution reality), and the
/// nearest-entity search now explicitly excludes the player's own entity
/// before picking the minimum-distance candidate.
///
/// ## Prevention
/// This test runs the real compiled binary end-to-end and asserts the
/// "Nearest entity to player" section reports a genuinely different entity
/// than the player's own post-resolution entity/position, with the exact
/// hand-derived correct answer (entity `1v1`, position `(6, 3)`, distance
/// `1`) -- not just "not self", to also catch a fix that excludes self but
/// still computes from the stale position (which would produce an
/// ambiguous tie between two different entities at distance 3, never
/// landing on this unique distance-1 answer).
///
/// ## Pitfall
/// Re-introducing a one-time `Position` capture (instead of re-deriving
/// live position from a stored `Entity` handle) anywhere between a
/// mutating system call (like collision resolution) and a later
/// "current state" query silently reintroduces this exact class of bug --
/// it will not panic or warn, it will just silently query the wrong
/// location.
#[ test ]
fn bug_reproducer_bug_515_nearest_entity_search_uses_live_position_and_excludes_self()
{
  let output = Command::new( env!( "CARGO_BIN_EXE_ecs_collision_demo" ) )
    .output()
    .expect( "failed to run ecs_collision_demo binary" );
  assert!( output.status.success(), "binary must exit successfully" );
  let stdout = String::from_utf8_lossy( &output.stdout );

  // The player is entity `0v1` (first entity spawned); confirm its real
  // post-collision-resolution position, independently of the (possibly
  // buggy) "Nearest entity" section.
  let player_line = stdout.lines()
    .find( |l| l.starts_with( "Entity 0v1 now at" ) )
    .expect( "collision resolution must print entity 0v1's post-resolution position" );
  assert!(
    player_line.contains( "(5, 3)" ),
    "player (entity 0v1) must resolve to (5, 3) per the demo's own fixed collision setup -- got: {player_line}"
  );

  // The nearest-*other*-entity search must not just rediscover the player.
  let nearest_line = stdout.lines()
    .find( |l| l.starts_with( "Nearest entity to player:" ) )
    .expect( "must print a 'Nearest entity to player' line" );
  assert!(
    !nearest_line.contains( "0v1" ),
    "nearest entity to the player must not be the player's own entity -- self-match is a vacuous, zero-information result: {nearest_line}"
  );

  // Hand-derived unique correct answer using the player's LIVE position
  // (5, 3): entity `1v1` at (6, 3), Manhattan distance 1 -- uniquely
  // nearest (the next closest, `3v1` at (3, 4), is distance 3). Using the
  // stale pre-resolution position (5, 5) instead would tie two different
  // entities at distance 3, never landing on this unique answer.
  assert!( nearest_line.contains( "1v1" ), "expected nearest entity to be 1v1, got: {nearest_line}" );

  let position_line = stdout.lines()
    .skip_while( |l| !l.starts_with( "Nearest entity to player:" ) )
    .nth( 1 )
    .expect( "'Nearest entity' section must print a Position line" );
  assert!( position_line.contains( "(6, 3)" ), "expected nearest entity position (6, 3), got: {position_line}" );

  let distance_line = stdout.lines()
    .skip_while( |l| !l.starts_with( "Nearest entity to player:" ) )
    .nth( 2 )
    .expect( "'Nearest entity' section must print a Distance line" );
  assert!( distance_line.trim() == "Distance: 1", "expected Distance: 1, got: {distance_line}" );
}

//! Tests for the `game_of_life` library — `Cell` lifecycle and per-grid
//! simulation construction.
//!
//! Relocated from `src/main.rs`, per the all-tests-in-tests/ convention.

use game_of_life::{ Cell, HexGameOfLife, SquareGameOfLife, TriangularGameOfLife };
use tiles_tools::coordinates::hexagonal::{ Coordinate as HexCoord, Axial, Pointy };

#[ test ]
fn test_cell_lifecycle()
{
  let mut cell = Cell::new();
  assert!( cell.is_alive() );
  assert_eq!( cell.age, 0 );

  cell.age();
  assert_eq!( cell.age, 1 );

  cell.kill();
  assert!( !cell.is_alive() );

  cell.revive();
  assert!( cell.is_alive() );
  assert_eq!( cell.age, 0 );
}

#[ test ]
fn test_square_game_creation()
{
  let game = SquareGameOfLife::new( 10, 10 );
  assert_eq!( game.width(), 10 );
  assert_eq!( game.height(), 10 );
  assert_eq!( game.generation(), 0 );
}

#[ test ]
fn test_hex_game_creation()
{
  let game = HexGameOfLife::new();
  assert_eq!( game.generation(), 0 );
}

#[ test ]
fn test_triangular_game_creation()
{
  let game = TriangularGameOfLife::new();
  assert_eq!( game.generation(), 0 );
}

/// ## Root Cause
/// `HexGameOfLife::step` computed `neighbors_count` correctly (via the
/// hex-specific `Neighbors` adjacency) but never derived or applied a next
/// generation from it -- no `Cell` component was ever revived, aged, or
/// killed, so the seed pattern never evolved no matter how many generations
/// were advanced.
///
/// ## Why Not Caught
/// `test_hex_game_creation` only asserted `generation() == 0` right after
/// construction; nothing called `step()` and then inspected any cell's alive
/// state, so a `step()` that silently discarded its own computation produced
/// no test failure.
///
/// ## Fix Applied
/// `step` now derives `next_generation` from `neighbors_count` using the
/// rule already documented in its own comment (survive on 2-3 neighbors,
/// born on exactly 2), then calls a new `world_state_update` (mirroring
/// `SquareGameOfLife`'s) that persists every decision into the ECS world via
/// `world.get_mut::<Cell>` / `world.spawn`. `is_cell_alive` was made `pub` so
/// this test can observe the result directly.
///
/// ## Prevention
/// This test hand-derives the expected outcome from the crate's own axial
/// neighbor offsets for the built-in seed pattern and asserts a death, a
/// survival, and a birth all actually landed in the ECS world after one
/// `step()` -- the three branches `world_state_update` can take, not a
/// single pinned snapshot.
///
/// ## Pitfall
/// A neighbor-counting loop that never writes its result anywhere is easy to
/// mistake for a working simulation -- `step()` still printed a plausible
/// "Hex Generation N: ..." line every call. Always confirm a computed value
/// is actually persisted into the state other code queries, not just logged.
#[ test ]
fn bug_reproducer_bug_486_hex_game_step_applies_rules()
{
  let mut game = HexGameOfLife::new();

  let center = HexCoord::< Axial, Pointy >::new( 0, 0 );
  let survivor = HexCoord::< Axial, Pointy >::new( 1, 0 );
  let newborn = HexCoord::< Axial, Pointy >::new( 1, 1 );

  // Seed pattern is `[(0,0),(1,0),(0,1),(-1,1),(-1,0),(0,-1)]`, all alive.
  assert!( game.is_cell_alive( center ), "center must start alive per the seed pattern" );
  assert!( game.is_cell_alive( survivor ), "(1,0) must start alive per the seed pattern" );
  assert!( !game.is_cell_alive( newborn ), "(1,1) must start dead -- not part of the seed pattern" );

  game.step();

  assert_eq!( game.generation(), 1 );
  // Hand-derived from the crate's own axial neighbor offsets
  // ( (1,0), (1,-1), (0,-1), (-1,0), (-1,1), (0,1) ) applied to the seed:
  // the center has 5 living neighbors (dies of overcrowding), (1,0) has 2
  // (survives), and (1,1) has exactly 2 living neighbors (newly born).
  assert!( !game.is_cell_alive( center ), "center must die of overcrowding (5 neighbors)" );
  assert!( game.is_cell_alive( survivor ), "(1,0) must survive (2 neighbors)" );
  assert!( game.is_cell_alive( newborn ), "(1,1) must be born (2 neighbors)" );
}

/// ## Root Cause
/// `HexGameOfLife::step` (and `SquareGameOfLife::step`, which shares the
/// identical pattern) built `neighbors_count` solely by incrementing each
/// living cell's *neighbor* coordinates -- never the living cell's own
/// coordinate. A living cell with zero currently-living neighbors therefore
/// never became a key of `neighbors_count`, so it was never inserted into
/// `next_generation` either, so `world_state_update` never touched its
/// `Cell` component. The cell silently stayed alive forever, bypassing the
/// Conway rule (which -- correctly, if it had ever run -- maps `(true, 0)`
/// to death via the `_ => false` arm) no matter how many generations were
/// advanced.
///
/// ## Why Not Caught
/// `bug_reproducer_bug_486_hex_game_step_applies_rules` only advances one
/// generation from the built-in seed, where every living cell already has
/// 2+ living neighbors (none isolated yet), so the omission never
/// manifested. It takes the seed pattern spreading out over multiple
/// generations before a living cell drifts to zero living neighbors --
/// which is exactly what happens to `(-1, 2)` and `(-2, 1)` by generation 3
/// of the default seed (hand-derived here from the crate's own axial
/// neighbor offsets, and cross-checked against this example's actual
/// `cargo run -p game_of_life` output: `step()`'s own
/// `next_generation`-derived line printed "Hex Generation 3: 13 living
/// cells" while the true ECS world state -- what `state_print()` walks --
/// held 15 living cells for that same generation).
///
/// ## Fix Applied
/// Both `HexGameOfLife::step` and `SquareGameOfLife::step` now seed
/// `neighbors_count` with a `0` entry for every currently-alive cell's own
/// coordinate (`neighbors_count.entry(pos.coord).or_insert(0)`) alongside
/// incrementing its neighbors, guaranteeing every living cell is always a
/// candidate for re-evaluation even when totally isolated.
///
/// ## Prevention
/// This test advances the real default seed pattern by the same 3
/// generations the `game_of_life` binary's own `main()` does, then checks
/// the two specific coordinates hand-derived to go isolated at generation 3
/// are dead -- not a `step()`-internal count (which the bug also corrupts,
/// but only as a downstream symptom), but the actual persisted ECS `Cell`
/// state, the same source `state_print()` reads from. A third assertion
/// confirms a cell that legitimately keeps living neighbors at every
/// generation still survives, guarding against an over-corrected fix that
/// kills everything.
///
/// ## Pitfall
/// A `next_generation` map that is only ever populated by *incrementing*
/// neighbor entries can never produce a `0`-neighbor entry by construction
/// -- re-simplifying the loop back to just the neighbor-increment (dropping
/// the explicit self `.entry(..).or_insert(0)`) silently reintroduces
/// exactly this bug.
#[ test ]
fn bug_reproducer_bug_511_hex_game_isolated_survivor_never_reevaluated()
{
  let mut game = HexGameOfLife::new();

  for _ in 1..=3
  {
    game.step();
  }

  assert_eq!( game.generation(), 3 );

  // Hand-derived: by generation 3 of the built-in seed, (-1,2) and (-2,1)
  // are alive survivors from generation 2 with zero living neighbors within
  // the generation-2 population -- the Conway rule, if applied, maps
  // `(true, 0)` to death via `step`'s own `_ => false` arm.
  let isolated_a = HexCoord::< Axial, Pointy >::new( -1, 2 );
  let isolated_b = HexCoord::< Axial, Pointy >::new( -2, 1 );
  assert!
  (
    !game.is_cell_alive( isolated_a ),
    "(-1,2) has 0 living neighbors at generation 2 and must die of isolation by generation 3, not persist forever unevaluated"
  );
  assert!
  (
    !game.is_cell_alive( isolated_b ),
    "(-2,1) has 0 living neighbors at generation 2 and must die of isolation by generation 3, not persist forever unevaluated"
  );

  // Regression guard: a cell that legitimately keeps living neighbors at
  // every generation (never isolated) must still survive -- the fix must
  // not over-correct into killing everything.
  let true_survivor = HexCoord::< Axial, Pointy >::new( 1, 0 );
  assert!
  (
    game.is_cell_alive( true_survivor ),
    "(1,0) has living neighbors at every generation and must remain alive -- fix must not over-kill"
  );
}

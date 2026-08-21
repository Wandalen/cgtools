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

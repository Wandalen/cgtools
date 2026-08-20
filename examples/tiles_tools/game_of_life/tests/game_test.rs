//! Tests for the `game_of_life` library — `Cell` lifecycle and per-grid
//! simulation construction.
//!
//! Relocated from `src/main.rs`, per the all-tests-in-tests/ convention.

use game_of_life::{ Cell, HexGameOfLife, SquareGameOfLife, TriangularGameOfLife };

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

//! Conway's Game of Life demo driver across three coordinate systems —
//! square, hexagonal, and triangular — over the `game_of_life` library
//! crate ( see `src/lib.rs` for the `Cell` component and the per-grid
//! simulation types ).
//!
//! Run with: `cd examples/tiles_tools/game_of_life && cargo run --release`

use game_of_life::{ SquareGameOfLife, HexGameOfLife, TriangularGameOfLife };

/// Demonstrates Game of Life across different coordinate systems.
fn main()
{
  println!( "Conway's Game of Life - Multi-Coordinate System Demo" );
  println!( "====================================================" );

  // Square Grid Game of Life
  println!( "\n🟩 SQUARE GRID (8-connected neighbors)" );
  let mut square_game = SquareGameOfLife::new( 20, 20 );
  square_game.print_state();

  for i in 1..=5
  {
    square_game.step();
    square_game.print_state();

    if i < 5
    {
      std::thread::sleep( std::time::Duration::from_secs( 1 ) );
    }
  }

  // Hexagonal Grid Game of Life
  println!( "\n🔶 HEXAGONAL GRID (6-connected neighbors)" );
  let mut hex_game = HexGameOfLife::new();
  hex_game.print_state();

  for _ in 1..=3
  {
    hex_game.step();
    hex_game.print_state();
  }

  // Triangular Grid Game of Life
  println!( "\n🔺 TRIANGULAR GRID (3-connected neighbors)" );
  let mut tri_game = TriangularGameOfLife::new();
  tri_game.print_state();

  for _ in 1..=3
  {
    tri_game.step();
    tri_game.print_state();
  }

  println!( "\n✨ Demo Complete!" );
  println!( "This example showcases how tiles_tools ECS works seamlessly" );
  println!( "across different coordinate systems with proper neighbor" );
  println!( "calculations and grid-aware game logic." );
}

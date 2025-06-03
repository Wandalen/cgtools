// Import the ndarray_cg crate's prelude for array manipulation
use ndarray_cg::prelude::*;

// Include the input file as a byte array at compile time
const INPUT : &[u8] = include_bytes!( "life.txt" );

// Define the size of the grid (25x25)
const N : usize = 25;

// Define a struct to represent a cell
#[derive( Clone, Copy )] // Add `Copy` to allow copying of `Cell` values
pub struct Cell( u8 );

// Define a type alias for a 2D array of `Cell` values
type Board = Array2< Cell >;

// Implement the `Add` and `AddAssign` traits for `Cell` to allow addition
use std::ops::{ Add, AddAssign };

impl Add for Cell
{
  type Output = Cell;

  fn add( self, other : Cell ) -> Cell
  {
    Cell( self.0 + other.0 ) // Add the inner `u8` values
  }
}

impl AddAssign for Cell
{
  fn add_assign( &mut self, other : Cell )
  {
    self.0 += other.0; // Add the inner `u8` values
  }
}

// Function to parse the input file into a 2D grid (Board)
fn parse( x : &[u8] ) -> Board
{
  // Create a grid with a border of `Cell(0)` (dead cells)
  let mut map = Board::from_elem( ( N + 2, N + 2 ), Cell( 0 ));

  // Convert the input bytes into a 1D array of `Cell(1)` (live cells) and `Cell(0)` (dead cells)
  let a = Array::from_iter( x.iter().filter_map( |&b| match b
  {
    b'#' => Some( Cell( 1 )), // '#' represents a live cell
    b'.' => Some( Cell( 0 )), // '.' represents a dead cell
    _ => None,                // Ignore other characters
  }));

  // Reshape the 1D array into an NxN grid
  let a = a.into_shape_with_order( ( N, N )).unwrap();

  // Copy the NxN grid into the center of the larger grid with a border
  map.slice_mut( s![ 1..-1, 1..-1 ]).assign( &a );

  // Return the grid with the border
  map
}

// Function to apply the rules of the Game of Life to the grid
fn iterate( z : &mut Board, scratch : &mut Board )
{
  // Create a mutable view of the scratch array to store neighbor counts
  let mut neigh = scratch.view_mut();
  neigh.fill( Cell( 0 )); // Reset the scratch array to `Cell(0)`

  // Add the values of the 8 neighboring cells to calculate neighbor counts
  neigh += &z.slice( s![ 0..-2, 0..-2 ]); // Top-left neighbors
  neigh += &z.slice( s![ 0..-2, 1..-1 ]); // Top neighbors
  neigh += &z.slice( s![ 0..-2, 2.. ]);   // Top-right neighbors
  neigh += &z.slice( s![ 1..-1, 0..-2 ]); // Left neighbors
  neigh += &z.slice( s![ 1..-1, 2.. ]);   // Right neighbors
  neigh += &z.slice( s![ 2.., 0..-2 ]);   // Bottom-left neighbors
  neigh += &z.slice( s![ 2.., 1..-1 ]);   // Bottom neighbors
  neigh += &z.slice( s![ 2.., 2.. ]);     // Bottom-right neighbors

  // Create a mutable view of the inner grid (excluding the border)
  let mut zv = z.slice_mut( s![ 1..-1, 1..-1 ]);

  // Apply the Game of Life rules to each cell
  zv.zip_mut_with( &neigh, |y, &n|
  {
    y.0 = (( n.0 == 3 ) || ( n.0 == 2 && y.0 > 0 )) as u8; // Birth or survival
  });
}

// Function to ensure the four corners of the grid are always alive
fn turn_on_corners( z : &mut Board )
{
  let n = z.nrows(); // Get the number of rows
  let m = z.ncols(); // Get the number of columns
  z[[ 1, 1 ]] = Cell( 1 );         // Top-left corner
  z[[ 1, m - 2 ]] = Cell( 1 );     // Top-right corner
  z[[ n - 2, 1 ]] = Cell( 1 );     // Bottom-left corner
  z[[ n - 2, m - 2 ]] = Cell( 1 ); // Bottom-right corner
}

// Function to render the grid to the console
fn render( a : &Board )
{
  // Iterate over each row of the grid
  for row in a.rows()
  {
    // Iterate over each cell in the row
    for x in row // Remove the `&` to avoid moving `Cell`
    {
      if x.0 > 0
      {
        print!( "#" ); // Print '#' for live cells
      }
      else
      {
        print!( "." ); // Print '.' for dead cells
      }
    }
    println!(); // Move to the next line after each row
  }
}

// Main function to run the simulation
fn main()
{
  let mut a = parse( INPUT ); // Parse the input into the initial grid
  let mut scratch = Board::from_elem( ( N, N ), Cell( 0 )); // Create a scratch array for neighbor calculations
  let steps = 50; // Number of steps to simulate

  turn_on_corners( &mut a ); // Turn on the corners of the grid

  // Simulate the Game of Life for the specified number of steps
  for _ in 0..steps
  {
    iterate( &mut a, &mut scratch ); // Apply the game rules
    turn_on_corners( &mut a );       // Ensure the corners stay alive
  }

  render( &a ); // Render the final grid

  // Count the number of live cells in the final grid
  let alive = a.iter().filter( |x| x.0 > 0 ).count(); // Remove extra `&` to avoid moving `Cell`
  println!( "After {} steps there are {} cells alive", steps, alive ); // Print the result
}

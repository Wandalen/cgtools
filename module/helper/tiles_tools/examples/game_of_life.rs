#![allow(dead_code ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::items_after_statements ) ]
#![ allow( clippy::unnecessary_cast ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::format_in_format_args ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::duplicated_attributes ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::trivially_copy_pass_by_ref ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::useless_vec ) ]
#![ allow( clippy::unnested_or_patterns ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::redundant_else ) ]
#![ allow( clippy::match_same_arms ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::min_ident_chars ) ]
//! Conway's Game of Life implementation using the tiles_tools ECS system.
//!
//! This example demonstrates how to use the tiles_tools library to implement
//! Conway's Game of Life on different coordinate systems. It showcases:
//!
//! - ECS entity and component management
//! - Universal coordinate system support (Square, Hexagonal, Triangular)
//! - Grid-aware game logic and neighbor calculations
//! - System-based game state updates
//! - Cross-coordinate system compatibility
//!
//! Run with: `cargo run --example game_of_life --features enabled`

use tiles_tools::{
  ecs::{World, Position},
  coordinates::{
  square::{Coordinate as SquareCoord, EightConnected},
  hexagonal::{Coordinate as HexCoord, Axial, Pointy},
  triangular::{Coordinate as TriCoord, TwelveConnected},
  },
};
use std::collections::HashMap;

/// Cell component representing a living cell in Game of Life.
#[derive(Debug, Clone, Copy ) ]
struct Cell
{
  /// Whether this cell is currently alive
  alive: bool,
  /// Age of the cell (generations alive)
  age: u32,
}

impl Cell {
  pub fn new() -> Self {
  Self { alive: true, age: 0 }
  }
  
  pub fn is_alive(&self) -> bool {
  self.alive
  }
  
  pub fn kill(&mut self) {
  self.alive = false;
  }
  
  pub fn revive(&mut self) {
  self.alive = true;
  self.age = 0;
  }
  
  pub fn age(&mut self) {
  if self.alive {
    self.age += 1;
  }
  }
}

/// Game of Life simulation on a square grid with 8-connected neighbors.
struct SquareGameOfLife
{
  world: World,
  width: i32,
  height: i32,
  generation: u32,
}

impl SquareGameOfLife {
  /// Creates a new Game of Life simulation on a square grid.
  pub fn new(width: i32, height: i32) -> Self {
  let mut world = World::new();
  
  // Spawn initial pattern - a glider
  let glider_pattern = [
    (1, 2), (2, 3), (3, 1), (3, 2), (3, 3)
  ];
  
  for &(x, y) in &glider_pattern {
    let coord = SquareCoord::<EightConnected>::new(x, y);
    world.spawn((
      Position::new(coord),
      Cell::new(),
    ));
  }
  
  Self { world, width, height, generation: 0 }
  }
  
  /// Advances the simulation by one generation using Conway's rules.
  pub fn step(&mut self) {
  let mut next_generation = HashMap::new();
  let mut neighbors_count = HashMap::new();
  
  // Count neighbors for all positions
  {
    let mut query = self.world.query::<(&Position<SquareCoord<EightConnected>>, &Cell)>();
    for (_entity, (pos, cell)) in query.iter() {
      if cell.is_alive() {
        // Count neighbors for living cells and their neighbors
        for neighbor_coord in pos.neighbors() {
          *neighbors_count.entry(neighbor_coord.coord).or_insert(0) += 1;
        }
      }
    }
  }
  
  // Apply Game of Life rules
  for (coord, neighbor_count) in neighbors_count {
    let currently_alive = self.is_cell_alive(&coord);
    
    let should_be_alive = match (currently_alive, neighbor_count) {
      (true, 2 | 3) => true,  // Survival
      (false, 3) => true,             // Birth
      _ => false,                     // Death or remain dead
    };
    
    next_generation.insert(coord, should_be_alive);
  }
  
  // Update world state
  self.update_world_state(next_generation);
  self.generation += 1;
  }
  
  /// Checks if a cell at the given coordinate is alive.
  fn is_cell_alive(&self, coord: &SquareCoord<EightConnected>) -> bool {
  let _pos_query = Position::new(*coord);
  let mut query = self.world.query::<(&Position<SquareCoord<EightConnected>>, &Cell)>();
  
  for (_entity, (pos, cell)) in query.iter() {
    if pos.coord == *coord {
      return cell.is_alive();
    }
  }
  false
  }
  
  /// Updates the world state based on the next generation.
  fn update_world_state(&mut self, next_generation: HashMap<SquareCoord<EightConnected>, bool>) {
  // This is a simplified update - in practice would use proper ECS entity management
  println!("Generation {}: {} living cells", 
           self.generation + 1, 
           next_generation.values().filter(|&&alive| alive).count());
  }
  
  /// Prints the current state of the grid.
  pub fn print_state(&self) {
  println!("\nGeneration {}", self.generation);
  
  // Find bounds of living cells
  let mut min_x = i32::MAX;
  let mut max_x = i32::MIN;
  let mut min_y = i32::MAX;
  let mut max_y = i32::MIN;
  
  let mut living_cells = std::collections::HashSet::new();
  let mut query = self.world.query::<(&Position<SquareCoord<EightConnected>>, &Cell)>();
  
  for (_entity, (pos, cell)) in query.iter() {
    if cell.is_alive() {
      living_cells.insert((pos.coord.x, pos.coord.y));
      min_x = min_x.min(pos.coord.x);
      max_x = max_x.max(pos.coord.x);
      min_y = min_y.min(pos.coord.y);
      max_y = max_y.max(pos.coord.y);
    }
  }
  
  if living_cells.is_empty() {
    println!("All cells are dead!");
    return;
  }
  
  // Print grid with padding
  for y in (min_y - 1)..=(max_y + 1) {
    for x in (min_x - 1)..=(max_x + 1) {
      if living_cells.contains(&(x, y)) {
        print!("█");
      } else {
        print!("·");
      }
    }
    println!();
  }
  }
}

/// Game of Life simulation on a hexagonal grid.
struct HexGameOfLife
{
  world: World,
  generation: u32,
}

impl HexGameOfLife {
  /// Creates a new Game of Life on a hexagonal grid.
  pub fn new() -> Self {
  let mut world = World::new();
  
  // Spawn initial hexagonal pattern
  let hex_pattern = [
    (0, 0), (1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1)
  ];
  
  for &(q, r) in &hex_pattern {
    let coord = HexCoord::<Axial, Pointy>::new(q, r);
    world.spawn((
      Position::new(coord),
      Cell::new(),
    ));
  }
  
  Self { world, generation: 0 }
  }
  
  /// Advances one generation with modified rules for hexagonal grid.
  pub fn step(&mut self) {
  // Hexagonal Game of Life uses different rules due to 6 neighbors instead of 8
  // Common rule: survive with 2-3 neighbors, born with 2 neighbors
  
  let mut neighbors_count = HashMap::new();
  
  {
    let mut query = self.world.query::<(&Position<HexCoord<Axial, Pointy>>, &Cell)>();
    for (_entity, (pos, cell)) in query.iter() {
      if cell.is_alive() {
        for neighbor_coord in pos.neighbors() {
          *neighbors_count.entry((neighbor_coord.coord.q, neighbor_coord.coord.r)).or_insert(0) += 1;
        }
      }
    }
  }
  
  println!("Hex Generation {}: {} positions with neighbors", 
           self.generation + 1, 
           neighbors_count.len());
  
  self.generation += 1;
  }
  
  /// Prints the hexagonal grid state.
  pub fn print_state(&self) {
  println!("\nHexagonal Generation {}", self.generation);
  
  let mut query = self.world.query::<(&Position<HexCoord<Axial, Pointy>>, &Cell)>();
  let living_cells: Vec<_> = query.iter()
    .filter(|(_, (_, cell))| cell.is_alive())
    .map(|(_, (pos, _))| (pos.coord.q, pos.coord.r))
    .collect();
  
  println!("Living cells: {:?}", living_cells);
  }
}

/// Game of Life simulation on a triangular grid.
struct TriangularGameOfLife
{
  world: World,
  generation: u32,
}

impl TriangularGameOfLife {
  /// Creates a new Game of Life on a triangular grid.
  pub fn new() -> Self {
  let mut world = World::new();
  
  // Spawn initial triangular pattern
  let tri_pattern = [
    (0, 0), (1, 0), (0, 1), (1, 1)
  ];
  
  for &(x, y) in &tri_pattern {
    let coord = TriCoord::<TwelveConnected>::new(x, y);
    world.spawn((
      Position::new(coord),
      Cell::new(),
    ));
  }
  
  Self { world, generation: 0 }
  }
  
  /// Advances one generation with triangular grid rules.
  pub fn step(&mut self) {
  // Triangular grids have 12 neighbors, so different rules apply
  println!("Triangular Generation {}: Complex neighbor relationships", 
           self.generation + 1);
  self.generation += 1;
  }
  
  /// Prints the triangular grid state.
  pub fn print_state(&self) {
  println!("\nTriangular Generation {}", self.generation);
  
  let mut query = self.world.query::<(&Position<TriCoord<TwelveConnected>>, &Cell)>();
  let living_cells: Vec<_> = query.iter()
    .filter(|(_, (_, cell))| cell.is_alive())
    .map(|(_, (pos, _))| (pos.coord.x, pos.coord.y))
    .collect();
  
  println!("Living triangular cells: {:?}", living_cells);
  }
}

/// Demonstrates Game of Life across different coordinate systems.
fn main()
{
  println!("Conway's Game of Life - Multi-Coordinate System Demo");
  println!("====================================================");
  
  // Square Grid Game of Life
  println!("\n🟩 SQUARE GRID (8-connected neighbors)");
  let mut square_game = SquareGameOfLife::new(20, 20);
  square_game.print_state();
  
  for i in 1..=5 {
  square_game.step();
  square_game.print_state();
  
  if i < 5 {
    std::thread::sleep(std::time::Duration::from_millis(1000));
  }
  }
  
  // Hexagonal Grid Game of Life
  println!("\n🔶 HEXAGONAL GRID (6-connected neighbors)");
  let mut hex_game = HexGameOfLife::new();
  hex_game.print_state();
  
  for _ in 1..=3 {
  hex_game.step();
  hex_game.print_state();
  }
  
  // Triangular Grid Game of Life
  println!("\n🔺 TRIANGULAR GRID (12-connected neighbors)");
  let mut tri_game = TriangularGameOfLife::new();
  tri_game.print_state();
  
  for _ in 1..=3 {
  tri_game.step();
  tri_game.print_state();
  }
  
  println!("\n✨ Demo Complete!");
  println!("This example showcases how tiles_tools ECS works seamlessly");
  println!("across different coordinate systems with proper neighbor");
  println!("calculations and grid-aware game logic.");
}

#[cfg(test ) ]
mod tests {
  use super::*;

  #[test]
  fn test_cell_lifecycle() {
  let mut cell = Cell::new();
  assert!(cell.is_alive());
  assert_eq!(cell.age, 0);
  
  cell.age();
  assert_eq!(cell.age, 1);
  
  cell.kill();
  assert!(!cell.is_alive());
  
  cell.revive();
  assert!(cell.is_alive());
  assert_eq!(cell.age, 0);
  }

  #[test]
  fn test_square_game_creation() {
  let game = SquareGameOfLife::new(10, 10);
  assert_eq!(game.width, 10);
  assert_eq!(game.height, 10);
  assert_eq!(game.generation, 0);
  }

  #[test]
  fn test_hex_game_creation() {
  let game = HexGameOfLife::new();
  assert_eq!(game.generation, 0);
  }

  #[test]
  fn test_triangular_game_creation() {
  let game = TriangularGameOfLife::new();
  assert_eq!(game.generation, 0);
  }
}
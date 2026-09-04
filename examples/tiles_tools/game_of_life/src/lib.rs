//! Conway's Game of Life on the `tiles_tools` ECS across three coordinate
//! systems ( square, hexagonal, triangular ): the [`Cell`] component plus one
//! simulation type per grid, exposed as a library so the binary stays a thin
//! demo driver and the tests can live in `tests/`.
//!
//! It showcases:
//!
//! - ECS entity and component management
//! - Universal coordinate system support (Square, Hexagonal, Triangular)
//! - Grid-aware game logic and neighbor calculations
//! - System-based game state updates
//! - Cross-coordinate system compatibility

use tiles_tools::
{
  ecs::{ World, Position },
  coordinates::
  {
    square::{ Coordinate as SquareCoord, EightConnected },
    hexagonal::{ Coordinate as HexCoord, Axial, Pointy },
    triangular::{ Coordinate as TriCoord, FlatSided },
  },
};
use std::collections::HashMap;

/// Cell component representing a living cell in Game of Life.
#[ derive( Debug, Clone, Copy ) ]
pub struct Cell
{
  /// Whether this cell is currently alive.
  pub alive : bool,
  /// Age of the cell ( generations alive ).
  pub age : u32,
}

impl Cell
{
  /// Creates a freshly-born ( alive, age 0 ) cell.
  #[ must_use ]
  pub fn new() -> Self
  {
    Self { alive : true, age : 0 }
  }

  /// Whether this cell is currently alive.
  #[ must_use ]
  pub fn is_alive( self ) -> bool
  {
    self.alive
  }

  /// Marks the cell dead.
  pub fn kill( &mut self )
  {
    self.alive = false;
  }

  /// Brings the cell back to life with its age reset to 0.
  pub fn revive( &mut self )
  {
    self.alive = true;
    self.age = 0;
  }

  /// Ages a living cell by one generation; dead cells stay unchanged.
  pub fn age( &mut self )
  {
    if self.alive
    {
      self.age += 1;
    }
  }
}

impl Default for Cell
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Game of Life simulation on a square grid with 8-connected neighbors.
pub struct SquareGameOfLife
{
  world : World,
  width : i32,
  height : i32,
  generation : u32,
}

impl SquareGameOfLife
{
  /// Creates a new Game of Life simulation on a `width` x `height` square
  /// grid, seeded with a glider.
  #[ must_use ]
  pub fn new( width : i32, height : i32 ) -> Self
  {
    let mut world = World::new();

    // Spawn initial pattern - a glider
    let glider_pattern = [ ( 1, 2 ), ( 2, 3 ), ( 3, 1 ), ( 3, 2 ), ( 3, 3 ) ];

    for &( x, y ) in &glider_pattern
    {
      let coord = SquareCoord::< EightConnected >::new( x, y );
      world.spawn( ( Position::new( coord ), Cell::new() ) );
    }

    Self { world, width, height, generation : 0 }
  }

  /// Grid width in cells.
  #[ must_use ]
  pub fn width( &self ) -> i32
  {
    self.width
  }

  /// Grid height in cells.
  #[ must_use ]
  pub fn height( &self ) -> i32
  {
    self.height
  }

  /// Generations advanced so far.
  #[ must_use ]
  pub fn generation( &self ) -> u32
  {
    self.generation
  }

  /// Advances the simulation by one generation using Conway's rules.
  pub fn step( &mut self )
  {
    let mut next_generation = HashMap::new();
    let mut neighbors_count = HashMap::new();

    // Count neighbors for all positions
    {
      let mut query = self.world.query::< ( &Position< SquareCoord< EightConnected > >, &Cell ) >();
      for ( pos, cell ) in &mut query
      {
        if cell.is_alive()
        {
          // Fix(BUG-511): register the living cell itself as a candidate
          // (defaulting to 0 neighbors) before tallying its neighbors.
          // Root cause: `neighbors_count` was previously populated only by
          // incrementing *neighbor* coordinates, so a living cell with zero
          // living neighbors never became a key and was silently dropped
          // from `next_generation`, leaving it alive forever regardless of
          // the Conway rule below.
          // Pitfall: dropping this line and going back to only the
          // neighbor-increment loop silently reintroduces the bug, since
          // that loop can never itself produce a `0`-neighbor entry.
          neighbors_count.entry( pos.coord ).or_insert( 0 );

          // Count neighbors for living cells and their neighbors
          for neighbor_coord in pos.neighbors()
          {
            *neighbors_count.entry( neighbor_coord.coord ).or_insert( 0 ) += 1;
          }
        }
      }
    }

    // Apply Game of Life rules within the bounded grid
    for ( coord, neighbor_count ) in neighbors_count
    {
      if coord.x < 0 || coord.x >= self.width || coord.y < 0 || coord.y >= self.height
      {
        continue;
      }

      let currently_alive = self.is_cell_alive( coord );

      let should_be_alive = match ( currently_alive, neighbor_count )
      {
        ( true, 2 | 3 ) | ( false, 3 ) => true, // Survival or birth
        _ => false,                             // Death or remain dead
      };

      next_generation.insert( coord, should_be_alive );
    }

    // Update world state
    self.world_state_update( &next_generation );
    self.generation += 1;
  }

  /// Checks if a cell at the given coordinate is alive.
  fn is_cell_alive( &self, coord : SquareCoord< EightConnected > ) -> bool
  {
    let mut query = self.world.query::< ( &Position< SquareCoord< EightConnected > >, &Cell ) >();

    for ( pos, cell ) in &mut query
    {
      if pos.coord == coord
      {
        return cell.is_alive();
      }
    }
    false
  }

  /// Updates the world state based on the next generation, aging surviving
  /// cells and reviving/killing entities that changed state.
  fn world_state_update( &mut self, next_generation : &HashMap< SquareCoord< EightConnected >, bool > )
  {
    let mut existing = HashMap::new();
    {
      let mut query = self.world.query::< ( hecs::Entity, &Position< SquareCoord< EightConnected > > ) >();
      for ( entity, pos ) in &mut query
      {
        existing.insert( pos.coord, entity );
      }
    }

    for ( &coord, &should_be_alive ) in next_generation
    {
      if let Some( &entity ) = existing.get( &coord )
      {
        if let Ok( mut cell ) = self.world.get_mut::< Cell >( entity )
        {
          if should_be_alive
          {
            if cell.is_alive() { cell.age(); } else { cell.revive(); }
          }
          else
          {
            cell.kill();
          }
        }
      }
      else if should_be_alive
      {
        self.world.spawn( ( Position::new( coord ), Cell::new() ) );
      }
      else
      {
        // Cell doesn't exist and shouldn't be alive — nothing to do.
      }
    }

    println!
    (
      "Generation {}: {} living cells",
      self.generation + 1,
      next_generation.values().filter( | &&alive | alive ).count()
    );
  }

  /// Prints the current state of the grid.
  pub fn state_print( &self )
  {
    println!( "\nGeneration {}", self.generation );

    // Find bounds of living cells
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    let mut living_cells = std::collections::HashSet::new();
    let mut query = self.world.query::< ( &Position< SquareCoord< EightConnected > >, &Cell ) >();

    for ( pos, cell ) in &mut query
    {
      if cell.is_alive()
      {
        living_cells.insert( ( pos.coord.x, pos.coord.y ) );
        min_x = min_x.min( pos.coord.x );
        max_x = max_x.max( pos.coord.x );
        min_y = min_y.min( pos.coord.y );
        max_y = max_y.max( pos.coord.y );
      }
    }

    if living_cells.is_empty()
    {
      println!( "All cells are dead!" );
      return;
    }

    // Print grid with padding
    for y in ( min_y - 1 )..=( max_y + 1 )
    {
      for x in ( min_x - 1 )..=( max_x + 1 )
      {
        if living_cells.contains( &( x, y ) )
        {
          print!( "█" );
        }
        else
        {
          print!( "·" );
        }
      }
      println!();
    }
  }
}

/// Game of Life simulation on a hexagonal grid.
pub struct HexGameOfLife
{
  world : World,
  generation : u32,
}

impl HexGameOfLife
{
  /// Creates a new Game of Life on a hexagonal grid, seeded with a ring of
  /// six cells around the origin.
  #[ must_use ]
  pub fn new() -> Self
  {
    let mut world = World::new();

    // Spawn initial hexagonal pattern
    let hex_pattern = [ ( 0, 0 ), ( 1, 0 ), ( 0, 1 ), ( -1, 1 ), ( -1, 0 ), ( 0, -1 ) ];

    for &( q, r ) in &hex_pattern
    {
      let coord = HexCoord::< Axial, Pointy >::new( q, r );
      world.spawn( ( Position::new( coord ), Cell::new() ) );
    }

    Self { world, generation : 0 }
  }

  /// Generations advanced so far.
  #[ must_use ]
  pub fn generation( &self ) -> u32
  {
    self.generation
  }

  /// Advances one generation with modified rules for hexagonal grid.
  // Fix(BUG-486): `step` computed `neighbors_count` (correctly, via the hex-specific
  // `Neighbors` adjacency) but never derived or applied a next generation from it --
  // no `Cell` component was ever revived, aged, or killed, so the hex simulation's
  // seed pattern never evolved across any number of generations.
  // Root cause: the function stopped after printing a raw neighbor-position count;
  // nothing translated that count into survive/birth/death decisions or persisted
  // them into the ECS world, unlike `SquareGameOfLife::step`/`world_state_update`
  // earlier in this same file.
  // Pitfall: this file's own comment two lines below already stated the intended
  // rule ("survive with 2-3 neighbors, born with 2 neighbors") -- a documented rule
  // that is never implemented is easy to mistake for a live one on read-through.
  pub fn step( &mut self )
  {
    // Hexagonal Game of Life uses different rules due to 6 neighbors instead of 8
    // Common rule: survive with 2-3 neighbors, born with 2 neighbors

    let mut neighbors_count = HashMap::new();

    {
      let mut query = self.world.query::< ( &Position< HexCoord< Axial, Pointy > >, &Cell ) >();
      for ( pos, cell ) in &mut query
      {
        if cell.is_alive()
        {
          // Fix(BUG-511): register the living cell itself as a candidate
          // (defaulting to 0 neighbors) before tallying its neighbors.
          // Root cause: `neighbors_count` was previously populated only by
          // incrementing *neighbor* coordinates, so a living cell with zero
          // living neighbors never became a key and was silently dropped
          // from `next_generation` -- e.g. the built-in seed's `(-1,2)` and
          // `(-2,1)` by generation 3 -- leaving it alive forever regardless
          // of the Conway rule below.
          // Pitfall: dropping this line and going back to only the
          // neighbor-increment loop silently reintroduces the bug, since
          // that loop can never itself produce a `0`-neighbor entry.
          neighbors_count.entry( pos.coord ).or_insert( 0 );

          for neighbor_coord in pos.neighbors()
          {
            *neighbors_count.entry( neighbor_coord.coord ).or_insert( 0 ) += 1;
          }
        }
      }
    }

    let mut next_generation = HashMap::new();
    for ( &coord, &neighbor_count ) in &neighbors_count
    {
      let currently_alive = self.is_cell_alive( coord );
      let should_be_alive = match ( currently_alive, neighbor_count )
      {
        ( true, 2 | 3 ) | ( false, 2 ) => true, // Survival (2-3) or birth (exactly 2)
        _ => false,                             // Death or remain dead
      };
      next_generation.insert( coord, should_be_alive );
    }

    self.world_state_update( &next_generation );

    println!
    (
      "Hex Generation {}: {} living cells",
      self.generation + 1,
      next_generation.values().filter( | &&alive | alive ).count()
    );

    self.generation += 1;
  }

  /// Checks if a cell at the given coordinate is alive. `pub` ( unlike
  /// `SquareGameOfLife`'s private equivalent ) so regression tests can observe
  /// simulation state without parsing `state_print`'s console output.
  #[ must_use ]
  pub fn is_cell_alive( &self, coord : HexCoord< Axial, Pointy > ) -> bool
  {
    let mut query = self.world.query::< ( &Position< HexCoord< Axial, Pointy > >, &Cell ) >();

    for ( pos, cell ) in &mut query
    {
      if pos.coord == coord
      {
        return cell.is_alive();
      }
    }
    false
  }

  /// Updates the world state based on the next generation, aging surviving
  /// cells and reviving/killing entities that changed state.
  fn world_state_update( &mut self, next_generation : &HashMap< HexCoord< Axial, Pointy >, bool > )
  {
    let mut existing = HashMap::new();
    {
      let mut query = self.world.query::< ( hecs::Entity, &Position< HexCoord< Axial, Pointy > > ) >();
      for ( entity, pos ) in &mut query
      {
        existing.insert( pos.coord, entity );
      }
    }

    for ( &coord, &should_be_alive ) in next_generation
    {
      if let Some( &entity ) = existing.get( &coord )
      {
        if let Ok( mut cell ) = self.world.get_mut::< Cell >( entity )
        {
          if should_be_alive
          {
            if cell.is_alive() { cell.age(); } else { cell.revive(); }
          }
          else
          {
            cell.kill();
          }
        }
      }
      else if should_be_alive
      {
        self.world.spawn( ( Position::new( coord ), Cell::new() ) );
      }
      else
      {
        // Cell doesn't exist and shouldn't be alive — nothing to do.
      }
    }
  }

  /// Prints the hexagonal grid state.
  pub fn state_print( &self )
  {
    println!( "\nHexagonal Generation {}", self.generation );

    let mut query = self.world.query::< ( &Position< HexCoord< Axial, Pointy > >, &Cell ) >();
    let living_cells : Vec< _ > = query.iter()
    .filter( | ( _, cell ) | cell.is_alive() )
    .map( | ( pos, _ ) | ( pos.coord.q, pos.coord.r ) )
    .collect();

    println!( "Living cells: {living_cells:?}" );
  }
}

impl Default for HexGameOfLife
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Game of Life simulation on a triangular grid.
pub struct TriangularGameOfLife
{
  world : World,
  generation : u32,
}

impl TriangularGameOfLife
{
  /// Creates a new Game of Life on a triangular grid, seeded with a small
  /// four-cell patch.
  ///
  /// # Panics
  ///
  /// Panics if a seed coordinate is not a valid triangular coordinate — the
  /// pattern is hardcoded here, so a failure is an authoring mistake in this
  /// crate, not a runtime condition.
  #[ must_use ]
  pub fn new() -> Self
  {
    let mut world = World::new();

    // Spawn initial triangular pattern
    let tri_pattern = [ ( 0, 0, 1 ), ( 1, 0, 1 ), ( 2, -1, 1 ), ( 1, -1, 2 ) ];

    for &( a, b, c ) in &tri_pattern
    {
      let coord = TriCoord::< FlatSided >::new( a, b, c ).unwrap();
      world.spawn( ( Position::new( coord ), Cell::new() ) );
    }

    Self { world, generation : 0 }
  }

  /// Generations advanced so far.
  #[ must_use ]
  pub fn generation( &self ) -> u32
  {
    self.generation
  }

  /// Advances one generation with triangular grid rules.
  pub fn step( &mut self )
  {
    // Triangular grids have 12 neighbors, so different rules apply
    println!( "Triangular Generation {}: Complex neighbor relationships", self.generation + 1 );
    self.generation += 1;
  }

  /// Prints the triangular grid state.
  pub fn state_print( &self )
  {
    println!( "\nTriangular Generation {}", self.generation );

    let mut query = self.world.query::< ( &Position< TriCoord< FlatSided > >, &Cell ) >();
    let living_cells : Vec< _ > = query.iter()
    .filter( | ( _, cell ) | cell.is_alive() )
    .map( | ( pos, _ ) | ( pos.coord.a, pos.coord.b, pos.coord.c ) )
    .collect();

    println!( "Living triangular cells: {living_cells:?}" );
  }
}

impl Default for TriangularGameOfLife
{
  fn default() -> Self
  {
    Self::new()
  }
}

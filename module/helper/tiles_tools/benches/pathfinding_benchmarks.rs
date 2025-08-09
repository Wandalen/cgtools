//! Benchmarks for pathfinding algorithms
//!
//! This benchmark suite tests the performance of pathfinding algorithms
//! across different coordinate systems and grid sizes.

use criterion::{ criterion_group, criterion_main, BenchmarkId, Criterion };
use tiles_tools::
{
  pathfind::astar,
  coordinates::
  {
    hexagonal::{ Coordinate as HexCoord, Axial, Pointy },
    square::{ Coordinate as SquareCoord, FourConnected, EightConnected },
    triangular::{ Coordinate as TriCoord, TwelveConnected },
    isometric::{ Coordinate as IsoCoord, Diamond },
  },
};

fn benchmark_astar_hexagonal( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "astar_hexagonal" );
  
  for distance in [ 5, 10, 20, 50 ].iter()
  {
    group.bench_with_input( BenchmarkId::new( "straight_line", distance ), distance, |b, &distance| 
    {
      let start = HexCoord::< Axial, Pointy >::new( 0, 0 );
      let goal = HexCoord::< Axial, Pointy >::new( distance, 0 );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
    
    group.bench_with_input( BenchmarkId::new( "diagonal", distance ), distance, |b, &distance| 
    {
      let start = HexCoord::< Axial, Pointy >::new( 0, 0 );
      let goal = HexCoord::< Axial, Pointy >::new( distance, distance );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
  }
  
  group.finish();
}

fn benchmark_astar_square( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "astar_square_4connected" );
  
  for distance in [ 5, 10, 20, 50 ].iter()
  {
    group.bench_with_input( BenchmarkId::new( "straight_line", distance ), distance, |b, &distance| 
    {
      let start = SquareCoord::< FourConnected >::new( 0, 0 );
      let goal = SquareCoord::< FourConnected >::new( distance, 0 );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
    
    group.bench_with_input( BenchmarkId::new( "diagonal", distance ), distance, |b, &distance| 
    {
      let start = SquareCoord::< FourConnected >::new( 0, 0 );
      let goal = SquareCoord::< FourConnected >::new( distance, distance );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
  }
  
  group.finish();
  
  let mut group8 = c.benchmark_group( "astar_square_8connected" );
  
  for distance in [ 5, 10, 20, 50 ].iter()
  {
    group8.bench_with_input( BenchmarkId::new( "straight_line", distance ), distance, |b, &distance| 
    {
      let start = SquareCoord::< EightConnected >::new( 0, 0 );
      let goal = SquareCoord::< EightConnected >::new( distance, 0 );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
    
    group8.bench_with_input( BenchmarkId::new( "diagonal", distance ), distance, |b, &distance| 
    {
      let start = SquareCoord::< EightConnected >::new( 0, 0 );
      let goal = SquareCoord::< EightConnected >::new( distance, distance );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
  }
  
  group8.finish();
}

fn benchmark_astar_triangular( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "astar_triangular" );
  
  for distance in [ 5, 10, 20 ].iter() // Fewer distances due to higher neighbor count
  {
    group.bench_with_input( BenchmarkId::new( "straight_line", distance ), distance, |b, &distance| 
    {
      let start = TriCoord::< TwelveConnected >::new( 0, 0 );
      let goal = TriCoord::< TwelveConnected >::new( distance, 0 );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
  }
  
  group.finish();
}

fn benchmark_astar_isometric( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "astar_isometric" );
  
  for distance in [ 5, 10, 20, 50 ].iter()
  {
    group.bench_with_input( BenchmarkId::new( "straight_line", distance ), distance, |b, &distance| 
    {
      let start = IsoCoord::< Diamond >::new( 0, 0 );
      let goal = IsoCoord::< Diamond >::new( distance, 0 );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
    
    group.bench_with_input( BenchmarkId::new( "diagonal", distance ), distance, |b, &distance| 
    {
      let start = IsoCoord::< Diamond >::new( 0, 0 );
      let goal = IsoCoord::< Diamond >::new( distance, distance );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, |_| 1 )
      });
    });
  }
  
  group.finish();
}

fn benchmark_astar_with_obstacles( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "astar_with_obstacles" );
  
  // Create a maze-like obstacle pattern
  let is_passable = |coord : &SquareCoord< FourConnected >| -> bool 
  {
    // Block every other column and row to create a maze
    !( coord.x % 3 == 1 && coord.y % 2 == 1 )
  };
  
  for distance in [ 10, 20, 30 ].iter()
  {
    group.bench_with_input( BenchmarkId::new( "maze_pathfinding", distance ), distance, |b, &distance| 
    {
      let start = SquareCoord::< FourConnected >::new( 0, 0 );
      let goal = SquareCoord::< FourConnected >::new( distance, distance );
      
      b.iter( || 
      {
        astar( &start, &goal, &is_passable, |_| 1 )
      });
    });
  }
  
  group.finish();
}

fn benchmark_astar_variable_costs( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "astar_variable_costs" );
  
  // Create terrain with varying movement costs
  let terrain_cost = |coord : &HexCoord< Axial, Pointy >| -> u32 
  {
    match ( coord.q.abs() + coord.r.abs() ) % 4
    {
      0 => 1, // Plains
      1 => 2, // Hills  
      2 => 3, // Forests
      3 => 5, // Mountains
      _ => 1,
    }
  };
  
  for distance in [ 10, 20, 30 ].iter()
  {
    group.bench_with_input( BenchmarkId::new( "varied_terrain", distance ), distance, |b, &distance| 
    {
      let start = HexCoord::< Axial, Pointy >::new( 0, 0 );
      let goal = HexCoord::< Axial, Pointy >::new( distance, 0 );
      
      b.iter( || 
      {
        astar( &start, &goal, |_| true, &terrain_cost )
      });
    });
  }
  
  group.finish();
}

criterion_group!(
  benches,
  benchmark_astar_hexagonal,
  benchmark_astar_square,
  benchmark_astar_triangular,
  benchmark_astar_isometric,
  benchmark_astar_with_obstacles,
  benchmark_astar_variable_costs
);

criterion_main!( benches );
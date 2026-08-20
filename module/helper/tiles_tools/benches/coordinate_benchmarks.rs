//! Benchmarks for coordinate system operations
//!
//! This benchmark suite tests the performance of various coordinate system
//! operations including distance calculations, neighbor finding, and conversions.

use criterion::{ criterion_group, criterion_main, BenchmarkId, Criterion };
use tiles_tools::coordinates::
{
  hexagonal::{ Coordinate as HexCoord, Axial, Pointy },
  square::{ Coordinate as SquareCoord, FourConnected, EightConnected },
  triangular::{ Coordinate as TriCoord, FlatSided },
  isometric::{ Coordinate as IsoCoord, Diamond },
  conversion::{ Convert, ApproximateConvert },
  { Distance, Neighbors },
};

fn benchmark_distance_calculations( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "distance_calculations" );

  // Hexagonal distance
  let hex_origin = HexCoord::< Axial, Pointy >::new( 0, 0 );
  let hex_target = HexCoord::< Axial, Pointy >::new( 10, 15 );
  group.bench_function( "hexagonal_distance", |b| b.iter( || hex_origin.distance( hex_target ) ) );

  // Square distance (4-connected)
  let square_four_origin = SquareCoord::< FourConnected >::new( 0, 0 );
  let square_four_target = SquareCoord::< FourConnected >::new( 10, 15 );
  group.bench_function( "square_4_distance", |b| b.iter( || square_four_origin.distance( &square_four_target ) ) );

  // Square distance (8-connected)
  let square_eight_origin = SquareCoord::< EightConnected >::new( 0, 0 );
  let square_eight_target = SquareCoord::< EightConnected >::new( 10, 15 );
  group.bench_function( "square_8_distance", |b| b.iter( || square_eight_origin.distance( &square_eight_target ) ) );

  // Triangular distance
  let tri_coord1 = TriCoord::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let tri_coord2 = TriCoord::< FlatSided >::new( 10, 15, -24 ).unwrap();
  group.bench_function( "triangular_distance", |b| b.iter( || tri_coord1.distance( &tri_coord2 ) ) );

  // Isometric distance
  let iso_coord1 = IsoCoord::< Diamond >::new( 0, 0 );
  let iso_coord2 = IsoCoord::< Diamond >::new( 10, 15 );
  group.bench_function( "isometric_distance", |b| b.iter( || iso_coord1.distance( &iso_coord2 ) ) );

  group.finish();
}

fn benchmark_neighbor_calculations( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "neighbor_calculations" );

  // Hexagonal neighbors (6 neighbors)
  let hex_coord = HexCoord::< Axial, Pointy >::new( 5, 8 );
  group.bench_function( "hexagonal_neighbors", |b| b.iter( || hex_coord.neighbors() ) );

  // Square neighbors (4-connected)
  let square_four_center = SquareCoord::< FourConnected >::new( 5, 8 );
  group.bench_function( "square_4_neighbors", |b| b.iter( || square_four_center.neighbors() ) );

  // Square neighbors (8-connected)
  let square_eight_center = SquareCoord::< EightConnected >::new( 5, 8 );
  group.bench_function( "square_8_neighbors", |b| b.iter( || square_eight_center.neighbors() ) );

  // Triangular neighbors (3 neighbors)
  let tri_coord = TriCoord::< FlatSided >::new( 5, 8, -12 ).unwrap();
  group.bench_function( "triangular_neighbors", |b| b.iter( || tri_coord.neighbors() ) );

  // Isometric neighbors (4 neighbors)
  let iso_coord = IsoCoord::< Diamond >::new( 5, 8 );
  group.bench_function( "isometric_neighbors", |b| b.iter( || iso_coord.neighbors() ) );

  group.finish();
}

fn benchmark_coordinate_conversions( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "coordinate_conversions" );

  // Exact conversions: Square ↔ Isometric
  let square_coord = SquareCoord::< FourConnected >::new( 5, 8 );
  group.bench_function( "square_to_isometric", |b|
  {
  b.iter( ||
  {
    let iso : IsoCoord< Diamond > = square_coord.convert();
    iso
  });
  });

  let iso_coord = IsoCoord::< Diamond >::new( 5, 8 );
  group.bench_function( "isometric_to_square", |b|
  {
  b.iter( ||
  {
    let square : SquareCoord< FourConnected > = iso_coord.convert();
    square
  });
  });

  // Approximate conversions: Hexagonal ↔ Square
  let hex_coord = HexCoord::< Axial, Pointy >::new( 5, 8 );
  group.bench_function( "hexagonal_to_square_approx", |b|
  {
  b.iter( ||
  {
    let square : SquareCoord< FourConnected > = hex_coord.approximate_convert();
    square
  });
  });

  group.bench_function( "square_to_hexagonal_approx", |b|
  {
  b.iter( ||
  {
    let hex : HexCoord< Axial, Pointy > = square_coord.approximate_convert();
    hex
  });
  });

  group.finish();
}

fn benchmark_coordinate_creation( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "coordinate_creation" );

  for size in &[ 10, 100, 1000 ]
  {
  group.bench_with_input( BenchmarkId::new( "hexagonal_creation", size ), size, |b, &size|
  {
    b.iter( ||
    {
      let mut coords = Vec::with_capacity( size );
      for i in 0..size
      {
        coords.push( HexCoord::< Axial, Pointy >::new( i as i32, i as i32 ) );
      }
      coords
    });
  });

  group.bench_with_input( BenchmarkId::new( "square_creation", size ), size, |b, &size|
  {
    b.iter( ||
    {
      let mut coords = Vec::with_capacity( size );
      for i in 0..size
      {
        coords.push( SquareCoord::< FourConnected >::new( i as i32, i as i32 ) );
      }
      coords
    });
  });
  }

  group.finish();
}

criterion_group!(
  benches,
  benchmark_distance_calculations,
  benchmark_neighbor_calculations,
  benchmark_coordinate_conversions,
  benchmark_coordinate_creation
);

criterion_main!( benches );

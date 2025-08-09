//! Benchmarks for coordinate system operations
//!
//! This benchmark suite tests the performance of various coordinate system
//! operations including distance calculations, neighbor finding, and conversions.

#![allow(clippy::needless_return)]
#![allow(clippy::implicit_return)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::format_in_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::similar_names)]
#![allow(clippy::duplicated_attributes)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_else)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::cast_possible_wrap)]
#![allow(missing_docs)]

use criterion::{ criterion_group, criterion_main, BenchmarkId, Criterion };
use tiles_tools::coordinates::
{
  hexagonal::{ Coordinate as HexCoord, Axial, Pointy },
  square::{ Coordinate as SquareCoord, FourConnected, EightConnected },
  triangular::{ Coordinate as TriCoord, TwelveConnected },
  isometric::{ Coordinate as IsoCoord, Diamond },
  conversion::{ Convert, ApproximateConvert },
  { Distance, Neighbors },
};

fn benchmark_distance_calculations( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "distance_calculations" );
  
  // Hexagonal distance
  let hex_coord1 = HexCoord::< Axial, Pointy >::new( 0, 0 );
  let hex_coord2 = HexCoord::< Axial, Pointy >::new( 10, 15 );
  group.bench_function( "hexagonal_distance", |b| b.iter( || hex_coord1.distance( hex_coord2 ) ) );
  
  // Square distance (4-connected)
  let square_coord1 = SquareCoord::< FourConnected >::new( 0, 0 );
  let square_coord2 = SquareCoord::< FourConnected >::new( 10, 15 );
  group.bench_function( "square_4_distance", |b| b.iter( || square_coord1.distance( &square_coord2 ) ) );
  
  // Square distance (8-connected)
  let square8_coord1 = SquareCoord::< EightConnected >::new( 0, 0 );
  let square8_coord2 = SquareCoord::< EightConnected >::new( 10, 15 );
  group.bench_function( "square_8_distance", |b| b.iter( || square8_coord1.distance( &square8_coord2 ) ) );
  
  // Triangular distance
  let tri_coord1 = TriCoord::< TwelveConnected >::new( 0, 0 );
  let tri_coord2 = TriCoord::< TwelveConnected >::new( 10, 15 );
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
  let square_coord = SquareCoord::< FourConnected >::new( 5, 8 );
  group.bench_function( "square_4_neighbors", |b| b.iter( || square_coord.neighbors() ) );
  
  // Square neighbors (8-connected)
  let square8_coord = SquareCoord::< EightConnected >::new( 5, 8 );
  group.bench_function( "square_8_neighbors", |b| b.iter( || square8_coord.neighbors() ) );
  
  // Triangular neighbors (12 neighbors)
  let tri_coord = TriCoord::< TwelveConnected >::new( 5, 8 );
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
  })
  });
  
  let iso_coord = IsoCoord::< Diamond >::new( 5, 8 );
  group.bench_function( "isometric_to_square", |b| 
  {
  b.iter( || 
  {
    let square : SquareCoord< FourConnected > = iso_coord.convert();
    square
  })
  });
  
  // Approximate conversions: Hexagonal ↔ Square
  let hex_coord = HexCoord::< Axial, Pointy >::new( 5, 8 );
  group.bench_function( "hexagonal_to_square_approx", |b| 
  {
  b.iter( || 
  {
    let square : SquareCoord< FourConnected > = hex_coord.approximate_convert();
    square
  })
  });
  
  group.bench_function( "square_to_hexagonal_approx", |b| 
  {
  b.iter( || 
  {
    let hex : HexCoord< Axial, Pointy > = square_coord.approximate_convert();
    hex
  })
  });
  
  group.finish();
}

fn benchmark_coordinate_creation( c : &mut Criterion )
{
  let mut group = c.benchmark_group( "coordinate_creation" );
  
  for size in [ 10, 100, 1000 ].iter()
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
    })
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
    })
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
//! Basic tests for hexagonal grid example.

// use hexagonal_grid::*;
// use std::collections::HashSet;
// use std::hash::{ Hash, Hasher };
// use std::collections::hash_map::DefaultHasher;
//
// #[test]
// fn test_coordinate_hash()
// {
//   // Create two identical coordinates with specific types
//   let coord1 = Coordinate::< Axial, PointyTopped, OddParity >::new( 1, 2 );
//   let coord2 = Coordinate::< Axial, PointyTopped, OddParity >::new( 1, 2 );
//
//   // Create a different coordinate with specific types
//   let coord3 = Coordinate::< Axial, PointyTopped, OddParity >::new( 3, 4 );
//
//   // Verify that identical coordinates produce the same hash
//   let mut hasher1 = DefaultHasher::new();
//   coord1.hash( &mut hasher1 );
//   let hash1 = hasher1.finish();
//
//   let mut hasher2 = DefaultHasher::new();
//   coord2.hash( &mut hasher2 );
//   let hash2 = hasher2.finish();
//
//   assert_eq!( hash1, hash2, "Hashes for identical coordinates should match" );
//
//   // Verify that different coordinates produce different hashes
//   let mut hasher3 = DefaultHasher::new();
//   coord3.hash( &mut hasher3 );
//   let hash3 = hasher3.finish();
//
//   assert_ne!( hash1, hash3, "Hashes for different coordinates should not match" );
//
//   // Verify that the hash works correctly in a HashSet
//   let mut set = HashSet::new();
//   set.insert( coord1 );
//   assert!( set.contains( &coord2 ), "HashSet should recognize identical coordinates" );
//   assert!( !set.contains( &coord3 ), "HashSet should not recognize different coordinates" );
// }
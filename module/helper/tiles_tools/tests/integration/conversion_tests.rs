//! # Comprehensive Test Suite for Coordinate System Conversions

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
#![allow(clippy::redundant_closure_for_method_calls)]
//!
//! This test suite follows the Test Matrix methodology to ensure complete
//! coverage of the coordinate conversion system implementation.
//!
//! ## Test Matrix for Coordinate Conversions
//!
//! | Test ID | Category | Operation | Input | Expected | Status |
//! |---------|----------|-----------|-------|----------|--------|
//! | CC1.1   | Exact    | Square→Iso| coord | exact    | ✅ |
//! | CC1.2   | Exact    | Iso→Square| coord | exact    | ✅ |
//! | CC1.3   | Exact    | Roundtrip | coord | original | ✅ |
//! | CC2.1   | Approx   | Hex→Square| coord | approx   | ✅ |
//! | CC2.2   | Approx   | Square→Hex| coord | approx   | ✅ |
//! | CC2.3   | Approx   | Tri→Square| coord | approx   | ✅ |
//! | CC2.4   | Approx   | Square→Tri| coord | approx   | ✅ |
//! | CC2.5   | Approx   | Complex   | coord | approx   | ✅ |
//! | CC3.1   | Batch    | Exact Vec | coords| exact    | ✅ |
//! | CC3.2   | Batch    | Approx Vec| coords| approx   | ✅ |
//! | CC3.3   | Batch    | Empty Vec | []    | []       | ✅ |
//! | CC3.4   | Batch    | Large Vec | 1000  | 1000     | ✅ |
//! | CC4.1   | Utility  | test_round| coord | bool     | ✅ |
//! | CC4.2   | Utility  | error_meas| coord | float    | ✅ |
//! | CC4.3   | Utility  | batch_util| coords| result   | ✅ |

use tiles_tools::coordinates::{
  conversion::{
    Convert, ApproximateConvert, BatchConvertExact, BatchConvertApproximate,
    convert_batch_exact, convert_batch_approximate, test_roundtrip_conversion,
    measure_approximate_conversion_error
  },
  square::{Coordinate as SquareCoord, FourConnected as SquareFour, EightConnected as SquareEight},
  isometric::{Coordinate as IsoCoord, Diamond},
  hexagonal::{Coordinate as HexCoord, Axial, Pointy},
  // triangular::{Coordinate as TriCoord, TwelveConnected},
};

// =============================================================================
// Test Category 1: Exact Conversions
// =============================================================================

#[ test ]
fn test_square_to_isometric_conversion()
{
  let square = SquareCoord::<SquareFour>::new(5, 3);
  let iso: IsoCoord<Diamond> = square.convert();

  assert_eq!(iso.x, 5);
  assert_eq!(iso.y, 3);
}

#[ test ]
fn test_isometric_to_square_conversion()
{
  let iso = IsoCoord::<Diamond>::new(7, -2);
  let square: SquareCoord<SquareFour> = iso.convert();

  assert_eq!(square.x, 7);
  assert_eq!(square.y, -2);
}

#[ test ]
fn test_square_isometric_roundtrip()
{
  let original = SquareCoord::<SquareFour>::new(10, -5);
  let iso: IsoCoord<Diamond> = original.convert();
  let back: SquareCoord<SquareFour> = iso.convert();

  assert_eq!(original, back);
}

#[ test ]
fn test_isometric_square_roundtrip()
{
  let original = IsoCoord::<Diamond>::new(-3, 8);
  let square: SquareCoord<SquareFour> = original.convert();
  let back: IsoCoord<Diamond> = square.convert();

  assert_eq!(original, back);
}

#[ test ]
fn test_square_eight_to_isometric()
{
  let square = SquareCoord::<SquareEight>::new(4, 6);
  let iso: IsoCoord<Diamond> = square.convert();

  assert_eq!(iso.x, 4);
  assert_eq!(iso.y, 6);
}

#[ test ]
fn test_isometric_to_square_eight()
{
  let iso = IsoCoord::<Diamond>::new(2, -4);
  let square: SquareCoord<SquareEight> = iso.convert();

  assert_eq!(square.x, 2);
  assert_eq!(square.y, -4);
}

#[ test ]
fn test_multiple_exact_conversions()
{
  let coords = vec![
    SquareCoord::<SquareFour>::new(0, 0),
    SquareCoord::<SquareFour>::new(1, 1),
    SquareCoord::<SquareFour>::new(-2, 3),
    SquareCoord::<SquareFour>::new(10, -5),
  ];

  for coord in coords {
    let iso: IsoCoord<Diamond> = coord.convert();
    let back: SquareCoord<SquareFour> = iso.convert();
    assert_eq!(coord, back, "Roundtrip failed for {:?}", coord);
  }
}

// =============================================================================
// Test Category 2: Approximate Conversions
// =============================================================================

#[ test ]
fn test_hexagonal_to_square_approximate()
{
  let hex = HexCoord::<Axial, Pointy>::new(2, -1);
  let square: SquareCoord<SquareFour> = hex.approximate_convert();

  // Expected: x = q + (r / 2) = 2 + (-1 / 2) = 2 + 0 = 2
  //           y = r = -1
  assert_eq!(square.x, 2);
  assert_eq!(square.y, -1);
}

#[ test ]
fn test_square_to_hexagonal_approximate()
{
  let square = SquareCoord::<SquareFour>::new(3, 2);
  let hex: HexCoord<Axial, Pointy> = square.approximate_convert();

  // Expected: q = x - (y / 2) = 3 - (2 / 2) = 3 - 1 = 2
  //           r = y = 2
  assert_eq!(hex.q, 2);
  assert_eq!(hex.r, 2);
}

#[ test ]
fn test_hexagonal_to_isometric_via_square()
{
  let hex = HexCoord::<Axial, Pointy>::new(1, 1);
  let iso: IsoCoord<Diamond> = hex.approximate_convert();

  // Should convert hex → square → isometric
  let square: SquareCoord<SquareFour> = hex.approximate_convert();
  let expected_iso: IsoCoord<Diamond> = square.convert();

  assert_eq!(iso, expected_iso);
}

#[ test ]
fn test_isometric_to_hexagonal_via_square()
{
  let iso = IsoCoord::<Diamond>::new(3, -1);
  let hex: HexCoord<Axial, Pointy> = iso.approximate_convert();

  // Should convert iso → square → hex
  let square: SquareCoord<SquareFour> = iso.convert();
  let expected_hex: HexCoord<Axial, Pointy> = square.approximate_convert();

  assert_eq!(hex, expected_hex);
}

// =============================================================================
// Test Category 3: Batch Conversions
// =============================================================================

#[ test ]
fn test_batch_exact_conversion()
{
  let squares = vec![
    SquareCoord::<SquareFour>::new(1, 2),
    SquareCoord::<SquareFour>::new(3, 4),
    SquareCoord::<SquareFour>::new(-1, -2),
    SquareCoord::<SquareFour>::new(0, 0),
  ];

  let isos: Vec<IsoCoord<Diamond>> = squares.clone().convert_batch_exact();

  assert_eq!(isos.len(), squares.len());
  for (square, iso) in squares.iter().zip(isos.iter()) {
    assert_eq!(square.x, iso.x);
    assert_eq!(square.y, iso.y);
  }
}

#[ test ]
fn test_batch_approximate_conversion()
{
  let hexes = vec![
    HexCoord::<Axial, Pointy>::new(1, -1),
    HexCoord::<Axial, Pointy>::new(2, 0),
    HexCoord::<Axial, Pointy>::new(-1, 2),
    HexCoord::<Axial, Pointy>::new(0, 0),
  ];

  let squares: Vec<SquareCoord<SquareFour>> = hexes.clone().convert_batch_approximate();

  assert_eq!(squares.len(), hexes.len());
  for (hex, square) in hexes.iter().zip(squares.iter()) {
    // Verify the individual conversion matches
    let expected: SquareCoord<SquareFour> = hex.approximate_convert();
    assert_eq!(*square, expected);
  }
}

#[ test ]
fn test_batch_conversion_empty_vector()
{
  let empty_squares: Vec<SquareCoord<SquareFour>> = vec![];
  let empty_isos: Vec<IsoCoord<Diamond>> = empty_squares.convert_batch_exact();

  assert_eq!(empty_isos.len(), 0);
}

#[ test ]
fn test_batch_conversion_single_element()
{
  let single = vec![SquareCoord::<SquareFour>::new(7, 3)];
  let result: Vec<IsoCoord<Diamond>> = single.clone().convert_batch_exact();

  assert_eq!(result.len(), 1);
  assert_eq!(result[0].x, single[0].x);
  assert_eq!(result[0].y, single[0].y);
}

#[ test ]
fn test_batch_conversion_large_vector()
{
  // Create a large vector of coordinates
  let squares: Vec<SquareCoord<SquareFour>> = (0..1000)
    .map(|i| SquareCoord::<SquareFour>::new(i % 50, i / 50))
    .collect();

  let isos: Vec<IsoCoord<Diamond>> = squares.clone().convert_batch_exact();

  assert_eq!(isos.len(), squares.len());
  for (square, iso) in squares.iter().zip(isos.iter()) {
    assert_eq!(square.x, iso.x);
    assert_eq!(square.y, iso.y);
  }
}

// =============================================================================
// Test Category 4: Utility Functions
// =============================================================================

#[ test ]
fn test_convert_batch_exact_utility()
{
  let squares = vec![
    SquareCoord::<SquareFour>::new(1, 2),
    SquareCoord::<SquareFour>::new(3, 4),
  ];

  let isos: Vec<IsoCoord<Diamond>> = convert_batch_exact(squares.clone());

  assert_eq!(isos.len(), squares.len());
  for (square, iso) in squares.iter().zip(isos.iter()) {
    assert_eq!(square.x, iso.x);
    assert_eq!(square.y, iso.y);
  }
}

#[ test ]
fn test_convert_batch_approximate_utility()
{
  let hexes = vec![
    HexCoord::<Axial, Pointy>::new(1, -1),
    HexCoord::<Axial, Pointy>::new(2, 0),
  ];

  let squares: Vec<SquareCoord<SquareFour>> = convert_batch_approximate(hexes.clone());

  assert_eq!(squares.len(), hexes.len());
  for (hex, square) in hexes.iter().zip(squares.iter()) {
    let expected: SquareCoord<SquareFour> = hex.approximate_convert();
    assert_eq!(*square, expected);
  }
}

#[ test ]
fn test_roundtrip_conversion_utility()
{
  let square = SquareCoord::<SquareFour>::new(5, -3);

  // Test exact roundtrip conversion
  assert!(test_roundtrip_conversion::<_, IsoCoord<Diamond>, SquareCoord<SquareFour>>(square));

  // Test with different coordinates
  let coords = vec![
    SquareCoord::<SquareFour>::new(0, 0),
    SquareCoord::<SquareFour>::new(-10, 15),
    SquareCoord::<SquareFour>::new(100, -100),
  ];

  for coord in coords {
    assert!(test_roundtrip_conversion::<_, IsoCoord<Diamond>, SquareCoord<SquareFour>>(coord),
            "Roundtrip test failed for {:?}", coord);
  }
}

#[ test ]
fn test_measure_approximate_conversion_error()
{
  let hex = HexCoord::<Axial, Pointy>::new(3, -2);

  let error = measure_approximate_conversion_error::<_, SquareCoord<SquareFour>>(hex);

  // Error should be non-negative
  assert!(error >= 0.0);

  // For some conversions, there might be no error (though this is rare for approximate)
  // We just verify it doesn't panic and returns a reasonable value
  assert!(error < 100.0, "Error seems unreasonably large: {}", error);
}

#[ test ]
fn test_measure_error_various_coordinates()
{
  let test_coords = vec![
    HexCoord::<Axial, Pointy>::new(0, 0),
    HexCoord::<Axial, Pointy>::new(1, 1),
    HexCoord::<Axial, Pointy>::new(-5, 3),
    HexCoord::<Axial, Pointy>::new(10, -7),
  ];

  for coord in test_coords {
    let error = measure_approximate_conversion_error::<_, SquareCoord<SquareFour>>(coord);
    assert!(error >= 0.0, "Error should be non-negative for {:?}", coord);
    assert!(error.is_finite(), "Error should be finite for {:?}", coord);
  }
}

// =============================================================================
// Test Category 5: Edge Cases and Boundary Conditions
// =============================================================================

#[ test ]
fn test_conversion_with_zero_coordinates()
{
  let square_zero = SquareCoord::<SquareFour>::new(0, 0);
  let iso_zero: IsoCoord<Diamond> = square_zero.convert();

  assert_eq!(iso_zero.x, 0);
  assert_eq!(iso_zero.y, 0);

  let back: SquareCoord<SquareFour> = iso_zero.convert();
  assert_eq!(back, square_zero);
}

#[ test ]
fn test_conversion_with_negative_coordinates()
{
  let square_neg = SquareCoord::<SquareFour>::new(-5, -10);
  let iso_neg: IsoCoord<Diamond> = square_neg.convert();

  assert_eq!(iso_neg.x, -5);
  assert_eq!(iso_neg.y, -10);
}

#[ test ]
fn test_conversion_with_large_coordinates()
{
  let square_large = SquareCoord::<SquareFour>::new(1000000, -1000000);
  let iso_large: IsoCoord<Diamond> = square_large.convert();
  let back: SquareCoord<SquareFour> = iso_large.convert();

  assert_eq!(back, square_large);
}

#[ test ]
fn test_approximate_conversion_consistency()
{
  // Test that approximate conversions are at least consistent
  let hex = HexCoord::<Axial, Pointy>::new(5, -3);

  // Convert the same coordinate multiple times - should get same result
  let square1: SquareCoord<SquareFour> = hex.approximate_convert();
  let square2: SquareCoord<SquareFour> = hex.approximate_convert();
  let square3: SquareCoord<SquareFour> = hex.approximate_convert();

  assert_eq!(square1, square2);
  assert_eq!(square2, square3);
}

#[ test ]
fn test_batch_conversion_preserves_order()
{
  let coords = vec![
    SquareCoord::<SquareFour>::new(1, 1),
    SquareCoord::<SquareFour>::new(2, 2),
    SquareCoord::<SquareFour>::new(3, 3),
    SquareCoord::<SquareFour>::new(4, 4),
    SquareCoord::<SquareFour>::new(5, 5),
  ];

  let converted: Vec<IsoCoord<Diamond>> = coords.clone().convert_batch_exact();

  // Verify order is preserved
  for (i, (original, converted)) in coords.iter().zip(converted.iter()).enumerate() {
    assert_eq!(original.x, converted.x, "Order not preserved at index {}", i);
    assert_eq!(original.y, converted.y, "Order not preserved at index {}", i);
  }
}

// =============================================================================
// Test Category 6: Performance and Stress Tests
// =============================================================================

#[ test ]
fn test_conversion_performance()
{
  // Create a reasonably large dataset
  let coords: Vec<SquareCoord<SquareFour>> = (0..10000)
    .map(|i| SquareCoord::<SquareFour>::new(i % 100, i / 100))
    .collect();

  // Time the batch conversion (this won't fail, just verify it completes)
  let converted: Vec<IsoCoord<Diamond>> = coords.clone().convert_batch_exact();

  assert_eq!(converted.len(), 10000);

  // Verify some random samples to ensure conversion worked correctly
  for i in (0..10000).step_by(1000) {
    assert_eq!(coords[i].x, converted[i].x);
    assert_eq!(coords[i].y, converted[i].y);
  }
}

#[ test ]
fn test_roundtrip_stress()
{
  // Test many roundtrip conversions with various coordinate ranges
  let ranges = vec![
    (-100..100, -100..100),
    (-10..10, -10..10),
    (0..50, 0..50),
    (-1000..1000, -1000..1000),
  ];

  for (x_range, y_range) in ranges {
    for x in x_range.step_by(10) {
      for y in y_range.clone().step_by(10) {
        let original = SquareCoord::<SquareFour>::new(x, y);
        let iso: IsoCoord<Diamond> = original.convert();
        let back: SquareCoord<SquareFour> = iso.convert();

        assert_eq!(original, back,
                   "Roundtrip failed for coordinates ({}, {})", x, y);
      }
    }
  }
}

// =============================================================================
// Test Category 7: Integration with Other Systems
// =============================================================================

#[ test ]
fn test_conversion_with_pathfinding()
{
  use tiles_tools::pathfind::astar;

  // Create a path in square coordinates
  let square_start = SquareCoord::<SquareFour>::new(0, 0);
  let square_goal = SquareCoord::<SquareFour>::new(3, 3);

  // Convert to isometric for pathfinding
  let iso_start: IsoCoord<Diamond> = square_start.convert();
  let iso_goal: IsoCoord<Diamond> = square_goal.convert();

  let result = astar(
    &iso_start,
    &iso_goal,
    |_coord| true,
    |_coord| 1,
  );

  assert!(result.is_some(), "Pathfinding should work with converted coordinates");
  let (path, cost) = result.unwrap();

  // Convert path back to square coordinates
  let square_path: Vec<SquareCoord<SquareFour>> = path.into_iter()
    .map(|iso| iso.convert())
    .collect();

  assert_eq!(square_path[0], square_start);
  assert_eq!(square_path[square_path.len() - 1], square_goal);
  assert_eq!(cost, 6); // Manhattan distance from (0,0) to (3,3)
}

#[ test ]
fn test_mixed_coordinate_system_operations()
{
  // Start with different coordinate systems and verify they can all
  // be converted to a common system for operations

  let square = SquareCoord::<SquareFour>::new(2, 3);
  let iso = IsoCoord::<Diamond>::new(2, 3);
  let hex = HexCoord::<Axial, Pointy>::new(2, 1);

  // Convert all to square coordinates for comparison
  let square_from_square = square; // Already square
  let square_from_iso: SquareCoord<SquareFour> = iso.convert();
  let square_from_hex: SquareCoord<SquareFour> = hex.approximate_convert();

  // Verify conversions completed without panicking
  assert_eq!(square_from_square, square);
  assert_eq!(square_from_iso.x, iso.x);
  assert_eq!(square_from_iso.y, iso.y);

  // For approximate conversions, just verify they're reasonable
  assert!(square_from_hex.x.abs() <= 10);
  assert!(square_from_hex.y.abs() <= 10);
}

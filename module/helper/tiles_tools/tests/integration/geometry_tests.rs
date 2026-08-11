//! Integration tests for the hexagon geometry generators.
//!
//! Pins the contracts documented in `docs/algorithm/004_hexagon_geometry_generation.md`:
//!
//! | Test ID | Generator | Contract Pinned |
//! |---------|-----------|-----------------|
//! | GG1.1   | `hexagon_vertices` | 6 corners, CCW from ( 1, 0 ), vertex `i` at `60° × i`, unit radius |
//! | GG2.1   | `hexagon_triangles` | 24 floats ( 4 standalone triangles ) |
//! | GG2.2   | `hexagon_triangles` | Summed triangle area equals the analytic unit-hexagon area |
//! | GG3.1   | `hexagon_lines` | 24 floats ( 6 standalone segments ) |
//! | GG3.2   | `hexagon_lines` | Segments join adjacent corners; last segment closes back to corner 0 |
//! | GG4.1   | `hexagon_triangles_with_transform` | Matches transforming each position of the untransformed fill |
//! | GG5.1   | `from_iter` | One shape copy per coordinate, offset by ( x, -y ) of the cell's pixel position |

use tiles_tools::geometry::{ from_iter, hexagon_lines, hexagon_triangles, hexagon_triangles_with_transform, hexagon_vertices };
use tiles_tools::coordinates::pixel::Pixel;
use ndarray_cg::{ F32x3x3, Vector };

const EPS : f32 = 1e-5;

/// GG1.1: six corners, counterclockwise from ( 1, 0 ), vertex `i` at angle
/// `60° × i`, each at distance 1.0 from the center.
#[ test ]
fn hexagon_vertices_pins_count_order_radius()
{
  let points = hexagon_vertices();
  assert_eq!( points.len(), 6 );

  for ( i, point ) in points.iter().enumerate()
  {
    let angle = ( ( 60 * i ) as f32 ).to_radians();
    assert!( ( point[ 0 ] - angle.cos() ).abs() < EPS, "vertex {i} x: {} vs {}", point[ 0 ], angle.cos() );
    assert!( ( point[ 1 ] - angle.sin() ).abs() < EPS, "vertex {i} y: {} vs {}", point[ 1 ], angle.sin() );

    let radius = ( point[ 0 ] * point[ 0 ] + point[ 1 ] * point[ 1 ] ).sqrt();
    assert!( ( radius - 1.0 ).abs() < EPS, "vertex {i} radius: {radius}" );
  }

  // First corner is exactly ( 1, 0 ) — the documented starting point of the CCW walk.
  assert!( ( points[ 0 ][ 0 ] - 1.0 ).abs() < EPS );
  assert!( points[ 0 ][ 1 ].abs() < EPS );
}

/// GG2.1 + GG2.2: 4 standalone triangles ( 24 floats ), whose summed area is the
/// analytic area of a unit-radius regular hexagon, `3 * sqrt( 3 ) / 2`. A fill
/// that dropped, duplicated, or degenerated a triangle would break the sum.
#[ test ]
fn hexagon_triangles_pins_count_and_area()
{
  let positions = hexagon_triangles();
  assert_eq!( positions.len(), 24, "4 triangles x 3 vertices x 2 floats" );

  let mut area_sum = 0.0f32;
  for triangle in positions.chunks( 6 )
  {
    let ( ax, ay ) = ( triangle[ 0 ], triangle[ 1 ] );
    let ( bx, by ) = ( triangle[ 2 ], triangle[ 3 ] );
    let ( cx, cy ) = ( triangle[ 4 ], triangle[ 5 ] );
    area_sum += 0.5 * ( ( bx - ax ) * ( cy - ay ) - ( cx - ax ) * ( by - ay ) ).abs();
  }

  let analytic = 3.0 * 3.0f32.sqrt() / 2.0;
  assert!( ( area_sum - analytic ).abs() < 1e-4, "area {area_sum} vs analytic {analytic}" );
}

/// GG3.1 + GG3.2: 6 standalone segments ( 24 floats ) — segments 0..=4 connect
/// corner `i` to corner `i + 1`, and segment 5 explicitly closes the outline
/// from corner 5 back to corner 0 ( no `LINE_LOOP` mode needed ).
#[ test ]
fn hexagon_lines_pins_segments_and_closure()
{
  let positions = hexagon_lines();
  assert_eq!( positions.len(), 24, "6 segments x 2 endpoints x 2 floats" );

  let corners = hexagon_vertices();
  for ( i, segment ) in positions.chunks( 4 ).enumerate()
  {
    let from = corners[ i % 6 ];
    let to = corners[ ( i + 1 ) % 6 ];
    assert!( ( segment[ 0 ] - from[ 0 ] ).abs() < EPS, "segment {i} start x" );
    assert!( ( segment[ 1 ] - from[ 1 ] ).abs() < EPS, "segment {i} start y" );
    assert!( ( segment[ 2 ] - to[ 0 ] ).abs() < EPS, "segment {i} end x" );
    assert!( ( segment[ 3 ] - to[ 1 ] ).abs() < EPS, "segment {i} end y" );
  }
}

/// GG4.1: the transform variant equals applying the same homogeneous transform
/// to every position of the untransformed fill — triangle assembly only copies
/// corner coordinates, so per-corner and per-position transformation agree.
#[ test ]
fn hexagon_triangles_with_transform_matches_manual_transformation()
{
  // Scale by 2, translate by ( 0.5, -1.0 ).
  let transform = F32x3x3::from_row_major
  (
    [
      2.0, 0.0, 0.5,
      0.0, 2.0, -1.0,
      0.0, 0.0, 1.0,
    ]
  );

  let transformed = hexagon_triangles_with_transform( transform );
  let untransformed = hexagon_triangles();
  assert_eq!( transformed.len(), untransformed.len() );

  for ( i, position ) in untransformed.chunks( 2 ).enumerate()
  {
    let expected = transform * Vector( [ position[ 0 ], position[ 1 ], 1.0 ] );
    assert!( ( transformed[ 2 * i ] - expected.x() ).abs() < EPS, "position {i} x" );
    assert!( ( transformed[ 2 * i + 1 ] - expected.y() ).abs() < EPS, "position {i} y" );
  }
}

/// GG5.1: `from_iter` concatenates one copy of the produced shape per
/// coordinate, each copy offset by the cell's pixel position with its y
/// negated ( pixel-space y points down; geometry-space y points up ).
#[ test ]
fn from_iter_replicates_shape_per_cell()
{
  let cells = [Pixel::new( 3.0, 2.0 ), Pixel::new( -1.0, 0.5 )];
  let shape = hexagon_triangles();

  let positions = from_iter( cells.iter().copied(), hexagon_triangles, F32x3x3::identity() );
  assert_eq!( positions.len(), cells.len() * shape.len() );

  for ( cell_index, cell_positions ) in positions.chunks( shape.len() ).enumerate()
  {
    let cell = cells[ cell_index ];
    for ( i, position ) in shape.chunks( 2 ).enumerate()
    {
      let expected_x = cell.x() + position[ 0 ];
      let expected_y = -cell.y() + position[ 1 ];
      assert!( ( cell_positions[ 2 * i ] - expected_x ).abs() < EPS, "cell {cell_index} position {i} x" );
      assert!( ( cell_positions[ 2 * i + 1 ] - expected_y ).abs() < EPS, "cell {cell_index} position {i} y" );
    }
  }
}

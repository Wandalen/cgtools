
use super::*;
use line_tools::d3;

#[ test ]
fn test_distance_single_point()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 5.0, 3.0, 1.0 ] );

  let expected = [ 0.0 ];
  assert_eq!( expected, line.distances_get() );
  assert_eq!( 0.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_two_points()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 4.0, 0.0 ] );

  let expected = [ 0.0, 5.0 ];
  assert_eq!( expected, line.distances_get() );
  assert_eq!( 5.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_diagonal_segments()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );
  line.point_add_back( &[ 2.0, 2.0, 2.0 ] );

  let d = ( 3.0_f32 ).sqrt();
  let expected = [ 0.0, d, d * 2.0 ];
  assert_eq!( expected, line.distances_get() );
  assert_eq!( d * 2.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_add_back_duplicate_ignored()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  let expected = [ 0.0, 1.0, 2.0 ];
  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_mixed_add_front_and_back()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_front( &[ -1.0, 0.0, 0.0 ] );

  let expected = [ 0.0, 1.0, 2.0 ];
  assert_eq!( expected, line.distances_get() );
  assert_eq!( 2.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_clear_and_rebuild()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  line.clear();
  assert_eq!( 0, line.distances_get().len() );
  assert_eq!( 0.0, line.total_distance_get() );

  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 0.0, 3.0, 4.0 ] );

  let expected = [ 0.0, 5.0 ];
  assert_eq!( expected, line.distances_get() );
  assert_eq!( 5.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_point_remove_by_index()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  // Remove the middle point at index 1
  line.point_remove( 1 );

  // Now line is: (0,0,0) -> (1,1,0) -> (1,1,1)
  let d01 = ( 2.0_f32 ).sqrt();
  let d12 = 1.0;
  let expected = [ 0.0, d01, d01 + d12 ];
  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_point_remove_to_single()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );

  line.point_remove_back();

  let expected = [ 0.0 ];
  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_remove_all_points()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );

  line.point_remove_back();
  line.point_remove_back();

  assert_eq!( 0, line.distances_get().len() );
}

#[ test ]
fn test_distance_point_set_first()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  // Move the first point; all distances should recalculate
  line.point_set( [ 0.0, 1.0, 0.0 ], 0 );

  // (0,1,0) -> (1,0,0): sqrt(2), (1,0,0) -> (2,0,0): 1
  let d01 = ( 2.0_f32 ).sqrt();
  let expected = [ 0.0, d01, d01 + 1.0 ];
  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_point_set_last()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  // Move the last point
  line.point_set( [ 1.0, 1.0, 0.0 ], 2 );

  // (0,0,0) -> (1,0,0): 1, (1,0,0) -> (1,1,0): 1
  let expected = [ 0.0, 1.0, 2.0 ];
  assert_eq!( expected, line.distances_get() );
  assert_eq!( 2.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_total_distance_consistency()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 4.0, 0.0 ] );

  assert_eq!( 7.0, line.total_distance_get() );

  let last_distance = *line.distances_get().last().unwrap();
  assert_eq!( last_distance, line.total_distance_get() );
}

#[ test ]
fn test_distance_add_after_remove()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  line.point_remove_back();

  // Now add a different point
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );

  // (0,0,0) -> (1,0,0): 1, (1,0,0) -> (1,1,0): 1
  let expected = [ 0.0, 1.0, 2.0 ];
  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_points_add_back_batch()
{
  let mut line = d3::Line::default();
  let points : Vec< [ f32; 3 ] > = vec!
  [
    [ 0.0, 0.0, 0.0 ],
    [ 1.0, 0.0, 0.0 ],
    [ 1.0, 1.0, 0.0 ],
  ];
  line.points_add_back( &points );

  let expected = [ 0.0, 1.0, 2.0 ];
  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_points_add_front_batch()
{
  let mut line = d3::Line::default();
  let points : Vec< [ f32; 3 ] > = vec!
  [
    [ 0.0, 0.0, 0.0 ],
    [ 1.0, 0.0, 0.0 ],
    [ 1.0, 1.0, 0.0 ],
  ];
  line.points_add_front( &points );

  let expected = [ 0.0, 1.0, 2.0 ];
  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_point_add_back()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  let expected = 
  [
    0.0,
    1.0,
    2.0,
    3.0
  ];

  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_point_add_front()
{
  let mut line = d3::Line::default();
  line.point_add_front( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 1.0, 1.0 ] );

  let expected = 
  [
    0.0,
    1.0,
    2.0,
    3.0
  ];

  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_add_at_the_same_position()
{
  let mut line = d3::Line::default();
  line.point_add_front( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 1.0, 0.0 ] );

  let expected = 
  [
    0.0,
    1.0,
    2.0,
  ];

  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_point_set()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 2.0, 1.0 ] );

  line.point_set( [ 0.0, 2.0, 0.0 ], 1 );
  line.point_set( [ 0.0, 2.0, 1.0 ], 2 );
  line.point_set( [ 1.0, 2.0, 1.0 ], 3 );

  let expected = 
  [
    0.0,
    2.0,
    3.0,
    4.0
  ];

  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_point_remove_back()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  line.point_remove_back();

  let expected = 
  [
    0.0,
    1.0,
    2.0
  ];

  assert_eq!( expected, line.distances_get() );

  line.point_remove_back();

  let expected = 
  [
    0.0,
    1.0
  ];

  assert_eq!( expected, line.distances_get() );
}

#[ test ]
fn test_distance_point_remove_front()
{
  let mut line = d3::Line::default();
  line.point_add_front( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 1.0, 1.0 ] );

  line.point_remove_front();

  let expected = 
  [
    0.0,
    1.0,
    2.0
  ];

  assert_eq!( expected, line.distances_get() );

  line.point_remove_front();

  let expected =
  [
    0.0,
    1.0
  ];

  assert_eq!( expected, line.distances_get() );
}

// === Remove back edge cases ===

#[ test ]
fn test_distance_remove_back_all_one_by_one()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  line.point_remove_back();
  assert_eq!( [ 0.0, 1.0, 2.0 ], line.distances_get() );
  assert_eq!( 2.0, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );
  assert_eq!( 1.0, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( [ 0.0 ], line.distances_get() );
  assert_eq!( 0.0, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( 0, line.distances_get().len() );
  assert_eq!( 0.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_remove_back_then_add_back()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );

  // Remove two from back
  line.point_remove_back();
  line.point_remove_back();
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );
  assert_eq!( 1.0, line.total_distance_get() );

  // Add two different points back
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  assert_eq!( [ 0.0, 1.0, 2.0, 3.0 ], line.distances_get() );
  assert_eq!( 3.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_remove_back_non_unit_segments()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 4.0, 0.0 ] ); // dist = 5
  line.point_add_back( &[ 6.0, 8.0, 0.0 ] ); // dist = 5
  line.point_add_back( &[ 6.0, 8.0, 12.0 ] ); // dist = 12

  assert_eq!( 22.0, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( [ 0.0, 5.0, 10.0 ], line.distances_get() );
  assert_eq!( 10.0, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( [ 0.0, 5.0 ], line.distances_get() );
  assert_eq!( 5.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_points_remove_back_batch()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 4.0, 0.0, 0.0 ] );

  line.points_remove_back( 3 );

  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );
  assert_eq!( 1.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_points_remove_back_all()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  line.points_remove_back( 3 );

  assert_eq!( 0, line.distances_get().len() );
  assert_eq!( 0.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_points_remove_back_more_than_length()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );

  line.points_remove_back( 5 );

  assert_eq!( 0, line.distances_get().len() );
}

#[ test ]
fn test_distance_remove_back_then_add_front()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  line.point_remove_back();

  // Add to front instead
  line.point_add_front( &[ -1.0, 0.0, 0.0 ] );

  assert_eq!( [ 0.0, 1.0, 2.0 ], line.distances_get() );
  assert_eq!( 2.0, line.total_distance_get() );
}

// === Remove front edge cases ===

#[ test ]
fn test_distance_remove_front_all_one_by_one()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  line.point_remove_front();
  assert_eq!( [ 0.0, 1.0, 2.0 ], line.distances_get() );

  line.point_remove_front();
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );

  line.point_remove_front();
  assert_eq!( [ 0.0 ], line.distances_get() );

  line.point_remove_front();
  assert_eq!( 0, line.distances_get().len() );
}

#[ test ]
fn test_distance_remove_front_then_add_front()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );

  // Remove two from front
  line.point_remove_front();
  line.point_remove_front();
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );

  // Add two new points to front
  line.point_add_front( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 0.0, 0.0, 0.0 ] );

  assert_eq!( [ 0.0, 1.0, 2.0, 3.0 ], line.distances_get() );
}

#[ test ]
fn test_distance_remove_front_then_add_back()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  line.point_remove_front();
  // Line is now: (1,0,0) -> (2,0,0), distances [0, 1]
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );

  // Add to back
  line.point_add_back( &[ 2.0, 1.0, 0.0 ] );

  assert_eq!( [ 0.0, 1.0, 2.0 ], line.distances_get() );
  assert_eq!( 2.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_remove_front_non_unit_segments()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 4.0, 0.0 ] ); // segment = 5
  line.point_add_back( &[ 3.0, 4.0, 12.0 ] ); // segment = 12
  line.point_add_back( &[ 3.0, 5.0, 12.0 ] ); // segment = 1

  assert_eq!( [ 0.0, 5.0, 17.0, 18.0 ], line.distances_get() );

  line.point_remove_front();
  // Remaining: (3,4,0) -> (3,4,12) -> (3,5,12)
  assert_eq!( [ 0.0, 12.0, 13.0 ], line.distances_get() );

  line.point_remove_front();
  // Remaining: (3,4,12) -> (3,5,12)
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );
}

#[ test ]
fn test_distance_points_remove_front_batch()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 4.0, 0.0, 0.0 ] );

  line.points_remove_front( 3 );

  // Remaining: (3,0,0) -> (4,0,0)
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );
}

#[ test ]
fn test_distance_points_remove_front_all()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );

  line.points_remove_front( 3 );

  assert_eq!( 0, line.distances_get().len() );
}

#[ test ]
fn test_distance_points_remove_front_more_than_length()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );

  line.points_remove_front( 5 );

  assert_eq!( 0, line.distances_get().len() );
}

// === Mixed remove front and back ===

#[ test ]
fn test_distance_remove_front_and_back_alternating()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 4.0, 0.0, 0.0 ] );

  line.point_remove_front();
  // (1,0,0) -> (2,0,0) -> (3,0,0) -> (4,0,0)
  assert_eq!( [ 0.0, 1.0, 2.0, 3.0 ], line.distances_get() );

  line.point_remove_back();
  // (1,0,0) -> (2,0,0) -> (3,0,0)
  assert_eq!( [ 0.0, 1.0, 2.0 ], line.distances_get() );

  line.point_remove_front();
  // (2,0,0) -> (3,0,0)
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );

  line.point_remove_back();
  // (2,0,0)
  assert_eq!( [ 0.0 ], line.distances_get() );
  assert_eq!( 0.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_remove_front_and_back_to_empty()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );

  line.point_remove_front();
  line.point_remove_back();

  assert_eq!( 0, line.distances_get().len() );
}

#[ test ]
fn test_distance_remove_front_and_back_then_rebuild()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );

  // Shrink from both ends
  line.point_remove_front();
  line.point_remove_back();
  // Remaining: (1,0,0) -> (2,0,0)
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );
  assert_eq!( 1.0, line.total_distance_get() );

  // Grow from both ends
  line.point_add_front( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );

  assert_eq!( [ 0.0, 1.0, 2.0, 3.0 ], line.distances_get() );
  assert_eq!( 3.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_remove_back_total_distance_stays_consistent()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 4.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 4.0, 12.0 ] );

  // total = 3 + 4 + 12 = 19
  assert_eq!( 19.0, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( 7.0, line.total_distance_get() );
  let last = *line.distances_get().last().unwrap();
  assert_eq!( last, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( 3.0, line.total_distance_get() );
  let last = *line.distances_get().last().unwrap();
  assert_eq!( last, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( 0.0, line.total_distance_get() );
  let last = *line.distances_get().last().unwrap();
  assert_eq!( last, line.total_distance_get() );
}

#[ test ]
fn test_distance_remove_front_total_distance_stays_consistent()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 4.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 4.0, 12.0 ] );

  line.point_remove_front();
  // (3,0,0) -> (3,4,0) -> (3,4,12): 4 + 12 = 16
  assert_eq!( [ 0.0, 4.0, 16.0 ], line.distances_get() );
  let last = *line.distances_get().last().unwrap();
  assert_eq!( last, line.total_distance_get() );

  line.point_remove_front();
  // (3,4,0) -> (3,4,12): 12
  assert_eq!( [ 0.0, 12.0 ], line.distances_get() );
  let last = *line.distances_get().last().unwrap();
  assert_eq!( last, line.total_distance_get() );
}

#[ test ]
fn test_distance_batch_remove_back_then_batch_remove_front()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 2.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 3.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 4.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 5.0, 0.0, 0.0 ] );

  line.points_remove_back( 2 );
  // (0) -> (1) -> (2) -> (3)
  assert_eq!( [ 0.0, 1.0, 2.0, 3.0 ], line.distances_get() );
  assert_eq!( 3.0, line.total_distance_get() );

  line.points_remove_front( 2 );
  // (2) -> (3)
  assert_eq!( [ 0.0, 1.0 ], line.distances_get() );
  assert_eq!( 1.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_remove_back_single_point_line()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 5.0, 5.0, 5.0 ] );

  assert_eq!( [ 0.0 ], line.distances_get() );
  assert_eq!( 0.0, line.total_distance_get() );

  line.point_remove_back();
  assert_eq!( 0, line.distances_get().len() );
  assert_eq!( 0.0, line.total_distance_get() );
}

#[ test ]
fn test_distance_remove_front_single_point_line()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 5.0, 5.0, 5.0 ] );

  line.point_remove_front();
  assert_eq!( 0, line.distances_get().len() );
}

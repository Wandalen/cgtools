
use super::*;
use line_tools::d3;

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

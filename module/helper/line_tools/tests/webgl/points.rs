use super::*;
use line_tools::d3;

#[ test ]
fn test_point_add_back()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  let expected = 
  [
    gl::F32x3::new( 0.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 1.0 ),
  ];

  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_points_add_back()
{
  let mut line = d3::Line::default();
  line.points_add_back( &[ 
    [ 0.0, 0.0, 0.0 ],
    [ 1.0, 0.0, 0.0 ],
    [ 1.0, 1.0, 0.0 ],
    [ 1.0, 1.0, 1.0 ]
  ] );

  let expected = 
  [
    gl::F32x3::new( 0.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 1.0 ),
  ];

  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_point_add_front()
{
  let mut line = d3::Line::default();
  line.point_add_front( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_front( &[ 1.0, 1.0, 1.0 ] );

  let expected = 
  [
    gl::F32x3::new( 1.0, 1.0, 1.0 ),
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
    gl::F32x3::new( 0.0, 0.0, 0.0 ),
  ];

  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_points_add_front()
{
  let mut line = d3::Line::default();
  line.points_add_front( &[ 
    [ 0.0, 0.0, 0.0 ],
    [ 1.0, 0.0, 0.0 ],
    [ 1.0, 1.0, 0.0 ],
    [ 1.0, 1.0, 1.0 ]
  ] );

  let expected = 
  [
    gl::F32x3::new( 1.0, 1.0, 1.0 ),
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
    gl::F32x3::new( 0.0, 0.0, 0.0 ),
  ];

  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_point_set()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  line.point_set( [ 0.5, 0.5, 0.5 ], 0 );
  line.point_set( [ 0.5, 0.5, 0.5 ], 3 );
  line.point_set( [ 0.5, 0.5, 0.5 ], 7 );

  let expected = 
  [
    gl::F32x3::new( 0.5, 0.5, 0.5 ),
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
    gl::F32x3::new( 0.5, 0.5, 0.5 ),
  ];

  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_point_remove()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  let p0 = line.point_remove( 0 );
  let p1 = line.point_remove( 2 );
  let p2 = line.point_remove( 2 );

  let expected = 
  [
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
  ];

  assert_eq!( p0, Some( gl::F32x3::new( 0.0, 0.0, 0.0 ) ) );
  assert_eq!( p1, Some( gl::F32x3::new( 1.0, 1.0, 1.0 ) ) );
  assert_eq!( p2, None );
  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_point_remove_back()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  let p0 = line.point_remove_back();
  let p1 = line.point_remove_back();

  let expected = 
  [
    gl::F32x3::new( 0.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
  ];

  assert_eq!( p0, Some( gl::F32x3::new( 1.0, 1.0, 1.0 ) ) );
  assert_eq!( p1, Some( gl::F32x3::new( 1.0, 1.0, 0.0 ) ) );
  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_point_remove_front()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  let p0 = line.point_remove_front();
  let p1 = line.point_remove_front();

  let expected = 
  [
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 1.0 ),
  ];

  assert_eq!( p0, Some( gl::F32x3::new( 0.0, 0.0, 0.0 ) ) );
  assert_eq!( p1, Some( gl::F32x3::new( 1.0, 0.0, 0.0 ) ) );
  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_points_remove_back()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  line.points_remove_back( 2 );

  let expected = 
  [
    gl::F32x3::new( 0.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
  ];

  assert_eq!( expected, line.points_get() );
}

#[ test ]
fn test_points_remove_front()
{
  let mut line = d3::Line::default();
  line.point_add_back( &[ 0.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 0.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 0.0 ] );
  line.point_add_back( &[ 1.0, 1.0, 1.0 ] );

  line.points_remove_front( 2 );

  let expected = 
  [
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 1.0 ),
  ];

  assert_eq!( expected, line.points_get() );
}
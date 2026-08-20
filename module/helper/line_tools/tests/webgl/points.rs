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

/// # What
/// Sets a color at an out-of-range index and confirms the call silently does nothing,
/// leaving all existing colors untouched.
///
/// # How
/// `color_add_back` three colors, then call `color_set` at index 0 (in-range, expected to
/// apply), index 7 (out-of-range, expected to be a no-op), and assert against `colors_get()`.
///
/// # Root Cause
/// `color_set`'s doc comment claimed "Will panic if index is out of range", but the
/// implementation reads via `.get_mut( index )` guarded by `if let Some(..)`, which silently
/// no-ops on an out-of-range index instead of panicking. `point_set` (the sibling method,
/// already covered against the identical scenario by `test_point_set` above) has the same
/// mismatch; this crate had zero test coverage of `color_set` at all before this test.
///
/// # Fix
/// Corrected `point_set`'s and `color_set`'s doc comments in `src/lib.rs` to describe the
/// actual (and, per this test, contractually pinned) no-op behavior. No code behavior changed.
///
/// # Notes
/// bug_reproducer(BUG-154)
#[ test ]
fn test_color_set()
{
  let mut line = d3::Line::default();
  line.color_add_back( [ 0.0, 0.0, 0.0 ] );
  line.color_add_back( [ 1.0, 0.0, 0.0 ] );
  line.color_add_back( [ 1.0, 1.0, 0.0 ] );

  line.color_set( [ 0.5, 0.5, 0.5 ], 0 );
  line.color_set( [ 0.5, 0.5, 0.5 ], 7 );

  let expected =
  [
    gl::F32x3::new( 0.5, 0.5, 0.5 ),
    gl::F32x3::new( 1.0, 0.0, 0.0 ),
    gl::F32x3::new( 1.0, 1.0, 0.0 ),
  ];

  assert_eq!( expected, line.colors_get() );
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
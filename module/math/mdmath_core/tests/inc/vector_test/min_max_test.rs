
use super::*;

use the_module::vector::{ min, max, min_mut, max_mut };

// T01/T02
#[ test ]
fn integer_i32()
{
  assert_eq!( min( &[ 3i32, 1, 2 ], &[ 1i32, 5, 0 ] ), [ 1, 1, 0 ] );
  assert_eq!( max( &[ 3i32, 1, 2 ], &[ 1i32, 5, 0 ] ), [ 3, 5, 2 ] );
}

// T03
#[ test ]
fn integer_i64()
{
  assert_eq!( min( &[ 3i64, 1, 2 ], &[ 1i64, 5, 0 ] ), [ 1, 1, 0 ] );
  assert_eq!( max( &[ 3i64, 1, 2 ], &[ 1i64, 5, 0 ] ), [ 3, 5, 2 ] );
}

// T03
#[ test ]
fn integer_u32()
{
  assert_eq!( min( &[ 3u32, 1, 2 ], &[ 1u32, 5, 0 ] ), [ 1, 1, 0 ] );
  assert_eq!( max( &[ 3u32, 1, 2 ], &[ 1u32, 5, 0 ] ), [ 3, 5, 2 ] );
}

// T03
#[ test ]
fn integer_u64()
{
  assert_eq!( min( &[ 3u64, 1, 2 ], &[ 1u64, 5, 0 ] ), [ 1, 1, 0 ] );
  assert_eq!( max( &[ 3u64, 1, 2 ], &[ 1u64, 5, 0 ] ), [ 3, 5, 2 ] );
}

// T04 — regression: existing float path unchanged
// `min`/`max` only select one of the two exact input values (no arithmetic), so the result
// is always bit-identical to one of the literals — exact equality is correct here.
#[ test ]
#[ allow( clippy::float_cmp ) ]
fn float_regression()
{
  assert_eq!( min( &[ 3.0f32, 1.0, 2.0 ], &[ 1.0f32, 5.0, 0.0 ] ), [ 1.0, 1.0, 0.0 ] );
  assert_eq!( max( &[ 3.0f32, 1.0, 2.0 ], &[ 1.0f32, 5.0, 0.0 ] ), [ 3.0, 5.0, 2.0 ] );
}

// T05 — NaN tie-break: r (the accumulator, seeded from `a`) always wins over an unordered
// comparison, so NaN in `a` propagates through and NaN in `b` is ignored.
// `min`/`max` only select one of the two exact input values (no arithmetic), so the result
// is always bit-identical to one of the literals — exact equality is correct here.
#[ test ]
#[ allow( clippy::float_cmp ) ]
fn float_nan_tie_break()
{
  let r = min( &[ 1.0f32, f32::NAN ], &[ 2.0f32, 3.0 ] );
  assert_eq!( r[ 0 ], 1.0 );
  assert!( r[ 1 ].is_nan(), "NaN in `a` must propagate to the result" );

  let r = min( &[ 1.0f32, 5.0 ], &[ 2.0f32, f32::NAN ] );
  assert_eq!( r, [ 1.0, 5.0 ], "NaN in `b` must be ignored, keeping `a`'s value" );

  let r = max( &[ 1.0f32, f32::NAN ], &[ 2.0f32, 3.0 ] );
  assert_eq!( r[ 0 ], 2.0 );
  assert!( r[ 1 ].is_nan(), "NaN in `a` must propagate to the result" );

  let r = max( &[ 1.0f32, 5.0 ], &[ 2.0f32, f32::NAN ] );
  assert_eq!( r, [ 2.0, 5.0 ], "NaN in `b` must be ignored, keeping `a`'s value" );
}

#[ test ]
fn mut_variants()
{
  let mut r = [ 3i32, 1, 2 ];
  min_mut( &mut r, &[ 1i32, 5, 0 ] );
  assert_eq!( r, [ 1, 1, 0 ] );

  let mut r = [ 3i32, 1, 2 ];
  max_mut( &mut r, &[ 1i32, 5, 0 ] );
  assert_eq!( r, [ 3, 5, 2 ] );
}

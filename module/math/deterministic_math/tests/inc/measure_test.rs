//! The ULP metric itself, checked before anything is measured with it.
//!
//! Every accuracy bound in this suite and every Δulp figure the `cost_vs_libm`
//! example prints is read off `ulp_diff`. An error here would not fail — it
//! would quietly restate every one of those numbers, so the ruler is graded
//! before it is used.
//!
//! The two cases worth naming are the ones a naive implementation gets wrong.
//! Subtracting raw bit patterns reports two adjacent values straddling zero as
//! astronomically far apart, and treats `+0.0` and `-0.0` as two distinct
//! values rather than one.

use super::*;

#[ test ]
fn the_two_zeros_are_one_value_not_two()
{
  // The sign-folding key exists so this holds without an `==` fast path.
  assert_eq!( ulp_diff( 0.0, -0.0 ), 0 );
  assert_eq!( ulp_diff( -0.0, 0.0 ), 0 );
}

#[ test ]
fn adjacent_values_straddling_zero_are_two_apart()
{
  // The case the naive bit-subtraction gets astronomically wrong: these are
  // the two representable values either side of zero.
  let up = f64::from_bits( 1 );
  assert_eq!( ulp_diff( up, -up ), 2 );
  assert_eq!( ulp_diff( up, 0.0 ), 1 );
  assert_eq!( ulp_diff( -up, 0.0 ), 1 );
}

#[ test ]
fn one_step_is_one_ulp_at_every_scale()
{
  for x in [ 1.0_f64, 1.0e-300, 1.0e300, -7.5, f64::MIN_POSITIVE ]
  {
    let next = f64::from_bits( x.abs().to_bits() + 1 ) * x.signum();
    assert_eq!( ulp_diff( x, next ), 1, "at {x:e}" );
  }
}

#[ test ]
fn non_finites_are_classified_rather_than_measured()
{
  assert_eq!( ulp_diff( f64::NAN, f64::NAN ), 0 );
  assert_eq!( ulp_diff( f64::NAN, 1.0 ), u64::MAX );
  assert_eq!( ulp_diff( f64::INFINITY, f64::INFINITY ), 0 );
  assert_eq!( ulp_diff( f64::NEG_INFINITY, f64::NEG_INFINITY ), 0 );
  assert_eq!( ulp_diff( f64::INFINITY, f64::NEG_INFINITY ), u64::MAX );
  assert_eq!( ulp_diff( f64::INFINITY, f64::MAX ), u64::MAX );
}

#[ test ]
fn it_is_symmetric()
{
  for ( a, b ) in [ ( 1.0, 2.0 ), ( -1.0, 1.0 ), ( 0.0, -0.0 ), ( f64::NAN, 1.0 ) ]
  {
    assert_eq!( ulp_diff( a, b ), ulp_diff( b, a ), "at {a:e} vs {b:e}" );
  }
}

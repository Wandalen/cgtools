//! `sin`, `cos`, `tan`, `sin_cos` — one argument reduction and two polynomials.

use super::*;
use the_module::{ cos, sin, sin_cos, tan, PI, SIN_COS_MAX };

#[ test ]
fn sin_and_cos_satisfy_pythagoras()
{
  for x in sweep( -1.0e4, 1.0e4, 800 )
  {
    let ( s, c ) = sin_cos( x );
    assert!( rel_err( s * s + c * c, 1.0 ) < TOL, "unit circle at {x}" );
  }
}

#[ test ]
fn sin_cos_agrees_with_the_separate_calls()
{
  // `sin` and `cos` are documented as thin wrappers over `sin_cos`, so this is an
  // exact check: a difference here means one of the three drifted.
  for x in sweep( -1.0e4, 1.0e4, 500 )
  {
    let ( s, c ) = sin_cos( x );
    assert_eq!( s.to_bits(), sin( x ).to_bits(), "sin vs sin_cos at {x}" );
    assert_eq!( c.to_bits(), cos( x ).to_bits(), "cos vs sin_cos at {x}" );
  }
}

#[ test ]
fn sin_and_cos_match_libm()
{
  for x in sweep( -1.0e4, 1.0e4, 800 )
  {
    let ( s, c ) = sin_cos( x );
    // Absolute, not relative: `sin` legitimately passes through zero, where a
    // relative bound is meaningless.
    assert!( ( s - x.sin() ).abs() < 1.0e-12, "sin vs libm at {x}" );
    assert!( ( c - x.cos() ).abs() < 1.0e-12, "cos vs libm at {x}" );
  }
}

#[ test ]
fn sin_is_odd_and_cos_is_even()
{
  // Exactly, not approximately — the reduction is sign-symmetric by
  // construction, and an asymmetry would mean it is not.
  for x in sweep( 0.0, 1.0e3, 400 )
  {
    assert_eq!( sin( -x ).to_bits(), ( -sin( x ) ).to_bits(), "sin parity at {x}" );
    assert_eq!( cos( -x ).to_bits(), cos( x ).to_bits(), "cos parity at {x}" );
  }
}

#[ test ]
fn sin_keeps_its_relative_accuracy_near_zero()
{
  // `sin x -> x` as `x -> 0`, so the whole result is significant digits here and
  // a reduction that adds and subtracts a multiple of `PI/2` would destroy them.
  // The check is relative rather than absolute because an absolute bound is
  // trivially satisfied by anything near zero, including a wrong answer.
  for x in decades( 3.7, 300 )
  {
    assert!( rel_err( sin( x ), x.sin() ) < 1.0e-14, "sin near zero at {x}" );
    assert!( rel_err( cos( x ), x.cos() ) < 1.0e-14, "cos near zero at {x}" );
  }
}

#[ test ]
fn tan_is_sin_over_cos()
{
  for x in sweep( -1.0, 1.0, 300 )
  {
    let ( s, c ) = sin_cos( x );
    assert!( rel_err( tan( x ), s / c ) < TOL, "tan quotient at {x}" );
    assert!( rel_err( tan( x ), x.tan() ) < TOL, "tan vs libm at {x}" );
  }
}

#[ test ]
fn the_range_limit_is_enforced_rather_than_merely_documented()
{
  // Past the reduction's range there is no answer to give: nothing about a
  // `1e20`-radian angle survives to the fifteenth digit, so a finite return would
  // be a number with no information in it — and those propagate silently.
  //
  // This was once documented and not checked, and an argument past it did not
  // produce a poor answer but an arithmetic overflow inside the rounding helper,
  // which panics in a debug build. It was reached from a diverging Newton
  // iterate, which is exactly when a caller least expects a library to abort.
  for x in [ SIN_COS_MAX * 1.5, 1.0e20, -1.0e20, f64::INFINITY, f64::NEG_INFINITY, f64::NAN ]
  {
    let ( s, c ) = sin_cos( x );
    assert!( s.is_nan() && c.is_nan(), "should refuse {x}" );
    assert!( sin( x ).is_nan() && cos( x ).is_nan() && tan( x ).is_nan(), "wrappers should refuse {x}" );
  }

  // And just inside the limit it still answers.
  let ( s, c ) = sin_cos( SIN_COS_MAX * 0.5 );
  assert!( s.is_finite() && c.is_finite(), "should still answer inside the range" );
}

#[ test ]
fn the_quadrant_reassembly_lands_on_the_right_branch()
{
  // The reduction returns a residual plus a quarter-turn count, and the count
  // selects which of four `( ±sin, ±cos )` pairings to emit. A wrong entry there
  // is invisible to `sin² + cos² == 1` — every pairing satisfies it — so the
  // known values at each quarter turn are checked directly.
  for q in -8_i32 ..= 8
  {
    let x = f64::from( q ) * PI / 2.0;
    let ( s, c ) = sin_cos( x );
    let ( want_s, want_c ) = match q.rem_euclid( 4 )
    {
      0 => ( 0.0, 1.0 ),
      1 => ( 1.0, 0.0 ),
      2 => ( 0.0, -1.0 ),
      _ => ( -1.0, 0.0 ),
    };
    // Absolute: `PI / 2` is not exactly a quarter turn in `f64`, so the residual
    // is the representation error of `PI`, not an error of the reduction.
    assert!( ( s - want_s ).abs() < 1.0e-14, "sin at quarter turn {q}" );
    assert!( ( c - want_c ).abs() < 1.0e-14, "cos at quarter turn {q}" );
  }
}

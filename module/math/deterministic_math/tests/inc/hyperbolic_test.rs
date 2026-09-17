//! `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`.
//!
//! Four of these were rebuilt around `exp_m1` and `ln_1p` rather than around
//! `exp` and `ln`, because every one of them is a difference that cancels near
//! zero: `sinh` as `( e^x - e^-x ) / 2` subtracts two nearly-equal numbers,
//! `tanh` divides one such difference by another, and both inverses take the
//! logarithm of something that approaches 1. Measured against the platform's
//! libm before that change: 7 163 ULP for `sinh`, 1 244 for `tanh`, 2 628 for
//! `asinh`.

use super::*;
use the_module::{ acosh, asinh, atanh, cosh, exp, ln, sinh, tanh, LN_2 };

#[ test ]
fn the_hyperbolic_identity_holds_where_f64_can_still_express_it()
{
  // `cosh² - sinh² == 1`, but only near the origin, and the limit is `f64`'s
  // rather than these functions': `cosh x` and `sinh x` both approach `eˣ/2`, and
  // by `x = 30` they agree to far more digits than the format carries. Their
  // difference is then pure cancellation — and no algebraic rearrangement
  // rescues it, because the information is gone from the two inputs before the
  // identity is evaluated. Asserting it out there would grade `f64`, not these.
  for x in sweep( -5.0, 5.0, 400 )
  {
    let ( s, c ) = ( sinh( x ), cosh( x ) );
    assert!( rel_err( c * c - s * s, 1.0 ) < 1.0e-11, "direct identity at {x}" );
  }

  // What is checkable across the full range is agreement with libm, which is the
  // claim the identity was standing in for anyway.
  for x in sweep( -300.0, 300.0, 400 )
  {
    assert!( rel_err( sinh( x ), x.sinh() ) < 1.0e-11, "sinh vs libm at {x}" );
    assert!( rel_err( cosh( x ), x.cosh() ) < 1.0e-11, "cosh vs libm at {x}" );
  }
}

#[ test ]
fn tanh_is_sinh_over_cosh()
{
  for x in sweep( -30.0, 30.0, 400 )
  {
    assert!( rel_err( tanh( x ), sinh( x ) / cosh( x ) ) < TOL, "tanh quotient at {x}" );
    assert!( rel_err( tanh( x ), x.tanh() ) < 1.0e-11, "tanh vs libm at {x}" );
  }

  // Saturating, and exactly so: past the point where `e^-2x` falls below the
  // last place of 1, the answer *is* 1 and anything else is noise.
  assert_eq!( tanh( 40.0 ), 1.0 );
  assert_eq!( tanh( -40.0 ), -1.0 );
  assert_eq!( tanh( f64::INFINITY ), 1.0 );
  assert_eq!( tanh( f64::NEG_INFINITY ), -1.0 );
}

#[ test ]
fn the_small_argument_band_keeps_every_digit()
{
  // Every function here tends to the identity — or, for `cosh`, to 1 — as the
  // argument goes to zero, so the whole result is significant digits and a
  // cancelling formula destroys them. This is the assertion the `exp`-based
  // forms failed, and it fails loudly: the error grew by roughly a decade for
  // every decade the argument shrank.
  for x in decades( 3.7, 300 )
  {
    assert!( rel_err( sinh( x ), x.sinh() ) < 1.0e-14, "sinh near zero at {x:e}" );
    assert!( rel_err( sinh( -x ), ( -x ).sinh() ) < 1.0e-14, "sinh near zero at -{x:e}" );
    assert!( rel_err( tanh( x ), x.tanh() ) < 1.0e-14, "tanh near zero at {x:e}" );
    assert!( rel_err( asinh( x ), x.asinh() ) < 1.0e-14, "asinh near zero at {x:e}" );
    assert!( rel_err( atanh( x ), x.atanh() ) < 1.0e-14, "atanh near zero at {x:e}" );
    assert!( rel_err( cosh( x ), x.cosh() ) < 1.0e-14, "cosh near zero at {x:e}" );

    // Far enough below the point where the leading correction reaches half the
    // last place, each of these is the identity *exactly* — an assertion that
    // needs no reference implementation at all.
    //
    // The margin is deliberate. The correction itself vanishes around `1e-8`, but
    // these formulas reach the answer through two or three intermediate roundings
    // (`1 - a`, a division, a series), each of which can move the result by half
    // a last place on its own. Below `1e-20` every intermediate is exact — `1 - a`
    // and `1 + a` are both exactly `1`, and `a²` cannot reach `a`'s mantissa — so
    // bit equality is a structural consequence rather than a lucky rounding.
    if x < 1.0e-20
    {
      assert_eq!( sinh( x ).to_bits(), x.to_bits(), "sinh should be the identity at {x:e}" );
      assert_eq!( tanh( x ).to_bits(), x.to_bits(), "tanh should be the identity at {x:e}" );
      assert_eq!( asinh( x ).to_bits(), x.to_bits(), "asinh should be the identity at {x:e}" );
      assert_eq!( atanh( x ).to_bits(), x.to_bits(), "atanh should be the identity at {x:e}" );
      assert_eq!( cosh( x ), 1.0, "cosh should be exactly 1 at {x:e}" );
    }
  }
}

#[ test ]
fn the_saturating_branches_agree_with_the_ones_they_replace()
{
  // Past a threshold `sinh` and `cosh` drop the vanishing term and evaluate
  // `e^( |x| - ln 2 )` instead. Both formulas are evaluated at the *same*
  // argument rather than at two points straddling the threshold — sampling
  // either side instead measures the derivative multiplied by the sampling gap,
  // which looks exactly like a jump of that size and says nothing about
  // continuity.
  for x in [ 18.0, 20.0, 22.0, 25.0 ]
  {
    let e = exp( x );
    assert!( rel_err( exp( x - LN_2 ), 0.5 * ( e - 1.0 / e ) ) < 1.0e-14, "sinh branches at {x}" );
    assert!( rel_err( exp( x - LN_2 ), 0.5 * ( e + 1.0 / e ) ) < 1.0e-14, "cosh branches at {x}" );
  }

  // And the same for the two-formula seam in `sinh` near 1, where the `exp_m1`
  // form gives way to the direct difference.
  for x in [ 0.98, 0.995, 1.0, 1.005, 1.02 ]
  {
    assert!( rel_err( sinh( x ), x.sinh() ) < 1.0e-14, "sinh across its band at {x}" );
    assert!( rel_err( asinh( x ), x.asinh() ) < 1.0e-14, "asinh across its band at {x}" );
  }
}

#[ test ]
fn asinh_inverts_sinh()
{
  for x in sweep( -20.0, 20.0, 400 )
  {
    assert!( rel_err( asinh( sinh( x ) ), x ) < 1.0e-9, "asinh of sinh at {x}" );
  }
  for x in sweep( -1.0e6, 1.0e6, 500 )
  {
    assert!( rel_err( asinh( x ), x.asinh() ) < 1.0e-11, "asinh vs libm at {x}" );
    assert_eq!( asinh( -x ).to_bits(), ( -asinh( x ) ).to_bits(), "asinh parity at {x}" );
  }

  // The large-argument identity branch, where `asinh x` becomes `ln x + ln 2`.
  for x in [ 1.0e9, 1.0e12, 1.0e20, 1.0e100, 1.0e300 ]
  {
    assert!( rel_err( asinh( x ), x.asinh() ) < 1.0e-11, "asinh far out at {x:e}" );
    assert!( rel_err( asinh( x ), ln( x ) + LN_2 ) < 1.0e-14, "asinh identity at {x:e}" );
  }
}

#[ test ]
fn acosh_inverts_cosh_and_refuses_what_has_no_inverse()
{
  // Below 1 there is no real answer — `cosh` never goes there — so NaN is the
  // only honest return.
  for x in sweep( -10.0, 0.999, 200 )
  {
    assert!( acosh( x ).is_nan(), "acosh should refuse {x}" );
  }
  assert!( acosh( f64::NAN ).is_nan() );
  assert_eq!( acosh( 1.0 ), 0.0, "the one point where the answer is exactly zero" );

  for x in sweep( 1.0, 1.0e6, 500 )
  {
    assert!( rel_err( acosh( x ), x.acosh() ) < 1.0e-11, "acosh vs libm at {x}" );
  }
  for x in sweep( 0.001, 20.0, 400 )
  {
    assert!( rel_err( acosh( cosh( x ) ), x ) < 1.0e-9, "acosh of cosh at {x}" );
  }

  // Just above 1, where `acosh` has an infinite derivative and the argument's own
  // representation is the limiting factor. `acosh( 1 + t ) -> sqrt( 2t )`, which
  // is the shape a formula built on `ln( x + sqrt( x² - 1 ) )` loses: `x² - 1`
  // cancels away most of `t` before the square root ever sees it.
  for t in decades( 3.7, 300 )
  {
    let want = ( 1.0_f64 + t ).acosh();
    assert!( rel_err( acosh( 1.0 + t ), want ) < 1.0e-14, "acosh just above 1 at 1+{t:e}" );
  }

  // The large-argument identity branch, matching `asinh`'s.
  for x in [ 1.0e9, 1.0e12, 1.0e20, 1.0e100, 1.0e300 ]
  {
    assert!( rel_err( acosh( x ), x.acosh() ) < 1.0e-11, "acosh far out at {x:e}" );
    assert!( rel_err( acosh( x ), ln( x ) + LN_2 ) < 1.0e-14, "acosh identity at {x:e}" );
  }
}

#[ test ]
fn atanh_inverts_tanh_and_refuses_what_has_no_inverse()
{
  for x in sweep( -0.99, 0.99, 500 )
  {
    assert!( rel_err( atanh( x ), x.atanh() ) < 1.0e-11, "atanh vs libm at {x}" );
    assert_eq!( atanh( -x ).to_bits(), ( -atanh( x ) ).to_bits(), "atanh parity at {x}" );
  }
  for x in sweep( -5.0, 5.0, 400 )
  {
    assert!( rel_err( atanh( tanh( x ) ), x ) < 1.0e-9, "atanh of tanh at {x}" );
  }

  // The poles, and past them.
  assert_eq!( atanh( 1.0 ), f64::INFINITY );
  assert_eq!( atanh( -1.0 ), f64::NEG_INFINITY );
  for x in [ 1.0001, 2.0, 1.0e6, -1.0001, -2.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN ]
  {
    assert!( atanh( x ).is_nan(), "atanh should refuse {x}" );
  }
}

#[ test ]
fn the_hyperbolics_propagate_nan_and_saturate_at_infinity()
{
  for f in [ sinh as fn( f64 ) -> f64, cosh, tanh, asinh ]
  {
    assert!( f( f64::NAN ).is_nan(), "nan should propagate" );
  }
  assert!( sinh( f64::INFINITY ).is_infinite() && sinh( f64::INFINITY ) > 0.0 );
  assert!( sinh( f64::NEG_INFINITY ).is_infinite() && sinh( f64::NEG_INFINITY ) < 0.0 );
  assert!( cosh( f64::INFINITY ).is_infinite() && cosh( f64::INFINITY ) > 0.0 );
  assert!( cosh( f64::NEG_INFINITY ).is_infinite() && cosh( f64::NEG_INFINITY ) > 0.0 );
  assert!( asinh( f64::INFINITY ).is_infinite() && asinh( f64::INFINITY ) > 0.0 );
  assert!( acosh( f64::INFINITY ).is_infinite() && acosh( f64::INFINITY ) > 0.0 );

  // `cosh` is even, exactly.
  for x in sweep( 0.0, 30.0, 300 )
  {
    assert_eq!( cosh( -x ).to_bits(), cosh( x ).to_bits(), "cosh parity at {x}" );
  }
}

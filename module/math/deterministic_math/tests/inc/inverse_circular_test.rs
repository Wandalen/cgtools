//! `atan`, `atan2`, `asin`, `acos` — the arctangent and the two functions built
//! on it.
//!
//! This family is where the crate's largest accuracy defect lived. `atan` reduced
//! every argument onto one of four bin centres, including arguments already far
//! smaller than the reduction's own residual, and reassembled the answer as
//! `ATAN_V[ 0 ] + atan_series( u )` — two terms equal and opposite to within the
//! size of the argument. The leading digits cancelled and what survived was the
//! rounding of a constant. Measured against the platform's libm: 95 ULP for
//! `atan`, and — through the two functions that route into it — 8 883 ULP for
//! `atan2` and 22 268 ULP for `asin`, concentrated below `| x | ~ 1e-4` and
//! fanning out steadily as the argument shrank.
//!
//! [`the_small_argument_band_keeps_every_digit`] is the guard that keeps it
//! fixed: a linear sweep bunched around 1 never reaches the band, so the check
//! has to walk decades.

use super::*;
use the_module::{ acos, asin, atan, atan2, tan, FRAC_PI_2, PI };

#[ test ]
fn atan_inverts_tan()
{
  // Restricted to atan's own principal range, which is where the inverse holds.
  for a in sweep( -1.5, 1.5, 400 )
  {
    assert!( rel_err( atan( tan( a ) ), a ) < TOL, "atan of tan at {a}" );
  }
}

#[ test ]
fn atan_matches_libm_across_the_whole_line()
{
  for x in sweep( -1.0e6, 1.0e6, 900 )
  {
    assert!( rel_err( atan( x ), x.atan() ) < TOL, "atan vs libm at {x}" );
  }

  // Exactly odd, and saturating at the asymptote rather than near it.
  for x in sweep( 0.0, 1.0e3, 300 )
  {
    assert_eq!( atan( -x ).to_bits(), ( -atan( x ) ).to_bits(), "atan parity at {x}" );
  }
  assert_eq!( atan( f64::INFINITY ), FRAC_PI_2 );
  assert_eq!( atan( f64::NEG_INFINITY ), -FRAC_PI_2 );
  assert!( atan( f64::NAN ).is_nan() );
}

#[ test ]
fn the_small_argument_band_keeps_every_digit()
{
  // `atan x -> x` as `x -> 0`, so the entire result is significant digits and
  // there is nothing for a lossy reduction to hide behind. This is the assertion
  // the bin-reduction defect failed, and it fails it loudly: the error grew by
  // roughly a decade for every decade the argument shrank.
  //
  // Relative, and tight. An absolute bound near zero is satisfied by any answer
  // at all, including the wrong one.
  for x in decades( 3.7, 300 )
  {
    assert!( rel_err( atan( x ), x.atan() ) < 1.0e-14, "atan near zero at {x:e}" );
    assert!( rel_err( atan( -x ), ( -x ).atan() ) < 1.0e-14, "atan near zero at -{x:e}" );
    assert!( rel_err( asin( x ), x.asin() ) < 1.0e-14, "asin near zero at {x:e}" );
    assert!( rel_err( atan2( x, 1.0 ), x.atan2( 1.0 ) ) < 1.0e-14, "atan2 near zero at {x:e}" );

    // The identity that makes the claim independent of libm: below the point
    // where the leading correction reaches half the last place of `x`, `atan x`
    // and `x` are the same double, and any reduction that perturbs the leading
    // digits breaks it. That point is `x² / 3 < 2^-53` for `atan` and `x² / 6`
    // for `asin`, so `1e-9` clears both by a wide margin.
    if x < 1.0e-9
    {
      assert_eq!( atan( x ).to_bits(), x.to_bits(), "atan should be the identity at {x:e}" );
      assert_eq!( asin( x ).to_bits(), x.to_bits(), "asin should be the identity at {x:e}" );
    }
  }
}

#[ test ]
fn atan_joins_its_two_strategies_without_a_step()
{
  // Either side of the threshold `atan` switches strategy entirely — direct
  // series below, bin reduction above. A discontinuity there is invisible to
  // every identity test, because both sides are individually self-consistent.
  //
  // Graded against libm, which is continuous across the seam by construction, at
  // a spacing far finer than any plausible step.
  for x in sweep( 0.05, 0.08, 400 )
  {
    assert!( rel_err( atan( x ), x.atan() ) < 1.0e-15, "atan across the seam at {x}" );
  }
}

#[ test ]
fn atan2_matches_libm_in_every_quadrant()
{
  for y in sweep( -100.0, 100.0, 60 )
  {
    for x in sweep( -100.0, 100.0, 60 )
    {
      // Skip the origin, where the value is a convention rather than a limit.
      if x.abs() < 1.0e-9 && y.abs() < 1.0e-9 { continue; }
      assert!( ( atan2( y, x ) - y.atan2( x ) ).abs() < 1.0e-12, "atan2 at {y},{x}" );
    }
  }
}

#[ test ]
fn atan2_lands_on_the_axes_exactly()
{
  // The four axis directions are the cases a quadrant table gets wrong, and each
  // is a value that can be stated rather than approximated.
  assert_eq!( atan2( 1.0, 0.0 ), FRAC_PI_2 );
  assert_eq!( atan2( -1.0, 0.0 ), -FRAC_PI_2 );
  assert_eq!( atan2( 0.0, 1.0 ), 0.0 );
  assert_eq!( atan2( 0.0, -1.0 ), PI );
  assert_eq!( atan2( 0.0, 0.0 ), 0.0, "the origin is a convention, and this is the one chosen" );
  assert!( atan2( f64::NAN, 1.0 ).is_nan() );
  assert!( atan2( 1.0, f64::NAN ).is_nan() );
}

#[ test ]
fn asin_and_acos_are_complementary()
{
  for x in sweep( -1.0, 1.0, 500 )
  {
    assert!( rel_err( asin( x ) + acos( x ), FRAC_PI_2 ) < TOL, "complement at {x}" );
    assert!( ( asin( x ) - x.asin() ).abs() < 1.0e-12, "asin vs libm at {x}" );
    assert!( ( acos( x ) - x.acos() ).abs() < 1.0e-12, "acos vs libm at {x}" );
  }

  // At the endpoints, where the derivative is infinite and an implementation
  // that reached them by a series would not arrive.
  assert_eq!( asin( 1.0 ), FRAC_PI_2 );
  assert_eq!( asin( -1.0 ), -FRAC_PI_2 );
  assert_eq!( acos( 1.0 ), 0.0 );
  assert!( rel_err( acos( -1.0 ), PI ) < TOL );
}

#[ test ]
fn asin_and_acos_clamp_rather_than_returning_nan()
{
  // A documented divergence from libm, asserted so it cannot regress quietly in
  // either direction — reverting to NaN would break the callers it protects, and
  // widening the clamp would hide a genuinely out-of-range argument.
  //
  // The case it protects: an argument a hair outside `[ -1, 1 ]` is the ordinary
  // result of rounding a normalised dot product, and a NaN there would propagate
  // into a value no later check would question.
  assert!( rel_err( asin( 2.0 ), FRAC_PI_2 ) < TOL );
  assert!( rel_err( asin( -2.0 ), -FRAC_PI_2 ) < TOL );
  assert!( acos( 2.0 ).abs() < 1.0e-12 );
  assert!( rel_err( acos( -2.0 ), PI ) < TOL );

  // The clamp is a boundary, not a rescale: values inside the domain are
  // untouched, so it cannot quietly bend a legitimate argument.
  for x in sweep( -1.0, 1.0, 300 )
  {
    assert!( ( asin( x ) - x.asin() ).abs() < 1.0e-12, "in-domain asin at {x}" );
  }

  // NaN itself still passes straight through, so the case that means "already
  // broken" is not swallowed by the clamp.
  assert!( asin( f64::NAN ).is_nan() );
  assert!( acos( f64::NAN ).is_nan() );
}

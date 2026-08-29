//! Arctangent and the two inverse functions built on it.

use crate::algebraic::{ atan_series, sqrt };
use crate::constant::{ ASIN_DIRECT, ATAN_B, ATAN_DIRECT, ATAN_V, FRAC_PI_2, PI };

/// Arctangent, over the whole real line, returning `[ -PI/2, PI/2 ]`.
///
/// # The small-argument path, and why it is not an optimisation
///
/// A mid-range argument is reduced onto the nearest of four bin centres and the
/// series evaluated on the residual, which is what makes eleven terms enough.
/// For an argument near zero that same reduction is destructive: the result is
/// assembled as `ATAN_V[ 0 ] + atan_series( u )` where the two terms are equal
/// and opposite to within the size of the argument, so the leading digits
/// cancel and what survives is the rounding of `0.1243...`, not the answer.
///
/// The cost was real and measured. Against the platform's libm the reduced path
/// disagreed by up to 95 ULP for `atan` and — through the two functions that
/// route into it — 8 883 ULP for `atan2` and 22 268 ULP for `asin`, all of it
/// concentrated below `| x | ≈ 1e-4` and fanning out steadily as the argument
/// shrank. Below `constant::ATAN_DIRECT` the series is already well inside its
/// own documented range, so skipping the reduction costs nothing and keeps
/// every digit.
///
/// ```
/// use deterministic_math::{ atan, tan };
/// let a = 0.7;
/// assert!( ( atan( tan( a ) ) - a ).abs() < 1e-15 );
///
/// // The band the bin reduction used to lose: relative accuracy all the way down.
/// let small = 1.0e-9;
/// assert!( ( atan( small ) / small - 1.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn atan( x : f64 ) -> f64
{
  if x.is_nan()
  {
    return x;
  }
  let neg = x < 0.0;
  let mut t = if neg { -x } else { x };

  // Above 1, invert; the two branches then share one series. An infinite
  // argument inverts to zero and lands on the direct path below, returning
  // exactly `PI/2`.
  let inverted = t > 1.0;
  if inverted
  {
    t = 1.0 / t;
  }

  let mut a = if t < ATAN_DIRECT
  {
    atan_series( t )
  }
  else
  {
    // Reduce onto the nearest of four bin centres, leaving | u | <= ~0.125.
    let j = ( ( t * 4.0 ) as usize ).min( 3 );
    let b = ATAN_B[ j ];
    let u = ( t - b ) / ( 1.0 + t * b );
    ATAN_V[ j ] + atan_series( u )
  };

  if inverted
  {
    a = FRAC_PI_2 - a;
  }
  if neg { -a } else { a }
}

/// Two-argument arctangent, returning `( -PI, PI ]`.
///
/// This is the function every element-recovery path leans on, because an angle
/// read back from a pair of components is only unambiguous when both signs are
/// available. `atan( y / x )` throws one of them away and lands in the wrong
/// half of the circle for every state west of the origin.
///
/// ```
/// use deterministic_math::{ atan2, PI };
/// assert!( ( atan2( 1.0, -1.0 ) - 3.0 * PI / 4.0 ).abs() < 1e-15 );
/// assert!( ( atan2( -1.0, -1.0 ) + 3.0 * PI / 4.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn atan2( y : f64, x : f64 ) -> f64
{
  if x.is_nan() || y.is_nan()
  {
    return f64::NAN;
  }
  if x > 0.0
  {
    return atan( y / x );
  }
  if x < 0.0
  {
    return if y < 0.0 { atan( y / x ) - PI } else { atan( y / x ) + PI };
  }
  // x is exactly zero: on the axis, or at the origin.
  if y > 0.0
  {
    return FRAC_PI_2;
  }
  if y < 0.0
  {
    return -FRAC_PI_2;
  }
  0.0
}

/// Arcsine, for `| x | <= 1`, returning `[ -PI/2, PI/2 ]`.
///
/// Arguments slightly outside the domain — the usual result of rounding in a
/// normalised dot product — are clamped rather than returned as `NaN`. A `NaN`
/// here would propagate silently into whatever consumed it; a clamp is wrong by
/// less than the rounding that produced it.
///
/// Below `constant::ASIN_DIRECT` the companion side is skipped and this is
/// [`atan`] — see that constant for why building `sqrt( 1 - c² )` for a tiny
/// argument costs a last place rather than buying one.
///
/// ```
/// use deterministic_math::{ asin, sin };
/// let a = -0.3;
/// assert!( ( asin( sin( a ) ) - a ).abs() < 1e-15 );
///
/// // Small enough that the arcsine is its own argument, to the last bit.
/// let small = 3.7e-12;
/// assert_eq!( asin( small ), small );
/// ```
#[ must_use ]
pub fn asin( x : f64 ) -> f64
{
  let c = clamp_unit( x );
  if c.abs() < ASIN_DIRECT
  {
    return atan( c );
  }
  atan2( c, sqrt( ( 1.0 - c ) * ( 1.0 + c ) ) )
}

/// Arccosine, for `| x | <= 1`, returning `[ 0, PI ]`. Clamps like [`asin`].
///
/// ```
/// use deterministic_math::{ acos, cos };
/// let a = 2.1;
/// assert!( ( acos( cos( a ) ) - a ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn acos( x : f64 ) -> f64
{
  let c = clamp_unit( x );
  atan2( sqrt( ( 1.0 - c ) * ( 1.0 + c ) ), c )
}

/// Clamp to `[ -1, 1 ]`, passing `NaN` through unchanged.
#[ must_use ]
fn clamp_unit( x : f64 ) -> f64
{
  if x > 1.0
  {
    return 1.0;
  }
  if x < -1.0
  {
    return -1.0;
  }
  x
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn clamp_unit_is_a_boundary_not_a_rescale()
  {
    // Compared as bits, like the in-domain case below: the clamp returns the
    // boundary exactly, so bit equality is the claim and an epsilon would be a
    // weaker one.
    assert_eq!( clamp_unit( 2.0 ).to_bits(), 1.0_f64.to_bits() );
    assert_eq!( clamp_unit( -2.0 ).to_bits(), ( -1.0_f64 ).to_bits() );
    assert!( clamp_unit( f64::NAN ).is_nan() );
    // Everything inside is returned bit-identically.
    let mut x = -1.0;
    while x <= 1.0
    {
      assert_eq!( clamp_unit( x ).to_bits(), x.to_bits(), "in-domain at {x}" );
      x += 0.017;
    }
  }

  #[ test ]
  fn the_direct_thresholds_are_the_powers_of_two_they_claim_to_be()
  {
    // Both are written as decimal literals, and a decimal literal that is a hair
    // off the intended power of two would still look right and still work — it
    // would just quietly move the seam. Asserted against the exponent rather
    // than restated.
    assert_eq!( ATAN_DIRECT.to_bits(), f64::exp2( -4.0 ).to_bits() );
    assert_eq!( ASIN_DIRECT.to_bits(), f64::exp2( -27.0 ).to_bits() );

    // And the seam is in the right place. Below the threshold `asin` *is* `atan`
    // by construction, so comparing them there proves nothing; what needs proving
    // is that the companion path it replaces was already indistinguishable from
    // `atan` on the far side, rather than the bypass cutting in partway through a
    // genuine divergence.
    // The bound tracks `c` rather than being flat, because the two functions
    // genuinely differ: `asin c - atan c` is `c³ / 2`, so their relative gap is
    // `c² / 2` and grows as the sample climbs away from the threshold. Anything
    // beyond that gap is the companion's own rounding, and a few last places is
    // all it may contribute. The check is therefore tightest right at the
    // threshold, which is where it needs to be.
    let mut c = ASIN_DIRECT;
    while c < 1.0e-6
    {
      let ( with_companion, direct ) = ( asin( c ), atan( c ) );
      let bound = c * c / 2.0 + 8.0 * f64::EPSILON;
      assert!
      (
        ( with_companion / direct - 1.0 ).abs() < bound,
        "the companion path adds more than rounding at {c:e}: {with_companion:e} vs {direct:e}"
      );
      c *= 1.5;
    }
  }

  #[ test ]
  fn the_bin_table_holds_the_arctangent_of_each_centre()
  {
    // `ATAN_V[ j ]` must be `atan( ATAN_B[ j ] )` to full precision, or the
    // reduction reassembles onto a wrong offset. Re-derived here by running the
    // series on the residual against a known-good bisection, so the table is
    // checked rather than restated.
    for j in 0 .. 4
    {
      let b = ATAN_B[ j ];
      // atan( b ) via the series directly — | b | <= 0.875 is outside the
      // series' range, so bisect on `tan` instead, which is independent of
      // everything this table feeds.
      let ( mut lo, mut hi ) = ( 0.0_f64, 1.0_f64 );
      for _ in 0 .. 200
      {
        let mid = 0.5 * ( lo + hi );
        if crate::tan( mid ) < b { lo = mid; } else { hi = mid; }
      }
      assert!( ( ATAN_V[ j ] - lo ).abs() < 1.0e-15, "ATAN_V[ {j} ]" );
    }
  }

  #[ test ]
  fn the_direct_path_and_the_reduced_path_agree_at_the_seam()
  {
    // Either side of `ATAN_DIRECT` the function switches strategy entirely. A
    // discontinuity there would be invisible to every identity test, because
    // both sides are individually self-consistent.
    //
    // Both strategies are evaluated at the *same* argument rather than at two
    // points straddling the threshold. Sampling either side instead measures
    // `d( atan )/dx ≈ 1` multiplied by the sampling gap, which looks exactly
    // like a jump of that size and says nothing about continuity.
    for t in [ ATAN_DIRECT, 0.1, 0.2 ]
    {
      let direct = atan_series( t );

      let j = ( ( t * 4.0 ) as usize ).min( 3 );
      let b = ATAN_B[ j ];
      let reduced = ATAN_V[ j ] + atan_series( ( t - b ) / ( 1.0 + t * b ) );

      assert!
      (
        ( direct / reduced - 1.0 ).abs() < 1.0e-14,
        "paths disagree at {t}: {direct} vs {reduced}"
      );
    }
  }
}

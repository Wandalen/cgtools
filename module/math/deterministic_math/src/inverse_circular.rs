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
pub fn clamp_unit( x : f64 ) -> f64
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

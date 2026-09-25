//! Sine, cosine, tangent — one argument reduction and two polynomials.

use crate::algebraic::round_half_away;
use crate::constant::{ INV_PIO2, PIO2_HI, PIO2_LO, PIO2_MD, RECIP_FACT, SIN_COS_MAX };

/// Sine and cosine together, for `| x | < 1e8`; `NaN` outside that range.
///
/// Computed as one reduction and two polynomials rather than two reductions,
/// which is both cheaper and the reason the pair always agree: `sin` and `cos`
/// of the same argument come from the same residual, so `sin² + cos² == 1`
/// holds to rounding rather than to twice the reduction error.
///
/// # Why the range is refused rather than merely documented
///
/// The three-part `PI/2` reduction runs out of bits past
/// [`crate::SIN_COS_MAX`], and the quarter-turn count is an `i32` that a larger
/// argument saturates. The range was once stated and not checked, so an
/// argument past it did not produce a poor answer — it produced an arithmetic
/// overflow inside the rounding helper, which panics in a debug build and wraps
/// in a release one. That was reached from a diverging Newton iterate, which is
/// exactly the situation where a caller has least reason to expect the library
/// to abort.
///
/// `NaN` is the answer because no answer exists: nothing about a `1e20`-radian
/// angle survives to the fifteenth digit, so a finite return would be a number
/// with no information in it, and those are the ones that propagate silently.
///
/// ```
/// use deterministic_math::{ sin_cos, PI };
/// let ( s, c ) = sin_cos( PI / 6.0 );
/// assert!( ( s - 0.5 ).abs() < 1e-15 );
/// assert!( ( s * s + c * c - 1.0 ).abs() < 1e-15 );
///
/// // Past the reduction's range, and it says so rather than guessing.
/// assert!( sin_cos( 1.0e20 ).0.is_nan() );
/// ```
#[ must_use ]
pub fn sin_cos( x : f64 ) -> ( f64, f64 )
{
  if !x.is_finite() || x.abs() > SIN_COS_MAX
  {
    return ( f64::NAN, f64::NAN );
  }

  let n = round_half_away( x * INV_PIO2 );
  let nf = f64::from( n );
  let r = x - nf * PIO2_HI - nf * PIO2_MD - nf * PIO2_LO;

  let s = sin_reduced( r );
  let c = cos_reduced( r );

  match n.rem_euclid( 4 )
  {
    0 => ( s, c ),
    1 => ( c, -s ),
    2 => ( -s, -c ),
    _ => ( -c, s ),
  }
}

/// Sine. See [`sin_cos`] for the reduction and its range.
///
/// ```
/// use deterministic_math::{ sin, PI };
/// assert!( sin( PI ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn sin( x : f64 ) -> f64
{
  sin_cos( x ).0
}

/// Cosine. See [`sin_cos`] for the reduction and its range.
///
/// ```
/// use deterministic_math::{ cos, PI };
/// assert!( ( cos( PI ) + 1.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn cos( x : f64 ) -> f64
{
  sin_cos( x ).1
}

/// Tangent, as the quotient of [`sin_cos`]'s pair.
///
/// ```
/// use deterministic_math::{ tan, PI };
/// assert!( ( tan( PI / 4.0 ) - 1.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn tan( x : f64 ) -> f64
{
  let ( s, c ) = sin_cos( x );
  s / c
}

/// `sin r` for `| r | <= PI / 4`, Taylor in Horner form on `r²`.
#[ must_use ]
pub fn sin_reduced( r : f64 ) -> f64
{
  let r2 = r * r;
  let mut acc = -RECIP_FACT[ 19 ];
  for j in ( 0 ..= 8 ).rev()
  {
    let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
    acc = acc * r2 + sign * RECIP_FACT[ 2 * j + 1 ];
  }
  r * acc
}

/// `cos r` for `| r | <= PI / 4`, Taylor in Horner form on `r²`.
#[ must_use ]
pub fn cos_reduced( r : f64 ) -> f64
{
  let r2 = r * r;
  let mut acc = -RECIP_FACT[ 18 ];
  for j in ( 0 ..= 8 ).rev()
  {
    let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
    acc = acc * r2 + sign * RECIP_FACT[ 2 * j ];
  }
  acc
}

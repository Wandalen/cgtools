//! The kernel, checked by identities it must satisfy and by agreement with the
//! platform's own libm.
//!
//! # Two separate claims, deliberately kept apart
//!
//! **Self-consistency** — the crate's functions agree with each other:
//! `ln( exp x ) == x`, `sin² + cos² == 1`, `atan( tan a ) == a`. An
//! implementation that was internally coherent and uniformly wrong would pass
//! all of these, so they establish reproducibility and say nothing at all about
//! accuracy.
//!
//! **Accuracy** — the crate agrees with `f64`'s own methods, which are an
//! independently written implementation. That is what rules out coherent
//! wrongness. It is also the only place in the crate where a libm call appears,
//! and it appears in a test rather than in `src/`, which is exactly the
//! separation this crate exists to enforce: callers must not reach libm, but the
//! test that grades the crate against libm has to. That separation is not left
//! to discipline — [`contract_test`] reads `src/` and fails on any libm call
//! that appears there.
//!
//! No assertion here compares against a recorded literal. Every expected side is
//! an identity, the other half of a round trip, or a second implementation of
//! the same function — so none of them has to be renumbered when a routine is
//! retuned, and none of them can drift into asserting its own output.
//!
//! # Why the tolerances are loose and the determinism check is exact
//!
//! These functions are not correctly rounded and do not claim to be — they claim
//! to be *the same everywhere*. So accuracy is asserted within a relative
//! tolerance, while reproducibility is asserted on `to_bits()`, with no tolerance
//! at all. Tightening the first would be a different guarantee than the one the
//! crate offers; loosening the second would erase the guarantee it does offer.

use super::*;

mod algebraic_test;
mod circular_test;
mod contract_test;
mod exponential_test;
mod hyperbolic_test;
mod inverse_circular_test;

/// Accuracy bound against libm. Chosen to be well inside the crate's own
/// documented error and well outside `f64` noise, so it catches a wrong
/// reduction without failing on a last-place disagreement.
pub const TOL : f64 = 1.0e-12;

/// Relative error, falling back to absolute where the expected value is zero.
pub fn rel_err( got : f64, want : f64 ) -> f64
{
  let d = ( got - want ).abs();
  if want.abs() > 0.0 { d / want.abs() } else { d }
}

/// Distance between two finites in units of last place, saturating at
/// [ `u64::MAX` ].
///
/// Needed wherever a relative tolerance stops meaning anything — chiefly in the
/// subnormal range, where consecutive representable values are 6% apart, so
/// "within `1e-11` relative" is a bound no implementation can meet and "within 1
/// ULP" is the strongest true statement available.
///
/// Monotone bit ordering only holds within one sign, so the mixed-sign case is
/// measured as the two distances to zero added together rather than by
/// subtracting the raw patterns — which would report two adjacent values
/// straddling zero as astronomically far apart.
pub fn ulp_diff( a : f64, b : f64 ) -> u64
{
  if a == b
  {
    return 0;
  }
  if !a.is_finite() || !b.is_finite()
  {
    return if a.is_nan() && b.is_nan() { 0 } else { u64::MAX };
  }

  let key = | v : f64 |
  {
    let bits = v.to_bits();
    if v.is_sign_negative() { ( 0x8000_0000_0000_0000_u64 ).wrapping_sub( bits & 0x7FFF_FFFF_FFFF_FFFF ) } else { bits | 0x8000_0000_0000_0000 }
  };

  key( a ).abs_diff( key( b ) )
}

/// A deterministic sweep of `count` points across `[ lo, hi ]`.
///
/// Integer-derived rather than random: a failure names an input that is the same
/// input on the next run and on another machine.
pub fn sweep( lo : f64, hi : f64, count : i32 ) -> impl Iterator< Item = f64 >
{
  ( 0..count ).map( move | i | lo + ( hi - lo ) * f64::from( i ) / f64::from( count - 1 ) )
}

/// A sweep across decades — `mantissa * 10^-e` for `e` from 1 to `decades`.
///
/// The small-argument accuracy work is the reason this exists. A defect in an
/// argument reduction near zero does not show up as one bad point; it shows up as
/// a relative error that grows steadily as the argument shrinks, which a linear
/// sweep bunched around `1` never reaches and a decade sweep walks straight
/// through. The mantissa is deliberately off a round number so no sample lands on
/// a value either implementation special-cases.
///
/// The scale comes from `powf` rather than `powi` because `powi` computes a
/// negative exponent as a reciprocal, so `10f64.powi( -320 )` divides by an
/// already-overflowed denominator and returns zero — which would silently turn
/// the deepest decades into repeated samples at the origin.
pub fn decades( mantissa : f64, decades : i32 ) -> impl Iterator< Item = f64 >
{
  ( 1 ..= decades ).map( move | e | mantissa * f64::powf( 10.0, f64::from( -e ) ) )
}

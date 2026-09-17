//! Measuring the distance between two results.
//!
//! Not a pinned function and not part of the arithmetic surface — a utility for
//! *grading* that surface. It lives in the library rather than in a test because
//! the crate's own test suite and its `cost_vs_libm` example both need it, and a
//! library item is the only thing those two targets can share.
//!
//! It is public for the same reason it exists: anyone auditing this crate, or
//! porting something to it, is asking the question it answers.

/// Distance between two values in units of last place, saturating at
/// [ `u64::MAX` ].
///
/// Needed wherever a relative tolerance stops meaning anything — chiefly in the
/// subnormal range, where consecutive representable values are 6% apart, so
/// "within `1e-11` relative" is a bound no implementation can meet and "within
/// 1 ULP" is the strongest true statement available.
///
/// Monotone bit ordering only holds within one sign, so the mixed-sign case is
/// measured as the two distances to zero added together rather than by
/// subtracting the raw patterns — which would report two adjacent values
/// straddling zero as astronomically far apart. The same mapping folds `+0.0`
/// and `-0.0` onto one key, so the two zeros are zero apart rather than two.
///
/// Non-finite inputs are classified rather than measured: two NaNs are treated
/// as equal, as are two infinities of the same sign, and every other pairing
/// involving one saturates.
///
/// ```rust
/// use deterministic_math::measure::ulp_diff;
///
/// assert_eq!( ulp_diff( 1.0, 1.0 ), 0 );
/// assert_eq!( ulp_diff( 0.0, -0.0 ), 0 );
/// assert_eq!( ulp_diff( 1.0, f64::from_bits( 1.0_f64.to_bits() + 1 ) ), 1 );
/// assert_eq!( ulp_diff( f64::INFINITY, f64::INFINITY ), 0 );
/// assert_eq!( ulp_diff( f64::INFINITY, 1.0 ), u64::MAX );
/// ```
#[ must_use ]
pub fn ulp_diff( a : f64, b : f64 ) -> u64
{
  if a.is_nan() || b.is_nan()
  {
    return if a.is_nan() && b.is_nan() { 0 } else { u64::MAX };
  }
  if !a.is_finite() || !b.is_finite()
  {
    // Compared as bits rather than with `==`: same-sign infinities are the same
    // value and everything else here is unboundedly far away. `to_bits` answers
    // both without an equality comparison on floats.
    return if a.to_bits() == b.to_bits() { 0 } else { u64::MAX };
  }

  // Maps the float line onto `u64` in increasing order. Positives keep their
  // pattern with the top bit set; negatives are reflected below it. `+0.0` and
  // `-0.0` both land on `0x8000_0000_0000_0000`, which is why no separate
  // equality fast path is needed for them.
  let key = | v : f64 |
  {
    let bits = v.to_bits();
    if v.is_sign_negative() { ( 0x8000_0000_0000_0000_u64 ).wrapping_sub( bits & 0x7FFF_FFFF_FFFF_FFFF ) } else { bits | 0x8000_0000_0000_0000 }
  };

  key( a ).abs_diff( key( b ) )
}

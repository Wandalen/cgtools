//! The functions IEEE-754 already pins, plus the two built from nothing but
//! pinned operations.
//!
//! Nothing in this module needed reimplementing to be reproducible — `sqrt` and
//! `mul_add` are two of the six operations the standard requires to be
//! correctly rounded, and `hypot` and `powi` are compositions of `*`, `+`, `/`
//! and `sqrt`. They are here so that a reader auditing a codebase for libm
//! calls finds every arithmetic entry point in one crate, including the ones
//! that were never at risk. A call site that spells `deterministic_math::hypot`
//! states its intent; one that spells `x.hypot( y )` looks exactly like the
//! mistake next to it.

/// Square root — IEEE-754 requires this one to be correctly rounded, so the
/// platform's own is already bit-identical everywhere and is used directly.
///
/// ```
/// use deterministic_math::sqrt;
/// assert_eq!( sqrt( 4.0 ), 2.0 );
/// ```
#[ must_use ]
#[ inline ]
pub fn sqrt( x : f64 ) -> f64
{
  x.sqrt()
}

/// `a * b + c`, computed with a single rounding.
///
/// # This is pinned, and the bare method form is still worth banning
///
/// Fused multiply-add is the sixth operation IEEE-754 requires to be correctly
/// rounded, so it is exactly as reproducible as `+` or `sqrt`, and a target
/// without the instruction reaches a software `fma` that is correctly rounded
/// too rather than quietly degrading to two steps. Determinism is not the
/// concern here.
///
/// The concern is that `a * b + c` and `fma( a, b, c )` are *different
/// functions* — one rounding versus two — and both are correct. Two call sites
/// that ought to agree will not if one is written each way, and at a glance
/// `x.mul_add( y, z )` and `x * y + z` read as the same expression. Routing the
/// fused form through a named function makes the choice legible, which is the
/// same reason [`sqrt`] is here.
///
/// Rust never contracts `a * b + c` into an FMA on its own, so the unfused
/// spelling stays unfused and the two forms never swap underfoot.
///
/// ```
/// use deterministic_math::mul_add;
/// // Exact in this case, and the point is that it is the *same* exact value
/// // on every target.
/// assert_eq!( mul_add( 2.0, 3.0, 1.0 ), 7.0 );
/// ```
#[ must_use ]
#[ inline ]
pub fn mul_add( a : f64, b : f64, c : f64 ) -> f64
{
  a.mul_add( b, c )
}

/// `sqrt( x² + y² )`, without overflowing on the way.
///
/// The reason this exists rather than being left to the caller: the obvious
/// `sqrt( x * x + y * y )` overflows to infinity as soon as either argument
/// passes `1.34e154`, even though the answer is an ordinary finite number, and
/// it underflows to zero for arguments below `1.5e-154`. Both failures are
/// silent and both are reachable from real data — a distance in metres between
/// two points a light-year apart is `9.5e15`, and squaring a stiffness ratio
/// gets small fast.
///
/// Scaling by the larger magnitude removes both: `a * sqrt( 1 + ( b / a )² )`
/// squares only a ratio in `[ 0, 1 ]`, which cannot overflow, and the leading
/// `a` carries the exponent unharmed.
///
/// ```
/// use deterministic_math::hypot;
/// assert!( ( hypot( 3.0, 4.0 ) - 5.0 ).abs() < 1.0e-15 );
///
/// // The naive form is already infinite here; this is not.
/// let big : f64 = 1.0e200;
/// assert!( ( big * big ).is_infinite() );
/// assert!( ( hypot( big, big ) / ( big * core::f64::consts::SQRT_2 ) - 1.0 ).abs() < 1.0e-15 );
/// ```
#[ must_use ]
pub fn hypot( x : f64, y : f64 ) -> f64
{
  if x.is_nan() || y.is_nan()
  {
    // An infinity wins over a NaN in the other argument, matching the IEEE
    // recommendation: the magnitude is known to be infinite whatever the other
    // component turns out to be.
    if x.is_infinite() || y.is_infinite()
    {
      return f64::INFINITY;
    }
    return f64::NAN;
  }

  let ( a, b ) = ( x.abs(), y.abs() );
  let ( large, small ) = if a > b { ( a, b ) } else { ( b, a ) };

  if large == 0.0
  {
    return 0.0;
  }
  if large.is_infinite()
  {
    return f64::INFINITY;
  }

  let r = small / large;
  large * sqrt( 1.0 + r * r )
}

/// `x` raised to an integer power, by binary exponentiation.
///
/// Distinct from [`crate::powf`] in two ways that both matter. It accepts a
/// negative base, where `powf` cannot — `powf` routes through `ln`, which is
/// undefined there — so `powi( -2.0, 3 )` is `-8.0` rather than `NaN`. And it
/// is built from multiplication alone, so for a small exponent it is both
/// faster and more accurate than the exponential round trip.
///
/// The squaring chain is written out rather than left to `f64::powi`, whose
/// association the optimiser is free to reorder; here the order of every
/// multiplication is fixed by the loop.
///
/// ```
/// use deterministic_math::powi;
/// assert_eq!( powi( 2.0, 10 ), 1024.0 );
/// assert_eq!( powi( -2.0, 3 ), -8.0 );
/// assert_eq!( powi( 5.0, 0 ), 1.0 );
/// assert_eq!( powi( 2.0, -2 ), 0.25 );
/// ```
#[ must_use ]
pub fn powi( x : f64, n : i32 ) -> f64
{
  if n == 0
  {
    return 1.0;
  }

  // `n.unsigned_abs()` rather than `-n`, which overflows at `i32::MIN`.
  let mut e = n.unsigned_abs();
  let mut base = x;
  let mut acc = 1.0;

  loop
  {
    if e & 1 == 1
    {
      acc *= base;
    }
    e >>= 1;
    if e == 0
    {
      break;
    }
    base *= base;
  }

  if n < 0 { 1.0 / acc } else { acc }
}

/// `atan u` for `| u | <= 0.2`, by its alternating series.
///
/// Eleven terms. The first omitted term is `u^23 / 23`, under `4e-19` at the
/// range edge.
///
/// Lives here rather than beside its caller because the logarithm needs the
/// same shape: `ln( 1 + x )` is `2 * atanh( x / ( 2 + x ) )`, and the two
/// series differ only in whether the signs alternate. Keeping one of them here
/// and duplicating the other was the alternative, and duplicating a
/// coefficient loop is how two copies of it come to disagree.
#[ must_use ]
pub fn atan_series( u : f64 ) -> f64
{
  // The division below looks like the defect `RECIP_FACT` documents and is not
  // one: `j` is loop-constant, so it folds. Tabulating it was measured and
  // changed nothing.
  let u2 = u * u;
  let mut acc = 1.0 / 23.0;
  for j in ( 0 ..= 10 ).rev()
  {
    let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
    acc = acc * u2 + sign / ( 2.0 * f64::from( j ) + 1.0 );
  }
  u * acc
}

/// `atanh s` for `| s | <= 0.18`, by its series — the same shape as
/// [`atan_series`] without the alternating sign.
///
/// Eleven terms, matching the arctangent's. Both logarithm paths reduce to a
/// small `s` before calling: `ln` re-centres its mantissa onto
/// `[ sqrt( 1/2 ), sqrt( 2 ) )` so that `s = ( m - 1 ) / ( m + 1 )` stays under
/// `0.1716`, and `ln_1p` forms `s = x / ( 2 + x )`, under `0.149` across the
/// band it uses this on.
#[ must_use ]
pub fn atanh_series( s : f64 ) -> f64
{
  let s2 = s * s;
  let mut acc = 1.0 / 21.0;
  for j in ( 0 ..= 9 ).rev()
  {
    acc = acc * s2 + 1.0 / ( 2.0 * f64::from( j ) + 1.0 );
  }
  s * acc
}

/// Multiply by `2^k`.
///
/// Written with bit manipulation rather than `x * 2f64.powi( k )` because that
/// is a multiply chain whose association the optimiser may reorder.
///
/// # Why this is two multiplications and not one
///
/// The `f64` exponent field encodes `2^k` directly only for
/// `-1022 <= k <= 1023`; outside that window there is no single `f64` equal to
/// `2^k` to multiply by. Doing it anyway is not a rounding error but a
/// wraparound: `1023 + k` goes negative, the cast to `u64` wraps to a value
/// whose low bits land in the sign and exponent fields, and the multiplier
/// becomes a large *negative* number. `exp( -745.0 )` returned
/// `-9.12e292` where the answer is `5e-324` — wrong sign, and wrong by six
/// hundred orders of magnitude.
///
/// This mattered nowhere until `exp2` was added, because the only test that
/// reached this path swept `exp` over `[ -300, 300 ]`, and `k` there never
/// leaves the window. `exp2`'s domain reaches `k = -1075` on its first sample.
///
/// Splitting `k` in half puts both factors inside the window. The first product
/// is exact — `x` is `O( 1 )` and the shift is representable — so the second is
/// the only rounding, which is what makes a subnormal result correctly rounded
/// rather than merely close.
#[ must_use ]
pub fn scale2( x : f64, k : i32 ) -> f64
{
  if ( -1022 ..= 1023 ).contains( &k )
  {
    return x * pow2( k );
  }
  let half = k / 2;
  x * pow2( half ) * pow2( k - half )
}

/// `2^k` as an exact `f64`, for `-1022 <= k <= 1023`.
///
/// The caller is responsible for the range; [`scale2`] is the one that knows how
/// to stay inside it.
#[ must_use ]
fn pow2( k : i32 ) -> f64
{
  let e = ( 1023i32 + k ) as u64;
  f64::from_bits( e << 52 )
}

/// Round to nearest, halfway away from zero, as an `i32`.
///
/// `f64::round` has exactly these semantics and is exact for every value the
/// reductions here produce, but it is a libm entry point on some targets; the
/// two comparisons below are not.
#[ must_use ]
pub fn round_half_away( x : f64 ) -> i32
{
  let t = x as i32;
  let frac = x - f64::from( t );
  if frac >= 0.5
  {
    return t + 1;
  }
  if frac <= -0.5
  {
    return t - 1;
  }
  t
}

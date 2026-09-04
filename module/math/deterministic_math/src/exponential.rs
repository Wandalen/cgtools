//! Exponentials, logarithms, and the powers built from them.
//!
//! Every function here reduces to the same two kernels: a Taylor series for
//! `e^r` on a narrow range, and an inverse-hyperbolic-tangent series for
//! `ln m` on a narrow range. Everything else is argument reduction, which is
//! exact because it only ever adds an integer to an exponent field.

use crate::algebraic::{ atanh_series, round_half_away, scale2 };
use crate::constant::
{
  LN2_HI, LN2_LO, LOG2_E, LOG10_2, LOG10_E, RECIP_FACT, SERIES_BAND,
};

/// `e^x`, for `x` in `[ -745, 709 ]`.
///
/// Outside that range the answer is `0.0` or `f64::INFINITY`; a `NaN` in gives
/// a `NaN` out. Accurate to under 2 ULP across the range, which the round trip
/// through [`ln`] in this crate's tests measures rather than assumes.
///
/// ```
/// use deterministic_math::{ exp, ln };
/// let x = 3.75;
/// assert!( ( ln( exp( x ) ) - x ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn exp( x : f64 ) -> f64
{
  if x.is_nan()
  {
    return x;
  }
  if x > 709.782_712_893_384
  {
    return f64::INFINITY;
  }
  if x < -745.133_219_101_941_1
  {
    return 0.0;
  }

  // Cody-Waite: strip k whole factors of 2, leaving | r | <= ln( 2 ) / 2.
  let k = round_half_away( x * LOG2_E );
  let kf = f64::from( k );
  let r = x - kf * LN2_HI - kf * LN2_LO;

  scale2( exp_reduced( r ), k )
}

/// `2^x`, for `x` in `[ -1074, 1024 ]`.
///
/// Reduced in base two directly rather than routed through `exp( x * LN_2 )`,
/// which would round the product before exponentiating it and lose a digit for
/// every large argument. Here the integer part of `x` moves into the exponent
/// field untouched and only the fraction reaches a series, so `exp2` of any
/// integer is exact.
///
/// ```
/// use deterministic_math::exp2;
/// assert_eq!( exp2( 10.0 ), 1024.0 );
/// assert_eq!( exp2( -3.0 ), 0.125 );
/// assert!( ( exp2( 0.5 ) - core::f64::consts::SQRT_2 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn exp2( x : f64 ) -> f64
{
  if x.is_nan()
  {
    return x;
  }
  if x > 1024.0
  {
    return f64::INFINITY;
  }
  if x < -1075.0
  {
    return 0.0;
  }

  // | r | <= 0.5, so | r * ln 2 | <= 0.347 — inside `exp_reduced`'s range.
  let k = round_half_away( x );
  let r = x - f64::from( k );

  scale2( exp_reduced( r * LN2_HI + r * LN2_LO ), k )
}

/// `e^r` for `| r | <= 0.35`, by Taylor series in Horner form.
///
/// Sixteen terms. The first omitted term is `r^17 / 17!`, under `1e-20` at the
/// range edge, so the series error sits well below the rounding error of
/// evaluating it.
#[ must_use ]
pub fn exp_reduced( r : f64 ) -> f64
{
  let mut s = RECIP_FACT[ 16 ];
  for n in ( 1 ..= 15 ).rev()
  {
    s = s * r + RECIP_FACT[ n ];
  }
  s * r + 1.0
}

/// Split `x` into a mantissa on `[ sqrt( 1/2 ), sqrt( 2 ) )` and a power of two.
///
/// `x == m * 2^k` exactly — the exponent is read straight out of the bit
/// pattern and the mantissa is the same bits with a new exponent, so nothing is
/// rounded. Re-centring onto a range that straddles `1`, rather than the
/// `[ 1, 2 )` the format hands over, is what keeps the logarithm's series
/// argument small on *both* sides of `1` and so leaves no cancellation for it
/// to amplify.
///
/// Callers must have excluded zero, negatives, infinities and `NaN` already.
///
/// # Subnormals are lifted first
///
/// A subnormal has a zero exponent field and *no implicit leading bit*, so the
/// extraction below — which restores that bit unconditionally — would read its
/// mantissa as a normal one and return an exponent up to 51 too high. It is not
/// a precision loss but a wrong answer: `log2( 5e-324 )` came back `-1023`
/// instead of `-1074`.
///
/// Multiplying by `2^54` first moves any subnormal into the normal range, and
/// does so exactly: a subnormal carries at most 52 significant bits and the
/// scaling only shifts its exponent, so nothing rounds. The matching subtraction
/// from `k` undoes it.
#[ must_use ]
pub fn mantissa_exponent( x : f64 ) -> ( f64, i32 )
{
  /// Enough to lift the smallest subnormal, `2^-1074`, clear of `2^-1022`.
  const LIFT : i32 = 54;

  let ( x, lifted ) = if x < f64::MIN_POSITIVE { ( scale2( x, LIFT ), LIFT ) } else { ( x, 0 ) };

  let bits = x.to_bits();
  let mut k = ( ( bits >> 52 ) & 0x7ff ) as i32 - 1023 - lifted;
  let mut m = f64::from_bits( ( bits & 0x000f_ffff_ffff_ffff ) | ( 1023u64 << 52 ) );

  if m > core::f64::consts::SQRT_2
  {
    m *= 0.5;
    k += 1;
  }

  ( m, k )
}

/// `ln m` for `m` on `[ sqrt( 1/2 ), sqrt( 2 ) )`, by the series
/// `2 * atanh( ( m - 1 ) / ( m + 1 ) )`.
///
/// `| s |` stays under `0.1716` across that range, which is why eleven terms
/// reach full precision.
#[ must_use ]
pub fn ln_mantissa( m : f64 ) -> f64
{
  2.0 * atanh_series( ( m - 1.0 ) / ( m + 1.0 ) )
}

/// Natural logarithm, for `x > 0`.
///
/// `ln( 0.0 )` is `-f64::INFINITY` and a negative or `NaN` argument gives
/// `NaN`, matching the shape of the IEEE recommendation without depending on
/// any particular libm to supply it.
///
/// # Accuracy
///
/// Within **3 ULP everywhere measured**, and within 1 ULP over most of the
/// range: sampling 200 000 points per band against the platform's own `ln`
/// gives a worst case of 1 ULP across `1e-300 .. 1e-1`, `1 .. 1e6` and
/// `1e6 .. 1e300`, rising to 3 ULP inside `[ 0.5, 2 ]`, where the result itself
/// is small enough that one ULP of it is a large relative step.
///
/// Near 1 it does not degrade, which is the claim an atanh-series logarithm
/// most needs to make — `mantissa_exponent` in this module is the re-centring
/// that buys it, named rather than linked because it is private and a rustdoc
/// link would render dead. Over the 10 000 consecutive doubles astride `1.0`
/// the error is at most 1 ULP, and `ln( 1.0 )` is exactly `0.0`.
///
/// What none of this rescues is a caller who forms `1.0 + u` for small `u`
/// before calling. That rounds `u`'s low bits away before `ln` is reached, and
/// no accuracy inside `ln` can recover what the addition already discarded —
/// pass `u` to [`ln_1p`] instead. The distinction is worth keeping straight:
/// `ln_1p` exists for the argument, not for a weakness in `ln`.
///
/// ```
/// use deterministic_math::{ exp, ln };
/// let y = 1234.5;
/// assert!( ( exp( ln( y ) ) / y - 1.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn ln( x : f64 ) -> f64
{
  if x.is_nan() || x < 0.0
  {
    return f64::NAN;
  }
  if x == 0.0
  {
    return f64::NEG_INFINITY;
  }
  if x.is_infinite()
  {
    return f64::INFINITY;
  }

  let ( m, k ) = mantissa_exponent( x );
  let kf = f64::from( k );
  kf * LN2_HI + ( kf * LN2_LO + ln_mantissa( m ) )
}

/// Base-2 logarithm, for `x > 0`.
///
/// The exponent term is an integer and contributes no rounding at all, so
/// `log2` of any power of two is exact — including the large ones, where
/// `ln( x ) / LN_2` would have rounded a big logarithm before dividing and
/// returned something a hair off an integer.
///
/// ```
/// use deterministic_math::log2;
/// assert_eq!( log2( 1024.0 ), 10.0 );
/// assert_eq!( log2( 0.125 ), -3.0 );
/// assert_eq!( log2( f64::MIN_POSITIVE ), -1022.0 );
/// ```
#[ must_use ]
pub fn log2( x : f64 ) -> f64
{
  if x.is_nan() || x < 0.0
  {
    return f64::NAN;
  }
  if x == 0.0
  {
    return f64::NEG_INFINITY;
  }
  if x.is_infinite()
  {
    return f64::INFINITY;
  }

  let ( m, k ) = mantissa_exponent( x );
  f64::from( k ) + ln_mantissa( m ) * LOG2_E
}

/// Base-10 logarithm, for `x > 0`.
///
/// Split the same way [`log2`] is, so the exponent contributes one exact
/// multiplication rather than passing through a logarithm.
///
/// ```
/// use deterministic_math::log10;
/// assert!( ( log10( 1000.0 ) - 3.0 ).abs() < 1e-15 );
/// assert!( ( log10( 0.01 ) + 2.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn log10( x : f64 ) -> f64
{
  if x.is_nan() || x < 0.0
  {
    return f64::NAN;
  }
  if x == 0.0
  {
    return f64::NEG_INFINITY;
  }
  if x.is_infinite()
  {
    return f64::INFINITY;
  }

  let ( m, k ) = mantissa_exponent( x );
  f64::from( k ) * LOG10_2 + ln_mantissa( m ) * LOG10_E
}

/// Logarithm of `x` in an arbitrary `base`.
///
/// Unlike [`log2`] and [`log10`] this is genuinely two logarithms and a
/// division, so it carries roughly twice their rounding error and there is no
/// reduction that avoids it. Where the base is 2 or 10, prefer the dedicated
/// function; where it is a runtime value, this is the honest cost.
///
/// ```
/// use deterministic_math::log;
/// assert!( ( log( 81.0, 3.0 ) - 4.0 ).abs() < 1e-14 );
/// ```
#[ must_use ]
pub fn log( x : f64, base : f64 ) -> f64
{
  ln( x ) / ln( base )
}

/// `x` raised to the real power `y`, for `x >= 0`.
///
/// Positive base only. A negative base raised to a non-integer power is not a
/// real number, and a negative base raised to an integer power is a loop rather
/// than an exponential — for that case use [`crate::powi`], which accepts a
/// negative base and is both faster and more accurate for a small exponent.
/// Rather than implement a branch that guesses which the caller meant, the
/// negative case returns `NaN` and says so here.
///
/// ```
/// use deterministic_math::powf;
/// // Path independence: x^a * x^b == x^( a + b ).
/// let ( x, a, b ) = ( 7.5, 0.4, 1.3 );
/// let lhs = powf( x, a ) * powf( x, b );
/// assert!( ( lhs / powf( x, a + b ) - 1.0 ).abs() < 1e-14 );
/// ```
#[ must_use ]
pub fn powf( x : f64, y : f64 ) -> f64
{
  if y == 0.0
  {
    return 1.0;
  }
  if x == 0.0
  {
    return if y > 0.0 { 0.0 } else { f64::INFINITY };
  }
  if x < 0.0
  {
    return f64::NAN;
  }
  exp( y * ln( x ) )
}

/// Cube root, defined for negative arguments.
///
/// The reason this exists rather than being left to the caller: the obvious
/// substitution `powf( x, 1.0 / 3.0 )` returns `NaN` for every negative `x`,
/// because `powf` is undefined for a negative base with a fractional exponent.
/// Cube root is not — every real has exactly one real cube root. So the
/// identity is not merely less accurate, it is wrong on half the domain, and
/// wrong silently.
///
/// Odd symmetry is applied around the sign, and `1/3` is never formed as an
/// `f64` — `exp( ln( m ) / 3.0 )` divides by an exact integer instead, which
/// avoids rounding the exponent before using it.
///
/// ```rust
/// use deterministic_math::cbrt;
/// assert!( ( cbrt( 27.0 ) - 3.0 ).abs() < 1.0e-12 );
/// assert!( ( cbrt( -27.0 ) + 3.0 ).abs() < 1.0e-12 );
/// assert_eq!( cbrt( 0.0 ), 0.0 );
/// ```
#[ must_use ]
pub fn cbrt( x : f64 ) -> f64
{
  if x == 0.0 || !x.is_finite()
  {
    return x;
  }

  let m = if x < 0.0 { -x } else { x };
  let r = exp( ln( m ) / 3.0 );

  // One Newton step on r^3 = m tightens the division's rounding without
  // reintroducing a libm call: r <- r - ( r - m / r^2 ) / 3.
  let r = r - ( r - m / ( r * r ) ) / 3.0;

  if x < 0.0 { -r } else { r }
}

/// `exp( x ) - 1`, accurate when `x` is near zero.
///
/// The reason this exists: `exp( x ) - 1.0` computes a value near `1` and then
/// subtracts `1`, so for small `x` the leading bits cancel and the result keeps
/// only whatever precision survived — catastrophic cancellation. At
/// `x = 1e-16` the naive form returns exactly `0.0` while the true answer is
/// `1e-16`. The precision this function exists to preserve is exactly the
/// precision the identity destroys.
///
/// Across `[ -0.25, 0.25 ]` the series `x * ( 1 + x/2! + x²/3! + ... )` is used
/// directly — sixteen terms, so the truncation error is below `1e-24` at the
/// band edge and the result is limited only by the rounding of evaluating it.
/// See `constant::SERIES_BAND` for why the band ends where it does; the short
/// version is that outside it the naive form has no cancellation left to suffer
/// from.
///
/// ```rust
/// use deterministic_math::exp_m1;
/// // The naive identity collapses to zero here; this does not.
/// assert!( exp_m1( 1.0e-16 ) > 0.0 );
/// assert!( ( exp_m1( 1.0e-16 ) / 1.0e-16 - 1.0 ).abs() < 1.0e-15 );
/// assert!( ( exp_m1( 1.0 ) - 1.718_281_828_459_045 ).abs() < 1.0e-15 );
///
/// // And it holds through the band the naive form used to be reached across.
/// let x = 1.0e-3;
/// assert!( ( exp_m1( x ) / x.exp_m1() - 1.0 ).abs() < 1.0e-15 );
/// ```
#[ must_use ]
pub fn exp_m1( x : f64 ) -> f64
{
  if !x.is_finite()
  {
    return if x < 0.0 { -1.0 } else { x };
  }

  if x.abs() < SERIES_BAND
  {
    // exp( x ) - 1 == x * sum( x^( n - 1 ) / n! ), which never forms the 1 that
    // the subtraction would have had to remove.
    let mut acc = RECIP_FACT[ 16 ];
    for n in ( 1 ..= 15 ).rev()
    {
      acc = acc * x + RECIP_FACT[ n ];
    }
    return x * acc;
  }

  exp( x ) - 1.0
}

/// `ln( 1 + x )`, accurate when `x` is near zero.
///
/// The mirror of [`exp_m1`], and the failure is the same shape from the other
/// side: forming `1.0 + x` for small `x` rounds away the low bits of `x`
/// *before* `ln` ever sees them, so the argument is already wrong when the
/// logarithm is taken. At `x = 1e-17` the sum is exactly `1.0` and the naive
/// form returns `0.0`.
///
/// Across `[ -0.25, 0.25 ]` the identity
/// `ln( 1 + x ) == 2 * atanh( x / ( 2 + x ) )` is used instead. It never forms
/// `1 + x` at all: the series argument `s = x / ( 2 + x )` is computed from `x`
/// directly and keeps every bit of it, so the accuracy of the result is limited
/// by the series rather than by an addition that happened before it.
///
/// ```rust
/// use deterministic_math::ln_1p;
/// // `1.0 + 1e-17` rounds to `1.0`, so the naive identity returns zero.
/// assert_eq!( ( 1.0_f64 + 1.0e-17 ).to_bits(), 1.0_f64.to_bits() );
/// assert!( ln_1p( 1.0e-17 ) > 0.0 );
/// assert!( ( ln_1p( 1.0e-17 ) / 1.0e-17 - 1.0 ).abs() < 1.0e-15 );
///
/// // And it holds through the band the naive form used to be reached across.
/// let x = 1.0e-3;
/// assert!( ( ln_1p( x ) / x.ln_1p() - 1.0 ).abs() < 1.0e-15 );
/// ```
#[ must_use ]
pub fn ln_1p( x : f64 ) -> f64
{
  if !x.is_finite() || x <= -1.0
  {
    return ln( 1.0 + x );
  }

  if x.abs() < SERIES_BAND
  {
    return 2.0 * atanh_series( x / ( 2.0 + x ) );
  }

  ln( 1.0 + x )
}

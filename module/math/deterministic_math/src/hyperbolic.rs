//! Hyperbolic functions and their inverses.
//!
//! Every one of these has a textbook closed form built from `exp` or `ln`, and
//! every one of those closed forms cancels catastrophically somewhere in its
//! domain — near zero for the direct functions, near zero or near one for the
//! inverses. Each function below therefore carries a second branch whose only
//! job is to compute the same quantity without ever forming the `1` that the
//! subtraction would have had to remove. [`crate::exp_m1`] and
//! [`crate::ln_1p`] exist for exactly this, and this module is their largest
//! consumer.

use crate::algebraic::sqrt;
use crate::constant::{ HYPERBOLIC_SATURATION, LARGE_ARGUMENT, LN_2 };
use crate::exponential::{ exp, exp_m1, ln, ln_1p };

/// Hyperbolic sine.
///
/// # Why this is not `( eˣ - e⁻ˣ ) / 2`
///
/// It was, and near zero that form loses most of its digits: `eˣ` and `e⁻ˣ` are
/// both close to `1` there, so the subtraction cancels the leading bits and
/// keeps only what the two roundings left behind. Against the platform's libm
/// the difference reached 7 163 ULP, all of it below `| x | ≈ 1e-4`.
///
/// The identity used instead follows from writing `u = eˣ - 1`:
/// `eˣ - e⁻ˣ` is `( 1 + u ) - 1 / ( 1 + u )`, which is `u( u + 2 ) / ( 1 + u )`.
/// No `1` is ever formed and removed, and `u` comes from [`crate::exp_m1`],
/// which keeps every bit of a small argument.
///
/// ```
/// use deterministic_math::{ asinh, sinh };
/// let h = 1.25;
/// assert!( ( asinh( sinh( h ) ) - h ).abs() < 1e-14 );
///
/// // The band the difference-of-exponentials form used to lose.
/// let small = 1.0e-9;
/// assert!( ( sinh( small ) / small - 1.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn sinh( x : f64 ) -> f64
{
  if !x.is_finite()
  {
    return x;
  }

  let a = x.abs();
  let r = if a < 1.0
  {
    let u = exp_m1( a );
    0.5 * u * ( u + 2.0 ) / ( 1.0 + u )
  }
  else if a > HYPERBOLIC_SATURATION
  {
    // `e⁻ᵃ` cannot change a bit here, and halving inside the exponent keeps the
    // result finite for an `a` whose `eᵃ` would already have overflowed.
    exp( a - LN_2 )
  }
  else
  {
    let e = exp( a );
    0.5 * ( e - 1.0 / e )
  };

  if x < 0.0 { -r } else { r }
}

/// Hyperbolic cosine.
///
/// The one member of the family with no cancellation anywhere — both terms of
/// `( eˣ + e⁻ˣ ) / 2` are positive, so nothing can subtract away. Only the
/// large-argument branch is needed, and only to keep the answer finite past
/// where `eˣ` alone would overflow.
///
/// ```
/// use deterministic_math::{ cosh, sinh };
/// let h = 0.6;
/// assert!( ( cosh( h ) * cosh( h ) - sinh( h ) * sinh( h ) - 1.0 ).abs() < 1e-14 );
/// ```
#[ must_use ]
pub fn cosh( x : f64 ) -> f64
{
  if x.is_nan()
  {
    return x;
  }

  let a = x.abs();
  if a > HYPERBOLIC_SATURATION
  {
    return exp( a - LN_2 );
  }

  let e = exp( a );
  0.5 * ( e + 1.0 / e )
}

/// Hyperbolic tangent.
///
/// Same cancellation as [`sinh`] and the same cure. `( e²ˣ - 1 ) / ( e²ˣ + 1 )`
/// destroys a small argument in its numerator; writing `u = e²ˣ - 1` turns it
/// into `u / ( u + 2 )`, which does not. The old form disagreed with libm by up
/// to 1 244 ULP below `| x | ≈ 1e-4`.
///
/// ```
/// use deterministic_math::{ cosh, sinh, tanh };
/// let h = -0.8;
/// assert!( ( tanh( h ) - sinh( h ) / cosh( h ) ).abs() < 1e-15 );
///
/// let small = 1.0e-9;
/// assert!( ( tanh( small ) / small - 1.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn tanh( x : f64 ) -> f64
{
  if x.is_nan()
  {
    return x;
  }

  let a = x.abs();
  let r = if a > HYPERBOLIC_SATURATION
  {
    // Exactly 1 to the last bit, and reached without forming a huge quotient.
    1.0
  }
  else
  {
    let u = exp_m1( 2.0 * a );
    u / ( u + 2.0 )
  };

  if x < 0.0 { -r } else { r }
}

/// Inverse hyperbolic sine, over the whole real line.
///
/// Three branches, each for a failure the single closed form
/// `ln( x + sqrt( x² + 1 ) )` has somewhere.
///
/// **Near zero** the argument of `ln` tends to `1`, so the logarithm is
/// evaluated where its own accuracy is worst and the small quantity that
/// matters has already been rounded into the leading `1`. Writing
/// `x + sqrt( x² + 1 )` as `1 + ( x + x² / ( 1 + sqrt( 1 + x² ) ) )` and passing
/// the bracket to [`crate::ln_1p`] keeps it. This is what closed a 2 628 ULP
/// disagreement with libm below `| x | ≈ 1e-4`.
///
/// **Above `2^28`** the branch is not an approximation but an identity: `x² + 1`
/// rounds to `x²` there (the gap between neighbouring doubles at `7.2e16` is
/// already `16`), and `sqrt( x² )` is exact, so the direct expression *is*
/// `ln( x + x )`. Without it, `x * x` overflows to infinity above `1.34e154` —
/// barely a tenth of the way through the exponent range, and far short of what
/// a hyperbolic anomaly actually asks for: an argument of `1e250` names a real
/// point, at a result near `576`.
///
/// **Negative arguments** go by odd symmetry rather than directly, because
/// `x + sqrt( x² + 1 )` cancels catastrophically for large negative `x` — the
/// two terms agree to as many digits as `x` has.
///
/// ```
/// use deterministic_math::{ asinh, sinh };
/// let h = -4.0;
/// assert!( ( sinh( asinh( h ) ) - h ).abs() < 1e-13 );
///
/// // Past where squaring the argument would overflow.
/// let big = 1.0e250;
/// assert!( ( asinh( big ) / 576.339_420_429_071_3 - 1.0 ).abs() < 1e-15 );
///
/// // And relative accuracy near zero, which the plain `ln` form did not have.
/// let small = 1.0e-9;
/// assert!( ( asinh( small ) / small - 1.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn asinh( x : f64 ) -> f64
{
  if !x.is_finite()
  {
    return x;
  }

  let a = x.abs();
  let r = if a < 1.0
  {
    ln_1p( a + a * a / ( 1.0 + sqrt( 1.0 + a * a ) ) )
  }
  else if a > LARGE_ARGUMENT
  {
    ln( a ) + LN_2
  }
  else
  {
    ln( a + sqrt( a * a + 1.0 ) )
  };

  if x < 0.0 { -r } else { r }
}

/// Inverse hyperbolic cosine, for `x >= 1`; `NaN` below.
///
/// The domain has a hard edge at `1`, and that edge is where the closed form
/// `ln( x + sqrt( x² - 1 ) )` fails: for `x` just above `1`, `x²` rounds and the
/// subtraction of `1` then removes most of what distinguished it, so the
/// square root is computed from a handful of surviving bits. At
/// `x = 1 + 1e-10` the naive form has already lost six digits.
///
/// Substituting `t = x - 1` — exact, by Sterbenz's lemma, for every `x` in
/// `[ 0.5, 2 ]` — gives `x² - 1 == t( t + 2 )` and `x + sqrt( x² - 1 ) ==
/// 1 + t + sqrt( t( t + 2 ) )`, which [`crate::ln_1p`] takes without ever
/// forming the leading `1`.
///
/// ```
/// use deterministic_math::{ acosh, cosh };
/// let h = 2.5;
/// assert!( ( acosh( cosh( h ) ) - h ).abs() < 1e-14 );
/// assert_eq!( acosh( 1.0 ), 0.0 );
/// assert!( acosh( 0.5 ).is_nan() );
///
/// // Just above the domain edge, where the naive form loses most of its
/// // digits. Graded against the leading-order `sqrt( 2t )` rather than a
/// // recorded literal, so the assertion cannot drift into echoing its own
/// // output.
/// let near_one = 1.0 + 1.0e-10;
/// let t = near_one - 1.0;
/// assert!( ( acosh( near_one ) / deterministic_math::sqrt( 2.0 * t ) - 1.0 ).abs() < 1e-10 );
/// ```
#[ must_use ]
pub fn acosh( x : f64 ) -> f64
{
  if x.is_nan()
  {
    return x;
  }
  if x < 1.0
  {
    return f64::NAN;
  }
  if x > LARGE_ARGUMENT
  {
    // Same identity as `asinh`'s large branch: `x² - 1` rounds to `x²` here.
    return ln( x ) + LN_2;
  }

  let t = x - 1.0;
  ln_1p( t + sqrt( t * ( t + 2.0 ) ) )
}

/// Inverse hyperbolic tangent, for `| x | < 1`.
///
/// `±1` returns the correct infinity; anything outside is `NaN`.
///
/// The closed form `ln( ( 1 + x ) / ( 1 - x ) ) / 2` cancels near zero, where
/// the quotient tends to `1`. The identity
/// `( 1 + x ) / ( 1 - x ) == 1 + 2x / ( 1 - x )` moves the small quantity out
/// of the leading `1` and into an argument [`crate::ln_1p`] can keep.
///
/// ```
/// use deterministic_math::{ atanh, tanh };
/// let h = 0.4;
/// assert!( ( atanh( tanh( h ) ) - h ).abs() < 1e-14 );
/// assert!( atanh( 1.0 ).is_infinite() );
/// assert!( atanh( 1.5 ).is_nan() );
///
/// let small = 1.0e-9;
/// assert!( ( atanh( small ) / small - 1.0 ).abs() < 1e-15 );
/// ```
#[ must_use ]
pub fn atanh( x : f64 ) -> f64
{
  if x.is_nan()
  {
    return x;
  }

  let a = x.abs();
  // Both cases outside the series share one boundary, and `1.0` is exactly
  // representable, so `>=` separates them without an equality comparison —
  // which is also what keeps `clippy::float_cmp` quiet without a suppression.
  // The boundary itself is exact rather than approximate: `atanh` is `±∞` at
  // exactly `±1` and undefined beyond it.
  if a >= 1.0
  {
    if a > 1.0
    {
      return f64::NAN;
    }
    return if x > 0.0 { f64::INFINITY } else { f64::NEG_INFINITY };
  }

  let r = 0.5 * ln_1p( 2.0 * a / ( 1.0 - a ) );
  if x < 0.0 { -r } else { r }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn the_saturation_branch_is_an_identity_not_an_approximation()
  {
    // The claim `HYPERBOLIC_SATURATION` rests on: at the threshold the second
    // exponential is already below half an ulp of the first, so dropping it
    // changes no bit. Checked by evaluating both forms just under the branch.
    let a = HYPERBOLIC_SATURATION;
    let e = exp( a );
    assert_eq!( ( 0.5 * ( e - 1.0 / e ) ).to_bits(), ( 0.5 * e ).to_bits(), "sinh" );
    assert_eq!( ( 0.5 * ( e + 1.0 / e ) ).to_bits(), ( 0.5 * e ).to_bits(), "cosh" );
    let u = exp_m1( 2.0 * a );
    assert_eq!( ( u / ( u + 2.0 ) ).to_bits(), 1.0_f64.to_bits(), "tanh" );
  }

  #[ test ]
  fn the_branches_meet_without_a_step()
  {
    // `sinh` and `asinh` each switch strategy at `| x | == 1`. A discontinuity
    // there is invisible to identity tests, because both sides are
    // self-consistent on their own.
    //
    // Both formulas are evaluated at the *same* argument. Sampling either side
    // of the threshold instead measures the function's own slope across the
    // sampling gap — `cosh( 1 ) ≈ 1.54` times `2e-12` is `3e-12`, which reads
    // as a jump of that size while saying nothing about continuity.
    for a in [ 1.0, 0.5, 0.75 ]
    {
      let u = exp_m1( a );
      let series_form = 0.5 * u * ( u + 2.0 ) / ( 1.0 + u );
      let e = exp( a );
      let direct_form = 0.5 * ( e - 1.0 / e );
      assert!
      (
        ( series_form / direct_form - 1.0 ).abs() < 1.0e-14,
        "sinh forms disagree at {a}: {series_form} vs {direct_form}"
      );

      let ln1p_form = ln_1p( a + a * a / ( 1.0 + sqrt( 1.0 + a * a ) ) );
      let plain_form = ln( a + sqrt( a * a + 1.0 ) );
      assert!
      (
        ( ln1p_form / plain_form - 1.0 ).abs() < 1.0e-14,
        "asinh forms disagree at {a}: {ln1p_form} vs {plain_form}"
      );
    }
  }
}

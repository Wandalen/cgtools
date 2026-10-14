//! The pieces the surface is built from, checked directly rather than through
//! a composed result.
//!
//! Everything here reaches `the_module::internal`, which exists only under the
//! `test_internals` feature. That indirection is the price of tests living in
//! `tests/` — a separate crate, which cannot see a private item the way an
//! inline `mod tests` could.
//!
//! Why it is worth paying: a wrong coefficient table and a wrong reduction can
//! cancel. `atan` built on a shifted `ATAN_V` and a compensating reduction
//! would satisfy every identity in `inverse_circular_test.rs` and still be
//! wrong on its own terms. These tests grade each piece against its own
//! definition — a series rebuilt term by term, a table re-derived by bisection,
//! a split reassembled — so a defect fails here rather than shifting an answer
//! somewhere downstream.

use super::*;
use the_module::*;
use the_module::internal::*;

#[ test ]
fn round_half_away_breaks_ties_away_from_zero()
{
  assert_eq!( round_half_away( 0.5 ), 1 );
  assert_eq!( round_half_away( -0.5 ), -1 );
  assert_eq!( round_half_away( 1.5 ), 2 );
  assert_eq!( round_half_away( -1.5 ), -2 );
  assert_eq!( round_half_away( 0.4999 ), 0 );
  assert_eq!( round_half_away( -0.4999 ), 0 );
}

#[ test ]
fn scale2_is_exact_multiplication_by_a_power_of_two()
{
  // The window where a single multiply works. `3.25 * 2^k` stays normal
  // throughout, so agreement here must be bit-exact.
  for k in -1020 ..= 1021
  {
    let want = 3.25_f64 * 2.0_f64.powi( k );
    assert_eq!( scale2( 3.25, k ).to_bits(), want.to_bits(), "scale2 at k = {k}" );
  }
}

#[ test ]
fn scale2_stays_correct_past_the_exponent_field()
{
  // The range the wraparound defect lived in, plus the overflow end where the
  // same arithmetic would have produced a spurious infinity.
  //
  // Graded against `f64::exp2`, which is correctly rounded at integer
  // arguments and reaches these values by an entirely different route — not
  // against `f64::powi`, which computes a negative exponent as `1 / 2^1074`
  // and returns zero here because the reciprocal's denominator overflows
  // first.
  //
  // Bit-exact rather than approximate: a sign flip is not something a
  // tolerance should be able to absorb.
  for k in ( -1074 ..= -1021 ).chain( 1022 ..= 1024 )
  {
    let want = f64::exp2( f64::from( k ) );
    assert_eq!( scale2( 1.0, k ).to_bits(), want.to_bits(), "scale2 at k = {k}" );
    assert!( scale2( 1.0, k ) >= 0.0, "sign lost at k = {k}" );
  }

  // A mantissa that occupies more than one bit, over the sub-range where the
  // product is still representable — `1.5 * 2^k` needs `2^( k - 1 )` to exist,
  // which it does down to `k = -1073`, and must stay under `f64::MAX` at the
  // top.
  for k in ( -1073 ..= -1021 ).chain( 1022 ..= 1023 )
  {
    let want = 1.5 * f64::exp2( f64::from( k ) );
    assert_eq!( scale2( 1.5, k ).to_bits(), want.to_bits(), "scale2 of 1.5 at k = {k}" );
  }
}

#[ test ]
fn the_two_series_agree_with_their_own_definitions()
{
  // Rebuilt term by term at run time, so a mistyped loop bound in either
  // series fails here rather than shifting an answer.
  let mut u = -0.18;
  while u < 0.18
  {
    let mut alternating_sum = 0.0;
    let mut positive_sum = 0.0;
    for j in 0 ..= 40u32
    {
      let p = 2 * j + 1;
      let term = powi( u, p as i32 ) / f64::from( p );
      alternating_sum += if j % 2 == 0 { term } else { -term };
      positive_sum += term;
    }
    // Absolute rather than relative because both series pass through zero.
    // The bound is set by the reference sum's own rounding — 41 naive
    // additions at magnitude ~0.18 accumulate a couple of parts in 1e15 —
    // and is still tight enough that any coefficient a typo could plausibly
    // reach would miss it by orders of magnitude.
    assert!( ( atan_series( u ) - alternating_sum ).abs() < 1.0e-14, "atan_series at {u}" );
    assert!( ( atanh_series( u ) - positive_sum ).abs() < 1.0e-14, "atanh_series at {u}" );
    u += 0.007;
  }
}

#[ test ]
fn the_reduced_series_match_their_own_definitions()
{
  // Rebuilt from run-time factorials, so a mistyped entry in `RECIP_FACT` or
  // a wrong loop bound fails here rather than shifting an angle.
  let mut r = -FRAC_PI_4;
  while r < FRAC_PI_4
  {
    let ( mut sin_want, mut cos_want ) = ( 0.0, 0.0 );
    let ( mut term, mut fact ) = ( 1.0, 1.0 );
    for n in 0 ..= 30u32
    {
      if n > 0
      {
        term *= r;
        fact *= f64::from( n );
      }
      let signed = if ( n / 2 ) % 2 == 0 { term / fact } else { -term / fact };
      if n % 2 == 0 { cos_want += signed; } else { sin_want += signed; }
    }
    // Absolute, because `sin` passes through zero on this range. The bound is
    // set by the reference sum's own rounding across 31 naive additions, not
    // by the series being checked.
    assert!( ( sin_reduced( r ) - sin_want ).abs() < 1.0e-14, "sin_reduced at {r}" );
    assert!( ( cos_reduced( r ) - cos_want ).abs() < 1.0e-14, "cos_reduced at {r}" );
    r += 0.021;
  }
}

#[ test ]
fn the_three_part_split_reassembles_to_pi_over_two()
{
  // The reduction subtracts these three in sequence; if they do not sum to
  // `PI/2` the residual is wrong by the difference on every single call.
  let sum = PIO2_HI + PIO2_MD + PIO2_LO;
  assert_eq!( sum.to_bits(), core::f64::consts::FRAC_PI_2.to_bits() );

  // And the high part must be exactly representable in the top bits, or
  // `n * PIO2_HI` rounds and the split buys nothing.
  assert_eq!( PIO2_HI.to_bits() & 0x0000_0000_03ff_ffff, 0 );
}

#[ test ]
fn mantissa_exponent_is_an_exact_split()
{
  let mut x = 1.0e-30;
  while x < 1.0e30
  {
    let ( m, k ) = mantissa_exponent( x );
    assert!( ( core::f64::consts::FRAC_1_SQRT_2 .. core::f64::consts::SQRT_2 ).contains( &m ), "range at {x}" );
    // Reassembling must return the identical bits — the split rounds nothing.
    assert_eq!( scale2( m, k ).to_bits(), x.to_bits(), "reassembly at {x}" );
    x *= 1.7;
  }
}

#[ test ]
fn exp_reduced_matches_its_own_series_definition()
{
  let mut r = -0.35;
  while r < 0.35
  {
    let mut want = 0.0;
    let mut term = 1.0;
    for n in 0 ..= 40u32
    {
      if n > 0
      {
        term = term * r / f64::from( n );
      }
      want += term;
    }
    assert!( ( exp_reduced( r ) / want - 1.0 ).abs() < 1.0e-15, "exp_reduced at {r}" );
    r += 0.011;
  }
}

#[ test ]
fn ln_mantissa_inverts_exp_reduced_across_the_recentred_range()
{
  let mut m = core::f64::consts::FRAC_1_SQRT_2;
  while m < core::f64::consts::SQRT_2
  {
    let l = ln_mantissa( m );
    assert!( ( exp_reduced( l ) / m - 1.0 ).abs() < 1.0e-15, "round trip at {m}" );
    m += 0.013;
  }
}

#[ test ]
fn the_series_band_is_where_both_paths_are_accurate()
{
  // The claim `SERIES_BAND` rests on: at the band edge the naive identity has
  // recovered to roughly 1e-15 relative, so the handover is not a cliff.
  let x = SERIES_BAND;
  let naive = exp( x ) - 1.0;
  assert!( ( naive / x.exp_m1() - 1.0 ).abs() < 1.0e-15, "exp_m1 handover" );
  let naive = ln( 1.0 + x );
  assert!( ( naive / x.ln_1p() - 1.0 ).abs() < 1.0e-15, "ln_1p handover" );
}

#[ test ]
fn the_named_log_constants_are_consistent()
{
  assert!( ( LN_2 * LOG2_E - 1.0 ).abs() < 1.0e-15 );
  assert!( ( LN_10 * LOG10_E - 1.0 ).abs() < 1.0e-15 );
  assert!( ( LOG10_2 * LN_10 - LN_2 ).abs() < 1.0e-15 );
}

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
      if tan( mid ) < b { lo = mid; } else { hi = mid; }
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

//! `exp`, `exp2`, `exp_m1`, `ln`, `ln_1p`, `log`, `log2`, `log10`, `powf`,
//! `cbrt` — everything that reduces to an exponent field plus a short series.

use super::*;
use the_module::
{
  cbrt, exp, exp2, exp_m1, ln, ln_1p, log, log10, log2, powf, LN_10, LN_2,
};

#[ test ]
fn exp_and_ln_invert_each_other()
{
  for x in sweep( -300.0, 300.0, 400 )
  {
    assert!( rel_err( ln( exp( x ) ), x ) < TOL, "ln of exp at {x}" );
  }
  for y in sweep( 1.0e-8, 1.0e8, 400 )
  {
    assert!( rel_err( exp( ln( y ) ), y ) < TOL, "exp of ln at {y}" );
  }
}

#[ test ]
fn exp_turns_addition_into_multiplication()
{
  for a in sweep( -20.0, 20.0, 120 )
  {
    let b = a * 0.5 - 3.0;
    assert!( rel_err( exp( a + b ), exp( a ) * exp( b ) ) < TOL, "exp addition at {a}" );
  }
}

#[ test ]
fn exp_and_ln_match_libm()
{
  for x in sweep( -300.0, 300.0, 400 )
  {
    assert!( rel_err( exp( x ), x.exp() ) < TOL, "exp vs libm at {x}" );
  }
  for y in sweep( 1.0e-8, 1.0e8, 400 )
  {
    assert!( rel_err( ln( y ), y.ln() ) < TOL, "ln vs libm at {y}" );
  }
}

#[ test ]
fn exp_stays_correct_where_the_result_is_subnormal()
{
  // The argument reduction hands a power of two to a scaling helper, and below
  // `2^-1022` there is no single `f64` equal to that power to scale by. Doing it
  // anyway is not a rounding error but a wraparound — the exponent field goes
  // negative, its low bits land in the sign field, and the multiplier becomes a
  // large negative number. `exp( -745.0 )` returned `-9.12e292` where the answer
  // is `5e-324`.
  //
  // Nothing caught it because the only test that reached this path swept
  // `[ -300, 300 ]`, where the power of two never leaves the representable
  // window. This sweep exists to keep that window covered.
  // Graded in ULP for the same reason [`exp2_matches_libm_across_its_whole_domain`]
  // is: the subnormal grid is far coarser than any relative bound worth setting.
  for x in sweep( -745.0, -700.0, 400 )
  {
    let ( got, want ) = ( exp( x ), x.exp() );
    assert!( got >= 0.0, "sign lost at {x}: {got:e}" );
    assert!( ulp_diff( got, want ) <= 2, "subnormal exp at {x}: {got:e} vs {want:e}" );
  }

  // Both ends of the domain, where the answer stops existing.
  assert_eq!( exp( -746.0 ), 0.0 );
  assert!( exp( 710.0 ).is_infinite() );
  assert!( exp( 709.0 ).is_finite() && exp( 709.0 ) > 0.0 );
}

#[ test ]
fn exp2_is_exact_on_integers()
{
  // The reason `exp2` is reduced in base two rather than routed through
  // `exp( x * LN_2 )`: that product rounds before it is exponentiated, so no
  // large integer power comes back exact. Here the integer part moves into the
  // exponent field untouched.
  for k in -1074 ..= 1023
  {
    let want = f64::exp2( f64::from( k ) );
    assert_eq!( exp2( f64::from( k ) ).to_bits(), want.to_bits(), "exp2 at integer {k}" );
  }
  assert!( exp2( 1024.0 ).is_infinite() );
  assert_eq!( exp2( -1075.0 ), 0.0 );
}

#[ test ]
fn exp2_matches_libm_across_its_whole_domain()
{
  // Graded in ULP rather than relatively, because the domain runs into the
  // subnormal range where consecutive representable values are up to 6% apart —
  // a relative bound there is one no implementation can meet, and loosening it to
  // 6% would stop catching anything.
  for x in sweep( -1074.0, 1023.0, 900 )
  {
    let ( got, want ) = ( exp2( x ), x.exp2() );
    assert!( got >= 0.0, "sign lost at {x}" );
    assert!( ulp_diff( got, want ) <= 2, "exp2 vs libm at {x}: {got:e} vs {want:e}" );
  }
}

#[ test ]
fn exp2_and_log2_invert_each_other()
{
  for x in sweep( -900.0, 900.0, 400 )
  {
    assert!( rel_err( log2( exp2( x ) ), x ) < TOL, "log2 of exp2 at {x}" );
  }
}

#[ test ]
fn the_logarithms_agree_with_each_other_and_with_libm()
{
  for y in sweep( 1.0e-8, 1.0e8, 500 )
  {
    assert!( rel_err( log2( y ), y.log2() ) < TOL, "log2 vs libm at {y}" );
    assert!( rel_err( log10( y ), y.log10() ) < TOL, "log10 vs libm at {y}" );

    // Each base is the natural logarithm rescaled, which is the relationship the
    // separate routines exist to compute more accurately than the division would.
    assert!( rel_err( log2( y ), ln( y ) / LN_2 ) < TOL, "log2 definition at {y}" );
    assert!( rel_err( log10( y ), ln( y ) / LN_10 ) < TOL, "log10 definition at {y}" );

    // And the arbitrary-base form reproduces both.
    assert!( rel_err( log( y, 2.0 ), log2( y ) ) < TOL, "log base 2 at {y}" );
    assert!( rel_err( log( y, 10.0 ), log10( y ) ) < TOL, "log base 10 at {y}" );
  }
}

#[ test ]
fn log2_and_log10_are_exact_on_their_own_powers()
{
  // The point of a dedicated `log2`: the exponent comes straight out of the bit
  // pattern, so every exact power of two returns an exact integer rather than
  // something a hair either side of one.
  for k in -300 ..= 300
  {
    assert_eq!( log2( f64::exp2( f64::from( k ) ) ), f64::from( k ), "log2 at 2^{k}" );
  }

  // `log10` cannot be exact the same way — no power of ten but `1` is a power of
  // two — so it is checked against being within half a last place of the integer.
  for k in -300 ..= 300
  {
    let p = f64::powf( 10.0, f64::from( k ) );
    assert!( ( log10( p ) - f64::from( k ) ).abs() < 1.0e-13, "log10 at 10^{k}" );
  }
}

#[ test ]
fn the_logarithms_accept_subnormal_arguments()
{
  // The mirror image of the subnormal *result* defect in `exp`. A subnormal has
  // no implicit leading mantissa bit, so reading its exponent field the ordinary
  // way returns a value up to 51 too high — not a precision loss but a wrong
  // answer, and one that grows worse the smaller the argument gets.
  //
  // Checked against libm on the exact powers of two, where the expected value is
  // an integer that can be stated outright, and then across arbitrary subnormals
  // where only libm can supply the reference.
  for k in -1074 ..= -1023
  {
    let x = f64::exp2( f64::from( k ) );
    assert!( x > 0.0 && x < f64::MIN_POSITIVE, "guard assumption: {x:e} is subnormal" );
    assert_eq!( log2( x ), f64::from( k ), "log2 of the subnormal 2^{k}" );
    assert!( rel_err( ln( x ), x.ln() ) < TOL, "ln of the subnormal 2^{k}" );
    assert!( rel_err( log10( x ), x.log10() ) < TOL, "log10 of the subnormal 2^{k}" );
  }

  for x in decades( 3.7, 323 ).filter( | x | *x < f64::MIN_POSITIVE )
  {
    assert!( rel_err( ln( x ), x.ln() ) < TOL, "ln at the subnormal {x:e}" );
    assert!( rel_err( log2( x ), x.log2() ) < TOL, "log2 at the subnormal {x:e}" );
    assert!( rel_err( log10( x ), x.log10() ) < TOL, "log10 at the subnormal {x:e}" );
  }

  // And the round trip closes across the boundary, which the defect broke by 51
  // powers of two.
  for k in -1074 ..= -1000
  {
    assert!( rel_err( exp2( log2( f64::exp2( f64::from( k ) ) ) ), f64::exp2( f64::from( k ) ) ) < TOL, "round trip at 2^{k}" );
  }
}

#[ test ]
fn the_logarithms_reject_what_has_no_logarithm()
{
  for f in [ ln as fn( f64 ) -> f64, log2, log10 ]
  {
    assert!( f( -1.0 ).is_nan(), "negative" );
    assert!( f( f64::NAN ).is_nan(), "nan" );
    assert!( f( 0.0 ).is_infinite() && f( 0.0 ) < 0.0, "zero is negative infinity" );
    assert!( f( f64::INFINITY ).is_infinite() && f( f64::INFINITY ) > 0.0, "infinity" );
    assert_eq!( f( 1.0 ), 0.0, "one is exactly zero" );
  }
}

#[ test ]
fn powf_matches_libm()
{
  for x in sweep( 1.0e-3, 100.0, 80 )
  {
    for y in sweep( -8.0, 8.0, 80 )
    {
      assert!( rel_err( powf( x, y ), x.powf( y ) ) < 1.0e-11, "powf at {x}^{y}" );
    }
  }
}

#[ test ]
fn powf_agrees_with_repeated_multiplication()
{
  for x in sweep( 0.1, 20.0, 300 )
  {
    assert!( rel_err( powf( x, 2.0 ), x * x ) < TOL, "square at {x}" );
    assert!( rel_err( powf( x, 3.0 ), x * x * x ) < TOL, "cube at {x}" );
    assert!( rel_err( powf( x, 0.5 ), the_module::sqrt( x ) ) < TOL, "root at {x}" );
  }
}

#[ test ]
fn cbrt_accepts_negative_inputs()
{
  // This is the whole reason `cbrt` is a function rather than a call to `powf`:
  // `powf` of a negative base with a fractional exponent is NaN, so the
  // obvious-looking substitution is silently wrong over half the domain.
  assert!( powf( -8.0, 1.0 / 3.0 ).is_nan(), "guard assumption: powf is NaN here" );

  for x in sweep( -1.0e4, -1.0e-4, 400 )
  {
    let r = cbrt( x );
    assert!( r.is_finite() && r < 0.0, "cbrt of negative at {x}" );
    assert!( rel_err( r * r * r, x ) < 1.0e-11, "cbrt cubes back at {x}" );
  }
}

#[ test ]
fn cbrt_is_odd_and_cubes_back()
{
  for x in sweep( 1.0e-4, 1.0e4, 400 )
  {
    assert!( rel_err( cbrt( x ) * cbrt( x ) * cbrt( x ), x ) < 1.0e-11, "cbrt cubes back at {x}" );
    assert!( rel_err( cbrt( -x ), -cbrt( x ) ) < TOL, "cbrt odd symmetry at {x}" );
    assert!( rel_err( cbrt( x ), x.cbrt() ) < 1.0e-11, "cbrt vs libm at {x}" );
  }
  assert_eq!( cbrt( 0.0 ).to_bits(), 0.0_f64.to_bits() );
}

#[ test ]
fn exp_m1_is_accurate_where_the_naive_form_cancels()
{
  // Far from zero the naive form is fine and the two agree.
  for x in sweep( -20.0, 20.0, 300 )
  {
    assert!( rel_err( exp_m1( x ), x.exp_m1() ) < 1.0e-11, "exp_m1 vs libm at {x}" );
  }

  // Near zero, `exp( x ) - 1.0` loses most of its significant digits to
  // cancellation. The series must not.
  for x in decades( 3.7, 300 )
  {
    assert!( rel_err( exp_m1( x ), x.exp_m1() ) < 1.0e-14, "exp_m1 near zero at {x}" );
    assert!( rel_err( exp_m1( -x ), ( -x ).exp_m1() ) < 1.0e-14, "exp_m1 near zero at -{x}" );
  }

  // And the failure being avoided is real, not hypothetical: at this magnitude
  // the naive form has already lost the value entirely.
  let tiny = 1.0e-17;
  assert!( rel_err( exp( tiny ) - 1.0, tiny.exp_m1() ) > 0.5, "naive form should be badly wrong" );
  assert!( rel_err( exp_m1( tiny ), tiny.exp_m1() ) < 1.0e-14, "the series form should be right" );
}

#[ test ]
fn ln_1p_is_accurate_where_the_naive_form_cancels()
{
  for x in sweep( -0.9, 20.0, 300 )
  {
    assert!( rel_err( ln_1p( x ), x.ln_1p() ) < 1.0e-11, "ln_1p vs libm at {x}" );
  }

  for x in decades( 3.7, 300 )
  {
    assert!( rel_err( ln_1p( x ), x.ln_1p() ) < 1.0e-14, "ln_1p near zero at {x}" );
    assert!( rel_err( ln_1p( -x ), ( -x ).ln_1p() ) < 1.0e-14, "ln_1p near zero at -{x}" );
  }

  // `1.0 + 1e-17` rounds to exactly `1.0`, so the naive form returns zero and
  // every significant digit is gone.
  let tiny = 1.0e-17;
  assert!( ln( 1.0 + tiny ).abs() < f64::MIN_POSITIVE, "naive form should collapse to zero" );
  assert!( rel_err( ln_1p( tiny ), tiny.ln_1p() ) < 1.0e-14, "the series form should be right" );
}

#[ test ]
fn the_two_series_bands_join_without_a_step()
{
  // `exp_m1` and `ln_1p` each switch from a series to a closed form at a fixed
  // threshold. A discontinuity there would be invisible to every test above,
  // because both sides are individually self-consistent and libm agrees with
  // each of them separately to well inside the tolerance.
  //
  // Sampled either side of the band edge and graded against libm, which is
  // continuous across it by construction.
  for scale in [ 0.98, 0.995, 1.0, 1.005, 1.02 ]
  {
    for edge in [ 0.25, -0.25 ]
    {
      let x = edge * scale;
      assert!( rel_err( exp_m1( x ), x.exp_m1() ) < 1.0e-14, "exp_m1 across the band at {x}" );
      assert!( rel_err( ln_1p( x ), x.ln_1p() ) < 1.0e-14, "ln_1p across the band at {x}" );
    }
  }
}

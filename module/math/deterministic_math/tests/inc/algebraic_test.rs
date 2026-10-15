//! `sqrt`, `mul_add`, `hypot`, `powi` — the functions IEEE-754 already pins, and
//! the two built only from operations it pins.
//!
//! These are the entries where agreement with libm can be demanded *exactly*
//! rather than within a tolerance, because for `sqrt` and `mul_add` the standard
//! specifies a single correct answer and both implementations are required to
//! produce it.

use super::*;
use the_module::{ hypot, mul_add, powi, sqrt };

#[ test ]
fn sqrt_is_bit_identical_to_the_hardware()
{
  // IEEE-754 requires `sqrt` to be correctly rounded, so there is exactly one
  // right answer and this crate simply calls it.
  for m in sweep( 0.0, 1.0e6, 500 )
  {
    assert_eq!( sqrt( m ).to_bits(), m.sqrt().to_bits(), "sqrt at {m}" );
  }
}

#[ test ]
fn sqrt_round_trips()
{
  for m in sweep( 1.0e-6, 1.0e6, 300 )
  {
    let r = sqrt( m );
    assert!( rel_err( r * r, m ) < TOL, "sqrt round trip at {m}" );
  }
}

#[ test ]
fn mul_add_is_the_fused_form_and_not_the_unfused_one()
{
  // Delegation is exact by construction, so this half is a guard against the
  // function quietly becoming `a * b + c` at some later date.
  for a in sweep( -100.0, 100.0, 60 )
  {
    for b in sweep( -100.0, 100.0, 60 )
    {
      let c = a * 0.25 - 7.0;
      assert_eq!( mul_add( a, b, c ).to_bits(), a.mul_add( b, c ).to_bits(), "mul_add at {a},{b},{c}" );
    }
  }

  // And the distinction being preserved is a real one: somewhere in that sweep
  // the fused and unfused forms must actually disagree, or the assertion above
  // would hold for the wrong reason. One rounding versus two is the entire
  // reason this function is spelled out rather than left to the caller.
  let differs = sweep( 0.1, 3.0, 200 ).any( | a | mul_add( a, a, -a * a ) != a * a - a * a );
  assert!( differs, "fused and unfused never diverged — is an FMA path actually being taken?" );
}

#[ test ]
fn hypot_survives_arguments_the_naive_form_cannot()
{
  // The whole reason this exists rather than being left to the caller: the
  // obvious `sqrt( x * x + y * y )` overflows to infinity long before the answer
  // does, and underflows to zero long after it should.
  let big : f64 = 1.0e200;
  assert!( ( big * big ).is_infinite(), "guard assumption: the naive form overflows here" );
  assert!( rel_err( hypot( big, big ), big * core::f64::consts::SQRT_2 ) < TOL );

  let small : f64 = 1.0e-200;
  assert_eq!( small * small, 0.0, "guard assumption: the naive form underflows here" );
  assert!( rel_err( hypot( small, small ), small * core::f64::consts::SQRT_2 ) < TOL );
}

#[ test ]
fn hypot_matches_libm_and_its_own_definition()
{
  for x in sweep( -1.0e4, 1.0e4, 90 )
  {
    for y in sweep( -1.0e4, 1.0e4, 90 )
    {
      assert!( rel_err( hypot( x, y ), x.hypot( y ) ) < TOL, "hypot vs libm at {x},{y}" );
      // Where the naive form is in no danger, it is also the definition.
      assert!( rel_err( hypot( x, y ), sqrt( x * x + y * y ) ) < TOL, "hypot definition at {x},{y}" );
    }
  }

  // Symmetric in both arguments and in sign, exactly.
  for x in sweep( 0.5, 100.0, 50 )
  {
    let y = x * 0.375 + 1.0;
    assert_eq!( hypot( x, y ).to_bits(), hypot( y, x ).to_bits(), "argument order at {x}" );
    assert_eq!( hypot( -x, y ).to_bits(), hypot( x, y ).to_bits(), "sign at {x}" );
  }
}

#[ test ]
fn hypot_handles_zero_infinity_and_nan()
{
  assert_eq!( hypot( 0.0, 0.0 ), 0.0 );
  assert_eq!( hypot( 0.0, 3.0 ), 3.0 );
  assert_eq!( hypot( -3.0, 0.0 ), 3.0 );
  assert!( hypot( f64::INFINITY, 1.0 ).is_infinite() );
  assert!( hypot( 1.0, f64::NEG_INFINITY ).is_infinite() );
  assert!( hypot( f64::NAN, 1.0 ).is_nan() );

  // An infinity beats a NaN in the other argument: the magnitude is known to be
  // infinite whatever that component turns out to be. This follows libm, and is
  // asserted so the two cannot silently diverge.
  assert!( hypot( f64::INFINITY, f64::NAN ).is_infinite() );
  assert_eq!( hypot( f64::INFINITY, f64::NAN ).is_infinite(), f64::INFINITY.hypot( f64::NAN ).is_infinite() );
}

#[ test ]
fn powi_is_repeated_multiplication()
{
  for x in sweep( -20.0, 20.0, 400 )
  {
    assert_eq!( powi( x, 0 ).to_bits(), 1.0_f64.to_bits(), "zeroth power at {x}" );
    assert_eq!( powi( x, 1 ).to_bits(), x.to_bits(), "first power at {x}" );
    assert_eq!( powi( x, 2 ).to_bits(), ( x * x ).to_bits(), "square at {x}" );
    // `x * x * x` and the squaring chain's `x * ( x * x )` differ only by
    // associativity, which multiplication's single rounding makes irrelevant at
    // this exponent.
    assert_eq!( powi( x, 3 ).to_bits(), ( x * x * x ).to_bits(), "cube at {x}" );

    if x != 0.0
    {
      assert!( rel_err( powi( x, -3 ), 1.0 / ( x * x * x ) ) < TOL, "negative power at {x}" );
    }
  }
}

#[ test ]
fn powi_accepts_the_negative_bases_powf_refuses()
{
  // This is the reason `powi` is a separate function rather than a call into
  // `powf`: `powf` routes through `ln` and is NaN for every negative base, so the
  // obvious substitution is silently wrong over half the domain.
  assert!( the_module::powf( -2.0, 3.0 ).is_nan(), "guard assumption: powf is NaN here" );

  for x in sweep( -50.0, -0.1, 200 )
  {
    for n in [ 3, 5, 7 ]
    {
      assert!( powi( x, n ) < 0.0, "odd power of a negative at {x}^{n}" );
      assert!( powi( x, n + 1 ) > 0.0, "even power of a negative at {x}^{}", n + 1 );
      assert!( rel_err( powi( x, n ), x.powi( n ) ) < TOL, "powi vs libm at {x}^{n}" );
    }
  }
}

#[ test ]
fn powi_handles_the_exponent_that_cannot_be_negated()
{
  // `i32::MIN` has no positive counterpart, so an implementation that reaches
  // the magnitude by writing `-n` overflows here — in debug it panics, in
  // release it wraps back to `i32::MIN` and computes an enormous positive power
  // instead of a vanishing negative one.
  assert_eq!( powi( 1.0, i32::MIN ), 1.0 );
  assert_eq!( powi( 2.0, i32::MIN ), 0.0, "an unreachable-small power underflows to zero" );
  assert!( powi( 0.5, i32::MIN ).is_infinite() );
}

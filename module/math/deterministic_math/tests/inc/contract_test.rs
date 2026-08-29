//! The claims the crate makes about itself, as opposed to about any one
//! function: that it reaches no libm, that it returns the same bits for the same
//! input, that its constants are the standard ones, and that a `NaN` in produces
//! a `NaN` out rather than a plausible number.

use super::*;
use the_module::*;

/// Every method spelling that routes to the platform's libm.
///
/// `sqrt` and `mul_add` are on this list even though IEEE-754 pins both. They
/// are reproducible, so their presence is not a correctness defect — but the
/// crate's entire premise is that a reader can find every arithmetic entry point
/// by looking in one place, and an unannounced call bypasses that whatever its
/// rounding behaviour. The two legitimate uses are declared below instead.
const LIBM_SPELLINGS : [ &str ; 28 ] =
[
  ".sin(", ".cos(", ".tan(", ".sin_cos(",
  ".asin(", ".acos(", ".atan(", ".atan2(",
  ".sinh(", ".cosh(", ".tanh(",
  ".asinh(", ".acosh(", ".atanh(",
  ".exp(", ".exp2(", ".exp_m1(",
  ".ln(", ".ln_1p(", ".log(", ".log2(", ".log10(",
  ".powf(", ".powi(", ".cbrt(", ".hypot(",
  ".sqrt(", ".mul_add(",
];

/// The calls that are allowed, and exactly how many of each.
///
/// A count rather than a permission: `sqrt` and `mul_add` each have one
/// legitimate site — the function that wraps them — and a second occurrence
/// anywhere would be an unwrapped call that happens to sit in a file where one
/// was expected.
const ALLOWED : [ ( &str, &str, usize ) ; 2 ] =
[
  ( "algebraic.rs", ".sqrt(", 1 ),
  ( "algebraic.rs", ".mul_add(", 1 ),
];

/// Each source file, paired with its name.
///
/// `include_str!` rather than a directory walk, so the check runs against the
/// sources this test binary was actually compiled from and cannot be defeated by
/// a stale or relocated tree. The cost is that a new module must be added here —
/// which [`every_source_file_is_covered_by_the_libm_scan`] enforces.
const SOURCES : [ ( &str, &str ) ; 7 ] =
[
  ( "lib.rs",              include_str!( "../../src/lib.rs" ) ),
  ( "constant.rs",         include_str!( "../../src/constant.rs" ) ),
  ( "algebraic.rs",        include_str!( "../../src/algebraic.rs" ) ),
  ( "circular.rs",         include_str!( "../../src/circular.rs" ) ),
  ( "exponential.rs",      include_str!( "../../src/exponential.rs" ) ),
  ( "hyperbolic.rs",       include_str!( "../../src/hyperbolic.rs" ) ),
  ( "inverse_circular.rs", include_str!( "../../src/inverse_circular.rs" ) ),
];

/// Strip a source down to the code that ships.
///
/// Two things are removed. Comments and doc comments, because they routinely
/// *quote* the very calls the scan bans — the documentation's job is partly to
/// name what not to write. And the trailing `#[ cfg( test ) ]` module, because a
/// test comparing against libm is the point rather than a violation, exactly as
/// [`super`]'s own header describes.
fn shipping_code( source : &str ) -> String
{
  source
  .split( "#[ cfg( test ) ]" )
  .next()
  .unwrap_or( "" )
  .lines()
  .filter( | l | !l.trim_start().starts_with( "//" ) )
  .collect::< Vec< _ > >()
  .join( "\n" )
}

#[ test ]
fn no_libm_call_survives_in_shipping_code()
{
  // The crate's central claim, checked mechanically rather than trusted. Every
  // function here exists because the platform's own version of it is free to
  // differ between machines, compilers and libc versions; one forgotten
  // `x.exp()` inside `src/` reintroduces exactly the divergence the crate was
  // built to remove, and nothing else in the test suite would notice — the
  // result would still be correct, still self-consistent, and still agree with
  // libm, because it *would* be libm.
  for ( name, source ) in SOURCES
  {
    let code = shipping_code( source );
    for spelling in LIBM_SPELLINGS
    {
      let found = code.matches( spelling ).count();
      let allowed = ALLOWED
      .iter()
      .find( | ( f, s, _ ) | *f == name && *s == spelling )
      .map_or( 0, | ( _, _, n ) | *n );

      assert_eq!
      (
        found, allowed,
        "{name} contains {found} occurrence(s) of `{spelling}` in shipping code, expected {allowed}"
      );
    }
  }
}

#[ test ]
fn every_source_file_is_covered_by_the_libm_scan()
{
  // A module added to `src/` and not to `SOURCES` would be silently exempt from
  // the scan above, which is the one failure mode that turns the guard into
  // decoration. `lib.rs` declares every module, so its own text is the register
  // to check against.
  let lib = SOURCES.iter().find( | ( n, _ ) | *n == "lib.rs" ).expect( "lib.rs is in SOURCES" ).1;

  for line in lib.lines().map( str::trim )
  {
    let Some( rest ) = line.strip_prefix( "mod " ).or_else( || line.strip_prefix( "pub mod " ) ) else { continue };
    let Some( module ) = rest.strip_suffix( ';' ) else { continue };

    let file = format!( "{module}.rs" );
    assert!
    (
      SOURCES.iter().any( | ( n, _ ) | *n == file ),
      "`{file}` is declared in lib.rs but missing from the libm scan's SOURCES table"
    );
  }
}

#[ test ]
fn the_scan_would_actually_catch_something()
{
  // A guard that cannot fail is not a guard. `shipping_code` does a fair amount
  // of stripping, and an over-eager version of it — one that removed too much, or
  // split on a marker that no longer appears — would report a clean scan over an
  // empty string.
  let source = "\
/// A doc comment mentioning x.sin( y ) that must be ignored.
fn real() -> f64 { let a = 1.0_f64; a.sin() }
#[ cfg( test ) ]
mod tests { fn t() { let b = 2.0_f64; b.cos(); } }";

  let code = shipping_code( source );
  assert_eq!( code.matches( ".sin(" ).count(), 1, "a genuine call must survive stripping" );
  assert_eq!( code.matches( ".cos(" ).count(), 0, "the test module must be stripped" );
  assert!( !code.contains( "doc comment" ), "doc comments must be stripped" );
}

#[ test ]
fn constants_are_the_core_ones()
{
  assert_eq!( PI.to_bits(), core::f64::consts::PI.to_bits() );
  assert_eq!( TAU.to_bits(), core::f64::consts::TAU.to_bits() );
  assert_eq!( FRAC_PI_2.to_bits(), core::f64::consts::FRAC_PI_2.to_bits() );
  assert_eq!( FRAC_PI_4.to_bits(), core::f64::consts::FRAC_PI_4.to_bits() );
  assert_eq!( LN_2.to_bits(), core::f64::consts::LN_2.to_bits() );
  assert_eq!( LN_10.to_bits(), core::f64::consts::LN_10.to_bits() );

  // Re-derived rather than restated: the reduction limit has to be a finite
  // positive magnitude, and asserting the literal would only echo the source.
  assert!( SIN_COS_MAX.is_finite() && SIN_COS_MAX > 0.0 );
}

#[ test ]
fn evaluation_is_bit_reproducible()
{
  // Reproducibility across machines cannot be tested from one machine. What can
  // be tested is the property that makes it possible: every function is a pure
  // composition of pinned operations, so it returns the identical bit pattern for
  // the identical input, with no accumulated or ambient state.
  //
  // Every exported function is listed rather than a representative few, because
  // "representative" is a judgement and a stateful one would be the exception.
  for x in sweep( -100.0, 100.0, 400 )
  {
    for f in unary_surface()
    {
      let ( a, b ) = ( f.1( x ), f.1( x ) );
      assert_eq!( a.to_bits(), b.to_bits(), "{} is not a pure function at {x}", f.0 );
    }
    assert_eq!( sin_cos( x ), sin_cos( x ), "sin_cos is not a pure function at {x}" );
    assert_eq!( atan2( x, 3.5 ).to_bits(), atan2( x, 3.5 ).to_bits(), "atan2 at {x}" );
    assert_eq!( hypot( x, 3.5 ).to_bits(), hypot( x, 3.5 ).to_bits(), "hypot at {x}" );
    assert_eq!( powf( x.abs(), 3.5 ).to_bits(), powf( x.abs(), 3.5 ).to_bits(), "powf at {x}" );
    assert_eq!( powi( x, 5 ).to_bits(), powi( x, 5 ).to_bits(), "powi at {x}" );
    assert_eq!( log( x.abs(), 3.5 ).to_bits(), log( x.abs(), 3.5 ).to_bits(), "log at {x}" );
    assert_eq!( mul_add( x, 2.5, 1.5 ).to_bits(), mul_add( x, 2.5, 1.5 ).to_bits(), "mul_add at {x}" );
  }
}

#[ test ]
fn nan_propagates_rather_than_being_swallowed()
{
  for ( name, f ) in unary_surface()
  {
    assert!( f( f64::NAN ).is_nan(), "{name} swallowed a NaN" );
  }
  assert!( sin_cos( f64::NAN ).0.is_nan() && sin_cos( f64::NAN ).1.is_nan() );
  assert!( atan2( f64::NAN, 1.0 ).is_nan() && atan2( 1.0, f64::NAN ).is_nan() );
  assert!( hypot( f64::NAN, 1.0 ).is_nan() );
  assert!( powf( f64::NAN, 1.0 ).is_nan() && powf( 1.0, f64::NAN ).is_nan() );
  assert!( log( f64::NAN, 2.0 ).is_nan() && log( 2.0, f64::NAN ).is_nan() );
  assert!( mul_add( f64::NAN, 1.0, 1.0 ).is_nan() );

  // Outside its domain, `ln` of a negative is NaN rather than a large negative.
  assert!( ln( -1.0 ).is_nan() );

  // `asin` and `acos` are the deliberate exception to the domain rule — they
  // clamp — but not to the NaN rule, which is asserted above with the rest.
  // See `inverse_circular_test` for why.
}

/// One exported one-argument function, paired with the name it is reported
/// under when an assertion over the whole surface fails.
type NamedUnary = ( &'static str, fn( f64 ) -> f64 );

/// Every exported one-argument function, paired with its name.
///
/// Listed once here rather than repeated in each cross-cutting test, so a
/// function added to the crate and forgotten in one of them is a single omission
/// rather than several.
fn unary_surface() -> [ NamedUnary ; 22 ]
{
  [
    ( "sqrt", sqrt ),
    ( "cbrt", cbrt ),
    ( "exp", exp ),
    ( "exp2", exp2 ),
    ( "exp_m1", exp_m1 ),
    ( "ln", ln ),
    ( "ln_1p", ln_1p ),
    ( "log2", log2 ),
    ( "log10", log10 ),
    ( "sin", sin ),
    ( "cos", cos ),
    ( "tan", tan ),
    ( "atan", atan ),
    ( "asin", asin ),
    ( "acos", acos ),
    ( "sinh", sinh ),
    ( "cosh", cosh ),
    ( "tanh", tanh ),
    ( "asinh", asinh ),
    ( "acosh", acosh ),
    ( "atanh", atanh ),
    ( "hypot_with_itself", | x | hypot( x, x ) ),
  ]
}

/// Every function name `lib.rs` re-exports, with the constants filtered out.
///
/// Parsed from the `pub use` lines rather than listed, so this register cannot
/// drift from what actually ships: adding a function to `lib.rs` immediately
/// widens what every test built on this helper has to account for, and none of
/// them can be satisfied by editing a list.
fn exported_function_names() -> Vec< &'static str >
{
  let lib = SOURCES.iter().find( | ( n, _ ) | *n == "lib.rs" ).expect( "lib.rs is in SOURCES" ).1;

  let mut names = Vec::new();
  for line in lib.lines().map( str::trim )
  {
    let Some( rest ) = line.strip_prefix( "pub use " ) else { continue };
    let Some( start ) = rest.find( '{' ) else { continue };
    let Some( end ) = rest.find( '}' ) else { continue };

    for name in rest[ start + 1 .. end ].split( ',' ).map( str::trim ).filter( | n | !n.is_empty() )
    {
      // Constants are `SCREAMING_CASE` and are checked by `constants_are_the_core_ones`.
      if name.chars().all( | c | c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit() )
      {
        continue;
      }
      names.push( name );
    }
  }
  names
}

#[ test ]
fn the_export_list_is_actually_being_parsed()
{
  // Every test below rests on `exported_function_names` returning the real
  // surface. If the parser silently returned nothing — a changed `pub use`
  // spelling, a reformatted brace — each of those tests would pass by iterating
  // an empty list, which is the one failure mode they cannot report themselves.
  let names = exported_function_names();
  assert!( names.len() >= 28, "parsed only {} exported names — the parser has stopped matching", names.len() );
  for expected in [ "sqrt", "sin_cos", "atan2", "ln_1p", "atanh" ]
  {
    assert!( names.contains( &expected ), "`{expected}` is exported but the parser missed it" );
  }
}

#[ test ]
fn the_unary_surface_covers_every_exported_one_argument_function()
{
  // The cross-cutting tests iterate `unary_surface`, so anything missing from it
  // is silently untested for purity and NaN handling. `lib.rs`'s re-export lists
  // are the register of what exists; this compares the two.

  // Names taken by more than one argument, which `unary_surface` cannot hold and
  // the cross-cutting tests therefore name individually.
  const MULTI_ARGUMENT : [ &str ; 6 ] = [ "atan2", "hypot", "log", "mul_add", "powf", "powi" ];
  // Not a function.
  const NOT_A_FUNCTION : [ &str ; 1 ] = [ "sin_cos" ];

  let listed : Vec< &str > = unary_surface().iter().map( | ( n, _ ) | *n ).collect();

  for name in exported_function_names()
  {
    if MULTI_ARGUMENT.contains( &name ) || NOT_A_FUNCTION.contains( &name )
    {
      continue;
    }
    assert!
    (
      listed.contains( &name ),
      "`{name}` is exported but missing from `unary_surface`, so it is untested for purity and NaN handling"
    );
  }
}

#[ test ]
fn the_benchmark_measures_every_exported_function()
{
  // `docs/non_functional_requirement/001` sets a cost ceiling per function and
  // `002` an accuracy ceiling, and one run of `examples/cost_vs_libm.rs` answers
  // both. A function exported but absent from that example's `all_rows()` is
  // held to neither — and nothing about the printed table would look wrong. It
  // would simply be one row shorter than the surface it claims to cover, which
  // is not something a reader can notice without this comparison.
  //
  // The search is for the *quoted* name, which is what disambiguates `log` from
  // `log2` and `log10`: `"log2"` does not contain the five characters `"log"`.
  const BENCHMARK : &str = include_str!( "../../examples/cost_vs_libm.rs" );

  for name in exported_function_names()
  {
    assert!
    (
      BENCHMARK.contains( &format!( "\"{name}\"" ) ),
      "`{name}` is exported but never named in `examples/cost_vs_libm.rs`, so it is measured against neither the cost nor the accuracy ceiling"
    );
  }
}

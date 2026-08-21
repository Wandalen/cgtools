
use super::*;

// All inputs are small integer-valued floats and `dot` only sums products of them, so the
// results are exactly representable with no rounding error — exact equality is correct here.
#[ test ]
#[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
fn test_dot_product()
{
  use the_module::vector;

  // Test with typical vectors
  let vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 4.0, 5.0, 6.0 ];
  let result = vector::dot( &vec_a, &vec_b );
  assert_eq!( result, 32.0, "Dot product calculation failed for typical vectors" );

  // Test with negative numbers
  let vec_c = [ -1.0, -2.0, -3.0 ];
  let vec_d = [ 4.0, 5.0, 6.0 ];
  let result_neg = vector::dot( &vec_c, &vec_d );
  assert_eq!( result_neg, -32.0, "Dot product calculation failed for negative numbers" );

  // Test with zero vectors
  let vec_zero = [ 0.0, 0.0, 0.0 ];
  let got = vector::dot( &vec_a, &vec_zero );
  assert_eq!( got, 0.0, "Dot product calculation failed for zero vector" );

  // Test with empty vectors (edge case)
  let vec_empty : [ f32; 0 ] = [];
  let result_empty = vector::dot( &vec_empty, &vec_empty );
  assert_eq!( result_empty, 0.0, "Dot product calculation failed for empty vectors" );

}

#[ test ]
fn test_magnitude2()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
  };

  let vec_a = [ 1.0, 2.0, 3.0 ];
  let result = vector::mag2( &vec_a );
  assert_ulps_eq!( result, 14.0 );

  let vec_zero = [ 0.0, 0.0, 0.0 ];
  let got = vector::mag2( &vec_zero );
  assert_ulps_eq!( got, 0.0 );

  let vec_empty : [ f32; 0 ] = [];
  let result_empty = vector::mag2( &vec_empty );
  assert_ulps_eq!( result_empty, 0.0 );
}

#[ test ]
fn test_magnitude()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
  };

  let vec_a = [ 3.0, 4.0 ];
  let result = vector::mag( &vec_a );
  assert_ulps_eq!( result, 5.0 );

  let vec_zero = [ 0.0, 0.0 ];
  let got = vector::mag( &vec_zero );
  assert_ulps_eq!( got, 0.0 );

  let vec_empty : [ f32; 0 ] = [];
  let result_empty = vector::mag( &vec_empty );
  assert_ulps_eq!( result_empty, 0.0 );
}

#[ test ]
fn test_normalize()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    Float,
    vector::{ IterFloat, IterExt },
  };

  // Test with a typical vector
  let vec_a = [ 3.0, 4.0 ];
  let mut result = vec_a;
  vector::normalize( &mut result, &vec_a );
  let expected = [ 0.6, 0.8 ];
  for ( a, b ) in result.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( a, b );
  }

  // Test with a zero vector
  let vec_zero = [ 0.0, 0.0 ];
  let mut got = vec_zero;
  vector::normalize( &mut got, &vec_zero );
  assert!( got.iter().map_is_nan().all_true(), "Expected NaN, got {got:?}" );

  for value in &got
  {
    assert!( value.is_nan(), "Expected NaN, got {value}" );
  }

}

// test_kind: bug_reproducer(BUG-124)
/// ## Root Cause
/// `vector::normalize(r, a)` (`vector/arithmetics.rs`) computes `mag` from `a` but its write
/// loop divided `*elem` (an element already sitting in `r`) by `mag`, never reading from `a`'s
/// own iterator at all — correct only by coincidence when the caller pre-seeds `r` to equal `a`
/// before calling. With `r != a`, the function silently normalizes whatever `r` already held
/// scaled by `a`'s magnitude, instead of normalizing `a`'s direction into `r`.
/// ## Why Not Caught
/// Every existing caller (this test included, previously) pre-set `r = a` before calling
/// `normalize`, making `*elem` and the corresponding element of `a` always numerically identical
/// — the missing read from `a` was unobservable under that calling convention. The sibling
/// `project_on(r, b)` in the same file correctly reads `b.vector_iter()` inside its own write
/// loop, serving as the in-file oracle for what `normalize` should have done.
/// ## Fix Applied
/// Added `let mut aiter = a.vector_iter();` before the loop and changed the write to
/// `*elem = *aiter.next().unwrap() / mag;`, so every written element is `a`'s own value (not
/// whatever `r` previously held) divided by `a`'s magnitude.
/// ## Prevention
/// This test uses `r != a` (`r` pre-set to an unrelated, already-unit-length vector) so a
/// write loop that reads from `r` instead of `a` produces a detectably wrong result.
/// ## Pitfall
/// A `fn(r: &mut R, a: &A)`-shaped API that computes a scalar from `a` but only reads/writes
/// through `r` in its loop body is only correct under a "caller pre-seeds r = a" convention
/// that the type signature itself never states or enforces — always check what a write loop
/// actually dereferences, not just what it assigns into.
#[ test ]
fn test_normalize_with_distinct_source_and_destination()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
  };

  // r starts as an unrelated, already-unit-length vector distinct from a.
  let mut r = [ 1.0, 0.0, 0.0 ];
  let a = [ 3.0, 4.0, 0.0 ];
  vector::normalize( &mut r, &a );
  let expected = [ 0.6, 0.8, 0.0 ];
  for ( got, exp ) in r.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( got, exp );
  }
}

#[ test ]
fn test_normalized()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    Float,
  };

  let vec_a = [ 3.0, 4.0 ];
  let result = vector::normalized( &vec_a );
  let expected = [ 0.6, 0.8 ];
  for ( a, b ) in result.iter().zip( expected.iter() )
  {
    assert_ulps_eq!( a, b );
  }

  let vec_zero = [ 0.0, 0.0 ];
  let got = vector::normalized( &vec_zero );

  for value in &got
  {
    assert!( value.is_nan(), "Expected NaN, got {value}" );
  }

}

#[ test ]
fn test_normalize_to()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    Float,
  };

  let mut vec_a = [ 3.0, 4.0 ];
  vector::normalize_to( &mut vec_a, 10.0 );
  let expected = [ 6.0, 8.0 ];
  assert_ulps_eq!( vec_a[ .. ], expected[ .. ] );

  let mut got = [ 0.0, 0.0 ];
  vector::normalize_to( &mut got, 10.0 );

  for value in &got
  {
    assert!( value.is_nan(), "Expected NaN, got {value}" );
  }

}

#[ test ]
fn test_normalized_to()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    Float,
  };

  let vec_a = [ 3.0, 4.0 ];
  let result = vector::normalized_to( &vec_a, 10.0 );
  let expected = [ 6.0, 8.0 ];
  assert_ulps_eq!( result[ .. ], expected[ .. ] );

  let vec_zero = [ 0.0, 0.0 ];
  let got = vector::normalized_to( &vec_zero, 10.0 );
  for value in &got
  {
    assert!( value.is_nan(), "Expected NaN, got {value}" );
  }
}

#[ test ]
fn test_project_on()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    // Float,
  };

  let mut vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 4.0, 5.0, 6.0 ];
  vector::project_on( &mut vec_a, &vec_b );
  let expected = [ 1.662_337_662_337_662_4, 2.077_922_077_922_078, 2.493_506_493_506_493_4 ];
  // approx has no fixed-size array impls, so whole-vector comparison goes through the
  // slice impl ( `UlpsEq< [ B ] > for [ A ]` ) via `[ .. ]`.
  assert_ulps_eq!( vec_a[ .. ], expected[ .. ] );

  let mut vec_zero = [ 0.0, 0.0, 0.0 ];
  vector::project_on( &mut vec_zero, &vec_b );
  // Projecting the zero vector yields exactly 0.0 (0 / anything = 0, 0 * anything = 0 in
  // IEEE-754) — no rounding is possible, so exact equality is correct here.
  #[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
  { assert_eq!( vec_zero, [ 0.0, 0.0, 0.0 ], "Projection failed for zero vector" ); }
}

#[ test ]
fn test_projected_on()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    // Float,
  };

  let vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 4.0, 5.0, 6.0 ];
  let result = vector::projected_on( &vec_a, &vec_b );
  let expected = [ 1.662_337_662_337_662_4, 2.077_922_077_922_078, 2.493_506_493_506_493_4 ];
  assert_ulps_eq!( result[ .. ], expected[ .. ], max_ulps = 10000 );

  let vec_zero = [ 0.0, 0.0, 0.0 ];
  let got = vector::projected_on( &vec_zero, &vec_b );
  // Projecting the zero vector yields exactly 0.0 (0 / anything = 0, 0 * anything = 0 in
  // IEEE-754) — no rounding is possible, so exact equality is correct here.
  #[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
  { assert_eq!( got, [ 0.0, 0.0, 0.0 ], "Projected on function failed for zero vector" ); }
}

// test_kind: bug_reproducer(BUG-448)
/// ## Root Cause
/// Not a code defect: `project_on(r, b)` computes `scalar = dot(r,b) / mag2(b)`, and
/// `projected_on(a, b)` is a thin wrapper around it. When `b` is the zero vector, `mag2(b)` is
/// exactly `0.0` and `dot(r,b)` is exactly `0.0` too (dotting anything with the zero vector is
/// `0.0`), so `scalar` is `0.0 / 0.0`, i.e. `NaN` -- and every written component is `b`'s own
/// (zero) element times that `NaN`, which is `NaN`, not `0.0`. This is the mathematically
/// correct, intentional result: projecting onto a degenerate (zero-length) axis is undefined,
/// so `NaN` is the honest encoding rather than an arbitrary fallback. BUG-448 resolved as
/// "working as intended, previously undocumented and untested for this specific input shape" --
/// see the `# Zero-magnitude b` doc sections now on `project_on`/`projected_on` above.
/// ## Why Not Caught
/// The pre-existing `test_project_on`/`test_projected_on` (above) only tested the *opposite*
/// degenerate shape -- a zero `r`/`a` projected onto a *nonzero* `b`, which yields an exact,
/// well-defined `0.0` (no division by zero at all: `dot` is `0.0`, `mag2(b)` is nonzero, so
/// `scalar` is exactly `0.0`). Neither test exercised a *zero `b`*, the actual degenerate-axis
/// case the doc sections above describe.
/// ## Fix Applied
/// Documentation only -- no source behavior changed. Added `# Zero-magnitude b` doc sections to
/// `project_on`/`projected_on` (this file) explaining the `NaN` contract explicitly, matching
/// the existing `# Zero-magnitude input` precedent already on `normalize`/`normalized`. An
/// `Option`-returning restructure was considered and rejected: 7 in-workspace call sites of
/// `project_on`/`projected_on` sit outside this task's edit scope and would all need updating.
/// ## Prevention
/// This test projects a nonzero `r`/`a` onto a zero `b` and asserts every resulting component
/// is `NaN`, closing the coverage gap the existing tests left (zero-`a`-onto-nonzero-`b` was
/// covered; nonzero-`a`-onto-zero-`b` was not) and locking in the documented contract so a
/// future change cannot silently alter it without a test failing.
/// ## Pitfall
/// A function's zero/degenerate-input behavior can be intentional and correct while still being
/// undocumented and untested -- "add a test and document the contract" is itself a valid bug
/// resolution when the investigation concludes the code is already right; it is not always a
/// signal that a code change is owed.
#[ test ]
fn test_project_on_zero_b_yields_nan()
{
  use the_module::vector;

  let vec_zero_b : [ f32 ; 3 ] = [ 0.0, 0.0, 0.0 ];

  // In-place `project_on`: nonzero `r`, zero `b`.
  let mut r = [ 1.0, 2.0, 3.0 ];
  vector::project_on( &mut r, &vec_zero_b );
  for value in &r
  {
    assert!( value.is_nan(), "project_on( nonzero r, zero b ) should yield NaN, got {value}" );
  }

  // Allocating `projected_on`: nonzero `a`, zero `b`.
  let vec_a = [ 1.0, 2.0, 3.0 ];
  let got = vector::projected_on( &vec_a, &vec_zero_b );
  for value in &got
  {
    assert!( value.is_nan(), "projected_on( nonzero a, zero b ) should yield NaN, got {value}" );
  }
}

#[ test ]
fn test_angle()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
    // Float,
  };

  let vec_a = [ 1.0, 0.0 ];
  let vec_b = [ 0.0, 1.0 ];
  let result = vector::angle( &vec_a, &vec_b );
  assert_ulps_eq!( result, core::f32::consts::FRAC_PI_2 );

  let vec_zero = [ 0.0, 0.0 ];
  let got = vector::angle( &vec_a, &vec_zero );
  assert!( got.is_nan(), "Angle calculation failed for zero vector" );
}

// test_kind: bug_reproducer(BUG-446)
/// ## Root Cause
/// `vector::angle(a, b)` (`vector/arithmetics.rs`) computes `cos_theta = dot(a,b) / (mag(a) *
/// mag(b))` and passed it straight into `.acos()`. `cos_theta` is mathematically bounded to
/// `[-1, 1]`, but `mag(a) = dot(a,a).sqrt()` is a rounded value -- squaring it back inside the
/// denominator does not always exactly reproduce `dot(a,a)`, so the ratio can land fractionally
/// outside `[-1, 1]` even for simple, exact integer-valued inputs. `.acos()` on an out-of-range
/// input is documented to return `NaN`.
/// ## Why Not Caught
/// The pre-existing `test_angle` only exercised orthogonal vectors (`cos_theta = 0.0` exactly,
/// no rounding involved) and one deliberately-NaN zero-vector case -- never a case where
/// `cos_theta` itself is the thing that rounds outside `[-1, 1]`, which is exactly what happens
/// for a vector's angle with itself (`cos_theta` should be exactly `1.0`) or with its own exact
/// negation (`cos_theta` should be exactly `-1.0`).
/// ## Fix Applied
/// Added `let cos_theta = cos_theta.max( -E::one() ).min( E::one() );` before `.acos()` in
/// `vector::angle` (`mdmath_core/src/vector/arithmetics.rs`) -- the same defensive-clamp pattern
/// already applied at BUG-272 (`ndarray_cg`'s `to_euler_xyz`) and BUG-447 (Shepperd's method).
/// ## Prevention
/// This test uses `a = b = [ 1.0, 0.0, 1.0 ]`, empirically confirmed (`rustc`, release mode) to
/// round `dot(a,a) / ( mag(a) * mag(a) )` to `1.000000119..._f32`, strictly greater than `1.0`
/// -- and its exact negation `[ -1.0, 0.0, -1.0 ]`, which rounds to `-1.000000119..._f32`.
/// Pre-fix, `.acos()` on both returns `NaN`; post-fix, the clamp recovers the mathematically
/// correct `0.0` and `PI` respectively.
/// ## Pitfall
/// Any `.acos()`/`.asin()` fed a ratio derived from a dot-product-over-magnitude computation
/// must clamp defensively -- the ratio is mathematically bounded to `[-1, 1]`, but IEEE-754
/// rounding in the `sqrt`-then-multiply chain can push it fractionally outside that range even
/// for exact, simple inputs (a vector compared with itself or its own negation), not just
/// extreme or adversarial ones.
#[ test ]
fn test_angle_self_and_negation_no_nan()
{
  use the_module::
  {
    assert_ulps_eq,
    vector,
  };

  // Empirically confirmed to round `dot(a,a) / ( mag(a) * mag(a) )` to a value strictly
  // greater than `1.0` in f32, which pre-fix drove `.acos()` to `NaN`.
  let vec_a = [ 1.0, 0.0, 1.0 ];
  let self_angle : f32 = vector::angle( &vec_a, &vec_a );
  assert!( !self_angle.is_nan(), "angle(a, a) must not be NaN, got {self_angle}" );
  assert_ulps_eq!( self_angle, 0.0 );

  // Exact negation rounds the ratio to strictly less than `-1.0`.
  let vec_neg_a = [ -1.0, 0.0, -1.0 ];
  let opposite_angle : f32 = vector::angle( &vec_a, &vec_neg_a );
  assert!( !opposite_angle.is_nan(), "angle(a, -a) must not be NaN, got {opposite_angle}" );
  assert_ulps_eq!( opposite_angle, core::f32::consts::PI );
}

#[ test ]
fn test_is_orthogonal()
{
  use the_module::
  vector;

  // Test with orthogonal vectors
  let vec_a = [ 1.0, 0.0 ];
  let vec_b = [ 0.0, 1.0 ];
  assert!( vector::is_orthogonal( &vec_a, &vec_b ), "Orthogonal test failed for orthogonal vectors" );

  // Test with non-orthogonal vectors
  let vec_c = [ 1.0, 1.0 ];
  let vec_d = [ 1.0, 0.0 ];
  assert!( !vector::is_orthogonal( &vec_c, &vec_d ), "Orthogonal test failed for non-orthogonal vectors" );

  // Test with zero vector
  let vec_zero = [ 0.0, 0.0 ];
  assert!( vector::is_orthogonal( &vec_a, &vec_zero ), "Orthogonal test failed for zero vector" );
}

// test_kind: bug_reproducer(BUG-270)
/// ## Root Cause
/// `Cargo.toml`'s `arithmetics = [ "float" ]` feature declaration omitted `approx`, but
/// `vector/arithmetics.rs`'s `is_orthogonal` unconditionally uses `crate::approx::ulps_eq` and
/// bounds `E : approx::UlpsEq` with no `#[cfg(feature = "approx")]` guard -- so the file has
/// always needed `approx` to compile, regardless of what `Cargo.toml` declared.
/// ## Why Not Caught
/// Every existing test run (including this file's own pre-existing `test_is_orthogonal`) goes
/// through `cargo test -p mdmath_core --all-features`, which enables `approx` unconditionally
/// alongside `arithmetics` -- masking that `arithmetics` alone (or the `full` bundle, which pulls
/// in `arithmetics` but not `approx`) fails to build with E0432/E0433. This crate's own sibling
/// `ndarray_cg` also always requests both features together in its own `Cargo.toml`, so the one
/// real in-workspace consumer never tripped over the gap either.
/// ## Fix Applied
/// Changed `Cargo.toml`'s `arithmetics = [ "float" ]` to `arithmetics = [ "float", "approx" ]`,
/// making the already-real dependency explicit and cargo-enforced.
/// ## Prevention
/// This test's regression value is specifically in *how* it's invoked, not its body: run in
/// isolation via `cargo test -p mdmath_core --no-default-features --features enabled,arithmetics`
/// (no `approx`, no `--all-features`), it fails to compile before the fix (E0432: unresolved
/// import `crate::approx`) and passes after. Run under the crate's normal `--all-features` suite
/// it still exercises the same `is_orthogonal` call as a plain assertion, alongside the
/// pre-existing `test_is_orthogonal` above.
/// ## Pitfall
/// A feature flag whose gated source file unconditionally uses a *second* feature's items only
/// fails to build under the one combination that selects the first without the second --
/// `--all-features` and any consumer that happens to always request both together never exercise
/// that gap, so the declared feature graph can silently diverge from the code's real requirements
/// for a long time.
#[ test ]
fn test_is_orthogonal_builds_under_arithmetics_feature_alone()
{
  use the_module::vector;

  // Same call as `test_is_orthogonal` above -- the meaningful check here is that this file
  // compiles at all under an isolated `--features enabled,arithmetics` build (see `## Prevention`).
  let vec_a = [ 1.0, 0.0 ];
  let vec_b = [ 0.0, 1.0 ];
  assert!( vector::is_orthogonal( &vec_a, &vec_b ) );
}

#[ test ]
fn test_cross_mut()
{
  use the_module::
  {
    vector,
    assert_ulps_eq
  };

  let mut vec_a = [ 1.0, 0.0, 0.0 ];
  let vec_b = [ 0.0, 1.0, 0.0 ];
  vector::cross_mut( &mut vec_a, &vec_b );

  let exp = [ 0.0, 0.0, 1.0 ];
  for ( r, e ) in vec_a.iter().zip( exp.iter() )
  {
    assert_ulps_eq!( r, e );
  }

  let mut vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 1.0, 5.0, 7.0 ];
  vector::cross_mut( &mut vec_a, &vec_b );

  let exp = [ -1.0, -4.0, 3.0 ];
  for ( r, e ) in vec_a.iter().zip( exp.iter() )
  {
    assert_ulps_eq!( r, e );
  }
}

#[ test ]
fn test_cross()
{
  use the_module::
  {
    vector,
    assert_ulps_eq
  };

  let vec_a = [ 1.0, 0.0, 0.0 ];
  let vec_b = [ 0.0, 1.0, 0.0 ];
  let res = vector::cross( &vec_a, &vec_b );

  let exp = [ 0.0, 0.0, 1.0 ];
  for ( r, e ) in res.iter().zip( exp.iter() )
  {
    assert_ulps_eq!( r, e );
  }

  let vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 1.0, 5.0, 7.0 ];
  let res = vector::cross( &vec_a, &vec_b );

  let exp = [ -1.0, -4.0, 3.0 ];
  for ( r, e ) in res.iter().zip( exp.iter() )
  {
    assert_ulps_eq!( r, e );
  }
}

#[ test ]
fn test_integer_arithmetics()
{
  use the_module::vector;

  // `dot` / `mag2` work for any integer scalar (no subtraction).
  assert_eq!( vector::dot( &[ 1i32, 2, 3 ], &[ 4i32, 5, 6 ] ), 32 );
  assert_eq!( vector::mag2( &[ 1i32, 2, 3 ] ), 14 );

  // `cross` / `cross_mut` need a signed scalar; exercise both forms with i32.
  assert_eq!( vector::cross( &[ 1i32, 0, 0 ], &[ 0i32, 1, 0 ] ), [ 0, 0, 1 ] );
  assert_eq!( vector::cross( &[ 1i32, 2, 3 ], &[ 1i32, 5, 7 ] ), [ -1, -4, 3 ] );

  let mut v = [ 1i32, 2, 3 ];
  vector::cross_mut( &mut v, &[ 1i32, 5, 7 ] );
  assert_eq!( v, [ -1, -4, 3 ] );
}

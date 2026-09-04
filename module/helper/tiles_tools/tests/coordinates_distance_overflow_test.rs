//! Tests for `coordinates::{ hexagonal, square, isometric, triangular }`'s
//! `distance()` implementations -- overflow/panic and silent-wraparound
//! behavior for large-but-otherwise-valid `i32` coordinates.
//!
//! `Coordinate::new`/`new_uncheked` accept the full `i32` range unchecked on
//! every coordinate type in this module (all fields are `pub`), but every
//! `distance()` implementation subtracted/negated raw `i32`s -- or narrowed
//! an `i64` diff back to `u32` before summing -- both overflow well within
//! that same accepted domain.

// test_kind: bug_reproducer(BUG-350)
//
// ## Root Cause
// None of `coordinates::{ hexagonal, square, isometric, triangular }`'s
// `distance()` implementations bounded their arithmetic against the actual
// domain their own constructors allow. Hex's inherent `i32` method, hex's
// `Distance` trait method, and both square variants ran ordinary `i32`
// negation/subtraction directly on raw field values -- overflowing for
// coordinates on the order of `2e9` apart, or whenever a field equals
// `i32::MIN` (negating it alone overflows). The hex trait method separately
// narrowed each term to `u32` with `as u32` BEFORE the final sum, so the sum
// itself could still overflow `u32` even though each narrowed term fit
// individually. Triangular's impl already widened to `i64` for the whole
// subtract/abs/sum chain (so it never panics), but its FINAL `as u32` cast
// silently wrapped instead of saturating once that `i64` sum exceeded
// `u32::MAX` -- a true distance of `8_000_000_000` silently became
// `3_705_032_704` (`8e9 mod 2^32`), not a crash but silently wrong data.
//
// ## Why Not Caught
// Every existing `distance()` test (doctests, `tests/integration/*.rs`,
// `benches/coordinate_benchmarks.rs`) exercises only small, ordinary
// tile-grid-scale coordinates (single- and double-digit magnitudes). Nothing
// in the crate's prior coverage exercised coordinates anywhere near the
// `i32` boundary, so the gap between "what the constructors accept" and
// "what `distance()` can safely process" had no trigger.
//
// ## Fix Applied
// Every `distance()` implementation now widens to `i64` for the ENTIRE
// computation -- including hex's negations that build `s`/`other_s`, not
// just the subtractions -- before narrowing the final result back to the
// return type via `.clamp( 0, i64::from( {i32,u32}::MAX ) ) as {i32,u32}`,
// saturating instead of panicking or silently wrapping. See the
// `Fix(BUG-350)` comment directly above each of the 5 fixed methods in
// `src/coordinates/{ hexagonal, square, isometric, triangular }.rs`.
//
// ## Prevention
// Any `distance()`-shaped method on a coordinate type whose fields are
// `pub` and whose constructor performs no range validation must widen to a
// strictly larger integer type for its ENTIRE computation (every negation
// and subtraction, not only some of them) and saturate -- never silently
// wrap via a bare `as` -- when narrowing the final result back down.
//
// ## Pitfall
// Widening only part of a computation (e.g. the subtraction but not the
// negation feeding it, as hex's trait impl originally did) or narrowing
// each term before a final summation (as the same impl also did) both
// silently reintroduce the exact overflow the wider type was meant to
// prevent -- the widened type must cover every single operation, and
// narrowing must happen exactly once, at the very end, saturating.

use tiles_tools::coordinates::hexagonal::{ Axial, Coordinate as HexCoord, Pointy };
use tiles_tools::coordinates::square::{ Coordinate as SquareCoord, FourConnected, EightConnected };
use tiles_tools::coordinates::isometric::{ Coordinate as IsoCoord, Diamond };
use tiles_tools::coordinates::triangular::{ Coordinate as TriCoord, FlatSided };
use tiles_tools::coordinates::Distance;

/// Site 1: hex INHERENT `i32` `distance()` -- plain `i32` subtraction
/// overflows once the two coordinates are ~2e9 apart (`self.q - q` at
/// `hexagonal.rs`). Pre-fix: panics `attempt to subtract with overflow`.
/// Post-fix: saturates to `i32::MAX` (true distance `4_000_000_000` does not
/// fit in `i32`).
#[ test ]
fn hex_inherent_distance_saturates_on_extreme_subtraction()
{
  let a = HexCoord::< Axial, Pointy >::new( 2_000_000_000, 0 );
  let b = HexCoord::< Axial, Pointy >::new( -2_000_000_000, 0 );
  assert_eq!( a.distance( b ), i32::MAX );
}

/// Site 1: hex INHERENT `i32` `distance()` -- negating `i32::MIN` alone
/// overflows (`-self.q` at `hexagonal.rs`), independent of the subtraction
/// site above. Pre-fix: panics `attempt to negate with overflow`. Post-fix:
/// saturates to `i32::MAX` (true distance `2_147_483_648` does not fit).
#[ test ]
fn hex_inherent_distance_saturates_on_i32_min_negation()
{
  let a = HexCoord::< Axial, Pointy >::new( i32::MIN, 0 );
  let b = HexCoord::< Axial, Pointy >::new( 0, 0 );
  assert_eq!( a.distance( b ), i32::MAX );
}

/// Site 2: hex `Distance` TRAIT `u32` `distance()` -- reached only via UFCS
/// since the inherent method above always shadows `a.distance(b)` method-call
/// syntax on a concrete hex `Coordinate`. Same extreme-subtraction input as
/// the inherent test; here the true distance (`4_000_000_000`) DOES fit in
/// `u32`, so the fixed value is exact, not saturated. Pre-fix: panicked
/// `attempt to add with overflow` (each term was narrowed to `u32` before
/// the final sum, and the sum itself overflowed `u32`).
#[ test ]
fn hex_trait_distance_exact_on_extreme_subtraction()
{
  let a = HexCoord::< Axial, Pointy >::new( 2_000_000_000, 0 );
  let b = HexCoord::< Axial, Pointy >::new( -2_000_000_000, 0 );
  assert_eq!( Distance::distance( &a, &b ), 4_000_000_000u32 );
}

/// Site 2: hex `Distance` TRAIT `u32` `distance()` -- `i32::MIN` negation.
/// Pre-fix: panicked `attempt to negate with overflow` (the `i64::from(...)`
/// upgrade wrapped an already-overflowed `i32` negation, too late to help).
/// Post-fix: exact `2_147_483_648` (fits `u32`).
#[ test ]
fn hex_trait_distance_exact_on_i32_min_negation()
{
  let a = HexCoord::< Axial, Pointy >::new( i32::MIN, 0 );
  let b = HexCoord::< Axial, Pointy >::new( 0, 0 );
  assert_eq!( Distance::distance( &a, &b ), 2_147_483_648u32 );
}

/// Site 3: `square::Coordinate< FourConnected >::distance()` -- plain `i32`
/// subtraction overflow. Pre-fix: panics `attempt to subtract with
/// overflow`. Post-fix: exact `4_000_000_000` (fits `u32`).
#[ test ]
fn square_four_connected_distance_exact_on_extreme_subtraction()
{
  let a = SquareCoord::< FourConnected >::new( 2_000_000_000, 0 );
  let b = SquareCoord::< FourConnected >::new( -2_000_000_000, 0 );
  assert_eq!( a.distance( &b ), 4_000_000_000u32 );
}

/// Site 3: `square::Coordinate< FourConnected >::distance()` -- true
/// Manhattan distance at the absolute `i32` extremes exceeds `u32::MAX`.
/// Post-fix: saturates to `u32::MAX` rather than panicking or wrapping.
#[ test ]
fn square_four_connected_distance_saturates_beyond_u32_max()
{
  let a = SquareCoord::< FourConnected >::new( i32::MAX, i32::MAX );
  let b = SquareCoord::< FourConnected >::new( i32::MIN, i32::MIN );
  assert_eq!( a.distance( &b ), u32::MAX );
}

/// Site 4: `square::Coordinate< EightConnected >::distance()` -- same
/// subtraction-overflow pattern as `FourConnected` (shared root cause,
/// separate impl). Pre-fix: panics `attempt to subtract with overflow`.
/// Post-fix: exact `4_000_000_000`.
#[ test ]
fn square_eight_connected_distance_exact_on_extreme_subtraction()
{
  let a = SquareCoord::< EightConnected >::new( 2_000_000_000, 0 );
  let b = SquareCoord::< EightConnected >::new( -2_000_000_000, 0 );
  assert_eq!( a.distance( &b ), 4_000_000_000u32 );
}

/// Site 5: `isometric::Coordinate< Diamond >::distance()` -- identical
/// subtraction-overflow pattern to `square::FourConnected`. Pre-fix: panics
/// `attempt to subtract with overflow`. Post-fix: exact `4_000_000_000`.
#[ test ]
fn isometric_distance_exact_on_extreme_subtraction()
{
  let a = IsoCoord::< Diamond >::new( 2_000_000_000, 0 );
  let b = IsoCoord::< Diamond >::new( -2_000_000_000, 0 );
  assert_eq!( a.distance( &b ), 4_000_000_000u32 );
}

/// Site 6: `triangular::Coordinate::distance()` -- the one site that does
/// NOT panic pre-fix; it silently wraps instead. True distance for these
/// inputs is `8_000_000_000`, which does not fit `u32`; pre-fix the bare
/// `as u32` cast wrapped this to `3_705_032_704` (`8e9 mod 2^32`) with no
/// error at all. Post-fix: saturates to `u32::MAX`.
#[ test ]
fn triangular_distance_saturates_instead_of_wrapping()
{
  let a = TriCoord::< FlatSided >::new( 1_000_000_000, 1_000_000_000, -1_999_999_998 ).unwrap();
  let b = TriCoord::< FlatSided >::new( -1_000_000_000, -1_000_000_000, 2_000_000_002 ).unwrap();
  let d = a.distance( &b );
  assert_ne!( d, 3_705_032_704u32, "must not silently wrap back to the pre-fix corrupted value" );
  assert_eq!( d, u32::MAX );
}

/// Regression guard: ordinary, small-magnitude coordinates (the only scale
/// every pre-existing test/doctest/benchmark ever exercised) must still
/// produce the exact same results after widening every `distance()` impl to
/// `i64` internally -- the fix must not change behavior anywhere within the
/// domain that already worked.
#[ test ]
fn all_distance_impls_unchanged_for_ordinary_small_coordinates()
{
  let hex_a = HexCoord::< Axial, Pointy >::new( 0, 0 );
  let hex_b = HexCoord::< Axial, Pointy >::new( 1, 1 );
  assert_eq!( hex_a.distance( hex_b ), 2 );
  assert_eq!( Distance::distance( &hex_a, &hex_b ), 2u32 );

  let sq4_a = SquareCoord::< FourConnected >::new( 0, 0 );
  let sq4_b = SquareCoord::< FourConnected >::new( 3, 4 );
  assert_eq!( sq4_a.distance( &sq4_b ), 7 );

  let sq8_a = SquareCoord::< EightConnected >::new( 0, 0 );
  let sq8_b = SquareCoord::< EightConnected >::new( 3, 4 );
  assert_eq!( sq8_a.distance( &sq8_b ), 4 );

  let iso_a = IsoCoord::< Diamond >::new( 0, 0 );
  let iso_b = IsoCoord::< Diamond >::new( 3, 4 );
  assert_eq!( iso_a.distance( &iso_b ), 7 );

  let tri_a = TriCoord::< FlatSided >::new( 0, 0, 1 ).unwrap();
  let tri_b = TriCoord::< FlatSided >::new( 10, 15, -24 ).unwrap();
  assert_eq!( tri_a.distance( &tri_b ), 50 );
}

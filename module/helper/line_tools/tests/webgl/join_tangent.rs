//! Regression coverage for BUG-158: the join-tangent formula shared (as identical copy-pasted
//! GLSL) by `join_miter.vert`, `join_bevel.vert`, `join_round.vert`, `body.vert` and
//! `body_terminal.vert` produced `NaN` whenever three consecutive line points formed a ~180
//! degree cusp.
//!
//! This crate has no shader-execution test harness (`tests/webgl/*.rs` exercises the CPU-side
//! `Line` API only, never a GPU/WebGL context) and the tangent formula has no CPU-side twin
//! anywhere in `src/` -- so `guarded_tangent` below is a line-for-line Rust port of the fixed
//! GLSL, kept deliberately close to the shader source so the mapping stays auditable.

#![ expect( clippy::float_cmp, reason = "assertions check the guard's exact fallback/no-op identity (same deterministic operations in the same order, no accumulated rounding drift), not approximate equality" ) ]

fn normalize( v : [ f32; 2 ] ) -> [ f32; 2 ]
{
  let len = ( v[ 0 ] * v[ 0 ] + v[ 1 ] * v[ 1 ] ).sqrt();
  [ v[ 0 ] / len, v[ 1 ] / len ]
}

/// Port of the fixed `tangent` computation now shared by all 5 affected `.vert` files
/// (`vec2 dirIn = normalize( pointB - pointA ); vec2 dirOut = normalize( pointC - pointB );
/// vec2 tangentSum = dirOut + dirIn; vec2 tangent = dot( tangentSum, tangentSum ) > 1e-12 ?
/// normalize( tangentSum ) : dirIn;`).
fn guarded_tangent( point_a : [ f32; 2 ], point_b : [ f32; 2 ], point_c : [ f32; 2 ] ) -> [ f32; 2 ]
{
  let dir_in = normalize( [ point_b[ 0 ] - point_a[ 0 ], point_b[ 1 ] - point_a[ 1 ] ] );
  let dir_out = normalize( [ point_c[ 0 ] - point_b[ 0 ], point_c[ 1 ] - point_b[ 1 ] ] );
  let tangent_sum = [ dir_out[ 0 ] + dir_in[ 0 ], dir_out[ 1 ] + dir_in[ 1 ] ];
  let sq_len = tangent_sum[ 0 ] * tangent_sum[ 0 ] + tangent_sum[ 1 ] * tangent_sum[ 1 ];
  if sq_len > 1e-12 { normalize( tangent_sum ) } else { dir_in }
}

/// ## Root Cause
/// The pre-fix formula `normalize( normalize( pointC - pointB ) + normalize( pointB - pointA ) )`
/// divides by the sum vector's own length with no guard. When the path reverses ~180 degrees at
/// `pointB` (a cusp), `normalize(pointC-pointB)` and `normalize(pointB-pointA)` are equal and
/// opposite, their sum is the zero vector, and `normalize(vec2(0,0))` is `NaN` in GLSL (0/0 on
/// both components) -- propagating through `normal`/`sigma`/`offsetPoint` into `gl_Position` and
/// corrupting the joint's geometry.
///
/// ## Why Not Caught
/// No CPU-side twin of this formula exists to unit-test (`src/` has no `tangent`/`normalize`
/// line-join computation outside the 5 `.vert` files), and this crate has no shader-execution
/// test harness, so the NaN was reachable only via live rendering with a cusp in the input path.
///
/// ## Fix Applied
/// Added a squared-length guard (`dot(tangentSum,tangentSum) > 1e-12`) before the final
/// `normalize`, falling back to the incoming segment's own direction (`dirIn`, already a unit
/// vector) when the sum collapses -- identical guard applied to all 5 `.vert` files.
///
/// ## Prevention
/// This test mirrors the guarded formula in Rust and asserts it stays finite for an exact cusp
/// input, and stays numerically identical to the unguarded formula for an ordinary bend (proving
/// the guard changes nothing outside the degenerate case).
///
/// ## Pitfall
/// A `vec2` sum whose length can legitimately reach exactly zero must never be fed to `normalize`
/// without a guard -- GLSL (unlike some CPU math libraries) has no defined "safe normalize" and
/// silently produces `NaN`, not a panic or a zero vector.
// test_kind: bug_reproducer(BUG-158)
#[ test ]
fn guarded_tangent_stays_finite_at_a_cusp_bug_158()
{
  // A -> B -> C reverses ~180 degrees at B: dir_in = (1,0), dir_out = (-1,0), sum = (0,0).
  let point_a = [ 0.0, 0.0 ];
  let point_b = [ 1.0, 0.0 ];
  let point_c = [ -1.0, 0.0 ];

  // Pre-fix formula, reproduced inline (not called by any shader after the fix): confirms the
  // cusp genuinely produces NaN, so the guard below is verified against a real failure.
  let dir_in = normalize( [ point_b[ 0 ] - point_a[ 0 ], point_b[ 1 ] - point_a[ 1 ] ] );
  let dir_out = normalize( [ point_c[ 0 ] - point_b[ 0 ], point_c[ 1 ] - point_b[ 1 ] ] );
  let unguarded = normalize( [ dir_out[ 0 ] + dir_in[ 0 ], dir_out[ 1 ] + dir_in[ 1 ] ] );
  assert!( unguarded[ 0 ].is_nan() && unguarded[ 1 ].is_nan(), "expected the unguarded formula to reproduce the real BUG-158 NaN at an exact cusp, got {unguarded:?}" );

  let tangent = guarded_tangent( point_a, point_b, point_c );
  assert!( tangent[ 0 ].is_finite() && tangent[ 1 ].is_finite(), "guarded_tangent must never produce NaN/inf at a cusp, got {tangent:?}" );
  let len = ( tangent[ 0 ] * tangent[ 0 ] + tangent[ 1 ] * tangent[ 1 ] ).sqrt();
  assert!( ( len - 1.0 ).abs() < 1e-6, "guarded_tangent's fallback (dirIn) must stay unit-length, got length {len}" );
  assert_eq!( tangent, dir_in, "at an exact cusp, guarded_tangent must fall back to dirIn exactly" );
}

// A second, independently-constructed cusp (vertical segment folding back on itself) --
// guards against a fix that only special-cases the horizontal axis.
#[ test ]
fn guarded_tangent_stays_finite_at_a_vertical_cusp_bug_158()
{
  let point_a = [ 5.0, 5.0 ];
  let point_b = [ 5.0, 8.0 ];
  let point_c = [ 5.0, 5.0 ];

  let tangent = guarded_tangent( point_a, point_b, point_c );
  assert!( tangent[ 0 ].is_finite() && tangent[ 1 ].is_finite(), "guarded_tangent must never produce NaN/inf at a cusp, got {tangent:?}" );
}

// Outside the degenerate case, the guard must be a no-op: guarded_tangent's result for an
// ordinary bend must exactly match the original unguarded formula.
#[ test ]
fn guarded_tangent_matches_unguarded_formula_for_an_ordinary_bend()
{
  let point_a = [ 0.0, 0.0 ];
  let point_b = [ 1.0, 0.0 ];
  let point_c = [ 1.0, 1.0 ]; // 90-degree bend, not a cusp

  let dir_in = normalize( [ point_b[ 0 ] - point_a[ 0 ], point_b[ 1 ] - point_a[ 1 ] ] );
  let dir_out = normalize( [ point_c[ 0 ] - point_b[ 0 ], point_c[ 1 ] - point_b[ 1 ] ] );
  let unguarded = normalize( [ dir_out[ 0 ] + dir_in[ 0 ], dir_out[ 1 ] + dir_in[ 1 ] ] );

  let guarded = guarded_tangent( point_a, point_b, point_c );
  assert_eq!( guarded, unguarded, "guarded_tangent must be bit-for-bit identical to the original formula outside the degenerate case" );

  let expected = std::f32::consts::FRAC_1_SQRT_2;
  assert!( ( guarded[ 0 ] - expected ).abs() < 1e-6 && ( guarded[ 1 ] - expected ).abs() < 1e-6, "expected the 90-degree bend's tangent to be (1/sqrt(2), 1/sqrt(2)), got {guarded:?}" );
}

// A near-cusp (179.99 degrees, not exact) must still resolve through the ordinary branch --
// the guard's 1e-12 squared-length threshold must not misfire on a merely-sharp (but
// non-degenerate) bend and force the dirIn fallback where a real tangent is still computable.
#[ test ]
fn guarded_tangent_uses_the_real_formula_for_a_near_cusp_not_just_an_exact_one()
{
  let point_a = [ 0.0, 0.0 ];
  let point_b = [ 1.0, 0.0 ];
  // ~0.01 degrees off from an exact 180-degree reversal.
  let angle = std::f32::consts::PI - 0.0002_f32;
  let point_c = [ point_b[ 0 ] + angle.cos(), point_b[ 1 ] + angle.sin() ];

  let dir_in = normalize( [ point_b[ 0 ] - point_a[ 0 ], point_b[ 1 ] - point_a[ 1 ] ] );
  let dir_out = normalize( [ point_c[ 0 ] - point_b[ 0 ], point_c[ 1 ] - point_b[ 1 ] ] );
  let unguarded = normalize( [ dir_out[ 0 ] + dir_in[ 0 ], dir_out[ 1 ] + dir_in[ 1 ] ] );

  let guarded = guarded_tangent( point_a, point_b, point_c );
  assert!( guarded[ 0 ].is_finite() && guarded[ 1 ].is_finite(), "near-cusp must stay finite, got {guarded:?}" );
  assert_ne!( guarded, dir_in, "a near-cusp (not an exact cusp) must resolve through the real formula, not the dirIn fallback" );
  assert!( ( guarded[ 0 ] - unguarded[ 0 ] ).abs() < 1e-3 && ( guarded[ 1 ] - unguarded[ 1 ] ).abs() < 1e-3, "near-cusp guarded result should still track the unguarded formula closely, got {guarded:?} vs {unguarded:?}" );
}

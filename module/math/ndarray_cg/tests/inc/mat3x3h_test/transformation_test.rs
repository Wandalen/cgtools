use ndarray_cg::{ *, approx };
use approx::assert_abs_diff_eq;
use mat3x3h::{rot, scale, translation, look_to_rh};

// The translation matrix's row `i` is `[ 0, .., 1 ( at i ), .., 0, t_i ]`; dotted with the
// homogeneous origin `[ 0, 0, 0, 1 ]` this reduces to `t_i * 1 + 0 * 0 + .. `, i.e. multiplying
// by exactly 1.0 and adding exact zeros — both exact under IEEE-754 on every target, so the
// result is always bit-identical to the input translation component.
#[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
#[ test ]
fn test_translation()
{
  let vec = Vector( [ 0.0_f32, 0.0, 0.0, 1.0 ] );
  let translation = translation( [ 1.0_f32, 2.0, 3.0 ] );
  let res = translation * vec;

  assert_eq!( res.x(), 1.0 );
  assert_eq!( res.y(), 2.0 );
  assert_eq!( res.z(), 3.0 );
}

#[ test ]
fn test_rotation()
{
  let x = Vector( [ 1.0_f32, 0.0, 0.0, 1.0 ] );
  let y = Vector( [ 0.0_f32, 1.0, 0.0, 1.0 ] );
  let z = Vector( [ 0.0_f32, 0.0, 1.0, 1.0 ] );

  let angle = std::f32::consts::FRAC_PI_2;

  let rotation_x = rot( angle, 0.0, 0.0 );
  let rotation_y = rot( 0.0, angle, 0.0 );
  let rotation_z = rot( 0.0, 0.0, angle );

  let rotated_x = rotation_z * x;
  let rotated_y = rotation_x * y;
  let rotated_z = rotation_y * z;

  // Trig output (`sin`/`cos` of `FRAC_PI_2`) is not guaranteed bit-exact across targets —
  // this crate also builds for wasm32, whose libm can round the last bit differently than the
  // host's. Approximate comparison matches the pattern already used for rotation-derived
  // values in mat3x3_test/mat4x4_test's `from_quat`/`from_scale_rotation_translation` tests.
  assert_abs_diff_eq!( rotated_x.y(), 1.0 );
  assert_abs_diff_eq!( rotated_y.z(), 1.0 );
  assert_abs_diff_eq!( rotated_z.x(), 1.0 );
}

// The scale matrix's row `i` is `[ 0, .., s_i ( at i ), .., 0 ]`; dotted with the all-ones
// vector this reduces to `s_i * 1 + 0 * 1 + ..`, i.e. multiplying by exactly 1.0 and adding
// exact zeros — both exact under IEEE-754 on every target, so the result is always
// bit-identical to the input scale component.
#[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
#[ test ]
fn test_scale()
{
  let vec = Vector( [ 1.0_f32, 1.0, 1.0, 1.0 ] );
  let scale = scale( [ 0.1, 0.2, 0.3 ] );
  let scale_vec = scale * vec;

  assert_eq!( scale_vec.x(), 0.1 );
  assert_eq!( scale_vec.y(), 0.2 );
  assert_eq!( scale_vec.z(), 0.3 );
}

// test_kind: bug_reproducer(BUG-445)
/// ## Root Cause
/// `look_to_rh(eye, dir, up)` derives its `x` axis as `normalized(cross(z, up))` where
/// `z = normalized(dir)`, with no guard for `dir`/`up` being (numerically) parallel -- same
/// defect and root cause as `d2::rotation::Rotation::look_at` (see its own BUG-445 reproducer
/// in `d2_test/rotation_test.rs`). A top-down or bottom-up camera -- `dir = (0,-1,0)`,
/// `up = (0,1,0)`, the standard world-up vector -- makes `z` and `up` exactly parallel, so
/// `cross(z, up)` is the zero vector and `normalized()` divides `0.0 / 0.0`, propagating `NaN`
/// through the entire returned view matrix.
/// ## Why Not Caught
/// This file had no test at all for `look_to_rh`/`look_at_rh` before this task -- only
/// `rot`/`scale`/`translation` were covered.
/// ## Fix Applied
/// BUG-445 added a `mag(cross(z, up)) < 1e-6` guard in
/// `src/d2/mat3x3h/transformation.rs::look_to_rh`: when triggered, `x` is instead derived via
/// `non_parallel_hint(z.array_ref())` (`mdmath_core`), the same helper-axis technique shared
/// with `look_at`'s own fix.
/// ## Prevention
/// This test uses the exact top-down camera orientation from the root cause above, with
/// `eye = (0,0,0)` (so the translation terms `-dot_x`/`-dot_y`/`dot_z` are all exactly `0.0`,
/// keeping the expected matrix simple), and asserts the resulting view matrix bit-exactly
/// equals the deterministic basis `non_parallel_hint` produces for this input. Pre-fix, `NaN`
/// components make the comparison fail (`NaN` is unequal to everything, including itself);
/// post-fix, every component matches exactly.
/// ## Pitfall
/// Any `normalized(cross(a, b))` basis construction needs an explicit guard for `a`/`b` being
/// (numerically) parallel -- the zero cross product itself does not panic or produce an early
/// `NaN`, so the defect only surfaces once the degenerate basis is actually used, far from the
/// construction site. `look_at_rh` (which subtracts `center - eye` and delegates to
/// `look_to_rh`) is covered transitively -- it has no basis-construction logic of its own to
/// test separately.
#[ test ]
fn test_look_to_rh_parallel_up_no_nan()
{
  // Top-down camera: looking straight down -Y with world-up (+Y) as the up hint -- `dir` and
  // `up` are exactly parallel, the degenerate case BUG-445 fixes. `eye` at the origin keeps
  // every translation term in the expected matrix exactly `0.0`.
  let eye = Vector( [ 0.0_f32, 0.0, 0.0 ] );
  let dir = Vector( [ 0.0_f32, -1.0, 0.0 ] );
  let up = Vector( [ 0.0_f32, 1.0, 0.0 ] );

  let view = look_to_rh( eye, dir, up );

  // Empirically confirmed (see BUG-445's own reproduction, shared with `look_at`'s identical
  // test): `non_parallel_hint( (0,-1,0) )` picks the world X axis, giving this exact basis.
  let expected = Mat4::from_row_major
  ([
    0.0, 0.0, 1.0, 0.0,
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 1.0,
  ]);
  assert_eq!( view, expected, "look_to_rh( parallel dir/up ) must fall back to the non_parallel_hint basis, not NaN" );
}


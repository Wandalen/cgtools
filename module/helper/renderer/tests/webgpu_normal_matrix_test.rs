//! Unit-level validation tests for `renderer::webgpu::normal_matrix_compute` — the singular-matrix
//! fallback its own predecessor comment claimed matched `webgl::Node`'s BUG-171 fix, but didn't
//! ( see BUG-257 ). Pure CPU-side math, no GPU device needed.
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use renderer::webgpu::normal_matrix_compute;
use mingl::math;

#[ test ]
fn accepts_a_non_uniform_scale_and_computes_its_inverse_transpose()
{
  // diag( 2, 4, 1 ) -- invertible, non-uniform, and diagonal so its inverse-transpose is
  // hand-computable: inverse is diag( 0.5, 0.25, 1 ), transpose of a diagonal matrix is itself.
  let scale = math::F32x3x3::from_column_major
  (
    [
      2.0, 0.0, 0.0,
      0.0, 4.0, 0.0,
      0.0, 0.0, 1.0
    ]
  );

  let normal = normal_matrix_compute( scale );

  let expected = [ 0.5, 0.0, 0.0, 0.0, 0.25, 0.0, 0.0, 0.0, 1.0 ];
  let got = normal.to_array();
  #[ expect( clippy::float_cmp, reason = "exact expected values from a hand-computed diagonal inverse, not an approximation" ) ]
  let matches = got == expected;
  assert!( matches, "expected the inverse-transpose of a diagonal scale to be its reciprocal diagonal -- got {got:?}" );
}

/// ## Root Cause
/// `WebGpuRenderer`'s private `model_raw` ( `src/webgpu/renderer.rs` ) computed the world-space
/// normal transform as `rotation_scale.inverse().map_or( rotation_scale, | m | m.transpose() )` --
/// on a singular `rotation_scale` ( `inverse()` returns `None` ), this fell back to the raw,
/// un-inverted, un-transposed block itself, packed directly into the uniform buffer as the "normal
/// matrix". The accompanying comment claimed this was "the same degenerate result the WebGL node
/// path would produce lighting-wise" -- but `webgl::Node::world_matrix_set` (BUG-171) falls back to
/// identity, not the raw block. Using the raw block directly scales normals by the object's own
/// (possibly axis-collapsing) scale instead of leaving them unmodified -- e.g. for a
/// `diag( 2, 0, 2 )` world scale, every transformed normal's y-component is forced to exactly `0`,
/// producing non-unit-length, direction-distorted normals fed straight into the shader's lighting
/// math, instead of BUG-171's safe, well-formed identity fallback.
/// ## Why Not Caught
/// The normal-matrix computation was inlined directly in the private `model_raw` method with zero
/// test coverage of any kind -- no test exercised a singular world matrix through the WebGPU
/// renderer, and the inline comment's incorrect "same as WebGL" claim was never checked against
/// `webgl::Node`'s actual fallback value.
/// ## Fix Applied
/// The normal-matrix computation was extracted into its own `pub fn normal_matrix_compute`, whose
/// singular fallback now matches `webgl::Node`'s BUG-171 fix exactly: identity via
/// `gl::math::mat3x3::identity`, not the raw `rotation_scale` block.
/// ## Prevention
/// A comment asserting behavioral parity between two sibling implementations ( here, the `webgl`
/// and `webgpu` backends' normal-matrix computations ) is not itself proof of parity -- the sibling
/// path must be re-read and compared directly against the claim, not trusted at face value.
/// ## Pitfall
/// It's easy to assume a backend-parallel implementation inherited an earlier bug fix (here,
/// BUG-171) just because the code "looks similar" or a comment says so -- a fix applied to one
/// backend does not propagate to its sibling automatically, and the claim of parity must be
/// verified against the sibling's actual current source, not assumed.
#[ test ]
fn rejects_a_singular_matrix_by_falling_back_to_identity_not_the_raw_block()
{
  // An all-zero 3x3 block is singular ( determinant 0 ) -- inverse() must return None, exercising
  // the fallback branch.
  let singular = math::F32x3x3::_fill( 0.0 );

  let normal = normal_matrix_compute( singular );

  let identity = math::mat3x3::identity::< f32 >();
  #[ expect( clippy::float_cmp, reason = "identity is an exact, hand-known constant, not an approximation" ) ]
  let is_identity = normal.to_array() == identity.to_array();
  assert!( is_identity, "a singular rotation-scale block must fall back to identity ( matching webgl::Node's BUG-171 fix ), not the raw un-inverted block -- got {:?}", normal.to_array() );
}

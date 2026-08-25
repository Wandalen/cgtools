use super::*;

fn test_look_at_identity_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat3< f32, D > : Default + std::cmp::PartialEq + std::fmt::Debug,
  the_module::Mat3< f32, D > : the_module::Indexable< Index = the_module::Ix2 > + the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat3< f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat3, Rotation, Vector };

  // Looking down -Z with +Y up is the default OpenGL camera basis: the resulting rotation
  // should be the identity (x = +X, y = +Y, row -z = +Z).
  let dir = Vector::< f32, 3 >::from_array( [ 0.0, 0.0, -1.0 ] );
  let up = Vector::< f32, 3 >::from_array( [ 0.0, 1.0, 0.0 ] );

  let rotation = Mat3::< f32, D >::look_at( &dir, &up );
  let identity = Mat3::< f32, D >::from_row_major
  ([
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);
  assert_eq!( rotation, identity, "look_at( -Z, +Y ) must be the identity rotation" );
}

#[ test ]
fn test_look_at_identity_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_look_at_identity_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_look_at_identity_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_look_at_identity_generic::< DescriptorOrderColumnMajor >();
}

// test_kind: bug_reproducer(BUG-445)
/// ## Root Cause
/// `look_at(dir, up)` derives its `x` axis as `normalized(cross(z, up))` where `z =
/// normalized(dir)`, with no guard for `dir`/`up` being (numerically) parallel. A top-down or
/// bottom-up camera -- `dir = (0,-1,0)`, `up = (0,1,0)`, the standard world-up vector -- makes
/// `z` and `up` exactly parallel, so `cross(z, up)` is the zero vector and `normalized()`
/// divides `0.0 / 0.0`, propagating `NaN` into `x`, then into `y = cross(x, z)`, then into
/// every element of the returned matrix.
/// ## Why Not Caught
/// The pre-existing `test_look_at_identity_generic` above only exercises `dir = (0,0,-1)`,
/// `up = (0,1,0)` -- perpendicular, not parallel -- so `cross(z, up)` is always well away from
/// zero there. No test constructed a `dir`/`up` pair where `up` is parallel to `dir`, which is
/// exactly the degenerate top-down/bottom-up camera orientation real callers hit.
/// ## Fix Applied
/// BUG-445 added a `mag(cross(z, up)) < 1e-6` guard in `src/d2/rotation.rs::look_at`: when
/// triggered, `x` is instead derived via `non_parallel_hint(z)` (`mdmath_core`), a helper axis
/// guaranteed not to be parallel to `z`, cross-producted with `z` in its place.
/// ## Prevention
/// This test uses the exact top-down camera orientation from the root cause above and asserts
/// the resulting matrix bit-exactly equals the specific orthonormal basis `non_parallel_hint`
/// deterministically produces for this input: `x = (0,0,1)`, `y = (1,0,0)`, `-z = (0,1,0)`. A
/// single `assert_eq!` against that expected matrix is sufficient to prove the fix: pre-fix,
/// `NaN` components make `rotation` unequal to `expected` (`NaN` is unequal to everything,
/// including itself); post-fix, every component matches exactly.
/// ## Pitfall
/// Any `normalized(cross(a, b))` basis construction needs an explicit guard for `a`/`b` being
/// (numerically) parallel -- the zero cross product itself does not panic or produce an early
/// `NaN`, so the defect only surfaces once the degenerate basis is actually used, far from the
/// construction site. Always test the *parallel* `dir`/`up` case explicitly, not just a
/// perpendicular default.
fn test_look_at_parallel_up_no_nan_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat3< f32, D > : Default + std::cmp::PartialEq + std::fmt::Debug,
  the_module::Mat3< f32, D > : the_module::Indexable< Index = the_module::Ix2 > + the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat3< f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat3, Rotation, Vector };

  // Top-down camera: looking straight down -Y with world-up (+Y) as the up hint -- `dir` and
  // `up` are exactly parallel, the degenerate case BUG-445 fixes.
  let dir = Vector::< f32, 3 >::from_array( [ 0.0, -1.0, 0.0 ] );
  let up = Vector::< f32, 3 >::from_array( [ 0.0, 1.0, 0.0 ] );

  let rotation = Mat3::< f32, D >::look_at( &dir, &up );

  // Empirically confirmed (see BUG-445's own reproduction): `non_parallel_hint( (0,-1,0) )`
  // picks the world X axis, giving this exact, fully deterministic orthonormal basis. This
  // single comparison is itself sufficient to prove the fix: pre-fix, `rotation` contains
  // `NaN` (which is unequal to everything, including itself), so this assertion fails; post-fix
  // it is bit-exact to `expected`.
  let expected = Mat3::< f32, D >::from_row_major
  ([
    0.0, 0.0, 1.0,
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
  ]);
  assert_eq!( rotation, expected, "look_at( parallel dir/up ) must fall back to the non_parallel_hint basis, not NaN" );
}

#[ test ]
fn test_look_at_parallel_up_no_nan_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_look_at_parallel_up_no_nan_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_look_at_parallel_up_no_nan_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_look_at_parallel_up_no_nan_generic::< DescriptorOrderColumnMajor >();
}

fn test_between_vectors_self_alignment_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat3< f32, D > : Default + std::cmp::PartialEq + std::fmt::Debug,
  the_module::Mat3< f32, D > : the_module::Indexable< Index = the_module::Ix2 > + the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat3< f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat3, Rotation, Vector };

  // Aligning a vector with itself is a null rotation.
  let a = Vector::< f32, 3 >::from_array( [ 1.0, 0.0, 0.0 ] );
  let rotation = Mat3::< f32, D >::between_vectors( &a, &a );
  let identity = Mat3::< f32, D >::from_row_major
  ([
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);
  assert_eq!( rotation, identity, "between_vectors( a, a ) must be the identity rotation" );
}

#[ test ]
fn test_between_vectors_self_alignment_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_between_vectors_self_alignment_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_between_vectors_self_alignment_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_between_vectors_self_alignment_generic::< DescriptorOrderColumnMajor >();
}

fn test_between_vectors_and_vector_rotate_agree_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat3< f32, D > : Default + std::cmp::PartialEq + std::fmt::Debug,
  the_module::Mat3< f32, D > : the_module::Indexable< Index = the_module::Ix2 > + the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat3< f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat3, Rotation, Vector };

  let a = Vector::< f32, 3 >::from_array( [ 1.0, 0.0, 0.0 ] );
  let b = Vector::< f32, 3 >::from_array( [ 0.0, 1.0, 0.0 ] );

  let rotation = Mat3::< f32, D >::between_vectors( &a, &b );

  // `vector_rotate` applied to `a` must land on `b` -- the in-place rotation must agree with
  // the rotation that was constructed to produce exactly that alignment.
  let mut rotated = a;
  rotation.vector_rotate( &mut rotated );
  assert_eq!( rotated, b, "rotating `a` by between_vectors( a, b ) must yield `b`" );

  // The inverse rotation must undo it.
  let inverse = rotation.invert();
  let mut restored = rotated;
  inverse.vector_rotate( &mut restored );
  assert_eq!( restored, a, "rotating by the inverse must restore the original vector" );
}

#[ test ]
fn test_between_vectors_and_vector_rotate_agree_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_between_vectors_and_vector_rotate_agree_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_between_vectors_and_vector_rotate_agree_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_between_vectors_and_vector_rotate_agree_generic::< DescriptorOrderColumnMajor >();
}

fn test_inplace_look_at_matches_allocating_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat3< f32, D > : Default + std::cmp::PartialEq + std::fmt::Debug,
  the_module::Mat3< f32, D > : the_module::Indexable< Index = the_module::Ix2 > + the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat3< f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat3, Rotation, Vector };

  let dir = Vector::< f32, 3 >::from_array( [ 0.0, 0.0, -1.0 ] );
  let up = Vector::< f32, 3 >::from_array( [ 0.0, 1.0, 0.0 ] );

  let allocating = Mat3::< f32, D >::look_at( &dir, &up );

  let mut dst = Mat3::< f32, D >::default();
  the_module::inplace_look_at( &mut dst, &dir, &up );

  assert_eq!( dst, allocating, "inplace_look_at must agree with the allocating look_at" );
}

#[ test ]
fn test_inplace_look_at_matches_allocating_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_inplace_look_at_matches_allocating_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_inplace_look_at_matches_allocating_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_inplace_look_at_matches_allocating_generic::< DescriptorOrderColumnMajor >();
}

fn test_inplace_between_vectors_matches_allocating_generic< D : the_module::mat::Descriptor >()
where
  the_module::Mat3< f32, D > : Default + std::cmp::PartialEq + std::fmt::Debug,
  the_module::Mat3< f32, D > : the_module::Indexable< Index = the_module::Ix2 > + the_module::ScalarMut< Scalar = f32 >,
  the_module::Mat3< f32, D > : the_module::RawSliceMut< Scalar = f32 >,
{
  use the_module::{ Mat3, Rotation, Vector };

  let a = Vector::< f32, 3 >::from_array( [ 1.0, 0.0, 0.0 ] );
  let b = Vector::< f32, 3 >::from_array( [ 0.0, 1.0, 0.0 ] );

  let allocating = Mat3::< f32, D >::between_vectors( &a, &b );

  let mut dst = Mat3::< f32, D >::default();
  the_module::inplace_between_vectors( &mut dst, &a, &b );

  assert_eq!( dst, allocating, "inplace_between_vectors must agree with the allocating between_vectors" );
}

#[ test ]
fn test_inplace_between_vectors_matches_allocating_row_major()
{
  use the_module::mat::DescriptorOrderRowMajor;
  test_inplace_between_vectors_matches_allocating_generic::< DescriptorOrderRowMajor >();
}

#[ test ]
fn test_inplace_between_vectors_matches_allocating_column_major()
{
  use the_module::mat::DescriptorOrderColumnMajor;
  test_inplace_between_vectors_matches_allocating_generic::< DescriptorOrderColumnMajor >();
}

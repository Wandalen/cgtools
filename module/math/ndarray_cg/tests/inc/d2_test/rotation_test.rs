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

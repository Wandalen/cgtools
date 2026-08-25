use super::*;

use ndarray_cg::{ IndexingRef, QuatF64, approx };
use approx::assert_abs_diff_eq;
use the_module::
{
  Ix2,
  RawSliceMut,
  ScalarMut,
  RawSlice,
  ConstLayout,
  IndexingMut,
  Mat3,
  Mat4,
  mat
};

// `determinant` on these small-integer-valued matrices only sums/subtracts products of
// exactly-representable integers — no rounding is possible, so exact equality is correct.
#[ expect( clippy::float_cmp, reason = "assertions check exact expected values; no arithmetic drift is possible and epsilon comparison would weaken them" ) ]
fn test_determinant_generic< Descriptor : mat::Descriptor >()
where
  Mat4< f32, Descriptor > :
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > +
      ConstLayout< Index = Ix2 > +
      IndexingMut< Scalar = f32, Index = Ix2 >,
  Mat3< f32, Descriptor > :
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > +
      ConstLayout< Index = Ix2 > +
      IndexingMut< Scalar = f32, Index = Ix2 >
{
  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([
    1.0, 2.0, 3.0, 4.0,
    5.0, 6.0, 7.0, 8.0,
    9.0, 10.0, 11.0, 12.0,
    13.0, 14.0, 15.0, 16.0
  ]);

  let exp = 0.0;
  let got = mat.determinant();
  assert_eq!( got, exp );

  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]);
  let exp = 1.0;
  let got = mat.determinant();
  assert_eq!( got, exp );
}

#[ test ]
fn test_determinant_row_major()
{
  test_determinant_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_determinant_column_major()
{
  test_determinant_generic::< mat::DescriptorOrderColumnMajor >();
}

fn test_inverse_generic< Descriptor : mat::Descriptor >()
where
  Mat4< f32, Descriptor > :
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > +
      ConstLayout< Index = Ix2 > +
      IndexingMut< Scalar = f32, Index = Ix2 > +
      PartialEq,
  Mat3< f32, Descriptor > :
      RawSliceMut< Scalar = f32 > +
      ScalarMut< Scalar = f32, Index = Ix2 > +
      ConstLayout< Index = Ix2 > +
      IndexingMut< Scalar = f32, Index = Ix2 >
{
  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([
    1.0, 2.0, 3.0, 4.0,
    5.0, 6.0, 7.0, 8.0,
    9.0, 10.0, 11.0, 12.0,
    13.0, 14.0, 15.0, 16.0
  ]);
  let got = mat.inverse();
  assert!( got.is_none() );

  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([
    1.0, 3.0, 5.0, 9.0,
    1.0, 3.0, 1.0, 7.0,
    4.0, 3.0, 9.0, 7.0,
    5.0, 2.0, 0.0, 9.0
  ]);

  let exp = Mat4::< f32, Descriptor >::from_row_major
  ([
    -13.0 / 47.0,  2.0 / 47.0,   7.0 / 47.0,    6.0 / 47.0,
    -5.0 / 8.0,    7.0 / 8.0,    1.0 / 4.0,    -1.0 / 4.0,
    39.0 / 376.0, -53.0 / 376.0, 13.0 / 188.0, -9.0 / 188.0,
    55.0 / 188.0, -41.0 / 188.0, -13.0 / 94.0,  9.0 / 94.0
  ]);

  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]);
  let got = mat.inverse().unwrap();
  assert_eq!( got, mat );
}

#[ test ]
fn test_inverse_row_major()
{
  test_inverse_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_inverse_column_major()
{
  test_inverse_generic::< mat::DescriptorOrderColumnMajor >();
}

fn test_truncate_generic< Descriptor : mat::Descriptor >()
where
  Mat4< f32, Descriptor > :
      RawSliceMut< Scalar = f32 >,
  Mat3< f32, Descriptor > :
      RawSliceMut< Scalar = f32 > +
      IndexingRef< Scalar = f32, Index = Ix2 > +
      PartialEq
{
  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([
    1.0, 2.0, 3.0, 4.0,
    5.0, 6.0, 7.0, 8.0,
    9.0, 10.0, 11.0, 12.0,
    13.0, 14.0, 15.0, 16.0
  ]);

  let exp = Mat3::< f32, Descriptor >::from_row_major
  ([
    1.0, 2.0, 3.0,
    5.0, 6.0, 7.0,
    9.0, 10.0, 11.0,
  ]);

  let got = mat.truncate();
  assert_eq!( got, exp );

  let mat = Mat4::< f32, Descriptor >::from_row_major
  ([
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]);

  let exp = Mat3::< f32, Descriptor >::from_row_major
  ([
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);

  let got = mat.truncate();
  assert_eq!( got, exp );
}

#[ test ]
fn test_truncate_row_major()
{
  test_truncate_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_truncate_column_major()
{
  test_truncate_generic::< mat::DescriptorOrderColumnMajor >();
}


fn test_from_scale_rotation_translation_generic< Descriptor >()
where
  Descriptor : mat::Descriptor + PartialEq,
  Mat4< f64, Descriptor > :
      ScalarMut< Scalar = f64 > +
      RawSliceMut< Scalar = f64 > +
      IndexingMut< Scalar = f64, Index = Ix2 >
{
  let s = [ 1.0, 2.0, 3.0 ];
  let r = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ).normalize();
  let t = [ 0.0, 0.0, 0.0 ];

  let got = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let exp = Mat4::< f64, Descriptor >::from_column_major
  ([
    1.0, 0.0, 0.0, 0.0,
    0.0, 2.0, 0.0, 0.0,
    0.0, 0.0, 3.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]);

  assert_abs_diff_eq!( got, exp );

  let s = [ 1.0, 1.0, 1.0 ];
  let r = QuatF64::from( [ -5.0, 4.0, 1.0, 10.0 ] ).normalize();
  let t = [ 0.0, 0.0, 0.0 ];

  let got = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let exp = Mat4::< f64, Descriptor >::from_column_major
  ([
    0.760_563_380_281_690_1, -0.140_845_070_422_535_22, -0.633_802_816_901_408_5, 0.0,
    -0.422_535_211_267_605_6, 0.633_802_816_901_408_5, -0.647_887_323_943_661_9, 0.0,
    0.492_957_746_478_873_25, 0.760_563_380_281_690_1, 0.422_535_211_267_605_6, 0.0,
    0.0, 0.0, 0.0, 1.0
  ]);

  assert_abs_diff_eq!( got, exp );

  let s = [ 1.0, 1.0, 1.0 ];
  let r = QuatF64::from( [ 0.0, 0.0, 0.0, 1.0 ] ).normalize();
  let t = [ 1.0, -10.0, 30.0 ];

  let got = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let exp = Mat4::< f64, Descriptor >::from_column_major
  ([
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    1.0, -10.0, 30.0, 1.0
  ]);

  assert_abs_diff_eq!( got, exp );

  let s = [ 1.0, 2.0, 3.0 ];
  let r = QuatF64::from( [ -5.0, 4.0, 1.0, 10.0 ] ).normalize();
  let t = [ 1.0, -10.0, 30.0 ];

  let got = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let exp = Mat4::< f64, Descriptor >::from_column_major
  ([
    0.760_563_380_281_690_1, -0.140_845_070_422_535_22, -0.633_802_816_901_408_5, 0.0,
    -0.845_070_422_535_211_3, 1.267_605_633_802_816_8, -1.295_774_647_887_324_2,
    0.0, 1.478_873_239_436_619_7, 2.281_690_140_845_071, 1.267_605_633_802_816_5, 0.0,
    1.0, -10.0, 30.0, 1.0
  ]);

  assert_abs_diff_eq!( got, exp );
}

#[ test ]
fn test_from_scale_rotation_translation_row_major()
{
  test_from_scale_rotation_translation_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_from_scale_rotation_translation_column_major()
{
  test_from_scale_rotation_translation_generic::< mat::DescriptorOrderColumnMajor >();
}

/// ## Root Cause
/// `decompose()` divided by `inv_scale` (already a reciprocal) instead of multiplying,
/// re-squaring scale into the rotation matrix passed to `Quat::from` (BUG-250); separately,
/// `Quat::from(Mat3)` wrote its trace-derived `w` term into the `x` slot, cyclically
/// shifting all four components (BUG-119).
///
/// ## Why Not Caught
/// No test called `.decompose()` at all before this task — `test_from_scale_rotation_
/// translation_generic` only exercises the forward (build) direction, never the round trip.
///
/// ## Fix Applied
/// BUG-250 changed `decompose()`'s `rot_mat` column construction from `/ inv_scale` to `*
/// inv_scale`. BUG-119 reordered `Quat::from(Mat3)`'s final array literal to match the
/// crate's `[x,y,z,w]` storage convention. This test round-trips a matrix built with
/// deliberately non-uniform scale through `from_scale_rotation_translation` then
/// `.decompose()`, asserting the recovered scale/rotation/translation match the originals.
///
/// ## Prevention
/// A uniform or identity scale would not have exposed either bug (see each bug's own `##
/// Why Not Caught`) — this fixture uses `(2.0, 3.0, 0.5)` specifically to distinguish them.
///
/// ## Pitfall
/// A "build" test alone does not verify its own inverse ("decompose") operation — round-trip
/// coverage is required whenever both directions of a conversion exist. A round trip through
/// `decompose()` chains sqrt/reciprocal/quaternion-conversion arithmetic (unlike the single-pass
/// `test_from_scale_rotation_translation_generic` above, which compares against hand-computed
/// literals), so it accumulates a few ULP of rounding — comparing at the default epsilon
/// (`f64::EPSILON`) is too tight and fails on noise, not a real defect; `epsilon = 1e-9` is
/// loose enough to absorb that noise while remaining far tighter than any real bug's error.
fn test_decompose_recovers_scale_rotation_translation_generic< Descriptor : mat::Descriptor >()
where
  Mat4< f64, Descriptor > :
      RawSlice< Scalar = f64 > +
      RawSliceMut< Scalar = f64 > +
      ScalarMut< Scalar = f64, Index = Ix2 > +
      ConstLayout< Index = Ix2 > +
      IndexingMut< Scalar = f64, Index = Ix2 >
{
  let s = [ 2.0, 3.0, 0.5 ];
  let r = QuatF64::from( [ -5.0, 4.0, 1.0, 10.0 ] ).normalize();
  let t = [ 1.0, -10.0, 30.0 ];

  let mat = Mat4::< f64, Descriptor >::from_scale_rotation_translation( s, r, t );
  let ( got_t, got_r, got_s ) = mat.decompose().expect( "decompose should succeed for a valid TRS matrix" );

  assert_abs_diff_eq!( got_s, the_module::Vector::< f64, 3 >::from_array( s ), epsilon = 1e-9 );
  assert_abs_diff_eq!( got_r, r, epsilon = 1e-9 );
  assert_abs_diff_eq!( got_t, the_module::Vector::< f64, 3 >::from_array( t ) );
}

#[ test ]
fn test_decompose_recovers_scale_rotation_translation_row_major()
{
  test_decompose_recovers_scale_rotation_translation_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_decompose_recovers_scale_rotation_translation_column_major()
{
  test_decompose_recovers_scale_rotation_translation_generic::< mat::DescriptorOrderColumnMajor >();
}

fn test_identity_generic< Descriptor : mat::Descriptor >()
where 
  Mat4< f32, Descriptor > : 
    RawSlice< Scalar = f32 > +
    RawSliceMut< Scalar = f32 >
{
  let exp = &
  [ 
    1.0, 0.0, 0.0, 0.0, 
    0.0, 1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0, 
    0.0, 0.0, 0.0, 1.0, 
  ];

  let mat = the_module::mat4x4::identity::< f32 >();
  assert_eq!( mat.raw_slice(), exp );

  let mat = Mat4::< f32, Descriptor >::identity();
  assert_eq!( mat.raw_slice(), exp );
}

#[ test ]
fn test_identity_row_major()
{
  test_identity_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_identity_column_major()
{
  test_identity_generic::< mat::DescriptorOrderColumnMajor >();
}

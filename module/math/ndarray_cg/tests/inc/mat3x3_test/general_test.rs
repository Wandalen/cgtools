use super::*;
use ndarray_cg::{IndexingRef, QuatF64,approx };
use the_module::
{
  Ix2,
  RawSliceMut,
  ScalarMut,
  RawSlice,
  ConstLayout,
  IndexingMut,
  Mat3,
  Mat2,
  Mat4,
  mat
};

// `determinant` on these small-integer-valued matrices only sums/subtracts products of
// exactly-representable integers — no rounding is possible, so exact equality is correct.
#[ allow( clippy::float_cmp ) ]
fn test_determinant_generic< Descriptor : mat::Descriptor > ()
where
  Mat3< f32, Descriptor > :
  RawSliceMut< Scalar = f32 > +
  ScalarMut< Scalar = f32, Index = Ix2 > +
  ConstLayout< Index = Ix2 > +
  IndexingMut< Scalar = f32, Index = Ix2 >
{
  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let exp = 0.0;
  let got = mat.determinant();
  assert_eq!( got, exp );

  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
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
  Mat3< f32, Descriptor > :
  RawSliceMut< Scalar = f32 > +
  ScalarMut< Scalar = f32, Index = Ix2 > +
  ConstLayout< Index = Ix2 > +
  IndexingMut< Scalar = f32, Index = Ix2 > +
  PartialEq
{
  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let got = mat.inverse();
  assert!( got.is_none() );

  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, -1.0, 2.0, 4.0, 0.0, 6.0, 0.0, 1.0, -1.0 ] );
  let exp = Mat3::< f32, Descriptor >::from_row_major( [ 3.0, -0.5, 3.0, -2.0, 0.5, -1.0, -2.0, 0.5, -2.0 ] );
  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );

  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
  let exp = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ] );
  let got = mat.inverse().unwrap();
  assert_eq!( got, exp );
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
  Mat3< f32, Descriptor > :
    RawSliceMut< Scalar = f32 > +
    ConstLayout< Index = Ix2 > +
    IndexingMut< Scalar = f32, Index = Ix2 >,
  Mat2< f32, Descriptor > :
    RawSliceMut< Scalar = f32 > +
    IndexingRef< Scalar = f32 > +
    PartialEq
{
  let mat = Mat3::< f32, Descriptor >::from_row_major
  ([
    1.0, 2.0, 3.0,
    4.0, 5.0, 6.0,
    7.0, 8.0, 9.0,
  ]);

  let exp = Mat2::< f32, Descriptor >::from_row_major
  ([
    1.0, 2.0,
    4.0, 5.0,
  ]);

  let got = mat.truncate();
  assert_eq!( got, exp );

  let mat = Mat3::< f32, Descriptor >::from_row_major
  ([
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ]);

  let exp = Mat2::< f32, Descriptor >::from_row_major
  ([
    1.0, 0.0,
    0.0, 1.0,
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

fn test_from_quat_generic< Descriptor >()
where
  Descriptor : mat::Descriptor + PartialEq,
  Mat3< f64, Descriptor > :
      RawSliceMut< Scalar = f64 > +
      IndexingRef< Scalar = f64, Index = Ix2 > +
      PartialEq,
{
  let q = QuatF64::from( [ 1.0, 2.0, 3.0, 4.0 ] ).normalize();

  let exp = Mat3::< f64, Descriptor >::from_column_major
  ([
    0.133_333_333_333_333_53, 0.933_333_333_333_333_2, -0.333_333_333_333_333_26,
    -0.666_666_666_666_666_6, 0.333_333_333_333_333_5, 0.666_666_666_666_666_5,
    0.733_333_333_333_333_2, 0.133_333_333_333_333_36, 0.666_666_666_666_666_7,
  ]);

  assert_eq!( Mat3::< f64, Descriptor >::from_quat( q ), exp, " Mat3 from Quat mismatch" );

  let q = QuatF64::from( [ -5.0, 6.0, 1.0, 3.0 ] ).normalize();

  let exp = Mat3::< f64, Descriptor >::from_column_major
  ([
    -0.042_253_521_126_760_285, -0.760_563_380_281_69, -0.647_887_323_943_661_8,
    -0.929_577_464_788_732_3, 0.267_605_633_802_817, -0.253_521_126_760_563_4,
    0.366_197_183_098_591_45, 0.591_549_295_774_647_8, -0.718_309_859_154_929_3,
  ]);

  assert_eq!( Mat3::< f64, Descriptor >::from_quat( q ), exp, " Mat3 from Quat mismatch" );

   let q = QuatF64::from( [ -5.0, 4.0, 1.0, 10.0 ] ).normalize();

  let exp = Mat3::< f64, Descriptor >::from_column_major
  ([
    0.760_563_380_281_690_1, -0.140_845_070_422_535_22, -0.633_802_816_901_408_5,
    -0.422_535_211_267_605_6, 0.633_802_816_901_408_5, -0.647_887_323_943_661_9,
    0.492_957_746_478_873_25, 0.760_563_380_281_690_1, 0.422_535_211_267_605_6,
  ]);
  approx::assert_abs_diff_eq!( Mat3::< f64, Descriptor >::from_quat( q ), exp );
}

#[ test ]
fn test_from_quat_row_major()
{
  test_from_quat_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_from_quat_column_major()
{
  test_from_quat_generic::< mat::DescriptorOrderColumnMajor >();
}

fn test_identity_generic< Descriptor : mat::Descriptor >()
where 
  Mat3< f32, Descriptor > : 
    RawSlice< Scalar = f32 > +
    RawSliceMut< Scalar = f32 >
{
  let exp = &[ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0 ];

  let mat = the_module::mat3x3::identity::< f32 >();
  assert_eq!( mat.raw_slice(), exp );

  let mat = Mat3::< f32, Descriptor >::identity();
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

// `to_homogenous` only copies existing elements and inserts exact 0.0/1.0 padding — no
// arithmetic, so the result is bit-identical to the literal arrays compared against.
#[ allow( clippy::float_cmp ) ]
fn test_to_homogenous_generic< Descriptor : mat::Descriptor >()
where
  Mat4< f32, Descriptor > :
    RawSliceMut< Scalar = f32 >,
  Mat3< f32, Descriptor > :
    RawSlice< Scalar = f32 > +
    RawSliceMut< Scalar = f32 >
{
  let exp = 
  [ 
    1.0, 0.0, 0.0, 0.0, 
    0.0, 1.0, 0.0, 0.0, 
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0 
  ];
  let mat = Mat3::< f32, Descriptor >::from_row_major
  ( 
    [ 
      1.0, 0.0, 0.0, 
      0.0, 1.0, 0.0,
      0.0, 0.0, 1.0 
    ]
  );
  let mat = mat.to_homogenous();
  assert_eq!( mat.to_array(), exp );

  let exp = Mat4::< f32, Descriptor >::from_row_major
  ( 
    [ 
      1.0, 2.0, 3.0, 0.0,
      4.0, 5.0, 6.0, 0.0,
      7.0, 8.0, 9.0, 0.0,
      0.0, 0.0, 0.0, 1.0 
    ] 
  );
  let mat = Mat3::< f32, Descriptor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0 ] );
  let mat = mat.to_homogenous();
  assert_eq!( mat.to_array(), exp.to_array() );
}

#[ test ]
fn test_to_homogenous_row_major()
{
  test_to_homogenous_generic::< mat::DescriptorOrderRowMajor >();
}

#[ test ]
fn test_to_homogenous_column_major()
{
  test_to_homogenous_generic::< mat::DescriptorOrderColumnMajor >();
}

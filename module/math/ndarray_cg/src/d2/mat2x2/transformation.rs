use crate::*;

/// Produces a 2D rotation matrix for a given angle in radians.
///
/// # Parameters
/// - `angle_radians`: The angle of rotation in radians.
///
/// # Returns
/// - `Mat<E, 2, 2>`: A 2x2 rotation matrix.
#[ inline ]
pub fn rot< E >( angle_radians : E ) -> Mat2< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat2< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();
  Mat2::from_row_major
  ([
    cos_theta, -sin_theta,
    sin_theta, cos_theta,
  ])
}

/// Produces a 2D scaling matrix.
///
/// # Parameters
/// - `scaling`: A vector representing scaling factors along the x and y axes.
///
/// # Returns
/// - `Mat<E, 2, 2>`: A 2x2 scaling matrix.
#[ inline ]
pub fn scale< E, Scaling >( scaling : Scaling ) -> Mat2< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Scaling : VectorIter< E, 2 >,
  Mat2< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  let mut iter = scaling.vector_iter();
  let sx = *iter.next().unwrap();
  let sy = *iter.next().unwrap();
  Mat2::from_row_major
  ([
    sx,        E::zero(),
    E::zero(), sy,
  ])
}

/// Produces a 2D shearing matrix.
///
/// # Parameters
/// - `shearing`: A vector representing shearing factors along the x and y axes.
///
/// # Returns
/// - `Mat<E, 2, 2>`: A 2x2 shearing matrix.
#[ inline ]
pub fn shear< E, Shearing >( shearing : Shearing ) -> Mat2< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Shearing : VectorIter< E, 2 >,
  Mat2< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  let mut iter = shearing.vector_iter();
  let shx = *iter.next().unwrap();
  let shy = *iter.next().unwrap();
  Mat2::from_row_major
  ([
    E::one(), shx,
    shy,      E::one(),
  ])
}

/// Produces a 2D reflection matrix across the x-axis.
///
/// # Returns
/// - `Mat<E, 2, 2>`: A 2x2 reflection matrix.
#[ inline ]
pub fn reflect_x< E >() -> Mat2< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat2< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  Mat2::from_row_major
  ([
    E::one(),   E::zero(),
    E::zero(), -E::one(),
  ])
}

/// Produces a 2D reflection matrix across the y-axis.
///
/// # Returns
/// - `Mat<E, 2, 2>`: A 2x2 reflection matrix.
#[ inline ]
pub fn reflect_y< E >() -> Mat2< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat2< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  Mat2::from_row_major
  ([
    -E::one(), E::zero(),
    E::zero(), E::one(),
  ])
}


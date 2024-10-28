use crate::*;

/// Produces a 2D rotation matrix for a given angle in radians.
///
/// # Parameters
/// - `angle_radians`: The angle of rotation in radians.
///
/// # Returns
/// - A 3x3 rotation matrix.
#[ inline ]
pub fn rot< E >( angle_radians : E ) -> Mat3< E >
where
  E : MatEl + nd::NdFloat,
  Mat3< E > :  RawSliceMut< Scalar = E >
{
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();
  Mat3::from_row_major
  ([
    cos_theta, -sin_theta, E::zero(),
    sin_theta, cos_theta, E::zero(),
    E::zero(), E::zero(), E::one(),
  ])
}

/// Produces a 2D translation matrix.
///
/// # Parameters
/// - `translation`: A vector representing translation along the x and y axes.
///
/// # Returns
/// - A 3x3 translation matrix.
#[ inline ]
pub fn translate< E, Translation >( translation : Translation ) -> Mat3< E >
where
  E : MatEl + nd::NdFloat,
  Translation : VectorIter< E, 2 >,
  Mat3< E > :  RawSliceMut< Scalar = E >
{
  let mut iter = translation.vector_iter();
  let tx = *iter.next().unwrap();
  let ty = *iter.next().unwrap();
  Mat3::from_row_major
  ([
    E::one(),  E::zero(), tx,
    E::zero(), E::one(),  ty,
    E::zero(), E::zero(), E::one(),
  ])
}

/// Produces a 2D scaling matrix.
///
/// # Parameters
/// - `scaling`: A vector representing scaling factors along the x and y axes.
///
/// # Returns
/// - A 3x3 scaling matrix.
#[ inline ]
pub fn scale< E, Scaling >( scaling : Scaling ) -> Mat3< E >
where
  E : MatEl + nd::NdFloat,
  Scaling : VectorIter< E, 2 > + Collection< Scalar = E >,
  Mat3< E > :  RawSliceMut< Scalar = E >
{
  let mut iter = scaling.vector_iter();
  let sx = *iter.next().unwrap();
  let sy = *iter.next().unwrap();
  Mat3::from_row_major
  ([
    sx,        E::zero(), E::zero(),
    E::zero(), sy,        E::zero(),
    E::zero(), E::zero(), E::one(),
  ])
}

/// Produces a 2D shearing matrix.
///
/// # Parameters
/// - `shearing`: A vector representing shearing factors along the x and y axes.
///
/// # Returns
/// - A 3x3 shearing matrix.
#[ inline ]
pub fn shear< E, Shearing >( shearing : Shearing ) -> Mat3< E >
where
  E : MatEl + nd::NdFloat,
  Shearing : VectorIter< E, 2 >,
  Mat3< E > :  RawSliceMut< Scalar = E >
{
  let mut iter = shearing.vector_iter();
  let shx = *iter.next().unwrap();
  let shy = *iter.next().unwrap();
  Mat3::from_row_major
  ([
    E::one(),  shx,       E::zero(),
    shy,       E::one(),  E::zero(),
    E::zero(), E::zero(), E::one(),
  ])
}

/// Produces a 2D reflection matrix across the x-axis.
///
/// # Returns
/// - A 3x3 reflection matrix.
#[ inline ]
pub fn reflect_x< E >() -> Mat3< E >
where
  E : MatEl + nd::NdFloat,
  Mat3< E > :  RawSliceMut< Scalar = E >
{
  Mat3::from_row_major
  ([
    E::one(),  E::zero(), E::zero(),
    E::zero(), -E::one(), E::zero(),
    E::zero(), E::zero(), E::one(),
  ])
}

/// Produces a 2D reflection matrix across the y-axis.
///
/// # Returns
/// - A 3x3 reflection matrix.
#[ inline ]
pub fn reflect_y< E >() -> Mat3< E >
where
  E : MatEl + nd::NdFloat,
  Mat3< E > :  RawSliceMut< Scalar = E >
{
  Mat3::from_row_major
  ([
    -E::one(), E::zero(), E::zero(),
    E::zero(), E::one(), E::zero(),
    E::zero(), E::zero(), E::one(),
  ])
}

/// Produces a 2D rotation matrix around a specified point.
///
/// # Parameters
/// - `angle_radians`: The angle of rotation in radians.
/// - `p`: A vector representing the x and y coordinates of the point.
///
/// # Returns
/// - A 3x3 rotation matrix.
#[ inline ]
pub fn rot_around_point< E, Point >( angle_radians : E, p : Point ) -> Mat3< E >
where
  E : MatEl + nd::NdFloat,
  Point : VectorIter< E, 2 > + Collection< Scalar = E >,
  Mat3< E > :  RawSliceMut< Scalar = E >
{
  let mut iter = p.vector_iter();
  let px = *iter.next().unwrap();
  let py = *iter.next().unwrap();
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();
  Mat3::from_row_major
  ([
    cos_theta, -sin_theta, px - cos_theta * px + sin_theta * py,
    sin_theta, cos_theta,  py - sin_theta * px - cos_theta * py,
    E::zero(), E::zero(),  E::one(),
  ])
}

/// Produces a 2D scaling matrix relative to a specified point.
///
/// # Parameters
/// - `scaling`: A vector representing scaling factors along the x and y axes.
/// - `p`: A vector representing the x and y coordinates of the point.
///
/// # Returns
/// - A 3x3 scaling matrix.
#[ inline ]
pub fn scale_relative_to_point< E, Scaling, Point >( scaling : Scaling, p : Point ) -> Mat3< E >
where
  E : MatEl + nd::NdFloat,
  Scaling : VectorIter< E, 2 >,
  Point : VectorIter< E, 2 >,
  Mat3< E > :  RawSliceMut< Scalar = E >
{
  let mut s_iter = scaling.vector_iter();
  let sx = *s_iter.next().unwrap();
  let sy = *s_iter.next().unwrap();
  let mut p_iter = p.vector_iter();
  let px = *p_iter.next().unwrap();
  let py = *p_iter.next().unwrap();
  Mat3::from_row_major
  ([
    sx,        E::zero(), px - sx * px,
    E::zero(), sy,        py - sy * py,
    E::zero(), E::zero(), E::one(),
  ])
}

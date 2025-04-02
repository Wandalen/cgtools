use crate::*;
use mdmath_core::vector::arithmetics::*;

// #[ derive( Copy, Clone, Debug, PartialEq, Default ) ]
// pub struct Decomposed< E, Vec, Rot, const N : usize >
// where
//   Vec : VectorIter< E, N >,
// {
//   pub scale : Vec,
//   pub rot : Rot,
//   pub offset : Vec,
// }

/// Creates right-handed perspective transformation with z in range [ -1.0, 1.0 ].
/// This transformation corresponds to the transformation used in OpenGL:
/// https://registry.khronos.org/OpenGL-Refpages/gl2.1/xhtml/gluPerspective.xml
///
/// Similiar functions:
/// perspective_rh - return the same matrix, but with z in range [ 0.0, 1.0 ]
pub fn perspective_rh_gl< E >
(
  fovy : E,
  aspect : E,
  z_near : E,
  z_far : E
)
->  Mat4< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat4< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  let two = E::from( 2.0 ).unwrap();
  let f = E::one() / ( fovy / two ).tan();
  let dz = z_near - z_far;
  let sz = z_near + z_far;
  let mz = two * z_near * z_far;

  Mat4::from_row_major
  (
    [
      f / aspect, E::zero(),  E::zero(), E::zero(),
      E::zero(),  f,          E::zero(), E::zero(),
      E::zero(),  E::zero(),  sz / dz,   mz / dz,
      E::zero(),  E::zero(), -E::one(),  E::zero()
    ]
  )
}

/// Creates right-handed perspective transformation with z in range [ 0.0, 1.0 ].
/// This transformation can be used with WebGPU, for example.
///
/// Similiar functions:
/// perspective_rh_gl - return the same matrix, but with z in range [ -1.0, 1.0 ]
pub fn perspective_rh< E >
(
  fovy : E,
  aspect : E,
  z_near : E,
  z_far : E
)
->  Mat4< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat4< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  let two = E::from( 2.0 ).unwrap();
  let f = E::one() / ( fovy / two ).tan();
  let dz = z_near - z_far;
  let mz = z_near * z_far;

  Mat4::from_row_major
  (
    [
      f / aspect, E::zero(),  E::zero(),  E::zero(),
      E::zero(),  f,          E::zero(),  E::zero(),
      E::zero(),  E::zero(),  z_far / dz, mz / dz,
      E::zero(),  E::zero(), -E::one(),   E::zero()
    ]
  )
}

/// Make a right-handed view transformation from camera's position, camera's view directions,
/// and camera's "up" orientation.
/// (+)X - right, (+)Y - up, (+)Z - back
///
/// In other words, makes a transformation that first moves the eye positions to the origin,
/// and then makes rotates it so that the dir will point in -z direction.
///
/// Similiar functions:
/// look_at_rh - returns the same matrix, but takes camera's view center, instead of direction
pub fn loot_to_rh< E, Vec3 >
(
  eye : Vec3,
  dir : Vec3,
  up : Vec3
)
->  Mat4< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Vec3 : VectorIterMut< E, 3 > + ArrayRef< E, 3 > + Clone,
  Mat4< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  let z = normalized( &dir );
  let x = normalized( &cross( &z, &up ) );
  let y = cross( &x, &z );

  let x = x.array_ref();
  let y = y.array_ref();
  let z = z.array_ref();

  let dot_x = dot( &eye, x );
  let dot_y = dot( &eye, y );
  let dot_z = dot( &eye, z );

  Mat4::from_row_major
  (
    [
       x[ 0 ],    x[ 1 ],    x[ 2 ],   -dot_x,
       y[ 0 ],    y[ 1 ],    y[ 2 ],   -dot_y,
      -z[ 0 ],   -z[ 1 ],   -z[ 2 ],    dot_z,
       E::zero(), E::zero(), E::zero(), E::one()
    ]
  )
}

/// Make a right-handed view transformation from camera's position, camera's focal point,
/// and camera's "up" orientation.
/// X - (+)right, Y - (+)up, Z - (-)back
///
/// Similiar functions:
/// look_to_rh - returns the same matrix, but takes camera's view direction
pub fn loot_at_rh< E, Vec3 >
(
  eye : Vec3,
  center : Vec3,
  up : Vec3
)
->  Mat4< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Vec3 : VectorIterMut< E, 3 > + ArrayRef< E, 3 > + Clone,
  Mat4< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >,
{
  let dir = sub( &center, &eye );
  loot_to_rh( eye, dir, up )
}

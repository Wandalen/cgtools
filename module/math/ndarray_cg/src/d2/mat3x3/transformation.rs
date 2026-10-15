//! This module provides functions for creating 3x3 rotation matrices
//! from various representations, such as Euler angles (per-axis) and axis-angle.

use crate::{MatEl, nd, Mat3, mat, RawSliceMut, VectorIter};

/// Creates a 3x3 matrix for a rotation around the X-axis.
///
/// # Arguments
/// * `angle` - The rotation angle in radians.
#[ inline ]
pub fn from_angle_x< E >( angle : E ) -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  let ( s, c ) = angle.sin_cos();
  Mat3::from_row_major
  (
    [
      E::one(),  E::zero(), E::zero(),
      E::zero(), c,         -s,
      E::zero(), s,         c,
    ]
  )
}

/// Creates a 3x3 matrix for a rotation around the Y-axis.
///
/// # Arguments
/// * `angle` - The rotation angle in radians.
#[ inline ]
pub fn from_angle_y< E >( angle : E ) -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  let ( s, c ) = angle.sin_cos();
  Mat3::from_row_major
  (
    [
      c,         E::zero(), s,
      E::zero(), E::one(),  E::zero(),
      -s,        E::zero(), c
    ]
  )
}

/// Creates a 3x3 matrix for a rotation around the Z-axis.
///
/// # Arguments
/// * `angle` - The rotation angle in radians.
#[ inline ]
pub fn from_angle_z< E >( angle : E ) -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  let ( s, c ) = angle.sin_cos();
  Mat3::from_row_major
  (
    [
      c,         -s,        E::zero(),
      s,         c,         E::zero(),
      E::zero(), E::zero(), E::one()
    ]
  )
}

/// Creates a 3x3 rotation matrix from an axis and an angle.
///
/// # Arguments
/// * `axis` - The axis of rotation, which should be a normalized 3D vector.
/// * `angle` - The rotation angle in radians.
///
/// # Panics
///
/// Panics if `axis`'s iterator yields fewer than 3 elements.
// Fix(BUG-450): changed `angle`'s type from hardcoded `f32` to the function's own generic `E`.
// Root cause: every sibling constructor in this file (`from_angle_x`/`from_angle_y`/
// `from_angle_z`) correctly takes `angle : E`, but this function took `angle : f32` and routed
// it through `E::from( angle.sin() ).unwrap()` / `E::from( angle.cos() ).unwrap()` -- for
// `E = f64` callers, the angle was silently truncated to `f32` precision before `sin`/`cos` ever
// ran, discarding roughly half the caller's precision with no warning (not even a clippy lint,
// since the truncation happens at the call boundary via a normal, valid `f32` argument, not a
// lossy cast the compiler can see).
// Pitfall: a generic numeric function with one hardcoded concrete-typed parameter compiles
// fine and looks correct for the common `E = f32` case -- always check every parameter against
// the function's own generic type parameter, not just the return type and the other arguments,
// especially when sibling functions in the same file already establish the fully-generic
// pattern to follow.
#[ inline ]
pub fn from_axis_angle< E, Vec3 >( axis : Vec3, angle : E ) -> Mat3< E, mat::DescriptorOrderColumnMajor >
where
  E : MatEl + nd::NdFloat,
  Vec3 : VectorIter< E, 3 >,
  Mat3< E, mat::DescriptorOrderColumnMajor > : RawSliceMut< Scalar = E >
{
  let ( s, c ) = angle.sin_cos();
  let one_minus_c = E::one() - c ;

  let mut iter = axis.vector_iter();
  let ux = *iter.next().unwrap();
  let uy = *iter.next().unwrap();
  let uz = *iter.next().unwrap();

  let r1c1 = ux * ux * one_minus_c + c;
  let r1c2 = ux * uy * one_minus_c - uz * s;
  let r1c3 = ux * uz * one_minus_c + uy * s;

  let r2c1 = ux * uy * one_minus_c + uz * s;
  let r2c2 = uy * uy * one_minus_c + c;
  let r2c3 = uy * uz * one_minus_c - ux * s;

  let r3c1 = ux * uz * one_minus_c - uy * s;
  let r3c2 = uy * uz * one_minus_c + ux * s;
  let r3c3 = uz * uz * one_minus_c + c;
  Mat3::from_row_major
  (
    [
      r1c1, r1c2, r1c3,
      r2c1, r2c2, r2c3,
      r3c1, r3c2, r3c3
    ]
  )
}

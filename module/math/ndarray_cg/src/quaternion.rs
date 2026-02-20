//! This module defines the core data structure for quaternions, `Quat<E>`,
//! along with common type aliases. Quaternions are used to represent rotations
//! in 3D space in a compact and efficient manner, avoiding issues like gimbal lock.
mod private
{
  use crate::*;

  /// Represents a quaternion using a 4D vector for its components `[x, y, z, w]`.
  ///
  /// The `w` component is the scalar part, and `(x, y, z)` is the vector part.
  #[ derive( Clone, Copy, Debug, PartialEq, PartialOrd ) ]
  pub struct Quat< E >( pub Vector< E, 4 > )
  where E : MatEl;

  impl< E > Default for Quat< E >
  where
    E : MatEl + NdFloat
  {
    fn default() -> Self 
    {
      Quat( Vector( [ E::zero(), E::zero(), E::zero(), E::one() ] ) )
    } 
  }

  /// A type alias for a quaternion with `f32` components.
  pub type QuatF32 = Quat< f32 >;
  /// A type alias for a quaternion with `f64` components.
  pub type QuatF64 = Quat< f64 >;
}

// This macro organizes and exposes the public interface for the quaternion functionality.
crate::mod_interface!
{
  /// General quaternion operations and definitions.
  layer general;
  /// Operator overloads for quaternion arithmetic.
  layer operator;
  /// `From` trait implementations for converting to and from quaternions.
  layer from;
  /// Advanced arithmetic operations for quaternions.
  layer arithmetics;

  exposed use
  {
    Quat,
    QuatF32,
    QuatF64
  };
}

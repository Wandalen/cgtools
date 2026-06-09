//! Describe general floats and operations on them.

/// Internal namespace.
mod private
{
  /// Scalar element supporting field-agnostic arithmetic
  /// (`Add`/`Sub`/`Mul`/`Div`/`Rem`, `Zero`/`One` and their `*Assign`
  /// counterparts) without requiring float-specific operations like `sqrt` or
  /// trigonometry. Satisfied by all integer primitives and floats.
  pub trait Scalar
  where
    Self : Copy + ::num_traits::Num + ::num_traits::NumAssign,
  {
  }

  impl< T > Scalar for T
  where
    Self : Copy + ::num_traits::Num + ::num_traits::NumAssign,
  {
  }
}

crate::mod_interface!
{
  exposed use ::num_traits::Float;
  exposed use ::ndarray::NdFloat;
  exposed use Scalar;
}

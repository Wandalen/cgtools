/// Internal namespace.
mod private
{
  use crate::*;

  /// Trait for representing and manipulating rotations in 2D.
  pub trait Rotation2
  where
    Self : Rotation< 2 >,
  {

    fn from_angle< Dir, Up >( angle : < Self as Collection >::Scalar ) -> Self;

  }

}

crate::mod_interface!
{
  own use
  {
    Rotation2
  };

}

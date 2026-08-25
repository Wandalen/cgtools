/// Internal namespace.
mod private
{
  use crate::{Rotation, Collection};

  /// Trait for representing and manipulating rotations in 2D.
  pub trait Rotation2
  where
    Self : Rotation< 2 >,
  {

    // UX/DX: dropped the unused `< Dir, Up >` type parameters -- they appeared nowhere in the
    // signature (2D rotation has no "up"/"direction" axis choice the way a 3D basis does), had
    // zero implementors workspace-wide, and forced every caller to name two phantom types with
    // no way to know what they should be.
    /// Creates a new instance of the type from the given rotation angle.
    fn from_angle( angle : < Self as Collection >::Scalar ) -> Self;

  }

}

crate::mod_interface!
{
  own use
  {
    Rotation2
  };

}

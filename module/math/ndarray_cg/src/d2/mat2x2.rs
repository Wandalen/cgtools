mod private
{
  // use crate::*;
  // use std::fmt;

}

// mod rotation;
mod transformation;

crate::mod_interface!
{
  // own use rotation::
  // {
  //   Rotation2
  // };

  own use transformation::
  {
    rot,
    scale,
    shear,
    reflect_x,
    reflect_y,
  };

}

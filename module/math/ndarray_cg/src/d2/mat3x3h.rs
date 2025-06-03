mod private
{
  // use crate::*;
  // use std::fmt;

}

mod transformation;

crate::mod_interface!
{

  own use transformation::
  {
    // Decomposed,
    perspective_rh,
    perspective_rh_gl,
    look_to_rh,
    look_at_rh,
    rot,
    scale,
    translation,
  };

}

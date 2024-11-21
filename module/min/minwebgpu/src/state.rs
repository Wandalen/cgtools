/// Internal namespace.
mod private
{
  // use crate::*;
}

crate::mod_interface!
{
  layer vertex;
  layer fragment;
  layer blend;
  layer color_target;
  layer primitive;
  layer depth_stencil;
  layer stencil_face;
  layer multisample;
  layer programmable_stage;
}

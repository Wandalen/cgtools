/// Internal namespace.
mod private
{
  // use crate::*;
}

crate::mod_interface!
{
  /// Vertex shader state
  layer vertex;
  /// Fragment shader state
  layer fragment;
  /// Blend stat
  layer blend;
  /// Color target state
  layer color_target;
  /// Primitive state
  layer primitive;
  /// Depth stencil state
  layer depth_stencil;
  /// Stencil face state
  layer stencil_face;
  /// Multisample state
  layer multisample;
  /// Programmable stage state
  layer programmable_stage;
}

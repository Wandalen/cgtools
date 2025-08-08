/// Internal namespace.
mod private
{
  use crate::*;

  /// Returns a new `RenderPassDescriptor` with default settings.
  pub fn desc< 'a >() -> RenderPassDescriptor< 'a >
  {
    RenderPassDescriptor::new()
  }
}

crate::mod_interface!
{
  /// Color attachment related
  layer color_attachment;
  /// Depth stenctil attachment related
  layer depth_stencil_attachment;

  own use
  {
    desc
  };
}

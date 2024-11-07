/// Internal namespace.
mod private
{
  use crate::*;

  pub fn desc< 'a >() -> RenderPassDescriptor< 'a >
  {
    RenderPassDescriptor::new()
  }
}

crate::mod_interface!
{
  layer color_attachment;
  layer depth_stencil_attachment;

  own use
  {
    desc
  };
}

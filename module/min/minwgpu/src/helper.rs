//! This module contains helper shortcuts.

use mingl::mod_interface;

mod private
{
  /// A shortcut for creating a `wgpu::VertexAttribute`.
  #[ inline ]
  #[ must_use ]
  pub const fn attr
  (
    format : wgpu::VertexFormat,
    offset : wgpu::BufferAddress,
    shader_location : wgpu::ShaderLocation
  ) -> wgpu::VertexAttribute
  {
    wgpu::VertexAttribute
    {
      format,
      offset,
      shader_location,
    }
  }
}

mod_interface!
{
  layer adapter;
  layer device;
  own use attr;
}

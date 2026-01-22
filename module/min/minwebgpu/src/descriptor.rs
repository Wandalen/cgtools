/// Internal namespace.
mod private
{
  // use crate::*;
}

crate::mod_interface!
{
  /// Sampler descriptor
  layer sampler;
  /// Texture descriptor
  layer texture;
  /// Render pipeline descriptor
  layer render_pipeline;
  /// Render pass descriptor
  layer render_pass;
  /// Bind group descriptor
  layer bind_group;
  /// Pipeline layout descriptor
  layer pipeline_layout;
  /// Bind group descriptor
  layer bind_group_layout;
  /// Bind group layout entry descriptor
  layer bind_group_layout_entry;
  /// Buffer init descriptor
  layer buffer_init;
  /// Buffer descriptor
  layer buffer;
  /// Compute pipeline descriptor
  layer compute_pipeline;
}

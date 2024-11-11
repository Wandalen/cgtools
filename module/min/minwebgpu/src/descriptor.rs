/// Internal namespace.
mod private
{
  // use crate::*;
}

crate::mod_interface!
{
  layer sampler;
  layer texture;
  layer render_pipeline;
  layer render_pass;
  layer bind_group;
  layer pipeline_layout;
  layer bind_group_layout;
  layer bind_group_layout_entry;
  layer buffer_init;
  layer buffer;
}

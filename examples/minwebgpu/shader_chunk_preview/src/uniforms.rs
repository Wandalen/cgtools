//! GPU-side uniform buffer layout for the shader_chunk_preview example.
//! `time`/`resolution` are refreshed every frame by `app_run()`'s animation
//! loop; `noise_scale`/`warp_strength`/`brightness` are refreshed whenever
//! the browser-side slider UI reports a change (see `controls.rs`) --
//! field names match `shader/preview_fragment.wgsl`'s `Params` struct and
//! its `//@ param:` declarations 1:1, kept honest by
//! `tests/shader_source_test.rs`'s
//! `discovered_parameters_are_declared_as_uniform_fields`.

use minwebgpu as gl;

#[ repr( C ) ]
#[ derive( Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
pub( crate ) struct ParamsRaw
{
  pub( crate ) time : f32,
  pub( crate ) noise_scale : f32,
  pub( crate ) warp_strength : f32,
  pub( crate ) brightness : f32,

  /// .xy = drawing-buffer resolution in physical pixels, .zw unused.
  pub( crate ) resolution : [ f32; 4 ],
}

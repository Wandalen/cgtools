//! Graphics PBR renderer
// Pull the readme into the crate docs so its code blocks compile as doc tests —
// Quick Start drift then fails `cargo test --doc` instead of rotting silently (TASK-020).
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Webgl implementation of the renderer
  // Fix(BUG-241): was commented out, making this layer — and the `enabled`-only
  // deps its whole tree needs (minwebgl/web-sys/mingl/gltf/...) — unconditional
  // regardless of feature selection; broke any `--no-default-features` build
  // that didn't separately re-request `enabled` (e.g. `--features native`).
  #[ cfg( feature = "webgl" ) ]
  layer webgl;

  /// Canonical `gpu_hal`-based renderer — WebGPU-first, also runs on the
  /// WebGL2 backend ( `GpuContext::new_webgl` ) and, off-browser, on the
  /// native wgpu backend ( `GpuContext::new_native` ).
  #[ cfg( any
  (
    all( feature = "webgpu", target_arch = "wasm32" ),
    all( feature = "native", not( target_arch = "wasm32" ) )
  ) ) ]
  layer webgpu;
}

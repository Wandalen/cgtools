//! GPU hardware abstraction layer over the `min*` drivers.
//!
//! One WebGPU-shaped API through which rendering engines reach the GPU
//! without knowing which backend they run on. Backends are enum variants
//! behind features ( `webgpu`, `webgl`, `native` ), selected at runtime;
//! every handle offers a one-step drill-down to the raw driver object.
//! Shader sources are canonical WGSL with a per-backend override slot —
//! the WebGL backend requires the GLSL override pair.
//!
//! The browser backends ( `webgpu` over `minwebgpu`, `webgl` over
//! `minwebgl` ) exist on wasm32 only, like the drivers they wrap; the
//! `native` backend ( `wgpu` via `minwgpu` ) exists everywhere else and
//! renders into an offscreen texture readable through
//! `Surface::pixels_read`.
#![ doc( html_root_url = "https://docs.rs/gpu_hal/latest/gpu_hal/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

#[ cfg( feature = "enabled" ) ]
mod private {}

// The resource/device/pass layers exist only when at least one backend
// materializes on the current target — a browser feature on a native target
// ( or vice versa ) contributes nothing, and gating per backend-on-target
// keeps such builds down to the plain-data layers instead of producing
// uninhabited handle enums.
#[ cfg( feature = "enabled" ) ]
::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Error type shared by every backend.
  layer error;

  /// Backend-agnostic descriptors, formats and usage flags.
  layer types;

  /// WebGL2 backend data types and GL mappings.
  #[ cfg( all( feature = "webgl", target_arch = "wasm32" ) ) ]
  layer webgl;

  /// Native wgpu backend mappings and readback internals.
  #[ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]
  layer native;

  /// GPU resource handles: buffers, textures, samplers, shaders, bindings,
  /// pipelines.
  #[ cfg( any
  (
    all( feature = "webgpu", target_arch = "wasm32" ),
    all( feature = "webgl", target_arch = "wasm32" ),
    all( feature = "native", not( target_arch = "wasm32" ) )
  ) ) ]
  layer resource;

  /// Device, queue and presentation surface of the active backend.
  #[ cfg( any
  (
    all( feature = "webgpu", target_arch = "wasm32" ),
    all( feature = "webgl", target_arch = "wasm32" ),
    all( feature = "native", not( target_arch = "wasm32" ) )
  ) ) ]
  layer device;

  /// Command encoding and render pass recording.
  #[ cfg( any
  (
    all( feature = "webgpu", target_arch = "wasm32" ),
    all( feature = "webgl", target_arch = "wasm32" ),
    all( feature = "native", not( target_arch = "wasm32" ) )
  ) ) ]
  layer pass;
}

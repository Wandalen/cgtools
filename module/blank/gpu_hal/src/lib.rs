//! GPU hardware abstraction layer over the `min*` web drivers.
//!
//! One WebGPU-shaped API through which rendering engines reach the GPU
//! without knowing which backend they run on. Backends are enum variants
//! behind features ( `webgpu`, `webgl` ), selected at runtime; every handle
//! offers a one-step drill-down to the raw driver object. Shader sources
//! are canonical WGSL with a per-backend override slot — the WebGL backend
//! requires the GLSL override pair.
//!
//! Browser-only, like the drivers it wraps: on native targets the crate
//! compiles to a stub, mirroring `minwebgpu`.
#![ doc( html_root_url = "https://docs.rs/gpu_hal/latest/gpu_hal/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::exhaustive_enums ) ]
#![ allow( clippy::exhaustive_structs ) ]
#![ allow( clippy::must_use_candidate ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::return_self_not_must_use ) ]

#[ cfg( all( feature = "enabled", target_arch = "wasm32" ) ) ]
mod private {}

#[ cfg( all( feature = "enabled", target_arch = "wasm32" ) ) ]
::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Error type shared by every backend.
  layer error;

  /// Backend-agnostic descriptors, formats and usage flags.
  layer types;

  /// WebGL2 backend data types and GL mappings.
  #[ cfg( feature = "webgl" ) ]
  layer webgl;

  /// GPU resource handles: buffers, textures, samplers, shaders, bindings,
  /// pipelines.
  #[ cfg( any( feature = "webgpu", feature = "webgl" ) ) ]
  layer resource;

  /// Device, queue and presentation surface of the active backend.
  #[ cfg( any( feature = "webgpu", feature = "webgl" ) ) ]
  layer device;

  /// Command encoding and render pass recording.
  #[ cfg( any( feature = "webgpu", feature = "webgl" ) ) ]
  layer pass;
}

// Native target stub - mirrors minwebgpu's approach: compilation succeeds,
// functionality is absent.
#[ cfg( all( feature = "enabled", not( target_arch = "wasm32" ) ) ) ]
pub mod stub
{
  //! Stub for native targets, where no web GPU backend exists.

  /// Error stating that the HAL only functions on WebAssembly targets.
  #[ derive( Debug ) ]
  pub struct GpuHalNotAvailableError;

  impl std::fmt::Display for GpuHalNotAvailableError
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      write!( f, "gpu_hal functionality is only available on WebAssembly targets" )
    }
  }

  impl std::error::Error for GpuHalNotAvailableError {}
}

#[ cfg( all( feature = "enabled", not( target_arch = "wasm32" ) ) ) ]
pub use stub::*;

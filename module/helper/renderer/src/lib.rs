//! Graphics PBR renderer
// Pull the readme into the crate docs so its code blocks compile as doc tests —
// Quick Start drift then fails `cargo test --doc` instead of rotting silently (TASK-020).
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

// Crate-level lint policy — each allow below is a deliberate API decision :
// inlining is left to LLVM/wasm-opt across hundreds of small scene-graph accessors.
#![ allow( clippy::missing_inline_in_public_items ) ]
// Scene/material/light types are data bags callers construct and match field-by-field.
#![ allow( clippy::exhaustive_structs ) ]
#![ allow( clippy::exhaustive_enums ) ]
// GPU interop is cast-heavy by nature : sizes and indices cross u32/i32/f32/usize at
// every GL/WGPU call boundary; per-site annotation would drown the code.
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::cast_sign_loss ) ]
// Most fallible fns return `WebglError` from the same few GL call shapes; a per-fn
// `# Errors` section would repeat one sentence dozens of times. The forward-facing
// webgpu layer documents its errors explicitly.
#![ allow( clippy::missing_errors_doc ) ]
// Panics stem from `.unwrap()` on shader-location/context invariants established at
// construction; documenting each one adds noise without information.
#![ allow( clippy::missing_panics_doc ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Webgl implementation of the renderer
  //#[ cfg( feature = "webgl" ) ]
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

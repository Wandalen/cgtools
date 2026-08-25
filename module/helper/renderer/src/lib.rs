//! Graphics PBR renderer
// Pull the readme into the crate docs so its code blocks compile as doc tests —
// Quick Start drift then fails `cargo test --doc` instead of rotting silently (TASK-020).
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

mod private
{
  // This crate's `--lib` unit-test binary had no wasm-gated `#[cfg(test)]` code at all until
  // the inline reproducer tests added for BUG-432/433/434/435/436/437/438/439/440 ( scattered
  // across several `src/webgl/**` files, each nested inside its own `mod private` for
  // private-field access — see `rulebook.md § Test placement` ). Without this call, that one
  // compiled test binary defaults to running in Node.js, where `web_sys::window()` is always
  // `None` — same failure class as BUG-110 ( `tests/geometry_tests.rs` ), just surfacing here
  // for the first time because this binary never carried any wasm-gated test before.
  //
  // Root cause: `wasm_bindgen_test_configure!( run_in_browser )` must be linked into a test
  // binary at least once for that whole binary to run in a browser instead of Node — every
  // external `tests/*.rs` suite in this crate already carries its own copy ( each is a
  // separate binary ), but `src/lib.rs`'s own `--lib` binary never needed one before now.
  //
  // Pitfall: a missing `run_in_browser` config doesn't fail to compile — it fails at runtime
  // with an unrelated-looking `CanvasRetrievingError("Failed to get window")` on every single
  // test in the binary, which reads like a `minwebgl`/`mingl` regression rather than the
  // harness's own misconfiguration. One call anywhere in the binary's compiled `#[cfg(test)]`
  // code is enough to cover every `mod tests` block nested under `src/webgl/**`.
  #[ cfg( all( test, target_arch = "wasm32" ) ) ]
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
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

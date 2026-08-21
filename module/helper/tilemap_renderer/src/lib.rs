
//! Agnostic 2D rendering engine.
//!
//! Backend-agnostic rendering with POD commands and Y-up coordinate system.
//! Define commands once, render to any backend (SVG and WebGL today; terminal planned).
//!
//! ## Coordinate system
//!
//! All backends use a **Y-up** convention:
//! - `(0, 0)` is the **bottom-left** corner
//! - Positive Y points **up**
//! - Positive rotation is **counter-clockwise**
//!
//! ## Usage
//!
//! ```ignore
//! use tilemap_renderer::{ commands::*, types::*, assets::*, backend::* };
//! use tilemap_renderer::adapters::SvgBackend;
//!
//! let config = RenderConfig { width : 800, height : 600, ..Default::default() };
//! let mut svg = SvgBackend::new( config );
//! svg.assets_load( &assets )?;
//! svg.submit( &commands )?;
//! let Output::String( doc ) = svg.output()? else { unreachable!() };
//! ```

mod private
{
  // This crate's `--lib` unit-test binary had no wasm-gated `#[cfg(test)]` code at all until
  // the inline reproducer test added for BUG-441 ( `src/adapters/webgl.rs`, nested inside its
  // own `mod private` for private-field access -- see `rulebook.md § Test placement` ). Without
  // this call, that one compiled test binary defaults to running in Node.js, where
  // `web_sys::window()` is always `None` -- same failure class as `renderer`'s BUG-110
  // ( `renderer/tests/geometry_tests.rs` ) and this crate's own sibling fix in
  // `renderer/src/lib.rs` ( added for BUG-432..440's inline tests ).
  //
  // Root cause: `wasm_bindgen_test_configure!( run_in_browser )` must be linked into a test
  // binary at least once for that whole binary to run in a browser instead of Node -- this
  // crate's `--lib` binary never needed one before now.
  //
  // Pitfall: a missing `run_in_browser` config doesn't fail to compile -- it fails at runtime
  // with an unrelated-looking `CanvasRetrievingError("Failed to get window")` on every test in
  // the binary, which reads like a `minwebgl`/`mingl` regression rather than the harness's own
  // misconfiguration.
  #[ cfg( all( test, target_arch = "wasm32" ) ) ]
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
}

#[ cfg( feature = "enabled" ) ]
mod_interface::mod_interface!
{
  layer types;
  layer commands;
  layer assets;
  layer backend;

  #[ cfg( any
  (
    feature = "adapter-svg",
    feature = "adapter-terminal",
    feature = "adapter-webgl",
    feature = "adapter-webgpu",
    feature = "adapter-native",
    feature = "adapter-none",
  ) ) ]
  layer adapters;
}

// Scene-model has been extracted into its own crate: `tilemap_scene`. The
// `scene-model` feature now only gates the serde derives on the sampler
// types (`SamplerFilter`, `MipmapMode`, `WrapMode`) that `tilemap_scene`
// needs to serialize / deserialize alongside its own declaration types.

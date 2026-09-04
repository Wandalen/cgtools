
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

// Empty, but required: `mod_interface!` below resolves its own generated paths
// through a `private` module in every file that invokes it. This one briefly
// held a `wasm_bindgen_test_configure!( run_in_browser )` call, needed because
// the BUG-441 reproducer was an inline `#[cfg(test)]` block in
// `src/adapters/webgl.rs` and so compiled into this crate's `--lib` test binary,
// which defaults to Node where `web_sys::window()` is always `None`. That test
// now lives in `tests/webgl_context_loss_test.rs`, whose binary carries its own
// call, so nothing wasm-gated compiles into `--lib` any more.
mod private {}

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

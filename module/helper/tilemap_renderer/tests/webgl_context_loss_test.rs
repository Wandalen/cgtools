//! `WebGlBackend`'s context-loss flag lifecycle, against a live WebGL2 context.
//!
//! wasm32-only: `WebGlBackend::new` compiles real shaders against a live
//! `WebGl2RenderingContext`, which only exists under the workspace's
//! headless-browser wasm32 test runner ( see `.cargo/config.toml`'s
//! `[target.wasm32-unknown-unknown]` `runner` ) — a native `cargo nextest` run
//! cannot construct the subject at all. `webgl_backend_test.rs` holds this
//! adapter's context-free coverage instead.
//!
//! Reaches `context_lost` through the `test_internals` feature: the flag is
//! private state, and setting it is how this file stands in for a real
//! `webglcontextlost` DOM event.

#![ cfg( all( target_arch = "wasm32", feature = "adapter-webgl", feature = "test_internals" ) ) ]

use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::Assets;
use tilemap_renderer::backend::Backend;
use tilemap_renderer::types::RenderConfig;
use wasm_bindgen_test::wasm_bindgen_test;

// Browser, not Node: `web_sys::window()` is `None` under Node, so `gl_init`
// below fails at runtime there with a misleading
// `CanvasRetrievingError("Failed to get window")` rather than at compile time.
// Each file under `tests/` is its own binary, so this call belongs in each of
// them — it used to live in `src/lib.rs`'s `mod private`, back when this test
// was inline and therefore part of the `--lib` test binary.
wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

/// Builds a real, live WebGL2 context via a headless browser canvas. Same helper shape as
/// `renderer`'s `tests/webgl/*.rs::gl_init()` ( `gl::browser::setup` / `gl::canvas::make` /
/// `gl::context::from_canvas_with` ) — `tilemap_renderer` had no prior live-context test of
/// its own to reuse, so this mirrors that crate's proven pattern rather than introducing a
/// new one.
fn gl_init() -> gl::GL
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default();
  let canvas = gl::canvas::make().unwrap();
  gl::context::from_canvas_with( &canvas, options ).unwrap()
}

fn empty_assets() -> Assets
{
  Assets
  {
    fonts : Vec::new(),
    images : Vec::new(),
    sprites : Vec::new(),
    geometries : Vec::new(),
    gradients : Vec::new(),
    patterns : Vec::new(),
    clip_masks : Vec::new(),
    paths : Vec::new(),
  }
}

/// ## Root Cause
/// `context_lost` used to be cleared by the `webglcontextrestored` DOM listener the
/// instant the browser fired the event — before the caller had any chance to re-call
/// `assets_load` and re-upload GPU state. `submit`/`output` would then see
/// `context_lost == false` and proceed to issue GL calls against a context whose
/// textures/buffers/VAOs no longer existed.
///
/// ## Why Not Caught
/// No existing test exercised `context_lost`'s lifecycle at all — `webgl_backend_test.rs`
/// only covers `declared_capabilities()`, and nothing simulated a loss/restore cycle.
///
/// ## Fix Applied
/// `webglcontextrestored`'s listener no longer clears the flag — it only logs.
/// `assets_load` now clears `context_lost` itself, once GPU state has actually been
/// re-uploaded ( see the `Fix(BUG-441)` comments on both sites in `src/adapters/webgl.rs` ).
///
/// ## Prevention
/// This test simulates a loss ( `context_lost_set_for_test( true )` — the same effect the
/// real `webglcontextlost` listener has ) without needing to synthesize a real
/// `WEBGL_lose_context` DOM event ( no precedent for that exists anywhere in this
/// workspace ), then verifies `assets_load` — not merely the passage of time or a
/// restored-event — is what unblocks `submit`/`output` again.
///
/// ## Pitfall
/// Setting the private `context_lost` flag is a white-box shortcut standing in for a real
/// `webglcontextlost` event; it exercises the observable contract the fix changed
/// ( assets_load is now the sole place that clears the flag ), not the DOM listener
/// registration path itself.
// test_kind: bug_reproducer(BUG-441)
#[ wasm_bindgen_test ]
fn assets_load_clears_context_lost_after_simulated_loss()
{
  let gl = gl_init();
  let config = RenderConfig::default();
  let mut backend = WebGlBackend::new( config, gl ).unwrap();

  // Simulate the effect of a `webglcontextlost` event without needing a real one.
  backend.context_lost_set_for_test( true );

  assert!( backend.submit( &[] ).is_err(), "submit must reject while context_lost is true" );
  assert!( backend.output().is_err(), "output must reject while context_lost is true" );

  // The fix: re-uploading GPU state via assets_load is what clears the flag now, not the
  // (removed) listener-side clear.
  backend.assets_load( &empty_assets() ).unwrap();

  assert!( !backend.context_lost_for_test(), "assets_load must clear context_lost after re-uploading GPU state" );
  assert!( backend.submit( &[] ).is_ok(), "submit must succeed again once assets_load has run" );
  assert!( backend.output().is_ok(), "output must succeed again once assets_load has run" );
}

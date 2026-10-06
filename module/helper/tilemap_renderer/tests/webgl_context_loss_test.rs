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

mod helpers;

use helpers::empty_assets;
use helpers::webgl::{ gl_init, pixel_read, sleep };
use minwebgl as gl;
use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
use tilemap_renderer::backend::{ Backend, RenderError };
use tilemap_renderer::commands::{ Clear, RenderCommand, Sprite };
use tilemap_renderer::types::{ BlendMode, MipmapMode, RenderConfig, ResourceId, SamplerFilter, Transform, WrapMode };
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

// Browser, not Node: `web_sys::window()` is `None` under Node, so `gl_init`
// ( `helpers/webgl.rs` ) fails at runtime there with a misleading
// `CanvasRetrievingError("Failed to get window")` rather than at compile time.
// Each file under `tests/` is its own binary, so this call belongs in each of
// them — it used to live in `src/lib.rs`'s `mod private`, back when this test
// was inline and therefore part of the `--lib` test binary.
wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );

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

/// Calls `method` ( `loseContext` / `restoreContext` ) on a `WEBGL_lose_context`
/// extension object.
fn lose_context_call( extension : &gl::js_sys::Object, method : &str )
{
  let function : gl::js_sys::Function = gl::js_sys::Reflect::get( extension, &method.into() ).unwrap().dyn_into().unwrap();
  function.call0( extension ).unwrap();
}

/// One opaque red 1×1 bitmap ( id 0 ) and a sprite ( id 0 ) covering it.
fn red_sprite_assets() -> Assets
{
  Assets
  {
    images : vec!
    [
      ImageAsset
      {
        id : ResourceId::new( 0 ),
        source : ImageSource::Bitmap { bytes : vec![ 255, 0, 0, 255 ], width : 1, height : 1, format : PixelFormat::Rgba8 },
        filter : SamplerFilter::Nearest,
        mipmap : MipmapMode::Off,
        wrap : WrapMode::Clamp,
        premultiplied : false,
      }
    ],
    sprites : vec![ SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 1.0, 1.0 ] } ],
    ..empty_assets()
  }
}

/// A real loss and restore through `WEBGL_lose_context`, followed by the
/// `assets_load` the restore warning asks for, leaves the backend drawing again.
/// The restored context has none of the objects created before the loss and
/// starts from default state, so `assets_load` has to rebuild the shader
/// programs and re-apply `new()`'s GL state too. Before it did, the sprite
/// below used a program from the lost context and only the clear showed.
#[ wasm_bindgen_test ]
async fn assets_load_after_real_restore_draws_again()
{
  let gl = gl_init();
  let config = RenderConfig::default();
  let transform = Transform { scale : [ config.width as f32, config.height as f32 ], ..Transform::default() };
  let mut backend = WebGlBackend::new( config, gl.clone() ).unwrap();
  let assets = red_sprite_assets();
  backend.assets_load( &assets ).unwrap();
  let extension : gl::js_sys::Object = gl.get_extension( "WEBGL_lose_context" ).unwrap().expect( "WEBGL_lose_context" );

  lose_context_call( &extension, "loseContext" );
  for _ in 0..100
  {
    if backend.context_lost_for_test() { break; }
    sleep( 10 ).await;
  }
  assert!( backend.context_lost_for_test(), "webglcontextlost must reach the backend" );
  assert!( matches!( backend.assets_load( &assets ), Err( RenderError::ContextLost ) ), "assets_load can't upload into a lost context" );

  lose_context_call( &extension, "restoreContext" );
  for _ in 0..100
  {
    if !gl.is_context_lost() { break; }
    sleep( 10 ).await;
  }
  assert!( !gl.is_context_lost(), "the context must come back" );

  backend.assets_load( &assets ).unwrap();
  backend.submit
  (&[
    RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 1.0, 1.0 ] } ),
    RenderCommand::Sprite( Sprite { transform, sprite : ResourceId::new( 0 ), tint : [ 1.0; 4 ], blend : BlendMode::Normal, clip : None } ),
  ]).unwrap();

  let pixel = pixel_read( &gl );
  assert_eq!( pixel, [ 255, 0, 0, 255 ], "the sprite must draw after the restore" );
}

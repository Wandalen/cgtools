//! Orrery scene renderer with a Cargo-feature-selectable backend
//! ( webgl / webgpu / wgpu / vulkan ). Only the `wgpu` feature links the
//! `wgpu` crate — the other three do not, even transitively — see
//! `docs/adr/004_native_vulkan_hal_backend.md`.
//!
//! webgl/webgpu present live to a browser canvas, driven by
//! `mingl::web::exec_loop`, the same loop `orrery_webgpu` itself uses.
//! wgpu/vulkan have no windowing support in `gpu_hal` today
//! ( the native `Device::new` only ever builds an offscreen surface — see
//! `gpu_hal/tests/native_backend_test.rs` ), so those two render one frame
//! offscreen and save it as a PNG, the same pattern `minwgpu/hello_triangle`
//! uses.
//!
//! Every color/opacity/radius comes from `orrery_webgpu`'s own
//! `scene.rhai`, reused unchanged; task 203 explicitly defers
//! `orrery_webgpu`'s keyboard live-reparameterization out of scope, so the
//! per-frame parameters below ( seed/node count/grid density ) stay fixed.

#[ cfg( not( any(
  feature = "webgl",
  feature = "webgpu",
  feature = "wgpu",
  feature = "vulkan",
) ) ) ]
compile_error!( "orrery_flexible: select exactly one backend feature — webgl, webgpu, wgpu, or vulkan" );

// `wgpu` is the default feature, so selecting a different one also needs
// --no-default-features — otherwise both are active at once and, even
// though `gpu_hal::Device::new` itself disambiguates via a priority
// tie-break, the built binary would still transitively link `wgpu`
// whenever `vulkan` is also selected, violating the dependency-purity
// invariant this module's own top doc comment states ( ADR-004 ).
#[ cfg( any(
  all( feature = "webgl", feature = "webgpu" ),
  all( feature = "webgl", feature = "wgpu" ),
  all( feature = "webgl", feature = "vulkan" ),
  all( feature = "webgpu", feature = "wgpu" ),
  all( feature = "webgpu", feature = "vulkan" ),
  all( feature = "wgpu", feature = "vulkan" ),
) ) ]
compile_error!( "orrery_flexible: more than one backend feature is enabled — select exactly one ( webgl, webgpu, wgpu, or vulkan ). `wgpu` is the default feature, so building a different one also needs --no-default-features" );

use orrery_flexible::uniforms::UniformsRaw;
use orrery_webgpu::scene;

/// Star-field/node shuffle seed — fixed since keyboard reshuffling
/// ( `orrery_webgpu`'s " " key ) is out of scope here.
const SEED : f32 = 0.0;
/// Orbiting node count — fixed since keyboard adjustment ( `orrery_webgpu`'s
/// Up/Down keys ) is out of scope here.
const NODE_COUNT : i32 = 4;
/// Cartesian grid density — fixed since keyboard adjustment
/// ( `orrery_webgpu`'s Left/Right keys ) is out of scope here.
const GRID_DENSITY : f32 = 10.0;

// ==================== Browser backends ( webgpu / webgl ) ====================

#[ cfg( all( target_arch = "wasm32", any( feature = "webgpu", feature = "webgl" ) ) ) ]
use gpu_hal::{ Device, Queue, Surface };
#[ cfg( all( target_arch = "wasm32", any( feature = "webgpu", feature = "webgl" ) ) ) ]
use mingl::web::web_sys;

/// Runs the shared per-frame browser loop: resizes the canvas to its CSS
/// box every frame ( `getCurrentTexture`/the WebGL backbuffer both pick up
/// the new size on their own — mirrors `orrery_webgpu`'s own resize
/// handling, `examples/orrery/webgpu/src/main.rs` ), then uploads this
/// frame's uniforms and draws through [`orrery_flexible::scene_render`].
#[ cfg( all( target_arch = "wasm32", any( feature = "webgpu", feature = "webgl" ) ) ) ]
fn browser_loop
(
  device : Device,
  queue : Queue,
  surface : Surface,
  canvas : web_sys::HtmlCanvasElement,
  base_uniforms : UniformsRaw,
)
{
  let update_and_draw = move | t : f64 |
  {
    let dpr = web_sys::window().unwrap().device_pixel_ratio();
    let w = ( f64::from( canvas.client_width() ) * dpr ).round() as u32;
    let h = ( f64::from( canvas.client_height() ) * dpr ).round() as u32;
    if w == 0 || h == 0
    {
      return true; // collapsed/hidden layout — nothing to render this frame
    }
    if ( canvas.width(), canvas.height() ) != ( w, h )
    {
      canvas.set_width( w );
      canvas.set_height( h );
    }

    let time = ( t / 1000.0 ) as f32;
    let raw = base_uniforms.with_frame( time, SEED, NODE_COUNT, GRID_DENSITY, ( w, h ) );
    orrery_flexible::scene_render( &device, &queue, &surface, &raw.to_bytes() )
    .expect( "scene render failed" );

    true
  };

  mingl::web::exec_loop::run( update_and_draw );
}

/// Device creation is always awaited here : `gpu_hal::Device::new` really
/// awaits on the `webgpu` feature ( adapter/device request ) and resolves
/// immediately with no true await point on `webgl` — one call shape covers
/// both, so this crate never branches on which browser backend is active.
#[ cfg( all( target_arch = "wasm32", any( feature = "webgpu", feature = "webgl" ) ) ) ]
async fn app_run()
{
  let canvas = mingl::web::canvas::retrieve_or_make().expect( "canvas retrieval failed" );
  let ( device, queue, surface ) = Device::new( &canvas ).await
  .expect( "device creation failed — does this browser support the selected backend?" );

  let scene = scene::SceneConfig::load();
  let base_uniforms = UniformsRaw::from( &scene );

  browser_loop( device, queue, surface, canvas, base_uniforms );
}

#[ cfg( all( target_arch = "wasm32", any( feature = "webgpu", feature = "webgl" ) ) ) ]
fn main()
{
  wasm_bindgen_futures::spawn_local( app_run() );
}

// ==================== Native backends ( wgpu / vulkan ) ====================

/// Offscreen surface size — arbitrary; there is no window to size to.
/// Defined for every backend; the wasm32 backends size to the canvas
/// instead and leave this unused.
#[ allow( dead_code, reason = "only the wgpu/vulkan native backends read this -- the wasm32 backends size to the canvas instead" ) ]
const OFFSCREEN_SIZE : ( u32, u32 ) = ( 800, 600 );

/// Renders one offscreen frame and saves it as `-orrery_{backend}.png` —
/// `gpu_hal`'s native/Vulkan backends have no windowing support, so this is
/// the closest offscreen equivalent to the browser backends' live loop.
#[ cfg( all( not( target_arch = "wasm32" ), any( feature = "wgpu", feature = "vulkan" ) ) ) ]
fn main()
{
  let ( device, queue, surface ) = gpu_hal::Device::new( OFFSCREEN_SIZE.0, OFFSCREEN_SIZE.1 )
  .expect( "native device creation failed — is a Vulkan ICD installed ( a software one such as lavapipe suffices )?" );

  let scene = scene::SceneConfig::load();
  let base_uniforms = UniformsRaw::from( &scene );
  let raw = base_uniforms.with_frame( 0.0, SEED, NODE_COUNT, GRID_DENSITY, OFFSCREEN_SIZE );

  orrery_flexible::scene_render( &device, &queue, &surface, &raw.to_bytes() )
  .expect( "scene render failed" );

  let pixels = surface.pixels_read( &device, &queue ).expect( "pixel readback failed" );

  let backend = device.backend_name();
  let path = format!( "-orrery_{backend}.png" );
  image::save_buffer( &path, &pixels, OFFSCREEN_SIZE.0, OFFSCREEN_SIZE.1, image::ColorType::Rgba8 )
  .unwrap_or_else( | e | panic!( "failed to save {path} :: {e}" ) );
  println!( "orrery_flexible ( {backend} ): wrote {path}" );
}

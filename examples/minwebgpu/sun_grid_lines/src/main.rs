//! Procedural sci-fi HUD diagram ported to browser WebGPU: animated star,
//! orbit ring, and a Cartesian grid, driven by the same parameterization
//! uniforms as the WebGL2 version, adjustable live via the keyboard.
//!
//! Every color, opacity, and radius not listed above as keyboard-live comes
//! from `scene.rhai` (see `scene` module) instead of being a shader
//! constant — edit that file and rebuild to restyle the diagram.
//!
//! This example only works on WebAssembly (wasm32) targets where WebGPU
//! APIs are available.

#![ allow( clippy::cast_possible_truncation ) ]

// Compiled under `cargo test` (any target, so its own unit test runs
// natively) or when actually targeting wasm32 (where `run()` uses it). A
// plain native build/check/clippy has no consumer for it — the native path
// below is a stub — so it would otherwise be legitimately dead code there.
#[cfg( any( test, target_arch = "wasm32" ) )]
mod scene;

#[cfg( target_arch = "wasm32" )]
use minwebgpu as gl;
#[cfg( target_arch = "wasm32" )]
use std::rc::Rc;
#[cfg( target_arch = "wasm32" )]
use core::cell::RefCell;
#[cfg( target_arch = "wasm32" )]
use web_sys::{ wasm_bindgen::prelude::*, KeyboardEvent };

#[cfg( target_arch = "wasm32" )]
const SIZE : u32 = 800; // square canvas, matching the reference composition

#[cfg( target_arch = "wasm32" )]
#[ repr( C ) ]
#[ derive( Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
struct UniformsRaw
{
  time : f32,
  seed : f32,
  node_count : i32,
  grid_density : f32,

  // Static scene styling below, loaded once from `scene.rhai` (see
  // `scene::SceneConfig`) and left unchanged every frame. Every color is
  // `[ f32; 4 ]` ( unused w = 1.0 ) to match `scene.wgsl`'s `vec4f` fields —
  // WGSL's uniform-buffer layout aligns `vec3f` to 16 bytes anyway, so
  // packing as vec4 throughout avoids hand-deriving that padding.
  bg_top : [ f32; 4 ],
  bg_bottom : [ f32; 4 ],
  nebula_color : [ f32; 4 ],
  stars_color : [ f32; 4 ],
  grid_color : [ f32; 4 ],
  corona_inner : [ f32; 4 ],
  corona_mid : [ f32; 4 ],
  corona_outer : [ f32; 4 ],
  disc_dark : [ f32; 4 ],
  disc_mid : [ f32; 4 ],
  disc_bright : [ f32; 4 ],
  ring_color : [ f32; 4 ],

  /// x = nebula opacity, y = grid opacity, z = sun disc base radius, w = orbit ring radius
  scalars_a : [ f32; 4 ],
  /// x = star intensity, yzw = unused padding
  scalars_b : [ f32; 4 ]
}

/// Live-adjustable scene parameters, shared between the animation loop and
/// the keyboard handler below.
#[cfg( target_arch = "wasm32" )]
struct Params
{
  seed : f32,
  node_count : i32,
  grid_density : f32
}

/// Builds the static-styling portion of `UniformsRaw` from a loaded scene —
/// `time`/`seed`/`node_count`/`grid_density` are left zeroed, overwritten
/// every frame by `run()`'s animation loop via struct-update syntax.
#[cfg( target_arch = "wasm32" )]
fn base_uniforms_from_scene( scene : &scene::SceneConfig ) -> UniformsRaw
{
  UniformsRaw
  {
    time : 0.0,
    seed : 0.0,
    node_count : 0,
    grid_density : 0.0,

    bg_top : scene.background.top.to_array(),
    bg_bottom : scene.background.bottom.to_array(),
    nebula_color : scene.nebula.color.to_array(),
    stars_color : scene.stars.color.to_array(),
    grid_color : scene.grid.color.to_array(),
    corona_inner : scene.sun_corona.inner.to_array(),
    corona_mid : scene.sun_corona.mid.to_array(),
    corona_outer : scene.sun_corona.outer.to_array(),
    disc_dark : scene.sun_disc.dark.to_array(),
    disc_mid : scene.sun_disc.mid.to_array(),
    disc_bright : scene.sun_disc.bright.to_array(),
    ring_color : scene.orbit_ring.color.to_array(),

    scalars_a : [ scene.nebula.opacity as f32, scene.grid.opacity as f32, scene.sun_disc.base_radius as f32, scene.orbit_ring.radius as f32 ],
    scalars_b : [ scene.stars.intensity as f32, 0.0, 0.0, 0.0 ],
  }
}

#[cfg( target_arch = "wasm32" )]
async fn run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::retrieve_or_make()?;
  canvas.set_width( SIZE );
  canvas.set_height( SIZE );

  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::request_adapter().await;
  let device = gl::context::request_device( &adapter ).await;
  let queue = device.queue();
  let presentation_format = gl::context::preferred_format();
  gl::context::configure( &device, &context, presentation_format )?;

  let shader = gl::ShaderModule::new( include_str!( "../shaders/scene.wgsl" ) ).create( &device );

  let render_pipeline = gl::render_pipeline::create
  (
    &device,
    &gl::render_pipeline::desc( gl::VertexState::new( &shader ) )
    .fragment
    (
      gl::FragmentState::new( &shader )
      .target( gl::ColorTargetState::new().format( presentation_format ) )
    )
    .into()
  )?;

  // No explicit pipeline layout was given above, so WebGPU derives an
  // "auto" bind group layout from the shader's own uniform declaration;
  // get_bind_group_layout(0) retrieves it.
  let uniform_buffer = gl::BufferDescriptor::new( gl::BufferUsage::UNIFORM | gl::BufferUsage::COPY_DST )
  .size::< UniformsRaw >()
  .create( &device )?;

  let uniform_bind_group = gl::bind_group::desc( &render_pipeline.get_bind_group_layout( 0 ) )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( &uniform_buffer ) )
  .create( &device );

  let scene = scene::SceneConfig::load();
  let base_uniforms = base_uniforms_from_scene( &scene );

  let params = Rc::new( RefCell::new( Params { seed : 0.0, node_count : 1, grid_density : 10.0 } ) );

  // Keyboard controls demonstrate live re-parameterization of the shader:
  // Up/Down change how many nodes orbit the ring, Left/Right change grid
  // density, and Space reshuffles the star field and node layout.
  {
    let params = params.clone();
    let keydown_closure = move | e : KeyboardEvent |
    {
      let mut params = params.borrow_mut();
      match e.key().as_str()
      {
        "ArrowUp" => params.node_count = ( params.node_count + 1 ).min( 8 ),
        "ArrowDown" => params.node_count = ( params.node_count - 1 ).max( 1 ),
        "ArrowRight" => params.grid_density = ( params.grid_density + 2.0 ).min( 24.0 ),
        "ArrowLeft" => params.grid_density = ( params.grid_density - 2.0 ).max( 4.0 ),
        " " => params.seed = params.seed * 1.618_034 + 1.0,
        _ => {}
      }
    };
    let closure = Closure::< dyn FnMut( _ ) >::new( Box::new( keydown_closure ) );
    web_sys::window().unwrap().set_onkeydown( Some( closure.as_ref().unchecked_ref() ) );
    closure.forget();
  }

  let update_and_draw = move | t : f64 |
  {
    let time = ( t / 1000.0 ) as f32;
    let ( seed, node_count, grid_density ) =
    {
      let params = params.borrow();
      ( params.seed, params.node_count, params.grid_density )
    };

    let raw = UniformsRaw { time, seed, node_count, grid_density, ..base_uniforms };
    gl::queue::write_buffer( &queue, &uniform_buffer, &[ raw ] ).unwrap();

    let canvas_texture = gl::context::current_texture( &context ).unwrap();
    let canvas_view = gl::texture::view( &canvas_texture ).unwrap();

    let command_encoder = device.create_command_encoder();
    let render_pass = command_encoder.begin_render_pass
    (
      &gl::render_pass::desc()
      .color_attachment( gl::ColorAttachment::new( &canvas_view ) )
      .into()
    ).unwrap();

    render_pass.set_pipeline( &render_pipeline );
    render_pass.set_bind_group( 0, Some( &uniform_bind_group ) );
    render_pass.draw( 3 );
    render_pass.end();

    gl::queue::submit( &queue, command_encoder.finish() );

    true
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

#[cfg( target_arch = "wasm32" )]
fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}

// Stub main for native targets
#[cfg( not( target_arch = "wasm32" ) )]
fn main()
{
  println!( "This WebGPU example only works on WebAssembly targets." );
  println!( "To run this example, compile for wasm32-unknown-unknown target:" );
  println!( "  cargo build --target wasm32-unknown-unknown" );
}

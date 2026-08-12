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

// Only the wasm32 path (`app_run()`) consumes the scene module here; the native
// path below is a stub, and the scene tests live in `tests/scene_test.rs`
// against the library target.
#[cfg( target_arch = "wasm32" )]
use sun_grid_lines::scene;

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
  // `scene::SceneConfig`) and left unchanged every frame. Every field is
  // `[ f32; 4 ]` / `[ [ f32; 4 ]; N ]` to match `scene.wgsl`'s `vec4f` /
  // `array< vec4f, N >` fields — WGSL's uniform-buffer layout aligns
  // `vec3f` to 16 bytes and requires fixed-size arrays anyway, so packing
  // everything as vec4 slots avoids hand-deriving padding and keeps every
  // list's element count a compile-time constant on both sides.
  bg_top : [ f32; 4 ],
  bg_bottom : [ f32; 4 ],

  /// .xyz = color, .w = opacity
  nebula_colors : [ [ f32; 4 ]; scene::NEBULA_BAND_COUNT ],
  /// .x = vertical center, .y = thickness, .z = noise scale, .w = drift speed
  nebula_params : [ [ f32; 4 ]; scene::NEBULA_BAND_COUNT ],

  /// .xyz = color, .w = intensity
  star_colors : [ [ f32; 4 ]; scene::STAR_LAYER_COUNT ],
  /// .x = density, .y = point size, .z = twinkle speed, .w = unused
  star_params : [ [ f32; 4 ]; scene::STAR_LAYER_COUNT ],

  grid_color : [ f32; 4 ],
  /// x = opacity, y = line width, z = glow, w = unused
  grid_params : [ f32; 4 ],

  corona_inner : [ f32; 4 ],
  corona_mid : [ f32; 4 ],
  corona_outer : [ f32; 4 ],
  /// x = inner radius, y = mid radius, z = outer radius, w = unused
  corona_radii : [ f32; 4 ],
  /// x = flicker amplitude, y = flicker speed, zw = unused
  corona_flicker : [ f32; 4 ],

  disc_dark : [ f32; 4 ],
  disc_mid : [ f32; 4 ],
  disc_bright : [ f32; 4 ],
  /// x = base radius, y = pulsate amplitude, z = pulsate speed, w = granulation scale
  disc_params : [ f32; 4 ],

  /// .xyz = color, .w = glow amount
  ring_colors : [ [ f32; 4 ]; scene::ORBIT_RING_COUNT ],
  /// .x = radius, .y = stroke width, .z = pulse speed, .w = unused
  ring_params : [ [ f32; 4 ]; scene::ORBIT_RING_COUNT ],

  /// .xyz = color, .w = size
  node_colors : [ [ f32; 4 ]; scene::NODE_COUNT ],
  /// .x = orbit radius, .y = angular speed, .z = phase, .w = unused
  node_params : [ [ f32; 4 ]; scene::NODE_COUNT ],

  /// x = vignette strength, y = vignette radius, z = glow intensity, w = scanline intensity
  effects : [ f32; 4 ],
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

/// Packs a scene list — already asserted by `SceneConfig::load()` to have
/// exactly `N` elements, matching `scene.wgsl`'s fixed-size uniform arrays —
/// into a `[ [ f32; 4 ]; N ]` uniform slot, one call per list, one closure
/// per `array<vec4f, N>` field.
#[cfg( target_arch = "wasm32" )]
fn packed< T, F, const N : usize >( items : &[ T ], pack : F ) -> [ [ f32; 4 ]; N ]
where
  F : Fn( &T ) -> [ f32; 4 ],
{
  core::array::from_fn( | i | pack( &items[ i ] ) )
}

/// Builds the static-styling portion of `UniformsRaw` from a loaded scene —
/// `time`/`seed`/`node_count`/`grid_density` are left zeroed, overwritten
/// every frame by `app_run()`'s animation loop via struct-update syntax.
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

    nebula_colors : packed( &scene.nebula_bands, | band | { let [ r, g, b, _ ] = band.color.to_array(); [ r, g, b, band.opacity as f32 ] } ),
    nebula_params : packed( &scene.nebula_bands, | band | [ band.center as f32, band.thickness as f32, band.noise_scale as f32, band.drift_speed as f32 ] ),

    star_colors : packed( &scene.star_layers, | layer | { let [ r, g, b, _ ] = layer.color.to_array(); [ r, g, b, layer.intensity as f32 ] } ),
    star_params : packed( &scene.star_layers, | layer | [ layer.density as f32, layer.size as f32, layer.twinkle_speed as f32, 0.0 ] ),

    grid_color : scene.grid.color.to_array(),
    grid_params : [ scene.grid.opacity as f32, scene.grid.line_width as f32, scene.grid.glow as f32, 0.0 ],

    corona_inner : scene.sun_corona.inner.to_array(),
    corona_mid : scene.sun_corona.mid.to_array(),
    corona_outer : scene.sun_corona.outer.to_array(),
    corona_radii : [ scene.sun_corona.inner_radius as f32, scene.sun_corona.mid_radius as f32, scene.sun_corona.outer_radius as f32, 0.0 ],
    corona_flicker : [ scene.sun_corona.flicker_amplitude as f32, scene.sun_corona.flicker_speed as f32, 0.0, 0.0 ],

    disc_dark : scene.sun_disc.dark.to_array(),
    disc_mid : scene.sun_disc.mid.to_array(),
    disc_bright : scene.sun_disc.bright.to_array(),
    disc_params : [ scene.sun_disc.base_radius as f32, scene.sun_disc.pulsate_amplitude as f32, scene.sun_disc.pulsate_speed as f32, scene.sun_disc.granulation_scale as f32 ],

    ring_colors : packed( &scene.orbit_rings, | ring | { let [ r, g, b, _ ] = ring.color.to_array(); [ r, g, b, ring.glow as f32 ] } ),
    ring_params : packed( &scene.orbit_rings, | ring | [ ring.radius as f32, ring.stroke_width as f32, ring.pulse_speed as f32, 0.0 ] ),

    node_colors : packed( &scene.nodes, | node | { let [ r, g, b, _ ] = node.color.to_array(); [ r, g, b, node.size as f32 ] } ),
    node_params : packed( &scene.nodes, | node | [ node.radius as f32, node.speed as f32, node.phase as f32, 0.0 ] ),

    effects : [ scene.effects.vignette_strength as f32, scene.effects.vignette_radius as f32, scene.effects.glow_intensity as f32, scene.effects.scanline_intensity as f32 ],
  }
}

#[cfg( target_arch = "wasm32" )]
async fn app_run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::retrieve_or_make()?;
  canvas.set_width( SIZE );
  canvas.set_height( SIZE );

  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::adapter_request().await;
  let device = gl::context::device_request( &adapter ).await;
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
    gl::queue::buffer_write( &queue, &uniform_buffer, &[ raw ] ).unwrap();

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
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

// Stub main for native targets
#[cfg( not( target_arch = "wasm32" ) )]
fn main()
{
  println!( "This WebGPU example only works on WebAssembly targets." );
  println!( "To run this example, compile for wasm32-unknown-unknown target:" );
  println!( "  cargo build --target wasm32-unknown-unknown" );
}

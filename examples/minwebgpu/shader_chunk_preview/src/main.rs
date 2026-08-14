//! Interactive WebGPU preview of a composed shader_chunks set: renders
//! shader_chunk_preview's local `preview_fragment` chunk (domain-warped
//! fbm3 noise, built on shader_chunks_core's hash21/value_noise/fbm3
//! stack) live, with its `//@ param:`-declared uniforms
//! (`noise_scale`/`warp_strength`/`brightness`) wired to browser-side
//! slider controls -- see `readme.md` for the question this answers ("what
//! command opens a window with one chunk rendered, tunable by UI").
//!
//! This example only works on WebAssembly (wasm32) targets where WebGPU
//! APIs are available.

#[cfg(target_arch = "wasm32")]
use minwebgpu_shader_chunk_preview::shader_source;

#[cfg(target_arch = "wasm32")]
mod controls;
#[cfg(target_arch = "wasm32")]
mod uniforms;
#[cfg(target_arch = "wasm32")]
use uniforms::ParamsRaw;

#[cfg(target_arch = "wasm32")]
use minwebgpu as gl;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use core::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::prelude::*;

/// Slider-controlled tunables, named to match `controls::slider_add`'s
/// `property` argument and `shader/preview_fragment.wgsl`'s `Params`
/// fields 1:1 -- `serde_wasm_bindgen::from_value` deserializes
/// `controls.js`'s `getValues()` object straight into this shape.
#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize)]
struct Tunables
{
  noise_scale : f32,
  warp_strength : f32,
  brightness : f32,
}

#[cfg(target_arch = "wasm32")]
async fn app_run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( gl::browser::Config::default() );
  // Fill-parent canvas: mingl's make() sizes the drawing buffer to the CSS
  // box ( at devicePixelRatio ) and keeps it sized via ResizeObserver; the
  // frame loop below re-asserts the size anyway.
  let canvas = gl::canvas::retrieve_or_make()?;

  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::adapter_request().await;
  let device = gl::context::device_request( &adapter ).await;
  let queue = device.queue();
  let presentation_format = gl::context::preferred_format();
  gl::context::configure( &device, &context, presentation_format )?;

  // shader_source::assemble() composes shader_source::PREVIEW_CHUNKS --
  // the compile-time-selected shader_chunks_core chunks ( fullscreen
  // triangle vertex stage, noise stack ) plus the locally-defined
  // shader/preview_fragment.wgsl chunk -- dependency-before-dependent.
  let wgsl = shader_source::assemble();
  let shader = gl::ShaderModule::new( &wgsl ).create( &device );

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
  .size::< ParamsRaw >()
  .create( &device )?;

  let uniform_bind_group = gl::bind_group::desc( &render_pipeline.get_bind_group_layout( 0 ) )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( &uniform_buffer ) )
  .create( &device );

  let tunables = Rc::new( RefCell::new( Tunables { noise_scale : 4.0, warp_strength : 0.6, brightness : 1.2 } ) );

  // Slider ranges mirror `shader/preview_fragment.wgsl`'s own `//@ param:`
  // declarations -- `shader_chunks_params::chunk_discover` is the
  // machine-checked source of truth ( see
  // tests/shader_source_test.rs::discovered_parameters_are_declared_as_uniform_fields );
  // these three calls are this crate's UI-side copy of the same 3 ranges,
  // and their default values match `tunables` above.
  controls::slider_add( "Noise scale", "noise_scale", 4.0, 0.5, 20.0, 0.1 );
  controls::slider_add( "Warp strength", "warp_strength", 0.6, 0.0, 2.0, 0.01 );
  controls::slider_add( "Brightness", "brightness", 1.2, 0.0, 3.0, 0.01 );

  {
    let tunables = tunables.clone();
    let on_change_closure = move | values : JsValue |
    {
      let values : Tunables = serde_wasm_bindgen::from_value( values ).unwrap();
      *tunables.borrow_mut() = values;
    };
    let closure = Closure::< dyn FnMut( JsValue ) >::new( Box::new( on_change_closure ) );
    controls::on_change( closure.as_ref().unchecked_ref() );
    closure.forget();
  }

  let update_and_draw = move | t : f64 |
  {
    // The loop owns the buffer size: reconcile it with the canvas's CSS box
    // every frame, before acquiring the swap-chain texture ( WebGPU picks up
    // the new canvas size at the next getCurrentTexture ).
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
    let ( noise_scale, warp_strength, brightness ) =
    {
      let tunables = tunables.borrow();
      ( tunables.noise_scale, tunables.warp_strength, tunables.brightness )
    };

    let raw = ParamsRaw { time, noise_scale, warp_strength, brightness, resolution : [ w as f32, h as f32, 0.0, 0.0 ] };
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

#[cfg(target_arch = "wasm32")]
fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

// Stub main for native targets
#[cfg(not(target_arch = "wasm32"))]
fn main()
{
  println!( "This WebGPU example only works on WebAssembly targets." );
  println!( "To run this example, compile for wasm32-unknown-unknown target:" );
  println!( "  cargo build --target wasm32-unknown-unknown" );
}

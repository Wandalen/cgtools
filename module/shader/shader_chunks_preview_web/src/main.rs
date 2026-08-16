//! WebGPU browser runner for shader chunk previews: fetches the
//! `-preview.json` [`shader_chunks_preview_core::PreviewBundle`] the
//! `shader_chunks_preview` CLI wrote next to this crate's `index.html`,
//! compiles its composed WGSL, creates one slider per bundle parameter, and
//! renders full-screen — writing `time`/params/`resolution` into one
//! uniform buffer per the bundle's layout convention
//! ( [`shader_chunks_preview_core::resolution_index`] ).
//!
//! This runner only works on WebAssembly (wasm32) targets where WebGPU
//! APIs are available; it is served via `action/browser_serve` ( trunk ),
//! normally through `shader_chunks preview <name>`.

#[cfg(target_arch = "wasm32")]
mod controls;

#[cfg(target_arch = "wasm32")]
use minwebgpu as gl;
#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use core::cell::{ RefCell, Cell };
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::prelude::*;

/// Fetches and deserializes the preview bundle served next to
/// `index.html`. Panics loudly ( browser console ) when the bundle is
/// missing or stale — the CLI is the writer: `shader_chunks preview <name>`.
#[cfg(target_arch = "wasm32")]
async fn bundle_fetch() -> shader_chunks_preview_core::PreviewBundle
{
  let window = web_sys::window().expect( "no window" );
  let response = gl::JsFuture::from( window.fetch_with_str( "-preview.json" ) ).await
  .expect( "fetching `-preview.json` failed — write it first: `shader_chunks preview <name>`" );
  let response : web_sys::Response = response.dyn_into().expect( "fetch result must be a Response" );
  assert!
  (
    response.ok(),
    "fetching `-preview.json` returned HTTP {} — write it first: `shader_chunks preview <name>`",
    response.status()
  );
  let text = gl::JsFuture::from( response.text().expect( "Response.text()" ) ).await
  .expect( "reading `-preview.json` body failed" );
  let json = text.as_string().expect( "`-preview.json` body must be text" );
  serde_json::from_str( &json )
  .expect( "`-preview.json` must deserialize into a PreviewBundle — regenerate it: `shader_chunks preview <name>`" )
}

/// The pipeline and the bind group it derives `get_bind_group_layout(0)` from must always
/// change together -- one combined cell instead of two keeps a recompile from ever landing a
/// new pipeline next to a stale bind group ( or vice versa ) mid-frame.
#[cfg(target_arch = "wasm32")]
struct PipelineState
{
  pipeline : web_sys::GpuRenderPipeline,
  bind_group : web_sys::GpuBindGroup
}

// Pre-existing debt surfaced by this crate's first-ever wasm32-target clippy sweep (this
// function is `target_arch = "wasm32"`-gated, so a native-only clippy pass never compiled it,
// let alone linted it -- see BUG-162's verification). A straight-line WebGPU pipeline setup
// sequence like this one doesn't gain clarity from being split into single-call helpers.
#[ allow( clippy::too_many_lines, reason = "straight-line WebGPU pipeline setup sequence; splitting into single-call helpers reduces clarity" ) ]
#[cfg(target_arch = "wasm32")]
async fn app_run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let bundle = bundle_fetch().await;
  controls::chunk_title_set( &bundle.target );

  // Fill-parent canvas: mingl's make() sizes the drawing buffer to the CSS
  // box ( at devicePixelRatio ) and keeps it sized via ResizeObserver; the
  // frame loop below re-asserts the size anyway.
  let canvas = gl::canvas::retrieve_or_make()?;

  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::adapter_request().await?;
  let device = gl::context::device_request( &adapter ).await?;
  let queue = device.queue();
  let presentation_format = gl::context::preferred_format()?;
  gl::context::configure( &device, &context, presentation_format )?;

  // The bundle's WGSL is already composed, dependency-ordered, and
  // naga-validated by the CLI — compile it as-is.
  let shader = gl::ShaderModule::new( &bundle.wgsl ).create( &device );

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

  // One uniform buffer per the bundle layout convention: `time` at float 0,
  // each parameter after it in bundle order, `resolution : vec4f` at the
  // next 16-byte boundary.
  let resolution_index = shader_chunks_preview_core::resolution_index( bundle.parameters.len() );
  let float_count = resolution_index + 4;

  // No explicit pipeline layout was given above, so WebGPU derives an
  // "auto" bind group layout from the shader's own uniform declaration;
  // get_bind_group_layout(0) retrieves it.
  let uniform_buffer = gl::BufferDescriptor::new( gl::BufferUsage::UNIFORM | gl::BufferUsage::COPY_DST )
  .size_from_value( ( float_count * 4 ) as f64 )
  .create( &device )?;

  let uniform_bind_group = gl::bind_group::desc( &render_pipeline.get_bind_group_layout( 0 ) )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( &uniform_buffer ) )
  .create( &device );

  let pipeline_state : Rc< RefCell< PipelineState > > =
  Rc::new( RefCell::new( PipelineState { pipeline : render_pipeline, bind_group : uniform_bind_group } ) );

  // Guards against an older edit's recompile finishing after a newer one's and clobbering it --
  // each recompile task is independent and uncancelled, so a slow older edit can still be in
  // flight when a newer one settles first. Bumped synchronously on every edit; checked before
  // a recompile's result ( success or failure ) is ever applied.
  let generation : Rc< Cell< u64 > > = Rc::new( Cell::new( 0 ) );

  // Slider values in bundle order — the same order their uniform fields
  // follow `time` in the buffer.
  let values : Rc< RefCell< Vec< f32 > > > =
  Rc::new( RefCell::new( bundle.parameters.iter().map( | param | param.value as f32 ).collect() ) );
  let index_of : HashMap< String, usize > = bundle.parameters.iter().enumerate()
  .map( | ( index, param ) | ( param.property.clone(), index ) )
  .collect();

  for param in &bundle.parameters
  {
    controls::slider_add( &param.label, &param.property, param.value, param.min, param.max, param.step );
  }

  {
    let values = values.clone();
    let on_change_closure = move | changed : JsValue |
    {
      let changed : HashMap< String, f64 > = serde_wasm_bindgen::from_value( changed ).unwrap();
      let mut values = values.borrow_mut();
      for ( property, value ) in changed
      {
        if let Some( &index ) = index_of.get( &property )
        {
          values[ index ] = value as f32;
        }
      }
    };
    let closure = Closure::< dyn FnMut( JsValue ) >::new( Box::new( on_change_closure ) );
    controls::on_change( closure.as_ref().unchecked_ref() );
    closure.forget();
  }

  // Shadertoy-style live editor: seed it with the composed WGSL the CLI already
  // naga-validated, then wire a recompile task to the debounced edit callback. The uniform
  // buffer's layout ( param count/order, `resolution` slot ) stays fixed from the bundle
  // loaded at startup -- an edit only ever changes shader logic, never that layout.
  controls::editor_init( &bundle.wgsl );

  {
    let pipeline_state = pipeline_state.clone();
    let device = device.clone();
    let uniform_buffer = uniform_buffer.clone();
    let generation = generation.clone();
    let on_edit_closure = move | source : JsValue |
    {
      let pipeline_state = pipeline_state.clone();
      let device = device.clone();
      let uniform_buffer = uniform_buffer.clone();
      let generation = generation.clone();
      // Bump synchronously, before spawning -- ordering across rapid edits must track the
      // order `on_edit` itself fired, not whatever order the spawned tasks happen to resume in.
      let my_generation = generation.get() + 1;
      generation.set( my_generation );
      let source = source.as_string().unwrap();
      gl::spawn_local( async move
      {
        recompile( &device, &uniform_buffer, presentation_format, &source, &pipeline_state, &generation, my_generation ).await;
      });
    };
    let closure = Closure::< dyn FnMut( JsValue ) >::new( Box::new( on_edit_closure ) );
    controls::on_edit( closure.as_ref().unchecked_ref() );
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

    let mut floats = vec![ 0.0f32; float_count ];
    floats[ 0 ] = ( t / 1000.0 ) as f32;
    {
      let values = values.borrow();
      floats[ 1..=values.len() ].copy_from_slice( &values );
    }
    floats[ resolution_index ] = w as f32;
    floats[ resolution_index + 1 ] = h as f32;
    gl::queue::buffer_write( &queue, &uniform_buffer, &floats ).unwrap();

    let canvas_texture = gl::context::current_texture( &context ).unwrap();
    let canvas_view = gl::texture::view( &canvas_texture ).unwrap();

    let command_encoder = device.create_command_encoder();
    let render_pass = command_encoder.begin_render_pass
    (
      &gl::render_pass::desc()
      .color_attachment( gl::ColorAttachment::new( &canvas_view ) )
      .into()
    ).unwrap();

    let state = pipeline_state.borrow();
    render_pass.set_pipeline( &state.pipeline );
    render_pass.set_bind_group( 0, Some( &state.bind_group ) );
    render_pass.draw( 3 );
    render_pass.end();

    gl::queue::submit( &queue, command_encoder.finish() );

    true
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

/// Recompiles `source` into a new pipeline and swaps it into `pipeline_state` on success.
/// Every failure path ( shader compile error, pipeline validation error ) reports through the
/// diagnostics panel and returns without touching `pipeline_state` -- the last-good pipeline
/// keeps rendering every frame in the meantime, since `update_and_draw` only ever reads
/// whatever `pipeline_state` currently holds.
///
/// `generation`/`my_generation` guard against out-of-order completion: recompile tasks are
/// independent and uncancelled, so a slow older edit can still be in flight when a newer one
/// finishes first. If `generation` has moved past `my_generation` by the time this call would
/// report a result, a newer recompile is already authoritative and this one's outcome --
/// success or failure alike -- is discarded rather than applied or shown.
#[cfg(target_arch = "wasm32")]
async fn recompile
(
  device : &web_sys::GpuDevice,
  uniform_buffer : &web_sys::GpuBuffer,
  presentation_format : web_sys::GpuTextureFormat,
  source : &str,
  pipeline_state : &Rc< RefCell< PipelineState > >,
  generation : &Rc< Cell< u64 > >,
  my_generation : u64
)
{
  let shader = gl::ShaderModule::new( source ).create( device );

  let messages = gl::shader::compilation_messages_get( &shader ).await;
  if gl::shader::has_blocking_error( &messages )
  {
    if generation.get() == my_generation
    {
      controls::diagnostics_set( &format_messages( &messages ) );
    }
    return;
  }

  let descriptor : web_sys::GpuRenderPipelineDescriptor = gl::render_pipeline::desc( gl::VertexState::new( &shader ) )
  .fragment
  (
    gl::FragmentState::new( &shader )
    .target( gl::ColorTargetState::new().format( presentation_format ) )
  )
  .into();

  let pipeline = match gl::render_pipeline::create_async( device, &descriptor ).await
  {
    Ok( pipeline ) => pipeline,
    Err( error ) =>
    {
      if generation.get() == my_generation
      {
        controls::diagnostics_set( &error.to_string() );
      }
      return;
    }
  };

  // Same "auto" bind group layout convention as startup: no explicit `PipelineLayout`, so
  // WebGPU derives binding 0's layout from the new shader's own uniform declaration.
  let bind_group = gl::bind_group::desc( &pipeline.get_bind_group_layout( 0 ) )
  .auto_bindings()
  .entry_from_resource( &gl::BufferBinding::new( uniform_buffer ) )
  .create( device );

  if generation.get() == my_generation
  {
    *pipeline_state.borrow_mut() = PipelineState { pipeline, bind_group };
    controls::diagnostics_clear();
  }
}

/// Renders `GpuShaderModule.getCompilationInfo()` messages as one line per message --
/// `line:column: text` -- for the diagnostics panel.
#[cfg(target_arch = "wasm32")]
fn format_messages( messages : &[ gl::CompilationMessage ] ) -> String
{
  messages.iter()
  .map( | message | format!( "{}:{}: {}", message.line, message.column, message.text ) )
  .collect::< Vec< _ > >()
  .join( "\n" )
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
  println!( "This WebGPU runner only works on WebAssembly targets." );
  println!( "Serve it via the preview CLI instead:" );
  println!( "  shader_chunks preview <name>" );
}

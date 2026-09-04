//! The same triangle as `hello_triangle`, built with minwebgpu's quickstart
//! helpers instead of the raw step-by-step setup: `context::setup` replaces
//! the from_canvas/adapter_request/device_request/preferred_format/configure
//! sequence, and `render_pass::draw_to` replaces the command
//! encoder/pass/submit ceremony. Every value the helpers hand back — device,
//! queue, format, the render pass itself — is still the plain native
//! `web_sys` type, so dropping back to the manual API for any one step
//! stays a normal function call away. See `hello_triangle` for the fully
//! manual version these helpers were extracted from.
//!
//! This example only works on WebAssembly (wasm32) targets where WebGPU APIs are available.

#[cfg(target_arch = "wasm32")]
use minwebgpu as gl;

#[cfg(target_arch = "wasm32")]
async fn app_run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::retrieve_or_make()?;
  let gpu = gl::context::setup( &canvas ).await?;

  let shader = gl::ShaderModule::new( include_str!( "../shaders/shader.wgsl" ) ).create( &gpu.device );

  let render_pipeline = gl::render_pipeline::create
  (
    &gpu.device,
    &gl::render_pipeline::desc( gl::VertexState::new( &shader ) )
    .fragment
    (
      gl::FragmentState::new( &shader )
      .target
      (
        gl::ColorTargetState::new()
        .format( gpu.format )
      )
    )
    .into()
  )?;

  let canvas_view = gl::context::current_view( &gpu.context )?;

  gl::render_pass::draw_to( &gpu.device, &gpu.queue, &canvas_view, | pass |
  {
    pass.set_pipeline( &render_pipeline );
    pass.draw( 3 );
  })?;

  Ok(())
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
  println!("This WebGPU example only works on WebAssembly targets.");
  println!("To run this example, compile for wasm32-unknown-unknown target:");
  println!("  cargo build --target wasm32-unknown-unknown");
}

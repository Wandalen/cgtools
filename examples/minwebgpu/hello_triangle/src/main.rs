//! Classic "Hello Triangle" -- draws a single hardcoded 3-vertex triangle to the canvas.
//!
//! This example only works on WebAssembly (wasm32) targets where WebGPU APIs are available.

// Fix(BUG-306-A): the module doc comment above used to read "Just draw a large point in
// the middle of the screen" -- copy-pasted from an unrelated example and never updated
// to describe this crate's actual triangle-drawing shader (see shaders/shader.wgsl).
// Root cause: stale copy-paste doc comment, never cross-checked against this crate's own
// shader after being carried over.
// Pitfall: a demo crate's own top-of-file doc comment is not exempt from doc/source
// cross-checking just because it's "only an example".

#[cfg(target_arch = "wasm32")]
use minwebgpu as gl;

#[cfg(target_arch = "wasm32")]
async fn app_run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::retrieve_or_make()?;

  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::adapter_request().await?;
  let device = gl::context::device_request( &adapter ).await?;
  let queue = device.queue();
  let presentation_format = gl::context::preferred_format()?;
  gl::context::configure( &device, &context, presentation_format )?;
  
  let shader = gl::ShaderModule::new( include_str!( "../shaders/shader.wgsl" ) ).create( &device );
  
  let render_pipeline = gl::render_pipeline::create
  (
    &device, 
    &gl::render_pipeline::desc( gl::VertexState::new( &shader ) )
    .fragment
    ( 
      gl::FragmentState::new( &shader ) 
      .target
      ( 
        gl::ColorTargetState::new()
        .format( presentation_format ) 
      )
    )
    .into()
  )?;

  let canvas_texture = gl::context::current_texture( &context )?;
  let canvas_view = gl::texture::view( &canvas_texture )?;

  let command_encoder = device.create_command_encoder();
  let render_pass = command_encoder.begin_render_pass
  (
    &gl::render_pass::desc()
    .color_attachment( gl::ColorAttachment::new( &canvas_view ) )
    .into()
  ).unwrap();

  render_pass.set_pipeline( &render_pipeline );
  render_pass.draw( 3 );
  render_pass.end();

  gl::queue::submit( &queue, command_encoder.finish() );
  
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

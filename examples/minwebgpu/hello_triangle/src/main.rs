//! Just draw a large point in the middle of the screen.

use minwebgpu as gl;

async fn run() -> Result< (), gl::WebGPUError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::retrieve_or_make()?;

  let context = gl::context::from_canvas( &canvas )?;
  let adapter = gl::context::request_adapter().await;
  let device = gl::context::request_device( &adapter ).await;
  let queue = device.queue();
  let presentation_format = gl::context::preferred_format();
  gl::context::configure( &device, &context, presentation_format )?;
  
  let shader = gl::ShaderModule::new( include_str!( "../shaders/shader.wgsl" ) ).create( &device );
  
  let render_pipeline = gl::render_pipeline::create
  (
    &device, 
    gl::render_pipeline::desc( gl::VertexState::new( &shader ) )
    .fragment
    ( 
      gl::FragmentState::new( &shader ) 
      .target
      ( 
        gl::ColorTargetState::new()
        .format( presentation_format ) 
      )
    )
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

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}

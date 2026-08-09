//! Procedural sci-fi HUD diagram ported to browser WebGPU: animated star,
//! orbit ring, and a Cartesian grid, driven by the same parameterization
//! uniforms as the WebGL2 version, adjustable live via the keyboard.
//!
//! This example only works on WebAssembly (wasm32) targets where WebGPU
//! APIs are available.

#![ allow( clippy::cast_possible_truncation ) ]

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
  grid_density : f32
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

    let raw = UniformsRaw { time, seed, node_count, grid_density };
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

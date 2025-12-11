//! Postprocessing demo
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::implicit_return ) ]

mod lil_gui;
mod gui_setup;

use minwebgl as gl;
use renderer::webgl::{ Camera, Renderer, loaders, post_processing };
use post_processing::{ Pass, SwapFramebuffer };
use std::rc::Rc;
use core::cell::RefCell;

/// Entry point for the post-processing demo.
///
/// Demonstrates a multi-pass post-processing pipeline including HDR tone mapping,
/// color grading, and gamma correction applied to a 3D model.
fn main()
{
  gl::spawn_local( async { run().await.unwrap() } );
}

/// Sets up and runs the post-processing demo with interactive controls.
///
/// Creates a WebGL context, loads a GLTF model, configures the camera based on scene bounds,
/// and establishes a post-processing pipeline with three passes:
/// 1. Tone mapping (HDR to LDR conversion using ACES)
/// 2. Color grading (adjustable color correction in LDR space)
/// 3. Gamma correction (final sRGB conversion for display)
async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;

  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" )
  .expect( "Failed to enable EXT_shader_image_load_store extension" );

  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf_path = "skull_salazar_downloadable.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();

  // Calculate scene dimensions to automatically scale camera parameters
  // diagonal: full diagonal length of the bounding box for scale reference
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let dist = scene_bounding_box.max.mag();

  // Extract IEEE 754 exponent to determine scale order of magnitude
  // This helps dynamically adjust near/far planes based on scene size
  let exponent =
  {
    let bits = diagonal.to_bits();
    let exponent_field = ( ( bits >> 23 ) & 0xFF ) as i32;
    exponent_field - 127
  };

  // Camera setup: position eye along diagonal direction at calculated distance
  // Eye is positioned at (0,1,1) direction scaled by distance to fit the scene
  let mut eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  eye *= dist;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  // Camera projection parameters scaled to scene size
  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  // Near plane: scaled by 10^exponent but clamped to minimum of 1.0
  let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 );
  // Far plane: extends 100^|exponent| times beyond near plane for large scale range
  let far = near * 100.0f32.powi( exponent.abs() );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.bind_controls( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_use_emission( true );
  renderer.set_bloom_strength( 0.5 );
  renderer.set_bloom_radius( 0.5 );
  renderer.set_exposure( 1.0 );
  renderer.set_ibl( loaders::ibl::load( &gl, "envMap" ).await );
  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let color_grading = post_processing::ColorGradingPass::new( &gl )?;
  let color_grading = Rc::new( RefCell::new( color_grading ) );
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  gui_setup::setup( renderer.clone(), color_grading.clone() );

  let update_and_draw = move | _ : f64 |
  {
    renderer.borrow_mut().render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
    .expect( "Failed to render" );

    swap_buffer.reset();
    swap_buffer.bind( &gl );
    swap_buffer.set_input( renderer.borrow().get_main_texture() );

    // Post-processing pipeline - order matters for correct visual output:

    // Pass 1: Tone mapping (HDR → LDR conversion using ACES algorithm)
    // Must be first to compress HDR values into displayable LDR range (0-1)
    let res = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
    .expect( "Failed to render tonemapping pass" );

    swap_buffer.set_output( res );
    swap_buffer.swap();

    // Pass 2: Color grading (adjusts hue, saturation, brightness in LDR space)
    // Applied after tone mapping to work with perceptually linear LDR colors
    let res = color_grading.borrow().render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
    .expect( "Failed to render color grading pass" );

    swap_buffer.set_output( res );
    swap_buffer.swap();

    // Pass 3: Gamma correction (linear → sRGB for final display)
    // Must be last to ensure correct gamma for monitor display
    let _ = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
    .expect( "Failed to render ToSrgbPass" );

    true
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

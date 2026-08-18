//! Postprocessing demo

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
  gl::spawn_local( async { app_run().await.unwrap() } );
}

/// Sets up and runs the post-processing demo with interactive controls.
///
/// Creates a WebGL context, loads a GLTF model, configures the camera based on scene bounds,
/// and establishes a post-processing pipeline with three passes:
/// 1. Tone mapping (HDR to LDR conversion using ACES)
/// 2. Color grading (adjustable color correction in LDR space)
/// 3. Gamma correction (final sRGB conversion for display)
async fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default().antialias( false );
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

  let gltf_path = "static/skull_salazar_downloadable.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;
  scenes[ 0 ].borrow_mut().world_matrix_update();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();

  // Camera setup: frames the scene's bounding sphere from the (0,1,1) direction, deriving
  // distance/near/far from the box itself and the camera's own fov/aspect_ratio.
  let direction = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();

  let mut camera = Camera::from_bounding_box( &scene_bounding_box, direction, up, aspect_ratio, fov, 0.1 )?;
  camera.window_size_set( [ width, height ].into() );
  camera.controls_bind( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.use_emission_set( &gl, true );
  renderer.bloom_strength_set( 0.5 );
  renderer.bloom_radius_set( 0.5 );
  renderer.exposure_set( 1.0 );
  renderer.ibl_set( loaders::ibl::load( &gl, "static/envMap", None ).await );
  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let color_grading = post_processing::ColorGradingPass::new( &gl )?;
  let color_grading = Rc::new( RefCell::new( color_grading ) );
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  gui_setup::setup( &renderer, &color_grading );

  let update_and_draw = move | _ : f64 |
  {
    renderer.borrow_mut().render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
    .expect( "Failed to render" );

    swap_buffer.reset();
    swap_buffer.bind( &gl );
    swap_buffer.input_set( renderer.borrow().main_texture() );

    // Post-processing pipeline - order matters for correct visual output:

    // Pass 1: Tone mapping (HDR → LDR conversion using ACES algorithm)
    // Must be first to compress HDR values into displayable LDR range (0-1)
    let res = tonemapping.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
    .expect( "Failed to render tonemapping pass" );

    swap_buffer.output_set( res );
    swap_buffer.swap();

    // Pass 2: Color grading (adjusts hue, saturation, brightness in LDR space)
    // Applied after tone mapping to work with perceptually linear LDR colors
    let res = color_grading.borrow().render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
    .expect( "Failed to render color grading pass" );

    swap_buffer.output_set( res );
    swap_buffer.swap();

    // Pass 3: Gamma correction (linear → sRGB for final display)
    // Must be last to ensure correct gamma for monitor display
    let _ = to_srgb.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
    .expect( "Failed to render ToSrgbPass" );

    true
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

#[ cfg( test ) ]
mod tests
{
  /// ## Root Cause
  /// `lil_gui.rs`'s `name_set` binding declared `#[ wasm_bindgen( js_name = "getTitle" ) ]`, but
  /// `gui.js` exports no function named `getTitle` at all -- it exports `set_name`, which calls
  /// lil-gui's own `gui.name( name )` setter. Every other binding in `lil_gui.rs` has a `js_name`
  /// that exactly matches its corresponding `export function` in `gui.js`; this one didn't.
  ///
  /// ## Why Not Caught
  /// `name_set` is declared but never called anywhere in this crate, so the mismatch never
  /// reached the wasm/JS boundary at runtime. `wasm_bindgen` cannot verify a `js_name` target
  /// exists in the target JS module at compile time -- an `extern` binding to a nonexistent
  /// export compiles cleanly and only fails, with an opaque "is not a function"-style error, the
  /// first time something actually calls it.
  ///
  /// ## Fix Applied
  /// Changed `js_name` from `"getTitle"` to `"set_name"`, matching the actual `gui.js` export.
  ///
  /// ## Prevention
  /// `test_lil_gui_js_name_bindings_match_gui_js_exports` parses every `js_name = "..."` value
  /// out of `lil_gui.rs` and asserts each one has a matching `export function NAME(` in
  /// `gui.js`, rather than only checking that the crate compiles.
  ///
  /// ## Pitfall
  /// A `wasm_bindgen` extern binding is only checked structurally by the Rust compiler -- it has
  /// no way to confirm the named JS export actually exists in the target module. A stale or
  /// mistyped `js_name` is invisible until something calls the binding at runtime in a browser.
  /// Any binding that's currently unused is exactly the kind most likely to hide this silently.
  // Fix(BUG-XXX-F): reproducer for `name_set` binding to the nonexistent JS export "getTitle"
  // instead of the actual export "set_name".
  // test_kind: bug_reproducer(BUG-XXX-F)
  #[ test ]
  fn test_lil_gui_js_name_bindings_match_gui_js_exports()
  {
    let bindings_src = include_str!( "lil_gui.rs" );
    let gui_js_src = include_str!( "../gui.js" );

    let js_names : Vec< &str > = bindings_src
    .split( "js_name = \"" )
    .skip( 1 )
    .map( | rest | rest.split( '"' ).next().unwrap() )
    .collect();

    assert!( !js_names.is_empty(), "expected to find at least one js_name binding in lil_gui.rs" );

    for name in js_names
    {
      let expected_export = format!( "export function {name}(" );
      assert!
      (
        gui_js_src.contains( &expected_export ),
        "lil_gui.rs binds js_name = \"{name}\", but gui.js has no `{expected_export}` -- every js_name must match an actual gui.js export"
      );
    }
  }
}

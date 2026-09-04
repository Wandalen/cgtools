//! Renders skeletal animation from GLTF files.
#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/skeletal_animation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders skeleton animation from GLTF files" ) ]

use std::{ cell::RefCell, rc::Rc };
use mingl::F32x3;
use minwebgl as gl;
use renderer::webgl::
{
  post_processing::
  {
    self,
    Pass,
    SwapFramebuffer
  },
  Camera,
  Renderer
};
use animation::Sequencer;

mod lil_gui;
mod gui_setup;

/// Computes ( near, far ) clip-plane distances from `exponent` ( the scene bounding-box
/// diagonal's base-2 exponent ), scaling `near` down for smaller scenes while guaranteeing
/// `far` always clears `near` by a safe margin.
fn near_far_from_exponent( exponent : i32 ) -> ( f32, f32 )
{
  let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 ) * 10.0;
  // Fix(BUG-331): the unguarded `far` formula collapses to `far <= near` for
  // `exponent in [ -1, 0, 1 ]` ( e.g. `far == near` at `exponent == -1` and `1`, and
  // `far < near` at `exponent == 0` ), which `Camera::new` rejects ( requires `far > near` );
  // `main()`'s `.unwrap()` on `app_run()`'s `Result` then panics the whole demo. That band
  // covers scene bounding-box diagonals in `[ 0.5, 4.0 )` -- an ordinary size range for a
  // normalized glTF asset, including this demo's own bundled `bug_bunny.glb`.
  // Root cause: `100.0f32.powi( exponent.abs() ) / 100.0` is V-shaped in `exponent`
  // ( `.abs()` makes it shrink toward `exponent == 0` from both sides, reaching its minimum
  // of `1.0 / 100.0` exactly there ) instead of monotonically increasing with scene size.
  // Pitfall: near/far are two independently-constrained outputs of one shared `exponent`
  // input -- deriving `far` as a pure multiple of `near` without a floor lets the multiplier
  // silently erase the required margin at whatever exponent makes it collapse, instead of
  // failing loudly at the formula's own definition site.
  let far = ( near * 100.0f32.powi( exponent.abs() ) / 100.0 ).max( near * 10.0 );
  ( near, far )
}

async fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf_path = "static/gltf/bug_bunny.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;
  scenes[ 0 ].borrow_mut().world_matrix_update();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  gl::info!( "Scene boudnig box: {scene_bounding_box:?}" );
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let dist = scene_bounding_box.max.mag();
  let exponent =
  {
    let bits = diagonal.to_bits();
    let exponent_field = ( ( bits >> 23 ) & 0xFF ) as i32;
    exponent_field - 127
  };
  gl::info!( "Exponent: {exponent:?}" );

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  eye *= dist;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let ( near, far ) = near_far_from_exponent( exponent );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far )?;
  camera.window_size_set( [ width, height ].into() );
  camera.controls_bind( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.ibl_set( renderer::webgl::loaders::ibl::load( &gl, "static/envMap", None ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  for node in &scenes[ 0 ].borrow().children
  {
    let mut scale = node.borrow().scale_get();
    scale.0[ 0 ] *= -1.0;
    node.borrow_mut().scale_set( scale );
  }

  // Overrides the `Camera::new` eye with a near-top-down angle (mostly -y,
  // slight +z) close to the scene origin -- kept intentionally non-zero
  // rather than exactly `[0.0, 0.0, 0.0]` so it can never coincide with
  // `center` (`scene_bounding_box.center()`, line 86 above), which would
  // trigger the `eye == center` NaN precondition documented on
  // `CameraOrbitControls::eye` (`module/min/mingl/src/controls/camera_orbit_controls.rs`).
  camera.controls_get().borrow_mut().eye = F32x3::from_array( [-5.341_171e-6, -0.015_823_878, 0.007_656_166] );

  let last_time = Rc::new( RefCell::new( 0.0 ) );

  let current_animation = Rc::new( RefCell::new( gltf.animations[ 0 ].clone() ) );

  gui_setup::setup( gltf.animations.clone(), &current_animation );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = t / 1000.0;

      {
        let last_time = last_time.clone();

        let delta_time = time - *last_time.borrow();
        *last_time.borrow_mut() = time;

        if current_animation.borrow().inner_get::< Sequencer >().unwrap().is_completed()
        {
          current_animation.borrow_mut().inner_get_mut::< Sequencer >()
          .unwrap()
          .reset();
        }

        current_animation.borrow_mut().update( delta_time );
        current_animation.borrow().set();
      }

      renderer.borrow_mut().render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.input_set( renderer.borrow().main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.output_set( t );
      swap_buffer.swap();

      let _ = to_srgb.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
      .expect( "Failed to render ToSrgbPass" );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// ## Root Cause
  /// `near_far_from_exponent`'s `far` formula ( `near * 100.0f32.powi( exponent.abs() ) /
  /// 100.0` ) used `exponent.abs()`, making the scaling multiplier V-shaped around
  /// `exponent == 0` instead of monotonically increasing with scene size. Across
  /// `exponent in [ -1, 0, 1 ]` the multiplier drops low enough that `far` equals or falls
  /// below `near` -- a degenerate/inverted frustum `Camera::new` rejects ( it requires
  /// `far > near` ), which `main()`'s `.unwrap()` on `app_run()`'s `Result` then panics on.
  ///
  /// ## Why Not Caught
  /// This crate has no test file -- it is a `fn main()`-only WebGL demo binary, verified by
  /// running it in a browser against this demo's own bundled `bug_bunny.glb`. Whether the
  /// panic actually fires depends on that specific asset's bounding-box diagonal landing in
  /// the broken `[ 0.5, 4.0 )` band, which was never checked against the formula's own math.
  ///
  /// ## Fix Applied
  /// Extracted the near/far computation into `near_far_from_exponent`, keeping `near`'s
  /// formula unchanged and wrapping `far` in `.max( near * 10.0 )` -- a floor guaranteeing a
  /// minimum 10x margin regardless of what the exponent-scaled term evaluates to.
  ///
  /// ## Prevention
  /// This test sweeps every exponent in a wide, representative range and asserts `far > near`
  /// unconditionally, rather than checking only the exponents that happened to break.
  ///
  /// ## Pitfall
  /// A multiplier derived from the same shared input as the value it scales can collapse to a
  /// value that erases the relationship the caller depends on ( here, `far > near` ) -- always
  /// floor/ceiling such a derived value against its sibling rather than trusting the formula's
  /// shape to hold across the whole input domain.
  // Fix(BUG-331): reproducer for `far <= near` across `exponent in [ -1, 0, 1 ]`, rejected
  // by `Camera::new` and turned into a hard panic via `main()`'s `.unwrap()`.
  // test_kind: bug_reproducer(BUG-331)
  #[ test ]
  fn test_far_always_exceeds_near_across_exponent_range()
  {
    for exponent in -10_i32 ..= 10_i32
    {
      let ( near, far ) = near_far_from_exponent( exponent );
      assert!( near.is_finite() && near > 0.0, "near must be finite and positive at exponent {exponent}" );
      assert!( far.is_finite() && far > near, "far ({far}) must exceed near ({near}) at exponent {exponent}" );
    }
  }

  /// Pins the pre-fix formula's exact failures across the broken `[ -1, 0, 1 ]` band,
  /// confirming the bug was real and not a hypothetical edge case.
  #[ test ]
  fn test_pre_fix_formula_was_broken_for_exponents_negative_one_zero_and_one()
  {
    for exponent in [ -1_i32, 0_i32, 1_i32 ]
    {
      let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 ) * 10.0;
      let buggy_far = near * 100.0f32.powi( exponent.abs() ) / 100.0;
      assert!( buggy_far <= near, "pre-fix formula must be degenerate at exponent {exponent} (near={near}, far={buggy_far})" );

      let ( _, fixed_far ) = near_far_from_exponent( exponent );
      assert!( fixed_far > near, "fixed formula must clear the degenerate case at exponent {exponent}" );
    }
  }
}

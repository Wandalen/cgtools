//! Renders GLTF files using postprocess effects.
#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/gltf_viewer/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders GLTF files using postprocess effects" ) ]

#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::empty_docs ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::redundant_static_lifetimes ) ]
#![ allow( clippy::used_underscore_binding ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::clone_on_copy ) ]
#![ allow( clippy::ref_option ) ]
#![ allow( clippy::unnecessary_cast ) ]
#![ allow( clippy::assigning_clones ) ]
#![ allow( clippy::for_kv_map ) ]
#![ allow( clippy::useless_format ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::needless_bool ) ]
#![ allow( clippy::unnecessary_wraps ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::await_holding_refcell_ref ) ]
#![ allow( clippy::let_and_return ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::field_reassign_with_default ) ]
#![ allow( clippy::if_not_else ) ]

mod uniform_utils;
mod cube_normal_map_generator;
mod gem;
mod configurator;
mod scene_utilities;
mod ui;
mod debug;
mod surface_material;
mod gem_frag;
mod gem_vert;

use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::
{
  GL,
  F32x3,
  web_sys::HtmlCanvasElement
};
use renderer::webgl::
{
  post_processing::{ self, Pass, SwapFramebuffer }
};

use scene_utilities::*;
use configurator::*;
use crate::ui::set_renderer_loaded;

fn handle_camera_position( configurator : &Configurator )
{
  let camera_controls = configurator.camera.get_controls();
  let Some( ring ) = configurator.rings.get_ring() else { return; };
  let current_scene = ring.scene.clone();

  // Handle missing Plane node gracefully (e.g., if renamed or removed from GLTF)
  let Some( plane ) = current_scene.borrow().get_node( "Plane" )
  else
  {
    return;
  };

  if camera_controls.borrow().eye.y() <= plane.borrow().get_translation().y() + 0.1 || configurator.ui_state.state == "hero"
  {
    plane.borrow_mut().set_visibility( false, false );
  }
  else
  {
    plane.borrow_mut().set_visibility( true, false );
  }
}

/// Resizes canvas to window size with device pixel ratio applied for sharp rendering on high-DPI displays
fn resize_canvas_with_dpr( canvas : &HtmlCanvasElement )
{
  let Some( window ) = web_sys::window()
  else
  {
    return;
  };
  let dpr = window.device_pixel_ratio();

  // Use fallback defaults for robustness if DOM API calls fail
  let width = window.inner_width()
    .ok()
    .and_then( | v | v.as_f64() )
    .map_or( 1920, | v | ( v * dpr ) as u32 );

  let height = window.inner_height()
    .ok()
    .and_then( | v | v.as_f64() )
    .map_or( 911, | v | ( v * dpr ) as u32 );

  canvas.set_width( width );
  canvas.set_height( height );
}

/// Limits canvas size to prevent context loose if canvas is too big
fn clamp_canvas_size( canvas : &HtmlCanvasElement )
{
  let aspect = canvas.client_width() as f32 / canvas.client_height() as f32;

  if canvas.width() > 3840 || canvas.height() > 2160
  {
    canvas.set_width( ( 2160.0 * aspect ) as u32 );
    canvas.set_height( 2160 );
  }
}

/// Resets [`Renderer`] and updates [`renderer::webgl::Camera`] when [`HtmlCanvasElement`] is resized
fn handle_resize
(
  gl : &GL,
  configurator : &mut Configurator,
  swap_buffer : &mut SwapFramebuffer,
  canvas : &HtmlCanvasElement,
  is_resized : &Rc< RefCell< bool > >,
  last_eye : &mut F32x3
)
{
  if *is_resized.borrow()
  {
    resize_canvas_with_dpr( canvas );
    clamp_canvas_size( canvas );

    if configurator.renderer.borrow_mut().resize( gl, canvas.width(), canvas.height(), 4 ).is_ok()
    {
      *swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

      configurator.camera.set_window_size( [ canvas.width() as f32, canvas.height() as f32 ].into() );
      let aspect = canvas.width() as f32 / canvas.height() as f32;
      let perspective = gl::math::d2::mat3x3h::perspective_rh_gl( 40.0f32.to_radians(), aspect, 0.1, 1000.0 );
      configurator.camera.set_projection_matrix( perspective );

      if let Some( ui_state ) = ui::get_ui_state()
      {
        *last_eye = F32x3::from_array( ui_state.eye );
      }

      *is_resized.borrow_mut() = false;
    }
  }
}

/// Check changed fields of [`ui::UiState`] and updates depended [`Configutator`] features
fn handle_ui_change( configurator : &mut Configurator, last_eye : &mut F32x3 )
{
  if ui::is_changed()
  {
    if let Some( ui_state ) = ui::get_ui_state()
    {
      configurator.ui_state = ui_state.clone();
      // Avoid String allocations in hot path by using &str comparisons
      let ring_changed = ui_state.changed.iter().any( | s | s == "ring" );
      let gem_changed = ui_state.changed.iter().any( | s | s == "gem" );
      let metal_changed = ui_state.changed.iter().any( | s | s == "metal" );

      if ring_changed
      {
        configurator.rings.current_ring = ui_state.ring as usize;

        // When switching rings, load the ring's saved colors (unless user also changed color)
        if !gem_changed
        {
          configurator.load_gem_from_ring();
          ui::update_selection_highlight( "gem", &configurator.ui_state.gem );
        }
        if !metal_changed
        {
          configurator.load_metal_from_ring();
          ui::update_selection_highlight( "metal", &configurator.ui_state.metal );
        }
      }

      if gem_changed
      {
        // Save the new gem color to the current ring
        configurator.save_gem_to_ring();
        configurator.update_gem_color();
      }
      else if ring_changed
      {
        // Apply the loaded gem color for the new ring
        configurator.update_gem_color();
      }

      if metal_changed
      {
        // Save the new metal color to the current ring
        configurator.save_metal_to_ring();
        configurator.update_metal_color();
      }
      else if ring_changed
      {
        // Apply the loaded metal color for the new ring
        configurator.update_metal_color();
      }

      let new_eye = F32x3::from_array( ui_state.eye );

      // Avoid String allocations in hot path by using &str comparisons
      let position_changed = ui_state.changed.iter().any( | s | s == "position" );
      let center_changed = ui_state.changed.iter().any( | s | s == "center" );
      let state_changed = ui_state.changed.iter().any( | s | s == "state" );

      if ( position_changed || center_changed ) &&
      !state_changed &&
      !( ui_state.transition_animation_enabled && new_eye.distance( &last_eye ) > 0.75 )
      {
        let controls = configurator.camera.get_controls();
        controls.borrow_mut().up = F32x3::from_array( [ 0.0, 1.0, 0.0 ] );
        controls.borrow_mut().center = F32x3::from_array( ui_state.center );
        controls.borrow_mut().eye = F32x3::from_array( ui_state.eye );
        *last_eye = new_eye;
      }

      ui::clear_changed();
    }
  }
}

/// Inits configurator and starts render loop
async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::retrieve()?;
  resize_canvas_with_dpr( &canvas );

  let gl = gl::context::from_canvas_with( &canvas, options )?;

  clamp_canvas_size( &canvas );

  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .map_err( | _ | gl::WebglError::Other( "Failed to enable EXT_color_buffer_float extension" ) )?;
  let _ = gl.get_extension( "EXT_shader_image_load_store" )
  .map_err( | _ | gl::WebglError::Other( "Failed to enable EXT_shader_image_load_store extension" ) )?;

  // Enable debug controls if in debug mode
  ui::enable_debug_controls_if_needed();

  let mut configurator = Configurator::new( &gl, &canvas ).await?;

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let Some( is_resized ) = add_resize_callback()
  else
  {
    return Err( gl::WebglError::Other( "Failed to add resize callback" ) );
  };

  let mut last_eye = F32x3::from_array
  (
    ui::get_ui_state().unwrap_or_else
    (
      ||
      {
        gl::log::warn!( "UI state not available, using default camera position" );
        Default::default()
      }
    )
    .eye
  );

  set_renderer_loaded();

  // Define the update and draw logic
  let update_and_draw =
  {
    let mut prev_time = 0.0;
    move | t : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      let _time = t as f32 / 1000.0;

      let delta_time = t - prev_time;
      prev_time = t;
      handle_camera_position( &configurator );
      handle_resize( &gl, &mut configurator, &mut swap_buffer, &canvas, &is_resized, &mut last_eye );
      configurator.camera.update( delta_time );
      configurator.animation_state.update( delta_time );

      handle_ui_change( &mut configurator, &mut last_eye );

      let Some( ring ) = configurator.rings.get_ring() else { return true; };
      let scene = &ring.scene;
      let _ = configurator.renderer.borrow_mut().render( &gl, &mut scene.borrow_mut(), &configurator.camera );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( configurator.renderer.borrow().get_main_texture() );

      let t = match tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      {
        Ok(texture) => texture,
        Err(e) =>
        {
          gl::log::error!( "Tonemapping pass failed: {:?}, skipping frame", e );
          return true;
        }
      };

      swap_buffer.set_output( t );
      swap_buffer.swap();

      if let Err( e ) = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      {
        gl::log::error!( "ToSrgb pass failed: {:?}, skipping frame", e );
        return true;
      }

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local
  (
    async move
    {
      if let Err( e ) = run().await
      {
        gl::log::error!( "Failed to initialize 3D jewelry configurator: {:?}", e );
      }
    }
  );
}

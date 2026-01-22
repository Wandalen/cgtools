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

use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::
{
  GL,
  web_sys::HtmlCanvasElement
};
use renderer::webgl::
{
  Renderer, post_processing::{ self, Pass, SwapFramebuffer }
};

mod cube_normal_map_generator;
mod gem;
mod configurator;
mod helpers;
mod ui;
mod debug;
mod surface_material;

use helpers::*;
use configurator::*;

fn handle_camera_position( configurator : &Configurator )
{
  let camera_controls = configurator.camera.get_controls();
  let current_scene = &configurator.rings.rings[ configurator.rings.current_ring ];
  let plane = current_scene.borrow().get_node( "Plane" ).unwrap();
  if camera_controls.borrow().eye.y() <= plane.borrow().get_translation().y() + 0.1 || configurator.ui_state.state == "hero"
  {
    plane.borrow_mut().set_visibility( false, false );
  }
  else
  {
    plane.borrow_mut().set_visibility( true, false );
  }
}

/// Resets [`Renderer`] and updates [`renderer::webgl::Camera`] when [`HtmlCanvasElement`] is resized
fn handle_resize
(
  gl : &GL,
  configurator : &mut Configurator,
  swap_buffer : &mut SwapFramebuffer,
  canvas : &HtmlCanvasElement,
  is_resized : &Rc< RefCell< bool > >
)
{
  if *is_resized.borrow()
  {
    if let Ok( r ) = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )
    {
      {
        let mut renderer_mut = configurator.renderer.borrow_mut();
        *renderer_mut = r;
      }
      configurator.setup_renderer();

      *swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

      configurator.camera.set_window_size( [ canvas.width() as f32, canvas.height() as f32 ].into() );
      let aspect = canvas.width() as f32 / canvas.height() as f32;
      let perspective = gl::math::d2::mat3x3h::perspective_rh_gl( 40.0f32.to_radians(), aspect, 0.1, 1000.0 );
      configurator.camera.set_projection_matrix( perspective );

      *is_resized.borrow_mut() = false;
    }
  }
}

/// Check changed fields of [`ui::UiState`] and updates depended [`Configutator`] features
fn handle_ui_change( configurator : &mut Configurator )
{
  if ui::is_changed()
  {
    if let Some( ui_state ) = ui::get_ui_state()
    {
      configurator.ui_state = ui_state.clone();
      let ring_changed = ui_state.changed.contains( &"ring".to_string() );

      if ring_changed
      {
        configurator.rings.current_ring = ui_state.ring as usize;
      }

      if ui_state.changed.contains( &"gem".to_string() ) || ring_changed
      {
        configurator.update_gem_color();
      }

      if ui_state.changed.contains( &"metal".to_string() ) || ring_changed
      {
        configurator.update_metal_color();
      }

      if ( ui_state.changed.contains( &"position".to_string() ) ||
      ui_state.changed.contains( &"center".to_string() ) ) && !ui_state.changed.contains( &"state".to_string() )
      {
        let controls = configurator.camera.get_controls();
        controls.borrow_mut().up = gl::F32x3::from_array( [ 0.0, 1.0, 0.0 ] );
        controls.borrow_mut().center = gl::F32x3::from_array( ui_state.center );
        controls.borrow_mut().eye = gl::F32x3::from_array( ui_state.eye );
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

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  // Enable debug controls if in debug mode
  ui::enable_debug_controls_if_needed();

  let mut configurator = Configurator::new( &gl, &canvas ).await.unwrap();

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let is_resized = add_resize_callback();

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
      handle_resize( &gl, &mut configurator, &mut swap_buffer, &canvas, &is_resized );
      configurator.camera.update( delta_time );
      configurator.animation_state.update( delta_time );
      handle_ui_change( &mut configurator );

      let scene = &configurator.rings.rings[ configurator.rings.current_ring ];
      configurator.renderer.borrow_mut().render( &gl, &mut scene.borrow_mut(), &configurator.camera ).expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( configurator.renderer.borrow().get_main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.set_output( t );
      swap_buffer.swap();

      let _ = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
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
  gl::spawn_local( async move { run().await.unwrap(); } );
}

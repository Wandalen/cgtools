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
  Renderer, material::PBRMaterial, post_processing::{ self, Pass, SwapFramebuffer }
};

mod cube_normal_map_generator;
mod gem;
mod configurator;
mod helpers;
mod ui;
mod debug;

use helpers::*;
use configurator::*;

const MAX_DISTANCE : f32 = 50.0;

fn handle_camera_position( configurator : &Configurator )
{
  let camera_controls = configurator.camera.get_controls();
  let distance = camera_controls.borrow().eye.distance( &F32x3::default() );
  if distance > MAX_DISTANCE
  {
    camera_controls.borrow_mut().eye /= distance / MAX_DISTANCE;
  }

  {
    let mut material = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< PBRMaterial >( configurator.surface_material.borrow_mut() );
    if camera_controls.borrow().eye.y() < -20.0
    {
      material.base_color_factor.0[ 3 ] = 0.0;
      material.alpha_mode = renderer::webgl::AlphaMode::Blend;
    }
    else
    {
      material.base_color_factor.0[ 3 ] = 1.0;
      material.alpha_mode = renderer::webgl::AlphaMode::Opaque;
    }
    material.need_update = true;
  }
  configurator.renderer.borrow().update_material_uniforms( &gl, &configurator.surface_material, configurator.rings.current_ring.clone() );
}

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
      let perspective = gl::math::d2::mat3x3h::perspective_rh_gl( 70.0f32.to_radians(), aspect, 0.1, 1000.0 );
      configurator.camera.set_projection_matrix( perspective );

      *is_resized.borrow_mut() = false;
    }
  }
}

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
        if let Some( new_gem ) = configurator.rings.gems.get( ui_state.ring as usize ).cloned()
        {
          configurator.rings.current_gem = new_gem;
        }
        if let Some( new_ring ) = configurator.rings.rings.get( ui_state.ring as usize ).cloned()
        {
          remove_node_from_scene( &configurator.scene, &configurator.rings.current_ring );
          configurator.rings.current_ring = new_ring.clone();
          configurator.scene.borrow_mut().add( configurator.rings.current_ring.clone() );
          configurator.scene.borrow_mut().update_world_matrix();

          if configurator.surface_material.borrow().get_type_name() == "PBRMaterial"
          {
            let mut material = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< PBRMaterial >( configurator.surface_material.borrow_mut() );
            material.light_map = Some( configurator.rings.light_maps[ ui_state.ring as usize ].clone() );
            material.need_update = true;
          }
          configurator.renderer.borrow().update_material_uniforms( &gl, &configurator.surface_material, new_ring.clone() );
        }
      }

      if ui_state.changed.contains( &"gem".to_string() ) || ring_changed
      {
        configurator.update_gem_color();
      }

      if ui_state.changed.contains( &"metal".to_string() ) || ring_changed
      {
        configurator.update_metal_color();
      }

      ui::clear_changed();
    }
  }
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let mut configurator = Configurator::new( &gl, &canvas ).await.unwrap();

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let is_resized = add_resize_callback();

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      handle_camera_position( &configurator );
      handle_resize( &gl, &mut configurator, &mut swap_buffer, &canvas, &is_resized );
      handle_ui_change( &mut configurator );

      // If textures are of different size, gl.view_port needs to be called
      let _time = t as f32 / 1000.0;

      configurator.renderer.borrow_mut().render( &gl, &mut configurator.scene.borrow_mut(), &configurator.camera ).expect( "Failed to render" );

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
  #[ cfg( debug_assertions ) ]
  {
    gl::spawn_local( async move { debug::debug_run().await.unwrap(); } );
  }

  #[ cfg( not( debug_assertions ) ) ]
  {
    gl::spawn_local( async move { run().await.unwrap(); } );
  }
}

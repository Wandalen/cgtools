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

use renderer::webgl::
{
  post_processing::{self, Pass, SwapFramebuffer}, Camera, Renderer
};

mod lil_gui;
mod gui_setup;

fn canvas_size( canvas : &gl::web_sys::HtmlCanvasElement ) -> ( u32, u32 )
{
  let window = gl::web_sys::window().unwrap();
  let dpr = window.device_pixel_ratio();
  let css_w = canvas.client_width() as f64;
  let css_h = canvas.client_height() as f64;
  let w = ( css_w * dpr ) as u32;
  let h = ( css_h * dpr ) as u32;
  ( w.max( 1 ), h.max( 1 ) )
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContextOptions::default()
  .antialias( false )
  .depth( false )
  .stencil( false )
  .power_preference( minwebgl::context::PowerPreference::HighPerformance );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let ( pixel_w, pixel_h ) = canvas_size( &canvas );
  canvas.set_width( pixel_w );
  canvas.set_height( pixel_h );

  let gltf_path = "static/dodge-challenger/gltf/scene.gltf";
  // let gltf_path = "gambeson.glb";
  // let gltf_path = "old_rusty_car.glb";
  // let gltf_path = "sponza.glb";
  // let gltf_path = "nissan_titan_2017_transparent.glb";
  // let gltf_path = "transparent_cubes_oit_rendering_test_model.glb";
  // let gltf_path = "model.glb";
  // let gltf_path = "untitled.glb";
  // let gltf_path = "av-8b_harrier_ii.glb";
  // let gltf_path = "dae_crib_-_tommys_garage.glb";
  // let gltf_path = "low_poly_kids_playground.glb";
  // let gltf_path = "watchman_of_doom_2.0_special.glb";

  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  let center = scene_bounding_box.center();

  let norm_scale = if diagonal > 0.0 { 1.0 / diagonal } else { 1.0 };
  {
    let mut scene = scenes[ 0 ].borrow_mut();
    scene.set_scale( gl::math::F32x3::splat( norm_scale ) );
    scene.set_translation( center * -norm_scale );
    scene.update_world_matrix();
  }

  let eye = gl::math::F32x3::from( [ 0.0, 0.7, 0.7 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::splat( 0.0 );

  let fov = 70.0f32.to_radians();
  let near = 0.01;
  let far = 100.0;
  let aspect_ratio = pixel_w as f32 / pixel_h as f32;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ pixel_w as f32, pixel_h as f32 ].into() );
  camera.bind_controls( &canvas );

  let samples = 4;
  let mut renderer = Renderer::new( &gl, pixel_w, pixel_h, samples )?;

  let equirect = gl.create_texture();
  renderer::webgl::loaders::hdr_texture::load_to_mip_d2( &gl, equirect.as_ref(), 0, "static/venice_sunset_1k.hdr" ).await;

  let ibl = renderer::webgl::loaders::pmrem::generate( &gl, equirect.as_ref().unwrap(), 512 )?;
  renderer.set_ibl( ibl );
  renderer.set_clear_color( gl::math::F32x3::from( [ 0.01, 0.01, 0.01 ] ) );
  renderer.set_exposure( 0.0 );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, pixel_w, pixel_h );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  gui_setup::setup( renderer.clone() );

  let prev_size : Rc< RefCell< ( u32, u32 ) > > = Rc::new( RefCell::new( ( pixel_w, pixel_h ) ) );

  let update_and_draw =
  {
    let canvas = canvas.clone();
    let prev_size = prev_size.clone();
    move | _t : f64 |
    {
      let ( w, h ) = canvas_size( &canvas );
      let mut prev = prev_size.borrow_mut();
      if ( w, h ) != *prev
      {
        canvas.set_width( w );
        canvas.set_height( h );

        let proj = gl::math::mat3x3h::perspective_rh_gl( fov, w as f32 / h as f32, near, far );
        camera.set_projection_matrix( proj );
        camera.set_window_size( [ w as f32, h as f32 ].into() );

        renderer.borrow_mut().resize( &gl, w, h, samples ).expect( "Failed to resize renderer" );

        swap_buffer.free_gl_resources( &gl );
        swap_buffer = SwapFramebuffer::new( &gl, w, h );

        *prev = ( w, h );
      }

      renderer.borrow_mut().render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera ).expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( renderer.borrow().main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.set_output( t );
      swap_buffer.swap();

      let _ = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render ToSrgbPass" );

      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}

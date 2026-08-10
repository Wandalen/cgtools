//! Interactive dynamic text engraving demo.
//!
//! Loads a cylinder, binds an `engraving_config.json`-driven
//! [`renderer::webgl::engraving::EngravingSession`] to its `"Cylinder"` node (the glTF asset's
//! `TEXCOORD_1` is a dedicated unwrap of a flat rectangular section on the curved side, kept
//! separate from `TEXCOORD_0`'s cylindrical base-material UVs), and exposes text / font / bevel
//! controls through a lil-gui panel so the relief-normal and roughness/darkening shader path
//! (`USE_ENGRAVING` in `main.frag`) can be inspected live.

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
  post_processing::{ self, Pass, SwapFramebuffer },
  engraving::{ EngravingConfig, EngravingSession },
  Camera, Material, Object3D, Renderer, Scene,
};

mod gui_setup;
mod lil_gui;

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

/// Looks up `node_name`'s first primitive material, for wiring GUI sliders directly to
/// `PbrMaterial`'s engraving fields (bevel strength / groove roughness / groove darkening)
/// alongside the text itself, which goes through [`EngravingSession::set_text`].
fn find_material( scene : &Scene, node_name : &str ) -> Option< Rc< RefCell< Box< dyn Material > > > >
{
  let node = scene.get_node( node_name )?;
  let node_ref = node.borrow();
  let Object3D::Mesh( mesh ) = &node_ref.object else { return None };
  let mesh_ref = mesh.borrow();
  let primitive = mesh_ref.primitives.first()?;
  let material = primitive.borrow().material.clone();
  Some( material )
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContextOptions::default()
  .antialias( false )
  .depth( false )
  .stencil( false )
  .power_preference( gl::context::PowerPreference::HighPerformance );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  let ( pixel_w, pixel_h ) = canvas_size( &canvas );
  canvas.set_width( pixel_w );
  canvas.set_height( pixel_h );

  let gltf = renderer::webgl::loaders::gltf::load( &document, "static/cylinder.glb", &gl ).await?;
  let scene_rc = gltf.scenes[ 0 ].clone();

  let config = EngravingConfig::from_json( include_str!( "../engraving_config.json" ) )
  .expect( "engraving_config.json is valid" );

  let ( node_name, default_font, allowed_fonts ) =
  {
    let node_config = &config.nodes[ 0 ];
    ( node_config.node_name.to_string(), node_config.default_font.to_string(), node_config.allowed_fonts.clone() )
  };

  let session = EngravingSession::build( &gl, &scene_rc.borrow(), &config )
  .expect( "failed to bind engraving_config.json to the loaded scene" );

  let material = find_material( &scene_rc.borrow(), &node_name )
  .expect( "engraved node has a PBR material" );

  let eye = gl::math::F32x3::from( [ 0.0, 1.6, 1.8 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::splat( 0.0 );

  let fov = 45.0f32.to_radians();
  let near = 0.01;
  let far = 100.0;
  let aspect_ratio = pixel_w as f32 / pixel_h as f32;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ pixel_w as f32, pixel_h as f32 ].into() );
  camera.bind_controls( &canvas );

  let samples = 4;
  let mut renderer = Renderer::new( &gl, pixel_w, pixel_h, samples )?;

  let equirect = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "HDR equirect texture" ) )?;
  renderer::webgl::loaders::hdr_texture::load_to_mip_d2( &gl, Some( &equirect ), 0, "static/venice_sunset_1k.hdr" ).await;

  let ibl = renderer::webgl::loaders::pmrem::generate( &gl, &equirect, 512 )?;
  renderer.set_ibl( ibl );
  renderer.set_clear_color( gl::math::F32x3::from( [ 0.02, 0.02, 0.02 ] ) );
  renderer.set_exposure( 0.0 );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, pixel_w, pixel_h );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  gui_setup::setup( gl.clone(), node_name, default_font, allowed_fonts, Rc::new( RefCell::new( session ) ), material );

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

      renderer.borrow_mut().render( &gl, &mut scene_rc.borrow_mut(), &camera ).expect( "Failed to render" );

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

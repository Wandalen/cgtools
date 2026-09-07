//! Interactive PBR viewer: switch the loaded model at runtime from a debug
//! list, and toggle between the plain glTF viewer mode and the OpenPBR test
//! mode ( procedurally built scenes, starting with a solid-colour sphere ).

#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/gltf_viewer/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Interactive glTF / OpenPBR material viewer" ) ]

use std::{ cell::RefCell, rc::Rc };

use minwebgl as gl;
use gl::wasm_bindgen::{ prelude::Closure, JsValue };
use gl::js_sys::{ Object, Reflect };
use renderer::webgl::
{
  post_processing::{ self, Pass, SwapFramebuffer }, Camera, Renderer, Scene, DirectLight, Light, Node, Object3D
};

mod lil_gui;
mod gui_setup;
mod openpbr_scene;

/// Viewer modes.
const MODE_GLTF : &str = "gltf";
const MODE_OPENPBR : &str = "openpbr";

/// Model registry: ( display name, static path ). Kept in sync with the
/// `data-trunk rel="copy-file"` entries in `index.html`.
const MODELS : &[ ( &str, &str ) ] =
&[
  ( "Dodge Challenger", "static/dodge-challenger/gltf/scene.gltf" ),
  ( "AV-8B Harrier II", "static/av-8b_harrier_ii.glb" ),
  ( "Dae Crib", "static/dae_crib_-_tommys_garage.glb" ),
  ( "Gambeson", "static/gambeson.glb" ),
  ( "Low Poly Kids Playground", "static/low_poly_kids_playground.glb" ),
  ( "Bike", "static/bike.glb" ),
  ( "Nissan Titan 2017 (transparent)", "static/nissan_titan_2017_transparent.glb" ),
  ( "Old Rusty Car", "static/old_rusty_car.glb" ),
  ( "Transparent Cubes (OIT)", "static/transparent_cubes_oit_rendering_test_model.glb" ),
  ( "Watchman of Doom", "static/watchman_of_doom_2.0_special.glb" ),
];

/// What the viewer is currently showing.
#[ derive( Clone ) ]
struct ViewerChoice
{
  /// One of `MODE_GLTF` / `MODE_OPENPBR`.
  mode : String,
  /// glTF model path ( used only in glTF mode ).
  model : String,
}

struct ViewerState
{
  choice : RefCell< ViewerChoice >,
  scene : RefCell< Option< Rc< RefCell< Scene > > > >,
}

/// Normalizes a scene's scale/position so its bounding-box diagonal is 1 and
/// it sits at the origin ( matches the fixed camera framing ). Procedural
/// scenes may leave attribute bounding boxes at the inverted default
/// ( `min=+∞, max=−∞` ), which would produce a `∞` diagonal and a `NaN` center —
/// those are treated as "no box" and left untransformed.
fn scene_fit_to_view( scene : &Rc< RefCell< Scene > > )
{
  let scene_bounding_box = scene.borrow().bounding_box();
  let center = scene_bounding_box.center();
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();

  let has_box = diagonal.is_finite() && diagonal > 0.0 && center.mag().is_finite();
  let norm_scale = if has_box { 1.0 / diagonal } else { 1.0 };
  let translation = if has_box { center * -norm_scale } else { gl::math::F32x3::splat( 0.0 ) };

  let mut scene = scene.borrow_mut();
  scene.scale_set( gl::math::F32x3::splat( norm_scale ) );
  scene.translation_set( translation );
  scene.world_matrix_update();
}

/// (Re)loads whatever `state.choice` selects into `state.scene`.
async fn scene_load
(
  state : &Rc< ViewerState >,
  document : &gl::web_sys::Document,
  gl : &gl::WebGl2RenderingContext
) -> Result< (), gl::WebglError >
{
  let choice = state.choice.borrow().clone();

  if choice.mode == MODE_OPENPBR
  {
    // OpenPBR test mode — step 1: solid-colour sphere ( geometry check ).
    let gltf = openpbr_scene::solid_color_icosphere( gl, [ 0.2, 0.55, 1.0, 1.0 ] );
    let scene = gltf.scenes.into_iter().next().expect( "sphere scene exists" );

    // DIAGNOSTIC: kill IBL and light the sphere with a single directional light
    // + matte roughness so shading is a clean diffuse gradient. If the bright
    // quadrilateral that shows up under env lighting is a *flipped normal* it
    // will still show here; if it vanishes, it was environment reflection.
    if let Some( material ) = gltf.materials.first()
    {
      let mut material = renderer::webgl::cast_unchecked_material_to_ref_mut::< renderer::webgl::material::PbrMaterial >( material.borrow_mut() );
      material.need_use_ibl_set( false );
      material.roughness_factor = 0.8;
      material.metallic_factor = 0.0;
    }
    let key_light = Rc::new( RefCell::new( Node::new() ) );
    key_light.borrow_mut().object = Object3D::Light
    (
      Light::Direct
      (
        DirectLight
        {
          direction : gl::math::F32x3::from( [ 0.45, 0.8, 0.4 ] ).normalize(),
          color : gl::math::F32x3::from( [ 1.0, 1.0, 1.0 ] ),
          strength : 2.0,
        }
      )
    );
    scene.borrow_mut().children.push( key_light );

    scene_fit_to_view( &scene );
    *state.scene.borrow_mut() = Some( scene );
    return Ok( () );
  }

  let gltf = renderer::webgl::loaders::gltf::load( document, &choice.model, gl ).await?;
  let scene = gltf.scenes.into_iter().next().expect( "gltf has one scene" );
  scene_fit_to_view( &scene );
  *state.scene.borrow_mut() = Some( scene );

  Ok( () )
}

fn canvas_size( canvas : &gl::web_sys::HtmlCanvasElement ) -> ( u32, u32 )
{
  let window = gl::web_sys::window().unwrap();
  let dpr = window.device_pixel_ratio();
  let css_w = f64::from( canvas.client_width() );
  let css_h = f64::from( canvas.client_height() );
  let w = ( css_w * dpr ) as u32;
  let h = ( css_h * dpr ) as u32;
  ( w.max( 1 ), h.max( 1 ) )
}

/// Spawns a reload of the current selection.
fn reload
(
  state : &Rc< ViewerState >,
  document : &gl::web_sys::Document,
  gl : &gl::WebGl2RenderingContext
)
{
  let state = state.clone();
  let document = document.clone();
  let gl = gl.clone();
  gl::spawn_local( async move { let _ = scene_load( &state, &document, &gl ).await; } );
}

/// Wires the "Debug" folder: model list + viewer mode, both of which reload
/// the scene.
fn debug_ui_setup
(
  state : &Rc< ViewerState >,
  document : &gl::web_sys::Document,
  gl : &gl::WebGl2RenderingContext
)
{
  let js_object = Object::new();
  Reflect::set( &js_object, &JsValue::from_str( "model" ), &JsValue::from_str( &state.choice.borrow().model ) ).unwrap();
  Reflect::set( &js_object, &JsValue::from_str( "mode" ), &JsValue::from_str( &state.choice.borrow().mode ) ).unwrap();

  let gui = lil_gui::gui_new();
  let folder = lil_gui::folder_add( &gui, "Debug" );

  // Viewer mode ( glTF viewer / OpenPBR test ).
  let mode_map = Object::new();
  Reflect::set( &mode_map, &JsValue::from_str( "glTF model viewer" ), &JsValue::from_str( MODE_GLTF ) ).unwrap();
  Reflect::set( &mode_map, &JsValue::from_str( "OpenPBR test" ), &JsValue::from_str( MODE_OPENPBR ) ).unwrap();
  let mode_gui = lil_gui::dropdown_add( &folder, &js_object, "mode", &mode_map );
  let callback =
  {
    let state = state.clone();
    let document = document.clone();
    let gl = gl.clone();
    Closure::new( move | value : JsValue |
    {
      state.choice.borrow_mut().mode = value.as_string().unwrap_or_else( || MODE_GLTF.to_string() );
      reload( &state, &document, &gl );
    } )
  };
  lil_gui::on_finish_change( &mode_gui, &callback );
  callback.forget();

  // Model list ( glTF mode ).
  let catalog_map = Object::new();
  for ( name, path ) in MODELS
  {
    Reflect::set( &catalog_map, &JsValue::from_str( name ), &JsValue::from_str( path ) ).unwrap();
  }
  let catalog_gui = lil_gui::dropdown_add( &folder, &js_object, "model", &catalog_map );
  let callback =
  {
    let state = state.clone();
    let document = document.clone();
    let gl = gl.clone();
    Closure::new( move | value : JsValue |
    {
      if state.choice.borrow().mode != MODE_GLTF
      {
        return;
      }
      if let Some( path ) = value.as_string()
      {
        state.choice.borrow_mut().model = path;
        reload( &state, &document, &gl );
      }
    } )
  };
  lil_gui::on_finish_change( &catalog_gui, &callback );
  callback.forget();

  lil_gui::show( &gui );
}

async fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
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

  let default_model = MODELS[ 0 ].1;
  let state = Rc::new
  (
    ViewerState
    {
      choice : RefCell::new( ViewerChoice { mode : MODE_GLTF.to_string(), model : default_model.to_string() } ),
      scene : RefCell::new( None ),
    }
  );

  let fov = 70.0f32.to_radians();
  let near = 0.01;
  let far = 100.0;
  let aspect_ratio = pixel_w as f32 / pixel_h as f32;

  let mut camera = Camera::new( [ 0.0, 0.7, 0.7 ].into(), [ 0.0, 1.0, 0.0 ].into(), [ 0.0; 3 ].into(), aspect_ratio, fov, near, far )?;
  camera.window_size_set( [ pixel_w as f32, pixel_h as f32 ].into() );
  camera.controls_bind( &canvas );

  let samples = 4;
  let mut renderer = Renderer::new( &gl, pixel_w, pixel_h, samples )?;

  let equirect = gl.create_texture().ok_or( gl::WebglError::FailedToAllocateResource( "HDR equirect texture" ) )?;
  renderer::webgl::loaders::hdr_texture::load_to_mip_d2( &gl, Some( &equirect ), 0, "static/venice_sunset_1k.hdr" ).await;
  let ibl = renderer::webgl::loaders::pmrem::generate( &gl, &equirect, 512 )?;
  renderer.ibl_set( ibl );
  renderer.clear_color_set( gl::math::F32x3::from( [ 0.01, 0.01, 0.01 ] ) );
  renderer.exposure_set( 0.0 );

  let renderer = Rc::new( RefCell::new( renderer ) );

  gui_setup::setup( &renderer );
  debug_ui_setup( &state, &document, &gl );
  reload( &state, &document, &gl );

  let mut swap_buffer = SwapFramebuffer::new( &gl, pixel_w, pixel_h );
  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let prev_size : Rc< RefCell< ( u32, u32 ) > > = Rc::new( RefCell::new( ( pixel_w, pixel_h ) ) );

  let update_and_draw =
  {
    let canvas = canvas.clone();
    let prev_size = prev_size.clone();
    let state = state.clone();
    move | _t : f64 |
    {
      let ( w, h ) = canvas_size( &canvas );
      let mut prev = prev_size.borrow_mut();
      if ( w, h ) != *prev
      {
        canvas.set_width( w );
        canvas.set_height( h );

        let proj = gl::math::mat3x3h::perspective_rh_gl( fov, w as f32 / h as f32, near, far );
        camera.projection_matrix_set( proj ).expect( "resize produced a degenerate projection matrix" );
        camera.window_size_set( [ w as f32, h as f32 ].into() );

        renderer.borrow_mut().resize( &gl, w, h, samples ).expect( "Failed to resize renderer" );

        swap_buffer.gl_resources_free( &gl );
        swap_buffer = SwapFramebuffer::new( &gl, w, h );

        *prev = ( w, h );
      }

      if let Some( scene ) = state.scene.borrow().as_ref()
      {
        let mut scene = scene.borrow_mut();
        renderer.borrow_mut().render( &gl, &mut scene, &camera ).expect( "Failed to render" );
      }

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

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

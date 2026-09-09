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
  post_processing::{ self, Pass, SwapFramebuffer }, Camera, DirectLight, Light, Node, Object3D, PointLight,
  Renderer, Scene
};
use renderer::webgl::loaders::openpbr_mtlx::openpbr_surfaces_from_mtlx;
use renderer::webgl::material::OpenPbrSurface;

mod lil_gui;
mod gui_setup;
mod openpbr_scene;

/// Viewer modes.
const MODE_GLTF : &str = "gltf";
const MODE_OPENPBR : &str = "openpbr";
/// Scene built procedurally into `.usda` text and loaded through the real
/// `loaders::usd` pipeline ( composition + GL assembly ), with the material
/// arriving through a referenced `.mtlx` ( native OpenPBR lane ).
const MODE_USD : &str = "usd";

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
  /// OpenPBR material key ( used only in OpenPBR mode ), see
  /// [`OPENPBR_MATERIALS`].
  material : String,
}

/// OpenPBR test materials: ( display name, key ) — each is an embedded ASWF
/// `open_pbr_*.mtlx` sample or a scalar-only file from the OpenPBR Shader
/// Playground ( see `materials/ATTRIBUTION.md` ).
const OPENPBR_MATERIALS : &[ ( &str, &str ) ] =
&[
  ( "Velvet (fuzz)", "velvet" ),
  ( "Gold (metal)", "gold" ),
  ( "Glass (ior / transmission)", "glass" ),
  ( "Iridescent metal (thin film)", "iridescent" ),
  ( "Cord (coat)", "cord" ),
  ( "Straw (ior 2.83)", "straw" ),
  ( "Yellow paint", "yellowPaint" ),
];

/// Embedded `.mtlx` text for `key` ( ASWF spec examples + scalar-only material
/// files from the OpenPBR Shader Playground - see `materials/ATTRIBUTION.md` ).
#[ must_use ]
fn openpbr_material_mtlx( key : &str ) -> &'static str
{
  match key
  {
    "gold" => include_str!( "../materials/open_pbr_gold.mtlx" ),
    "glass" => include_str!( "../materials/open_pbr_glass.mtlx" ),
    "iridescent" => include_str!( "../materials/open_pbr_iridescent.mtlx" ),
    "cord" => include_str!( "../materials/playground_cord.mtlx" ),
    "straw" => include_str!( "../materials/playground_straw.mtlx" ),
    "yellowPaint" => include_str!( "../materials/playground_yellowPaint.mtlx" ),
    _ => include_str!( "../materials/open_pbr_velvet.mtlx" ),
  }
}

/// Parses the embedded `.mtlx` for `key` into a canonical surface.
#[ must_use ]
fn openpbr_material_surface( key : &str ) -> OpenPbrSurface
{
  let xml = openpbr_material_mtlx( key );
  let mut surfaces = openpbr_surfaces_from_mtlx( xml ).expect( "embedded OpenPBR material parses" );
  surfaces.pop().expect( "material file contains one surface" )
}

/// The renderer's shared material cell ( `Rc<RefCell<Box<dyn Material>>>` ).
type DynMaterial = Rc< RefCell< Box< dyn renderer::webgl::material::Material > > >;

struct ViewerState
{
  choice : RefCell< ViewerChoice >,
  scene : RefCell< Option< Rc< RefCell< Scene > > > >,
  /// Live-editable OpenPBR surface ( OpenPBR test mode ).
  surface : RefCell< Option< OpenPbrSurface > >,
  /// The material currently driving the sphere ( OpenPBR test mode ).
  material : RefCell< Option< DynMaterial > >,
}

/// Re-applies the edited surface to the live material ( marks uniforms for
/// re-upload ).
fn surface_apply( state : &Rc< ViewerState > )
{
  let material = state.material.borrow().clone();
  let surface = state.surface.borrow().clone();
  if let ( Some( material ), Some( surface ) ) = ( material, surface )
  {
    let mut m = renderer::webgl::cast_unchecked_material_to_ref_mut::< renderer::webgl::material::PbrMaterial >( material.borrow_mut() );
    m.openpbr_surface_apply( &surface );
  }
}

/// Adds one slider that edits a scalar field of the live OpenPBR surface.
/// `range` is `( min, max, step )`.
fn surface_slider
(
  state : &Rc< ViewerState >,
  js_object : &Object,
  gui : &JsValue,
  name : &str,
  default : f32,
  range : ( f64, f64, f64 ),
  set : fn( &mut OpenPbrSurface, f32 )
)
{
  let ( min, max, step ) = range;
  Reflect::set( js_object, &JsValue::from_str( name ), &JsValue::from_f64( f64::from( default ) ) ).unwrap();

  let prop = lil_gui::slider_add( gui, js_object, name, min, max, step );
  let callback =
  {
    let state = state.clone();
    Closure::new( move | value : f32 |
    {
      if let Some( surface ) = state.surface.borrow_mut().as_mut()
      {
        set( surface, value );
      }
      surface_apply( &state );
    } )
  };
  lil_gui::on_change( &prop, &callback );
  callback.forget();
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

/// Wraps a light in a node and adds it to the scene.
fn light_add( scene : &Rc< RefCell< Scene > >, light : Light ) -> Rc< RefCell< Node > >
{
  let node = Rc::new( RefCell::new( Node::new() ) );
  node.borrow_mut().object = Object3D::Light( light );
  scene.borrow_mut().children.push( node.clone() );
  node
}

/// Generates the Kulla–Conty energy-compensation LUT and gives it to the
/// renderer ( used by OpenPBR materials for direct-light multi-scattering ).
fn kulla_conty_setup( renderer : &mut Renderer, gl : &gl::WebGl2RenderingContext ) -> Result< (), gl::WebglError >
{
  let lut = renderer::webgl::loaders::kulla_conty::kulla_conty_lut_upload( gl, 32, 32, 512 )?;
  renderer.kulla_conty_lut_set( lut );
  Ok( () )
}

/// The OpenPBR / USD test rig: hemisphere-ish ambient ( two opposite direct
/// lights whose diffuse contribution scales with NoL — top = cool sky, bottom
/// = warm ground bounce, blending smoothly over the sphere ) plus a single key
/// point light. No env reflection, so the material's own response is readable.
fn studio_rig_add( scene : &Rc< RefCell< Scene > > )
{
  light_add( scene, Light::Direct( DirectLight { direction : gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] ), color : [ 0.8, 0.85, 1.0 ].into(), strength : 0.55 } ) );
  light_add( scene, Light::Direct( DirectLight { direction : gl::math::F32x3::from( [ 0.0, -1.0, 0.0 ] ), color : [ 0.3, 0.24, 0.2 ].into(), strength : 0.2 } ) );
  light_add( scene, Light::Point( PointLight { position : [ 1.5, 1.2, 1.6 ].into(), color : [ 1.0, 1.0, 1.0 ].into(), strength : 25.0, range : 8.0 } ) );
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

  if choice.mode == MODE_USD
  {
    // USD mode — the real `loaders::usd` pipeline on a multi-object set scene :
    // spheres + cubes under per-object `Xform`s ( translate / rotate / scale ops,
    // hierarchy composition ), five `.mtlx`-bound materials covering the distinct
    // OpenPBR carriers ( metal gold, coat cord, ior 2.8 straw, painted yellow,
    // fuzz velvet ) PLUS an inline `UsdPreviewSurface` - both material lanes in
    // one file. All fed as in-memory text ( the same shape `usd_scene_load_http`
    // builds from HTTP fetches ) through stage composition, mtlx binding
    // resolution and GL scene assembly. Studio rig, no env reflection.
    use openpbr_scene::UsdSetObject;

    let objects =
    [
      UsdSetObject { mesh: "sphere", material: "./gold.mtlx",        translate: [ -2.4, 0.0, 0.0 ], rotate_deg: [ 0.0, 0.0, 0.0  ], scale: 1.0 },
      UsdSetObject { mesh: "sphere", material: "./cord.mtlx",        translate: [ -1.4, 0.0, 0.5 ], rotate_deg: [ 0.0, 30.0, 0.0 ], scale: 0.8 },
      UsdSetObject { mesh: "sphere", material: "./straw.mtlx",       translate: [ 0.0, 0.0, 0.0  ], rotate_deg: [ 0.0, 0.0, 0.0  ], scale: 1.2 },
      UsdSetObject { mesh: "cube",   material: "./yellowPaint.mtlx", translate: [ 1.4, 0.0, -0.4 ], rotate_deg: [ 0.0, 0.0, 18.0 ], scale: 1.0 },
      UsdSetObject { mesh: "sphere", material: "./velvet.mtlx",      translate: [ 2.4, 0.0, 0.6  ], rotate_deg: [ 0.0, 0.0, 0.0  ], scale: 1.0 },
      UsdSetObject { mesh: "cube",   material: "preview",            translate: [ 1.0, 0.0, 1.6  ], rotate_deg: [ 25.0, 0.0, 0.0 ], scale: 0.7 },
      // §3.3 transmission test : glass ( transmission_weight = 1 ) IN FRONT of
      // the row, so it should visibly bend the image of the objects behind it.
      UsdSetObject { mesh: "sphere", material: "./glass.mtlx",       translate: [ -0.7, 0.0, 1.7 ], rotate_deg: [ 0.0, 0.0, 0.0  ], scale: 1.0 },
    ];
    let root = openpbr_scene::usd_set_scene_text( &objects );
    let assets =
    [
      ( "./gold.mtlx", openpbr_material_mtlx( "gold" ) ),
      ( "./cord.mtlx", openpbr_material_mtlx( "cord" ) ),
      ( "./straw.mtlx", openpbr_material_mtlx( "straw" ) ),
      ( "./yellowPaint.mtlx", openpbr_material_mtlx( "yellowPaint" ) ),
      ( "./velvet.mtlx", openpbr_material_mtlx( "velvet" ) ),
      ( "./glass.mtlx", openpbr_material_mtlx( "glass" ) ),
    ];
    let scene = renderer::webgl::loaders::usd::usd_scene_from_texts( gl, "scene.usda", &root, &assets )
    .map_err( | e |
    {
      gl::browser::error!( "USD scene load failed: {e:?}" );
      gl::WebglError::Other( "Failed to load USD scene" )
    })?;

    // The USD materials never went through `openpbr_surface_apply`'s IBL opt-out;
    // disable env sampling on every mesh material, like the OpenPBR mode does.
    let mut disable_ibl = | node : Rc< RefCell< Node > > | -> Result< (), gl::WebglError >
    {
      if let Object3D::Mesh( mesh ) = &node.borrow().object
      {
        for primitive in &mesh.borrow().primitives
        {
          let material_rc = primitive.borrow().material.clone();
          let mut m = renderer::webgl::cast_unchecked_material_to_ref_mut::< renderer::webgl::material::PbrMaterial >( material_rc.borrow_mut() );
          m.need_use_ibl_set( false );
        }
      }
      Ok( () )
    };
    scene.borrow().traverse( &mut disable_ibl )?;
    studio_rig_add( &scene );
    scene_fit_to_view( &scene );
    // No live-parameter surface binding yet ( the material lives behind the
    // usd assembly ); the sliders are re-enabled per mode in `gui_setup`.
    *state.material.borrow_mut() = None;
    *state.surface.borrow_mut() = None;
    *state.scene.borrow_mut() = Some( scene );
    return Ok( () );
  }

  if choice.mode == MODE_OPENPBR
  {
    // OpenPBR test mode — render the icosphere with a real `.mtlx` surface
    // through the OpenPbrSurface → PbrMaterial runtime bridge, lit by a small
    // studio rig ( no env reflection, so the material's own response is easy
    // to read ).
    let surface = openpbr_material_surface( &choice.material );
    let gltf = openpbr_scene::sphere_with_surface( gl, &surface );

    if let Some( material ) = gltf.materials.first()
    {
      let material_rc = material.clone();
      {
        let mut material = renderer::webgl::cast_unchecked_material_to_ref_mut::< renderer::webgl::material::PbrMaterial >( material_rc.borrow_mut() );
        material.need_use_ibl_set( false );
      }
      *state.material.borrow_mut() = Some( material_rc );
      *state.surface.borrow_mut() = Some( surface );
    }

    let scene = gltf.scenes.into_iter().next().expect( "sphere scene exists" );

    studio_rig_add( &scene );

    scene_fit_to_view( &scene );
    *state.scene.borrow_mut() = Some( scene );
    return Ok( () );
  }

  let gltf = renderer::webgl::loaders::gltf::load( document, &choice.model, gl ).await?;
  let scene = gltf.scenes.into_iter().next().expect( "gltf has one scene" );
  scene_fit_to_view( &scene );
  *state.material.borrow_mut() = None;
  *state.surface.borrow_mut() = None;
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
  Reflect::set( &js_object, &JsValue::from_str( "material" ), &JsValue::from_str( &state.choice.borrow().material ) ).unwrap();
  Reflect::set( &js_object, &JsValue::from_str( "mode" ), &JsValue::from_str( &state.choice.borrow().mode ) ).unwrap();

  let gui = lil_gui::gui_new();
  let folder = lil_gui::folder_add( &gui, "Debug" );

  // Viewer mode ( glTF viewer / OpenPBR test ).
  let mode_map = Object::new();
  Reflect::set( &mode_map, &JsValue::from_str( "glTF model viewer" ), &JsValue::from_str( MODE_GLTF ) ).unwrap();
  Reflect::set( &mode_map, &JsValue::from_str( "OpenPBR test" ), &JsValue::from_str( MODE_OPENPBR ) ).unwrap();
  Reflect::set( &mode_map, &JsValue::from_str( "USD scene ( loaders::usd )" ), &JsValue::from_str( MODE_USD ) ).unwrap();
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

  // OpenPBR material ( OpenPBR test mode ).
  let material_map = Object::new();
  for ( name, key ) in OPENPBR_MATERIALS
  {
    Reflect::set( &material_map, &JsValue::from_str( name ), &JsValue::from_str( key ) ).unwrap();
  }
  let material_gui = lil_gui::dropdown_add( &folder, &js_object, "material", &material_map );
  let callback =
  {
    let state = state.clone();
    let document = document.clone();
    let gl = gl.clone();
    Closure::new( move | value : JsValue |
    {
      if state.choice.borrow().mode != MODE_OPENPBR
      {
        return;
      }
      if let Some( key ) = value.as_string()
      {
        state.choice.borrow_mut().material = key;
        reload( &state, &document, &gl );
      }
    } )
  };
  lil_gui::on_finish_change( &material_gui, &callback );
  callback.forget();

  // Live OpenPBR surface parameters ( only active in OpenPBR test mode ).
  let params_folder = lil_gui::folder_add( &folder, "material params" );
  surface_slider( state, &js_object, &params_folder, "roughness", 1.0, ( 0.0, 1.0, 0.01 ), | s, v | s.specular_roughness = v );
  surface_slider( state, &js_object, &params_folder, "metalness", 0.0, ( 0.0, 1.0, 0.01 ), | s, v | s.base_metalness = v );
  surface_slider( state, &js_object, &params_folder, "specularIor", 1.5, ( 1.0, 2.5, 0.01 ), | s, v | s.specular_ior = v );
  surface_slider( state, &js_object, &params_folder, "clearcoat", 0.0, ( 0.0, 1.0, 0.01 ), | s, v | s.coat_weight = v );
  surface_slider( state, &js_object, &params_folder, "clearcoatRoughness", 0.0, ( 0.0, 1.0, 0.01 ), | s, v | s.coat_roughness = v );
  surface_slider( state, &js_object, &params_folder, "fuzzWeight", 0.5, ( 0.0, 1.0, 0.01 ), | s, v | s.fuzz_weight = v );
  surface_slider( state, &js_object, &params_folder, "fuzzRoughness", 0.5, ( 0.0, 1.0, 0.01 ), | s, v | s.fuzz_roughness = v );
  surface_slider( state, &js_object, &params_folder, "thinFilmWeight", 0.0, ( 0.0, 1.0, 0.01 ), | s, v | s.thin_film_weight = v );
  surface_slider( state, &js_object, &params_folder, "thinFilmThicknessNm", 450.0, ( 100.0, 1000.0, 5.0 ), | s, v | s.thin_film_thickness = v / 1000.0 );
  surface_slider( state, &js_object, &params_folder, "thinFilmIor", 1.4, ( 1.0, 2.0, 0.01 ), | s, v | s.thin_film_ior = v );

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
      choice : RefCell::new
      (
        ViewerChoice
        {
          mode : MODE_GLTF.to_string(),
          model : default_model.to_string(),
          material : "velvet".to_string(),
        }
      ),
      scene : RefCell::new( None ),
      surface : RefCell::new( None ),
      material : RefCell::new( None ),
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
  kulla_conty_setup( &mut renderer, &gl )?;
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

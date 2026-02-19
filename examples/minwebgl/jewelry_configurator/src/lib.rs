//! Jewelry configurator — WebGL renderer library.
//!
//! Provides [`JewelryRenderer`] for loading GLTF jewelry models and rendering them
//! with physically-based materials, image-based lighting, gem ray-tracing, soft shadows,
//! and tonemapping post-processing.
//!
//! Also exposes [`JewelryConfig`] for controlling gem color, metal color, exposure,
//! roughness, and metalness — usable from both Rust and JavaScript via wasm-bindgen.

#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::unsafe_derive_deserialize ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]

mod gem_material;
mod cube_normal_map_generator;
mod surface_material;
mod helpers;

use minwebgl as gl;
use gl::
{
  GL,
  F32x3,
  JsCast,
  canvas::HtmlCanvasElement,
  context::{ ContextOptions, PowerPreference },
  log,
  web_sys
};
use renderer::webgl::
{
  Camera,
  Node,
  Renderer,
  Scene,
  TextureInfo,
  loaders::{ gltf, ibl },
  post_processing,
  shadow::{ ShadowBaker, ShadowMap },
};
use gltf::GLTF;
use post_processing::{ Pass, SwapFramebuffer, ToneMappingPass, ToneMappingAces, ToSrgbPass };
use rustc_hash::FxHashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;
use serde::{ Serialize, Deserialize };
use std::rc::Rc;
use core::cell::RefCell;
use web_sys::ResizeObserver;
use cube_normal_map_generator::CubeNormalMapGenerator;
use helpers::
{
  CAMERA_FOV,
  CAMERA_NEAR,
  CAMERA_FAR,
  SHADOW_RESOLUTION,
  setup_camera,
  configure_metal_materials,
  normalize_scene_transform,
  add_ground_plane,
  generate_gem_normal_maps_and_apply,
  create_environment_texture,
  apply_gem_color,
  apply_plane_color,
};

/// Temporary brightness scale applied to gem color before uploading to the shader.
/// The gem rendering pipeline currently loses intensity somewhere, so we compensate here.
/// TODO: remove once the root cause in the pipeline is identified and fixed.
const GEM_COLOR_SHADER_SCALE : f32 = 1.7;

/// Global configuration applied to all jewelry items.
/// JS-compatible: all fields are accessible via getter/setter properties,
/// color arrays are exposed as JS arrays through `gem_color` / `metal_color` properties.
#[ wasm_bindgen ]
#[ derive( Clone, Copy, PartialEq, Serialize, Deserialize ) ]
pub struct JewelryConfig
{
  /// Gem (diamond) RGB color (0–1 range).
  gem_color : [ f32; 3 ],
  /// Metal base RGB color.
  metal_color : [ f32; 3 ],
  /// Background clear color (single value, applied as RGB splat).
  clear_color : f32,
  /// Renderer exposure.
  exposure : f32,
  /// Metal roughness.
  roughness : f32,
  /// Metal metalness.
  metalness : f32,
}

#[ wasm_bindgen ]
impl JewelryConfig
{
  /// Creates a config with default values.
  #[ must_use ]
  #[ wasm_bindgen( constructor ) ]
  pub fn new() -> Self { Self::default() }

  /// Gem color as a JS array (0–1 per channel).
  #[ must_use ]
  #[ wasm_bindgen( getter ) ]
  pub fn gem_color( &self ) -> Vec< f32 > { self.gem_color.to_vec() }

  /// Sets gem color from a JS array (0–1 per channel).
  #[ wasm_bindgen( setter ) ]
  pub fn set_gem_color( &mut self, v : &[ f32 ] )
  {
    if v.len() >= 3 { self.gem_color = [ v[ 0 ], v[ 1 ], v[ 2 ] ]; }
  }

  /// Metal color as a JS array (0–1 per channel).
  #[ must_use ]
  #[ wasm_bindgen( getter ) ]
  pub fn metal_color( &self ) -> Vec< f32 > { self.metal_color.to_vec() }

  /// Sets metal color from a JS array (0–1 per channel).
  #[ wasm_bindgen( setter ) ]
  pub fn set_metal_color( &mut self, v : &[ f32 ] )
  {
    if v.len() >= 3 { self.metal_color = [ v[ 0 ], v[ 1 ], v[ 2 ] ]; }
  }

  /// Background clear color (single value, applied as RGB splat).
  #[ must_use ]
  #[ wasm_bindgen( getter ) ]
  pub fn clear_color( &self ) -> f32 { self.clear_color }

  /// Sets background clear color.
  #[ wasm_bindgen( setter ) ]
  pub fn set_clear_color( &mut self, v : f32 ) { self.clear_color = v; }

  /// Renderer exposure.
  #[ must_use ]
  #[ wasm_bindgen( getter ) ]
  pub fn exposure( &self ) -> f32 { self.exposure }

  /// Sets renderer exposure.
  #[ wasm_bindgen( setter ) ]
  pub fn set_exposure( &mut self, v : f32 ) { self.exposure = v; }

  /// Metal roughness.
  #[ must_use ]
  #[ wasm_bindgen( getter ) ]
  pub fn roughness( &self ) -> f32 { self.roughness }

  /// Sets metal roughness.
  #[ wasm_bindgen( setter ) ]
  pub fn set_roughness( &mut self, v : f32 ) { self.roughness = v; }

  /// Metal metalness.
  #[ must_use ]
  #[ wasm_bindgen( getter ) ]
  pub fn metalness( &self ) -> f32 { self.metalness }

  /// Sets metal metalness.
  #[ wasm_bindgen( setter ) ]
  pub fn set_metalness( &mut self, v : f32 ) { self.metalness = v; }
}

impl JewelryConfig
{
  /// Returns gem color pre-scaled for the shader. See [`GEM_COLOR_SHADER_SCALE`].
  fn gem_color_for_shader( &self ) -> F32x3
  {
    F32x3::from( self.gem_color ) * GEM_COLOR_SHADER_SCALE
  }
}

impl Default for JewelryConfig
{

  fn default() -> Self
  {
    Self
    {
      gem_color   : [ 1.0, 1.0, 1.0 ],
      metal_color : [ 0.85, 0.85, 0.854 ],
      clear_color : 2.7,
      exposure    : 1.0,
      roughness   : 0.01,
      metalness   : 0.93,
    }
  }
}

/// GPU resources that need to be updated on resize.
struct RenderPipeline
{
  renderer : Renderer,
  swap_buffer : SwapFramebuffer,
  camera : Camera,
  tonemapping : ToneMappingPass< ToneMappingAces >,
  to_srgb : ToSrgbPass,
}

struct JewelryItem
{
  scene : Rc< RefCell< Scene > >,
  gems : FxHashMap< String, Rc< RefCell< Node > > >,
}

/// Jewelry renderer that can be used from both Rust and JavaScript.
#[ wasm_bindgen ]
pub struct JewelryRenderer
{
  gl : GL,
  config : JewelryConfig,
  pipeline : Rc< RefCell< RenderPipeline > >,
  loaded_gltfs : FxHashMap< Box< str >, JewelryItem >,
  environment_texture : Option< TextureInfo >,
  cube_normal_map_generator : Option< CubeNormalMapGenerator >,
  plane_template : Option< Rc< RefCell< Node > > >,
  shadow_map : Option< ShadowMap >,
  shadow_baker : Option< ShadowBaker >,
  _resize_observer : ResizeObserver,
  _resize_closure : Closure< dyn FnMut( js_sys::Array ) >,
}

impl JewelryRenderer
{
  /// Creates and fully initialises a jewelry renderer.
  /// Loads IBL at `ibl_path` and (if non-empty) a gem HDR environment at `gem_env_path`.
  pub async fn new( canvas : &HtmlCanvasElement, ibl_path : &str, gem_env_path : &str ) -> Self
  {
    let gl = create_gl( canvas );
    let config = JewelryConfig::default();
    let pipeline = create_pipeline( &gl, canvas, &config );
    let ( resize_observer, resize_closure ) = setup_resize_observer( canvas, &gl, &pipeline );

    let mut this = Self
    {
      config,
      pipeline,
      gl,
      loaded_gltfs : FxHashMap::default(),
      environment_texture : None,
      cube_normal_map_generator : None,
      plane_template : None,
      shadow_map : None,
      shadow_baker : None,
      _resize_observer : resize_observer,
      _resize_closure : resize_closure,
    };

    this.load_ibl( ibl_path ).await;

    if !gem_env_path.is_empty()
    {
      this.environment_texture = create_environment_texture( &this.gl, gem_env_path ).await;
    }

    this.init_gpu_resources();
    this.load_plane_template().await;

    this
  }

  /// Loads IBL data and applies initial renderer settings.
  async fn load_ibl( &mut self, ibl_path : &str )
  {
    let ibl_data = ibl::load( &self.gl, ibl_path, None ).await;
    let mut pipeline = self.pipeline.borrow_mut();
    pipeline.renderer.set_ibl( ibl_data );
    pipeline.renderer.set_skybox( None );
    pipeline.renderer.set_use_emission( true );
    pipeline.renderer.set_bloom_strength( 2.0 );
    pipeline.renderer.set_bloom_radius( 0.1 );
  }

  /// Creates GPU resources: cube normal map generator, shadow map, and shadow baker.
  fn init_gpu_resources( &mut self )
  {
    match CubeNormalMapGenerator::new( &self.gl )
    {
      Ok( generator ) => self.cube_normal_map_generator = Some( generator ),
      Err( err ) => log::error!( "Failed to create CubeNormalMapGenerator: {err}" ),
    }

    match ShadowMap::new( &self.gl, SHADOW_RESOLUTION )
    {
      Ok( sm ) => self.shadow_map = Some( sm ),
      Err( err ) => log::error!( "Failed to create ShadowMap: {err}" ),
    }

    match ShadowBaker::new( &self.gl )
    {
      Ok( sb ) => self.shadow_baker = Some( sb ),
      Err( err ) => log::error!( "Failed to create ShadowBaker: {err}" ),
    }
  }

  /// Loads the ground plane GLB and stores the `Plane` node as a reusable template.
  async fn load_plane_template( &mut self )
  {
    let document = web_sys::window().expect( "Should have a window" ).document().expect( "Should have a document" );
    match gltf::load( &document, "gltf/plane.glb", &self.gl ).await
    {
      Ok( plane_gltf ) => self.plane_template = plane_gltf.scenes[ 0 ].borrow().get_node( "Plane" ),
      Err( err ) => log::error!( "Failed to load plane: {err}" ),
    }
  }
}

#[ wasm_bindgen ]
impl JewelryRenderer
{
  /// Loads a GLTF jewelry model from a URL, then processes it.
  /// Returns `true` on success, `false` if loading failed.
  pub async fn load_jewelry_gltf( &mut self, path : &str ) -> bool
  {
    let document = web_sys::window().expect( "Should have a window" ).document().expect( "Should have a document" );
    let gltf = match gltf::load( &document, path, &self.gl ).await
    {
      Ok( gltf ) => gltf,
      Err( err ) => { log::error!( "{err}" ); return false; }
    };

    self.process_jewelry_gltf( path, &gltf );
    true
  }

  /// Processes a loaded GLTF jewelry model: configures materials, normalizes
  /// transforms, adds ground plane with shadow, and sets up gem materials.
  fn process_jewelry_gltf( &mut self, key : &str, gltf : &GLTF )
  {
    if gltf.scenes.is_empty()
    {
      log::error!( "GLTF '{key}' contains no scenes" );
      return;
    }
    let scene = &gltf.scenes[ 0 ];

    configure_metal_materials( scene, &self.config );
    normalize_scene_transform( scene );
    add_ground_plane
    (
      &self.gl,
      gltf,
      scene,
      &self.plane_template,
      &self.shadow_map,
      &self.shadow_baker,
      self.config.clear_color()
    );

    scene.borrow_mut().update_world_matrix();

    let gems = generate_gem_normal_maps_and_apply
    (
      &self.gl,
      scene,
      &self.cube_normal_map_generator,
      &self.environment_texture,
      self.config.gem_color_for_shader(),
    );

    self.loaded_gltfs.insert( key.into(), JewelryItem { scene : scene.clone(), gems } );
  }

  /// Renders a loaded jewelry scene with post-processing (tonemapping + sRGB conversion).
  pub fn render_jewelry( &mut self, name : &str, delta_time : f64 )
  {
    let Some( jewelry ) = self.loaded_gltfs.get( name )
    else
    {
      log::warn!( "render_jewelry: '{name}' is not loaded" );
      return;
    };
    let mut scene = jewelry.scene.borrow_mut();

    let mut pipeline = self.pipeline.borrow_mut();
    let pipeline : &mut RenderPipeline = &mut pipeline;
    pipeline.camera.update( delta_time );

    if let Err( err ) = pipeline.renderer.render( &self.gl, &mut scene, &pipeline.camera )
    {
      log::error!( "{err}" );
      return;
    }

    pipeline.swap_buffer.reset();
    pipeline.swap_buffer.bind( &self.gl );
    let tex = pipeline.renderer.get_main_texture();
    pipeline.swap_buffer.set_input( tex );

    match pipeline.tonemapping.render( &self.gl, pipeline.swap_buffer.get_input(), pipeline.swap_buffer.get_output() )
    {
      Ok( t ) =>
      {
        pipeline.swap_buffer.set_output( t );
        pipeline.swap_buffer.swap();
      }
      Err( err ) => { log::error!( "Tonemapping: {err}" ); return; }
    }

    if let Err( err ) = pipeline.to_srgb.render( &self.gl, pipeline.swap_buffer.get_input(), pipeline.swap_buffer.get_output() )
    {
      log::error!( "ToSrgb: {err}" );
    }
  }

  /// Returns a copy of the current configuration.
  #[ must_use ]
  pub fn config( &self ) -> JewelryConfig { self.config }

  /// Replaces the entire config and re-applies to the renderer and all loaded scenes.
  pub fn update_config( &mut self, config : JewelryConfig )
  {
    self.config = config;
    self.apply_config();
  }

  fn apply_config( &mut self )
  {
    {
      let mut pipeline = self.pipeline.borrow_mut();
      pipeline.renderer.set_clear_color( F32x3::splat( self.config.clear_color() ) );
      pipeline.renderer.set_exposure( self.config.exposure() );
    }

    for item in self.loaded_gltfs.values()
    {
      configure_metal_materials( &item.scene, &self.config );
      apply_gem_color( &item.gems, self.config.gem_color_for_shader() );
      apply_plane_color( &item.scene, self.config.clear_color() );
    }
  }
}

/// Sets up a `ResizeObserver` on the canvas that updates renderer, swap buffer and camera.
/// Creates a WebGL2 context from the canvas with high-performance settings and enables float texture extensions.
///
/// # Panics
/// Panics if the WebGL2 context cannot be created.
fn create_gl( canvas : &HtmlCanvasElement ) -> GL
{
  let options = ContextOptions
  {
    alpha : false,
    antialias : false,
    depth : false,
    stencil : false,
    power_preference : PowerPreference::HighPerformance,
    ..Default::default()
  };

  let gl = match gl::context::from_canvas_with( canvas, options )
  {
    Ok( gl ) => gl,
    Err( err ) => { log::error!( "{err}" ); panic!() }
  };

  _ = gl.get_extension( "EXT_color_buffer_float" );
  _ = gl.get_extension( "EXT_color_buffer_half_float" );

  gl
}

/// Builds the rendering pipeline: renderer, camera, swap buffer, tonemapping and sRGB passes.
///
/// # Panics
/// Panics if the renderer or post-processing passes cannot be created.
fn create_pipeline( gl : &GL, canvas : &HtmlCanvasElement, config : &JewelryConfig ) -> Rc< RefCell< RenderPipeline > >
{
  let width = canvas.width();
  let height = canvas.height();

  let mut renderer = match Renderer::new( gl, width, height, 2 )
  {
    Ok( renderer ) => renderer,
    Err( err ) => { log::error!( "{err}" ); panic!() }
  };
  renderer.set_clear_color( F32x3::splat( config.clear_color() ) );
  renderer.set_exposure( config.exposure() );

  let camera = setup_camera( canvas );
  let swap_buffer = SwapFramebuffer::new( gl, width, height );

  let tonemapping = ToneMappingPass::< ToneMappingAces >::new( gl )
  .expect( "Failed to create tonemapping pass" );

  let to_srgb = ToSrgbPass::new( gl, true )
  .expect( "Failed to create sRGB pass" );

  Rc::new( RefCell::new( RenderPipeline { renderer, swap_buffer, camera, tonemapping, to_srgb } ) )
}

fn setup_resize_observer
(
  canvas : &HtmlCanvasElement,
  gl : &GL,
  pipeline : &Rc< RefCell< RenderPipeline > >
) -> ( ResizeObserver, Closure< dyn FnMut( js_sys::Array ) > )
{
  let canvas_clone = canvas.clone();
  let gl = gl.clone();
  let pipeline = pipeline.clone();

  let closure = Closure::< dyn FnMut( js_sys::Array ) >::new
  (
    move | entries : js_sys::Array |
    {
      let Some( entry ) = entries.get( 0 ).dyn_ref::< web_sys::ResizeObserverEntry >().cloned()
      else
      {
        return;
      };

      let rect = entry.content_rect();
      let width = rect.width() as u32;
      let height = rect.height() as u32;
      if width == 0 || height == 0 { return; }

      canvas_clone.set_width( width );
      canvas_clone.set_height( height );

      let mut pipeline = pipeline.borrow_mut();

      if pipeline.renderer.resize( &gl, width, height, 2 ).is_ok()
      {
        pipeline.swap_buffer = SwapFramebuffer::new( &gl, width, height );

        let w = width as f32;
        let h = height as f32;
        let aspect = w / h;
        let perspective = gl::math::d2::mat3x3h::perspective_rh_gl( CAMERA_FOV, aspect, CAMERA_NEAR, CAMERA_FAR );
        pipeline.camera.set_window_size( [ w, h ].into() );
        pipeline.camera.set_projection_matrix( perspective );
      }
    }
  );

  let observer = ResizeObserver::new( closure.as_ref().unchecked_ref() )
  .expect( "Failed to create ResizeObserver" );

  observer.observe( canvas );

  ( observer, closure )
}

/// Creates and fully initialises a `JewelryRenderer`. JavaScript entry point.
/// Loads IBL at `ibl_path` and (if non-empty) a gem HDR environment at `gem_env_path`.
#[ wasm_bindgen ]
pub async fn create( canvas : HtmlCanvasElement, ibl_path : String, gem_env_path : String ) -> JewelryRenderer
{
  JewelryRenderer::new( &canvas, &ibl_path, &gem_env_path ).await
}

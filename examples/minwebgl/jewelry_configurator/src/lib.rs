//! Jewelry configurator

mod gem_material;
mod cube_normal_map_generator;
mod surface_material;
/// Scene processing helpers and configuration.
pub mod helpers;

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
use std::{ cell::RefCell, rc::Rc };
use web_sys::ResizeObserver;
use cube_normal_map_generator::CubeNormalMapGenerator;
use helpers::
{
  JewelryConfig,
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
  canvas : HtmlCanvasElement,
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

#[ wasm_bindgen ]
impl JewelryRenderer
{
  /// Create a new jewelry renderer instance.
  #[ wasm_bindgen( constructor ) ]
  pub fn new( canvas : &HtmlCanvasElement ) -> Self
  {
    let width = canvas.width();
    let height = canvas.height();

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

    let config = JewelryConfig::default();

    let mut renderer = match Renderer::new( &gl, width, height, 2 )
    {
      Ok( renderer ) => renderer,
      Err( err ) => { log::error!( "{err}" ); panic!() }
    };
    renderer.set_clear_color( config.clear_color );
    renderer.set_exposure( config.exposure );

    let camera = setup_camera( canvas );

    let swap_buffer = SwapFramebuffer::new( &gl, width, height );

    let tonemapping = ToneMappingPass::< ToneMappingAces >::new( &gl )
    .expect( "Failed to create tonemapping pass" );

    let to_srgb = ToSrgbPass::new( &gl, true )
    .expect( "Failed to create sRGB pass" );

    let pipeline = Rc::new( RefCell::new( RenderPipeline { renderer, swap_buffer, camera, tonemapping, to_srgb } ) );

    let ( resize_observer, resize_closure ) = setup_resize_observer( canvas, &gl, &pipeline );

    Self
    {
      canvas : canvas.clone(),
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
    }
  }

  /// Loads IBL environment maps, gem environment texture, and configures renderer settings.
  /// Must be called after construction and before rendering.
  /// `gem_env_path` is the path to the HDR environment map for gem reflections (empty string to skip).
  pub async fn init( &mut self, ibl_path : &str, gem_env_path : &str )
  {
    let ibl_data = ibl::load( &self.gl, ibl_path, None ).await;

    {
      let mut pipeline = self.pipeline.borrow_mut();
      pipeline.renderer.set_ibl( ibl_data );
      pipeline.renderer.set_skybox( None );
      pipeline.renderer.set_use_emission( true );
      pipeline.renderer.set_bloom_strength( 2.0 );
      pipeline.renderer.set_bloom_radius( 0.1 );
    }

    // Load gem environment texture if path provided
    if !gem_env_path.is_empty()
    {
      self.environment_texture = create_environment_texture( &self.gl, gem_env_path ).await;
    }

    // Create cube normal map generator
    match CubeNormalMapGenerator::new( &self.gl )
    {
      Ok( generator ) => self.cube_normal_map_generator = Some( generator ),
      Err( err ) => log::error!( "Failed to create CubeNormalMapGenerator: {err}" ),
    }

    // Create shadow resources
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

    // Load plane template
    let document = web_sys::window().expect( "Should have a window" ).document().expect( "Should have a document" );
    match gltf::load( &document, "gltf/plane.glb", &self.gl ).await
    {
      Ok( plane_gltf ) => self.plane_template = plane_gltf.scenes[ 0 ].borrow().get_node( "Plane" ),
      Err( err ) => log::error!( "Failed to load plane: {err}" ),
    }
  }

  /// Loads a GLTF jewelry model from a URL, then processes it.
  pub async fn load_jewelry_gltf( &mut self, path : &str )
  {
    let document = web_sys::window().expect( "Should have a window" ).document().expect( "Should have a document" );
    let gltf = match gltf::load( &document, path, &self.gl ).await
    {
      Ok( gltf ) => gltf,
      Err( err ) => { log::error!( "{err}" ); return; }
    };

    self.process_jewelry_gltf( path, &gltf );
  }

  /// Processes a loaded GLTF jewelry model: configures materials, normalizes
  /// transforms, adds ground plane with shadow, and sets up gem materials.
  fn process_jewelry_gltf( &mut self, key : &str, gltf : &GLTF )
  {
    let scene = &gltf.scenes[ 0 ];

    configure_metal_materials( scene, &self.config );
    normalize_scene_transform( scene );
    add_ground_plane
    (
      &self.gl,
      &gltf,
      scene,
      &self.plane_template,
      &self.shadow_map,
      &self.shadow_baker,
      self.config.clear_color
    );

    scene.borrow_mut().update_world_matrix();

    let gems = generate_gem_normal_maps_and_apply
    (
      &self.gl,
      scene,
      &self.cube_normal_map_generator,
      &self.environment_texture,
      self.config.gem_color,
    );

    self.loaded_gltfs.insert( key.into(), JewelryItem { scene : scene.clone(), gems } );
  }

  /// Renders a loaded jewelry scene with post-processing (tonemapping + sRGB conversion).
  pub fn render_jewelry( &mut self, name : &str, delta_time : f64 )
  {
    let Some( jewelry ) = self.loaded_gltfs.get( name ) else { return; };
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

  /// Sets gem (diamond) color and re-applies to all loaded items.
  pub fn set_gem_color( &mut self, r : f32, g : f32, b : f32 )
  {
    self.config.gem_color = F32x3::from_array( [ r, g, b ] );
    self.apply_config();
  }

  /// Sets metal color and re-applies to all loaded items.
  pub fn set_metal_color( &mut self, r : f32, g : f32, b : f32 )
  {
    self.config.metal_color = F32x3::from_array( [ r, g, b ] );
    self.apply_config();
  }

  /// Sets background clear color and re-applies to all loaded items.
  pub fn set_clear_color( &mut self, r : f32, g : f32, b : f32 )
  {
    self.config.clear_color = F32x3::from_array( [ r, g, b ] );
    self.apply_config();
  }

  /// Sets exposure and re-applies.
  pub fn set_exposure( &mut self, exposure : f32 )
  {
    self.config.exposure = exposure;
    self.apply_config();
  }

  /// Sets metal roughness and re-applies to all loaded items.
  pub fn set_roughness( &mut self, roughness : f32 )
  {
    self.config.roughness = roughness;
    self.apply_config();
  }

  /// Sets metal metalness and re-applies to all loaded items.
  pub fn set_metalness( &mut self, metalness : f32 )
  {
    self.config.metalness = metalness;
    self.apply_config();
  }

  /// Re-applies the current config to the renderer and all loaded items.
  fn apply_config( &mut self )
  {
    {
      let mut pipeline = self.pipeline.borrow_mut();
      pipeline.renderer.set_clear_color( self.config.clear_color );
      pipeline.renderer.set_exposure( self.config.exposure );
    }

    for item in self.loaded_gltfs.values()
    {
      configure_metal_materials( &item.scene, &self.config );
      apply_gem_color( &item.gems, self.config.gem_color );
      apply_plane_color( &item.scene, self.config.clear_color );
    }
  }
}

/// Sets up a `ResizeObserver` on the canvas that updates renderer, swap buffer and camera.
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

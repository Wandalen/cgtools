use minwebgl as gl;
use gl::
{
  GL,
  F32x3,
  log,
  math::mat3x3h,
};
use renderer::webgl::
{
  CullMode,
  AlphaMode,
  Camera,
  Material,
  Node,
  Object3D,
  Scene,
  Texture,
  TextureInfo,
  Sampler,
  WrappingMode,
  MinFilterMode,
  MagFilterMode,
  helpers,
  loaders::gltf,
  loaders::hdr_texture,
  material::PbrMaterial,
  shadow::{ ShadowMap, ShadowBaker, Light },
};
use rustc_hash::FxHashMap;
use std::{ cell::RefCell, rc::Rc };
use crate::cube_normal_map_generator::{ CubeNormalMapGenerator, CubeNormalData };
use crate::gem_material::GemMaterial;
use crate::surface_material::SurfaceMaterial;
use gl::canvas::HtmlCanvasElement;
use super::JewelryConfig;

pub const CAMERA_FOV : f32 = 0.6981317; // 40 degrees in radians, matches jewelry_3d_site
pub const CAMERA_NEAR : f32 = 0.1;
pub const CAMERA_FAR : f32 = 100.0;
pub const SHADOW_RESOLUTION : u32 = 2048;

/// Initializes a perspective camera with orbit controls configured for jewelry inspection.
pub fn setup_camera( canvas : &HtmlCanvasElement ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let eye = F32x3::from( [ 2.0, 2.0, 2.0 ] );
  let up = F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = F32x3::from( [ 0.0, 0.9, 0.0 ] );

  let aspect_ratio = width / height;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, CAMERA_FOV, CAMERA_NEAR, CAMERA_FAR );
  camera.set_window_size( [ width, height ].into() );
  camera.get_controls().borrow_mut().pan.enabled = false;
  camera.get_controls().borrow_mut().rotation.movement_smoothing_enabled = true;
  camera.get_controls().borrow_mut().rotation.speed = 50.0;
  camera.get_controls().borrow_mut().rotation.latitude_range_set( 90.0 );
  camera.get_controls().borrow_mut().zoom.min_distance_set( 2.0 );
  camera.get_controls().borrow_mut().zoom.max_distance_set( 6.0 );
  camera.bind_controls( canvas );

  camera
}

/// Traverses scene and configures PBR materials for metal parts.
pub fn configure_metal_materials( scene : &Rc< RefCell< Scene > >, config : &JewelryConfig )
{
  if let Err( err ) = scene.borrow().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let node = node.borrow();
      let Object3D::Mesh( mesh ) = &node.object else { return Ok( () ); };

      for primitive in &mesh.borrow().primitives
      {
        let material = &primitive.borrow().material;
        if material.borrow().type_name() != "PbrMaterial"
        {
          continue;
        }

        let mut material = helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >( material.borrow_mut() );

        material.base_color_factor = F32x3::from( config.metal_color ).to_homogenous();
        material.specular_factor = None;
        material.specular_color_factor = None;
        material.base_color_texture = None;
        material.metallic_roughness_texture = None;
        material.emissive_texture = None;
        material.specular_color_texture = None;
        material.specular_texture = None;
        material.cull_mode = Some( CullMode::Back );
        material.alpha_mode = AlphaMode::Opaque;
        material.roughness_factor = config.roughness;
        material.metallic_factor = config.metalness;
        material.needs_update = true;
      }

      Ok( () )
    }
  )
  {
    log::error!( "Error traversing scene: {err}" );
  }
}

/// Normalizes scale of scene children and positions them on the ground plane.
pub fn normalize_scene_transform( scene : &Rc< RefCell< Scene > > )
{
  for node in &scene.borrow().children
  {
    let mut node = node.borrow_mut();
    node.normalize_scale();
    node.compute_local_bounding_box();
    let bb = node.local_bounding_box_hierarchical();
    let t = mat3x3h::translation( [ 0.0, -bb.min.y() + 0.01, 0.0 ] ); // + 0.01 against z-fighting
    node.apply_matrix( t );
  }
}

/// Clones the plane template, adds it to the scene, and bakes a shadow onto it.
pub fn add_ground_plane
(
  gl : &GL,
  gltf : &gltf::GLTF,
  scene : &Rc< RefCell< Scene > >,
  plane_template : &Option< Rc< RefCell< Node > > >,
  shadow_map : &Option< ShadowMap >,
  shadow_baker : &Option< ShadowBaker >,
  clear_color : f32,
)
{
  let Some( plane_template ) = plane_template else { return; };

  let plane_node = plane_template.borrow().clone_tree();
  plane_node.borrow_mut().set_translation( F32x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
  plane_node.borrow_mut().set_scale( F32x3::from_array( [ 3.0, 1.0, 3.0 ] ) );
  scene.borrow_mut().add( plane_node.clone() );
  scene.borrow_mut().update_world_matrix();

  if let Err( err ) = bake_plane_shadow( gl, gltf, &plane_node, shadow_map, shadow_baker, clear_color )
  {
    log::error!( "Shadow baking failed: {err}" );
    setup_plane_material( gl, &plane_node, None, clear_color );
  }
}

/// Finds gem nodes, generates cube normal maps, and applies gem materials.
pub fn generate_gem_normal_maps_and_apply
(
  gl : &GL,
  scene : &Rc< RefCell< Scene > >,
  cube_normal_map_generator : &Option< CubeNormalMapGenerator >,
  environment_texture : &Option< TextureInfo >,
  gem_color : F32x3,
) -> FxHashMap< String, Rc< RefCell< Node > > >
{
  let gems = find_gem_nodes( scene );

  if let Some( generator ) = cube_normal_map_generator
  {
    for ( name, gem ) in &gems
    {
      let cube_normal_data = match generator.generate( gl, gem )
      {
        Ok( data ) => data,
        Err( err ) =>
        {
          log::error!( "Failed to generate cube normal map for {name}: {err}" );
          CubeNormalData::default()
        }
      };

      setup_gem_material( gl, gem, environment_texture, &cube_normal_data, gem_color );
    }
  }

  gems
}

/// Finds gem/diamond/crystal nodes in a scene by name substring (case-insensitive).
fn find_gem_nodes( scene : &Rc< RefCell< Scene > > ) -> FxHashMap< String, Rc< RefCell< Node > > >
{
  let keywords = [ "gem", "diamond", "crystal" ];
  let mut gems = FxHashMap::default();

  let _ = scene.borrow().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      if let Some( name ) = node.borrow().get_name()
      {
        let lower = name.to_lowercase();
        if keywords.iter().any( | kw | lower.contains( kw ) )
        {
          gems.insert( name.to_string(), node.clone() );
        }
      }
      Ok( () )
    }
  );

  gems
}

/// Creates an empty 2D texture with linear filtering, then loads HDR data into it.
pub async fn create_environment_texture( gl : &GL, hdr_path : &str ) -> Option< TextureInfo >
{
  let texture = gl.create_texture()?;

  let sampler = Sampler::former()
  .min_filter( MinFilterMode::Linear )
  .mag_filter( MagFilterMode::Linear )
  .wrap_r( WrappingMode::ClampToEdge )
  .wrap_s( WrappingMode::ClampToEdge )
  .wrap_t( WrappingMode::ClampToEdge )
  .end();

  let tex = Texture::former()
  .target( GL::TEXTURE_2D )
  .source( texture.clone() )
  .sampler( sampler )
  .end();

  let info = TextureInfo
  {
    texture : Rc::new( RefCell::new( tex ) ),
    uv_position : 0,
  };

  hdr_texture::load_to_mip_d2( gl, Some( &texture ), 0, hdr_path ).await;

  Some( info )
}

/// Replaces all materials on a gem mesh with a configured `GemMaterial`.
fn setup_gem_material
(
  gl : &GL,
  gem_node : &Rc< RefCell< Node > >,
  environment_texture : &Option< TextureInfo >,
  cube_normal_map_texture : &CubeNormalData,
  gem_color : F32x3,
)
{
  let Object3D::Mesh( mesh ) = &gem_node.borrow().object else { return; };
  let primitives = &mesh.borrow().primitives;
  let mut gem_material = GemMaterial::new( gl );
  gem_material.color = gem_color;
  gem_material.cube_normal_map_texture = cube_normal_map_texture.clone();
  gem_material.environment_texture = environment_texture.clone();
  for primitive in primitives
  {
    let material = &primitive.borrow().material;
    *material.borrow_mut() = gem_material.dyn_clone();
  }
}

/// Bakes a soft shadow from the ring onto the ground plane.
fn bake_plane_shadow
(
  gl : &GL,
  gltf : &gltf::GLTF,
  plane_node : &Rc< RefCell< Node > >,
  shadow_map : &Option< ShadowMap >,
  shadow_baker : &Option< ShadowBaker >,
  clear_color : f32,
) -> Result< (), gl::WebglError >
{
  let shadow_map = shadow_map.as_ref().ok_or( gl::WebglError::Other( "ShadowMap not initialized" ) )?;
  let shadow_baker = shadow_baker.as_ref().ok_or( gl::WebglError::Other( "ShadowBaker not initialized" ) )?;

  let light = Light::new
  (
    F32x3::from_array( [ 5.0, 5.0, 5.0 ] ),
    F32x3::from_array( [ -1.0, -1.0, -1.0 ] ).normalize(),
    gl::math::mat3x3h::perspective_rh_gl( 30.0_f32.to_radians(), 1.0, 0.1, 15.0 ),
    0.5
  );

  // Mark all meshes as shadow casters
  let _ = gltf.scenes[ 0 ].borrow().traverse
  (
    &mut | node |
    {
      if let Object3D::Mesh( mesh ) = &node.borrow().object
      {
        mesh.borrow_mut().is_shadow_caster = true;
      }
      Ok( () )
    }
  );

  // Render depth from light's perspective
  shadow_map.render( &gltf.scenes[ 0 ].borrow(), light )?;

  // Create shadow texture with mipmaps
  let mip_levels = ( ( SHADOW_RESOLUTION as f32 ).log2().floor() as i32 ) + 1;
  let shadow_texture = create_shadow_texture( gl, SHADOW_RESOLUTION, mip_levels );

  // Bake PCSS soft shadows onto the plane
  shadow_baker.render_soft_shadow
  (
    &plane_node.borrow(),
    shadow_texture.as_ref(),
    SHADOW_RESOLUTION,
    SHADOW_RESOLUTION,
    shadow_map,
    light
  )?;

  // Generate mipmaps
  gl.active_texture( gl::TEXTURE0 );
  gl.bind_texture( gl::TEXTURE_2D, shadow_texture.as_ref() );
  gl.generate_mipmap( gl::TEXTURE_2D );

  // Wrap shadow texture and apply SurfaceMaterial to the plane
  let mut tex = Texture::new();
  tex.source = shadow_texture;
  let texture_info = TextureInfo
  {
    texture : Rc::new( RefCell::new( tex ) ),
    uv_position : 0,
  };

  setup_plane_material( gl, plane_node, Some( texture_info ), clear_color );

  Ok( () )
}

/// Creates an R8 texture for shadow baking.
fn create_shadow_texture( gl : &GL, res : u32, mip_levels : i32 ) -> Option< gl::web_sys::WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
  gl.tex_storage_2d( gl::TEXTURE_2D, mip_levels, gl::R8, res.cast_signed(), res .cast_signed() );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR.cast_signed() );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR.cast_signed() );
  gl::texture::d2::wrap_clamp( gl );
  texture
}

/// Applies `SurfaceMaterial` to the ground plane node.
fn setup_plane_material( gl : &GL, plane_node : &Rc< RefCell< Node > >, shadow_texture : Option< TextureInfo >, clear_color : f32 )
{
  let Object3D::Mesh( mesh ) = &plane_node.borrow().object else { return; };
  let primitives = &mesh.borrow().primitives;
  let Some( primitive ) = primitives.first() else { return; };

  let mut surface_material = SurfaceMaterial::new( gl );
  surface_material.color = F32x3::splat( clear_color );
  surface_material.texture = shadow_texture;
  surface_material.needs_update = true;
  let material : Rc< RefCell< Box< dyn Material > > > = Rc::new( RefCell::new( Box::new( surface_material ) ) );
  primitive.borrow_mut().material = material;
}

/// Updates gem color on all gem nodes in the map.
pub fn apply_gem_color( gems : &FxHashMap< String, Rc< RefCell< Node > > >, gem_color : F32x3 )
{
  for gem in gems.values()
  {
    let node = gem.borrow();
    let Object3D::Mesh( mesh ) = &node.object else { continue; };
    for primitive in &mesh.borrow().primitives
    {
      let material = &primitive.borrow().material;
      if material.borrow().type_name() != "GemMaterial" { continue; }
      let mut mat = helpers::cast_unchecked_material_to_ref_mut::< GemMaterial >( material.borrow_mut() );
      mat.color = gem_color;
      mat.needs_update = true;
    }
  }
}

/// Updates surface material color on plane nodes in the scene.
pub fn apply_plane_color( scene : &Rc< RefCell< Scene > >, clear_color : f32 )
{
  let color = F32x3::splat( clear_color );
  let _ = scene.borrow().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let node = node.borrow();
      let Object3D::Mesh( mesh ) = &node.object else { return Ok( () ); };
      for primitive in &mesh.borrow().primitives
      {
        let material = &primitive.borrow().material;
        if material.borrow().type_name() != "SurfaceMaterial" { continue; }
        let mut mat = helpers::cast_unchecked_material_to_ref_mut::< SurfaceMaterial >( material.borrow_mut() );
        mat.color = color;
        mat.needs_update = true;
      }
      Ok( () )
    }
  );
}

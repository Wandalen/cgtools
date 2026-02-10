use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::{ GL, WebglError };
use renderer::webgl::
{
    Camera,
    Node,
    Object3D,
    Texture,
    TextureInfo,
    Material,
    shadow::{ ShadowBaker, ShadowMap }
};
use renderer::webgl::loaders::gltf::GLTF;
use crate::
{
    cube_normal_map_generator::CubeNormalData,
    gem::GemMaterial,
    surface_material::SurfaceMaterial,
};

use super::CLEAR_COLOR;

/// Generates and applies a runtime soft shadow texture to the ground plane.
///
/// This function performs real-time shadow baking by rendering the scene's shadow map,
/// generating a soft shadow texture with mipmaps, and applying it to the plane mesh.
/// All meshes in the scene are marked as shadow casters before rendering.
///
/// # Process
/// 1. Marks all mesh objects in the scene as shadow casters
/// 2. Renders the scene to a shadow map from the light's perspective
/// 3. Creates a texture with mipmaps (calculated from lightmap resolution)
/// 4. Renders soft shadows onto the plane using the shadow baker
/// 5. Generates mipmaps for the shadow texture
/// 6. Creates a SurfaceMaterial with the shadow texture and applies it to the plane
///
/// # Parameters
/// - `gl`: WebGL context for GPU operations
/// - `lightmap_res`: Resolution of the shadow texture (typically 2048)
/// - `light`: Light configuration (position, direction, projection matrix)
/// - `shadowmap`: Pre-configured shadow map for rendering depth information
/// - `shadow_baker`: Utility for rendering soft shadows to textures
/// - `gltf`: The complete scene containing all ring geometry
/// - `plane_node`: The ground plane node that will receive the shadow texture
///
/// # Errors
/// Returns `WebglError` if:
/// - Shadow map rendering fails (GPU resource exhaustion, framebuffer errors)
/// - Shadow texture creation fails
/// - Plane mesh has no primitives
/// - WebGL texture operations fail
#[ inline ]
pub fn bake_plane_shadow
(
  gl: &GL,
  lightmap_res: u32,
  light: renderer::webgl::shadow::Light,
  shadowmap: &ShadowMap,
  shadow_baker: &ShadowBaker,
  gltf: &GLTF,
  plane_node: Rc< RefCell< Node > >
) -> Result< (), WebglError >
{
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

  shadowmap.render( &gltf.scenes[ 0 ].borrow(), light )?;

  let mip_levels = ( ( lightmap_res as f32 ).log2().floor() as i32 ) + 1;
  let shadow_texture = create_shadow_texture( &gl, lightmap_res, mip_levels );
  shadow_baker.render_soft_shadow
  (
    &plane_node.borrow(),
    shadow_texture.as_ref(),
    lightmap_res,
    lightmap_res,
    shadowmap,
    light
  )?;

  gl.active_texture( gl::TEXTURE0 );
  gl.bind_texture( gl::TEXTURE_2D, shadow_texture.as_ref() );
  gl.generate_mipmap( gl::TEXTURE_2D );

  if let Object3D::Mesh( mesh ) = &plane_node.borrow().object
  {
    let primitives = &mesh.borrow().primitives;
    let Some( primitive ) = primitives.first()
    else
    {
      return Err( WebglError::Other( "Plane mesh has no primitives" ) );
    };

    let mut texture = Texture::new();
    texture.source = shadow_texture;
    let texture_info = TextureInfo
    {
      texture : Rc::new( RefCell::new( texture ) ),
      uv_position : 0,
    };

    let mut surface_material = SurfaceMaterial::new( &gl );
    surface_material.color = CLEAR_COLOR;
    surface_material.texture = Some( texture_info.clone() );
    surface_material.needs_update = false;
    let surface_material_boxed : Rc< RefCell< Box< dyn Material > > > = Rc::new( RefCell::new( Box::new( surface_material ) ) );
    primitive.borrow_mut().material = surface_material_boxed;
  }

  Ok( () )
}

/// Creates a single-channel shadow texture with mipmaps
/// for soft shadow rendering.
#[ must_use ]
#[ inline ]
pub fn create_shadow_texture( gl : &GL, res : u32, mip_levels : i32 ) -> Option< web_sys::WebGlTexture >
{
  let texture = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
  gl.tex_storage_2d( gl::TEXTURE_2D, mip_levels, gl::R8, res as i32, res as i32 );

  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
  gl::texture::d2::wrap_clamp( &gl );

  texture
}

/// Initializes a perspective camera with orbit controls
/// configured for jewelry inspection.
#[ must_use ]
#[ inline ]
pub fn setup_camera( canvas : &web_sys::HtmlCanvasElement ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let eye = gl::math::F32x3::from( [ 0.637_357_6, 1.144_155_9, -0.912_740_5 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.555_956_96, 0.557_413_94, -1.033_113_6 ] );

  let aspect_ratio = width / height;
  let fov = 40.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.get_controls().borrow_mut().pan.enabled = false;
  camera.get_controls().borrow_mut().rotation.movement_smoothing_enabled = true;
  camera.get_controls().borrow_mut().rotation.speed = 50.0;
  camera.get_controls().borrow_mut().rotation.latitude_range_set( 90.0 );
  camera.get_controls().borrow_mut().zoom.min_distance_set( 2.0 );
  camera.get_controls().borrow_mut().zoom.max_distance_set( 6.0 );
  camera.bind_controls( &canvas );

  camera
}

/// Replaces all materials on a gem mesh with a configured
/// `GemMaterial`, including environment and cube normal maps.
#[ inline ]
pub fn setup_gem_material
(
  gl : &GL,
  gem_node : &Rc< RefCell< Node > >,
  environment_texture : &Option< TextureInfo >,
  cube_normal_map_texture : &CubeNormalData
)
{
  if let Object3D::Mesh( mesh ) = &gem_node.borrow().object
  {
    let primitives = &mesh.borrow().primitives;
    let mut gem_material = GemMaterial::new( &gl );
    gem_material.cube_normal_map_texture = cube_normal_map_texture.clone();
    gem_material.environment_texture = environment_texture.clone();
    for primitive in primitives
    {
      let material = &primitive.borrow().material;
      *material.borrow_mut() = gem_material.dyn_clone();
    }
  }
}

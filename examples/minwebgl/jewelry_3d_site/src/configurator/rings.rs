use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::{ GL, F32x3, WebglError, math::mat3x3h };
use renderer::webgl::
{
    AlphaMode,
    Object3D,
    TextureInfo,
    loaders::gltf,
    material::PbrMaterial,
    helpers,
    shadow::{ ShadowBaker, ShadowMap }
};
use rustc_hash::FxHashMap;
use crate::
{
    cube_normal_map_generator::{ CubeNormalMapGenerator, CubeNormalData },
    scene_utilities::filter_nodes,
    ui::{ clear_changed, get_ui_state },
};
use renderer::webgl::loaders::gltf::GLTF;

use super::RINGS_NUMBER;
use super::state::{ RingsInfo, RingColors, Ring, RcVec, remove_numbers };
use super::rendering::{ bake_plane_shadow, setup_gem_material };
use super::gpu_sync::GpuSync;

/// Initializes the ring loading system with lazy loading and GPU synchronization.
///
/// This async function sets up the infrastructure for on-demand ring loading, creating
/// a shared ring collection and a loader closure that handles complete ring initialization.
/// The lazy loading system prevents race conditions through GPU fence synchronization,
/// ensuring rings are fully processed on the GPU before being made available.
///
/// # Lazy Loading System
/// Ring GLTF models are loaded on-demand rather than all at initialization. The returned
/// `RingsInfo` contains a `ring_loader` closure that:
/// - Loads and processes each ring's GLTF scene asynchronously
/// - Configures metal materials (PBR settings for realistic jewelry rendering)
/// - Normalizes geometry scales and computes bounding boxes
/// - Positions rings on the ground plane (Y=0 at the bottom of the bounding box)
/// - Bakes soft shadows onto the ground plane (errors logged but non-fatal)
/// - Detects gem nodes (by "gem", "diamond", "crystal" substrings)
/// - Generates cube normal maps for gems with caching (shared across numbered instances)
/// - Uses GPU sync fences to prevent race conditions before storing the ring
///
/// # Race Condition Prevention
/// GPU operations are asynchronous, and subsequent ring switches could occur before
/// GPU processing completes. The `GpuSync::sync()` call inserts a fence that blocks
/// until all pending GPU commands finish, ensuring the ring is fully ready before
/// being stored in the shared collection.
///
/// # Shadow Baking
/// Each ring includes a ground plane that receives baked soft shadows at 2048x2048
/// resolution. Shadow baking failures are gracefully handled (logged but non-fatal)
/// to prevent WebGL resource exhaustion from crashing the application.
///
/// # Gem Material Setup
/// Gems are identified by name patterns and receive:
/// - Environment map reflections for realistic lighting
/// - Cube normal maps for surface detail (cached by base name, e.g., "gem" for "gem001", "gem002")
/// - Custom GemMaterial with refraction and dispersion properties
///
/// # Parameters
/// - `gl`: WebGL context for all GPU operations
/// - `environment_texture`: Optional HDR environment map for reflections
/// - `cube_normal_map_generator`: Generator for creating cube normal maps from gem geometry
///
/// # Returns
/// `RingsInfo` containing:
/// - Shared collection of ring slots (initially empty, filled by lazy loading)
/// - Default color selections for all rings
/// - Current ring index from UI state
/// - Ring loader closure for on-demand loading
///
/// # Errors
/// Returns `WebglError` if:
/// - Window or document object unavailable (required for GLTF loading)
/// - Plane GLTF fails to load
/// - Shadow map or baker initialization fails
#[ inline ]
pub async fn setup_rings
(
  gl : &GL,
  environment_texture : Option< TextureInfo >,
  cube_normal_map_generator : CubeNormalMapGenerator
)
-> Result< RingsInfo, WebglError >
{
  let window = gl::web_sys::window()
  .ok_or( WebglError::Other( "Failed to get window object" ) )?;
  let document = window.document()
  .ok_or( WebglError::Other( "Failed to get document object" ) )?;
  let gl = gl.clone();

  let rings : RcVec< Option< Ring > > = Rc::new( RefCell::new( vec![ None; RINGS_NUMBER ] ) );

  let plane_gltf = gltf::load( &document, "gltf/plane.glb", &gl ).await?;
  let plane_template = plane_gltf.scenes[ 0 ].borrow().get_node( "Plane" );

  let shadowmap_res = 2048;
  let lightmap_res = 2048;

  let light_pos = F32x3::from_array( [ 5.0, 5.0, 5.0 ] );
  let light_dir = F32x3::from_array( [ -1.0, -1.0, -1.0 ] ).normalize();

  let light = renderer::webgl::shadow::Light::new
  (
    light_pos,
    light_dir,
    gl::math::mat3x3h::perspective_rh_gl( 30.0_f32.to_radians(), 1.0, 0.1, 15.0 ),
    0.5
  );

  let shadowmap = ShadowMap::new( &gl, shadowmap_res )?;
  let shadow_baker = ShadowBaker::new( &gl )?;

  let ring_loader =
  {
    let rings = rings.clone();
    let gl = gl.clone();
    let gpu_sync = GpuSync::new( &gl ).ok();

    move | gltf : GLTF, index : usize |
    {
      if let Err( e ) = gltf.scenes[ 0 ].borrow().traverse
      (
        &mut | node |
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

            material.alpha_mode = AlphaMode::Opaque;

            let color = F32x3::from_array( [ 1.2, 1.2, 1.2 ] ).to_homogenous();
            material.base_color_factor = color;
            material.specular_factor = Some( 0.0 );
            material.specular_color_factor = Some( F32x3::splat( 1.0 ) );
            // Roughness 0.1 provides visually pleasing subtle surface variation
            // (0.04 appeared too mirror-like for realistic jewelry rendering)
            material.roughness_factor = 0.1;
            // Metallic 0.9 prevents oversaturation while maintaining metal appearance
            material.metallic_factor = 0.9;
            material.needs_update = true;
          }

          Ok( () )
        }
      )
      {
        gl::log::error!( "Error traversing scene: {:?}", e );
      }

      for node in &gltf.scenes[ 0 ].borrow().children
      {
        let mut node = node.borrow_mut();
        node.normalize_scale();
        node.compute_local_bounding_box();
        let bb = node.local_bounding_box_hierarchical();
        let t = mat3x3h::translation( [ 0.0, -bb.min.y(), 0.0 ] );
        node.apply_matrix( t );
      }

      if let Some( plane_template ) = &plane_template
      {
        let plane_node = plane_template.borrow().clone_tree();
        plane_node.borrow_mut().set_translation( F32x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
        plane_node.borrow_mut().set_scale( F32x3::from_array( [ 3.0, 1.0, 3.0 ] ) );
        gltf.scenes[ 0 ].borrow_mut().add( plane_node.clone() );
        gltf.scenes[ 0 ].borrow_mut().update_world_matrix();

        // Handle shadow baking errors gracefully (WebGL resource exhaustion, framebuffer failures)
        // If shadow baking fails, continue without shadows rather than crashing
        let _ = bake_plane_shadow( &gl, lightmap_res, light, &shadowmap, &shadow_baker, &gltf, plane_node );
      }

      let mut ring_gems = FxHashMap::default();
      for substring in [ "gem", "diamond", "crystal" ]
      {
        let nodes = filter_nodes( &gltf.scenes[ 0 ], substring.to_string(), false );
        ring_gems.extend( nodes );
      }

      let mut normal_maps = FxHashMap::< String, CubeNormalData >::default();
      for ( name, gem ) in &ring_gems
      {
        let root_name = remove_numbers( name.as_str() );
        let cube_normal_map_texture = if let Some( normal_map ) = normal_maps.get( &root_name )
        {
          normal_map.clone()
        }
        else
        {
          match cube_normal_map_generator.generate( &gl, &gem )
          {
            Ok( normal_map ) =>
            {
              normal_maps.insert( name.clone(), normal_map.clone() );
              normal_map
            },
            Err( e ) =>
            {
              gl::log::error!( "Failed to generate cube normal map for {}: {:?}", name, e );
              CubeNormalData::default()
            }
          }
        };

        setup_gem_material( &gl, &gem, &environment_texture, &cube_normal_map_texture );
      }

      let ring = Ring
      {
        scene : gltf.scenes[ 0 ].clone(),
        gems : ring_gems,
      };

      if let Some( ref sync ) = gpu_sync
      {
        sync.sync();
      }
      rings.borrow_mut()[ index ] = Some( ring.clone() );
    }
  };

  let current_ring = get_ui_state().map_or( 0, | inner | inner.ring ) as usize;
  clear_changed();

  // Initialize color selections for all rings with defaults
  let ring_colors : Vec< RingColors > = ( 0..RINGS_NUMBER ).map( | _ | RingColors::default() ).collect();

  Ok
  (
    RingsInfo::new( &gl, rings, ring_colors, current_ring, ring_loader )
  )
}

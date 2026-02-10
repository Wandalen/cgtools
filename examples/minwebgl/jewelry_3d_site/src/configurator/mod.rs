use std::{ cell::RefCell, rc::Rc };
use mingl::web::canvas;
use minwebgl as gl;
use gl::
{
  GL,
  F32x3,
  WebglError
};
use renderer::webgl::
{
  Camera,
  IBL,
  Material,
  Node,
  Object3D,
  Renderer,
  TextureInfo,
  helpers,
  material::PbrMaterial,
};
use crate::
{
  cube_normal_map_generator::CubeNormalMapGenerator,
  gem::GemMaterial,
  scene_utilities::*,
  ui::{ UiState, get_ui_state },
};
use ::animation::{ AnimatablePlayer, Tween, easing::{ Linear, EasingBuilder } };

// Module declarations
mod animation;
mod state;
mod gpu_sync;
mod rendering;
mod rings;

// Public re-exports
// Note: These re-exports are used via wildcard imports (e.g., `use configurator::*`)
// but clippy cannot detect this usage pattern, hence the allow attribute
#[allow(unused_imports)]
pub use self::state::{ Ring, RingColors, RingsInfo, RcScene, RcNode, RcVec, get_color, remove_numbers };
#[allow(unused_imports)]
pub use self::animation::{ AnimationState, MaterialAnimationCallback };
#[allow(unused_imports)]
pub use self::rings::setup_rings;
#[allow(unused_imports)]
pub use self::rendering::
{
    bake_plane_shadow,
    create_shadow_texture,
    setup_camera,
    setup_gem_material
};
#[allow(unused_imports)]
pub use self::gpu_sync::GpuSync;

/// Duration of color transition animation in milliseconds (MS)
pub const TRANSITION_DURATION_MS : f64 = 1000.0;
/// Default clear color used for background and floor color.
pub const CLEAR_COLOR : F32x3 = F32x3::splat( 2.0 );

pub(crate) const RINGS_NUMBER : usize = 5;

/// High-level scene configurator responsible for:
/// - Renderer and camera setup
/// - Ring and gem scene management
/// - UI-driven material updates
/// - Color transition animations
#[ non_exhaustive ]
pub struct Configurator
{
  /// Shared WebGL renderer instance
  pub renderer : Rc< RefCell< Renderer > >,

  /// Scene camera with user controls
  pub camera : Camera,

  /// Image-based lighting configuration
  pub ibl : IBL,

  /// Optional skybox texture
  pub skybox : Option< TextureInfo >,

  /// Loaded ring scenes and gem node mappings
  pub rings : RingsInfo,

  /// Current UI state (colors, ring index, etc.)
  pub ui_state : UiState,

  /// Animation controller for material transitions
  pub animation_state : AnimationState
}

impl Configurator
{
  /// Creates and initializes the entire rendering pipeline:
  /// - Loads environment maps
  /// - Initializes renderer and camera
  /// - Loads ring scenes and gems
  /// - Applies initial UI-driven material states
  ///
  /// # Errors
  /// Returns an error if resource loading or initialization fails
  #[ inline ]
  pub async fn new( gl : &GL, canvas : &canvas::HtmlCanvasElement ) -> Result< Self, WebglError >
  {
    let mut cube_normal_map_generator = CubeNormalMapGenerator::new( gl )?;
    cube_normal_map_generator.set_texture_size( gl, 512, 512 );

    let ibl = renderer::webgl::loaders::ibl::load( gl, "environment_maps/studio", None ).await;

    let env_map = create_empty_texture( &gl ).await
    .ok_or(  WebglError::Other( "Failed to create environment texture" ) )?;
    let env_texture_source = env_map.texture.borrow().source.clone();
    renderer::webgl::loaders::hdr_texture::load_to_mip_d2
    (
      gl,
      env_texture_source.as_ref(),
      0,
      "environment_maps/studio3/env-gem-4.hdr"
    )
    .await;

    let rings = setup_rings( gl, Some( env_map ), cube_normal_map_generator ).await?;

    let renderer = Renderer::new( gl, canvas.width(), canvas.height(), 4 )?;
    let renderer = Rc::new( RefCell::new( renderer ) );

    let camera = setup_camera( &canvas );

    let ui_state = get_ui_state()
    .ok_or( WebglError::Other( "Failed to get UI state from JavaScript" ) )?;

    let skybox = None;

    let mut configurator = Configurator
    {
      renderer,
      camera,
      ibl,
      skybox,
      rings,
      ui_state,
      animation_state : AnimationState::new()
    };

    configurator.setup_renderer();
    configurator.update_gem_color();
    configurator.update_metal_color();

    Ok( configurator )
  }

  /// Saves current UI gem color to the current ring's context.
  #[ inline ]
  pub fn save_gem_to_ring( &mut self )
  {
    if let Some( colors ) = self.rings.ring_colors.get_mut( self.rings.current_ring )
    {
      colors.gem = self.ui_state.gem.clone();
    }
  }

  /// Saves current UI metal color to the current ring's context.
  #[ inline ]
  pub fn save_metal_to_ring( &mut self )
  {
    if let Some( colors ) = self.rings.ring_colors.get_mut( self.rings.current_ring )
    {
      colors.metal = self.ui_state.metal.clone();
    }
  }

  /// Loads gem color from the current ring's context into UI state.
  #[ inline ]
  pub fn load_gem_from_ring( &mut self )
  {
    if let Some( colors ) = self.rings.ring_colors.get( self.rings.current_ring )
    {
      self.ui_state.gem = colors.gem.clone();
    }
  }

  /// Loads metal color from the current ring's context into UI state.
  #[ inline ]
  pub fn load_metal_from_ring( &mut self )
  {
    if let Some( colors ) = self.rings.ring_colors.get( self.rings.current_ring )
    {
      self.ui_state.metal = colors.metal.clone();
    }
  }

  /// Updates gem material color based on current UI selection.
  /// Uses animated transitions when applicable.
  #[ inline ]
  pub fn update_gem_color( &mut self )
  {
    match self.ui_state.gem.as_str()
    {
      "white" => self.set_gem_color( F32x3::from_array( [ 1.2, 1.2, 1.2 ] ) ),
      "red" => self.set_gem_color( F32x3::from_array( [ 0.8, 0.05, 0.05 ] ) ),
      "orange" => self.set_gem_color( F32x3::from_array( [ 1.0, 0.3, 0.05 ] ) * 2.0 ),
      "yellow" => self.set_gem_color( F32x3::from_array( [ 1.7, 1.0, 0.15 ] ) ),
      "green" => self.set_gem_color( F32x3::from_array( [ 0.1, 0.45, 0.15 ] ) ),
      "turquoise" => self.set_gem_color( F32x3::from_array( [ 0.25, 0.83, 0.77 ] ) * 1.2 ),
      "blue" => self.set_gem_color( F32x3::from_array( [ 0.1, 0.3, 1.0 ] ) * 1.2 ),
      "pink" => self.set_gem_color( F32x3::from_array( [ 1.0, 0.41, 0.81 ] ) * 2.0 ),
      "custom" =>
      {
        #[ cfg( debug_assertions ) ]
        {
          if self.ui_state.gem_custom_color.len() >= 3
          {
            let base_color = F32x3::from_array
            (
              [
                self.ui_state.gem_custom_color[ 0 ],
                self.ui_state.gem_custom_color[ 1 ],
                self.ui_state.gem_custom_color[ 2 ]
              ]
            );
            let final_color = base_color * self.ui_state.gem_multiplier;
            self.set_gem_color( final_color );
          }
        }
      },
      _ => ()
    }
  }

  /// Updates metal (ring body) material color from UI state.
  /// Applies physically-based parameters suitable for jewelry.
  #[ inline ]
  pub fn update_metal_color( &mut self )
  {
    match self.ui_state.metal.as_str()
    {
      "silver" => self.set_metal_color( F32x3::from_array( [ 0.753, 0.753, 0.753 ] ) * 1.2 ),
      "copper" => self.set_metal_color( F32x3::from_array( [ 1.0, 0.4, 0.2 ] ) * 1.8 ),
      "gold" => self.set_metal_color( F32x3::from_array( [ 1.0, 0.5, 0.1 ] ) * 2.0 ),
      "custom" =>
      {
        #[ cfg( debug_assertions ) ]
        {
          if self.ui_state.metal_custom_color.len() >= 3
          {
            let base_color = F32x3::from_array
            (
              [
                self.ui_state.metal_custom_color[ 0 ],
                self.ui_state.metal_custom_color[ 1 ],
                self.ui_state.metal_custom_color[ 2 ]
              ]
            );
            let final_color = base_color * self.ui_state.metal_multiplier;
            self.set_metal_color( final_color );
          }
        }
      },
      _ => ()
    }
  }

  /// Animates and applies a new gem color to all gem materials
  /// in the currently selected ring.
  #[ inline ]
  pub fn set_gem_color( &mut self, color : F32x3 )
  {
    let delay = self.animation_state.animations.time();
    let get_player = | old_color : F32x3 |
    {
      let mut tween = Tween::new( old_color, color, TRANSITION_DURATION_MS, Linear::new() )
      .with_delay( delay );
      tween.update( delay );
      tween
    };

    let Some( ring ) = self.rings.get_ring() else
    {
      return;
    };

    for ( _, gem ) in &ring.gems
    {
      let Object3D::Mesh( mesh ) = &gem.borrow().object
      else
      {
        continue;
      };

      for primitive in &mesh.borrow().primitives
      {
        let material = &primitive.borrow().material;

        self.animation_state.add_material_animation
        (
          material,
          get_player( get_color( material ) ),
          | player : &dyn AnimatablePlayer, material : &Rc< RefCell< Box< dyn Material > > > |
          {
            if material.borrow().type_name() != "GemMaterial"
            {
              return;
            }
            let Some( player ) = player.as_any().downcast_ref::< Tween< F32x3 > >()
            else
            {
              return;
            };
            let color = player.value_get();
            let mut material = helpers::cast_unchecked_material_to_ref_mut::< GemMaterial >( material.borrow_mut() );
            material.color = color;
            material.needs_update = true;
          }
        );
      }
    }
  }

  /// Animates and applies a new metal color to all non-gem meshes
  /// in the current ring scene.
  #[ inline ]
  pub fn set_metal_color
  (
    &mut self,
    color : F32x3
  )
  {
    let Some( ring ) = self.rings.get_ring() else { return; };

    let gems = &ring.gems;

    let delay = self.animation_state.animations.time();
    let get_player = | old_color : F32x3 |
    {
      let mut tween = Tween::new( old_color, color, TRANSITION_DURATION_MS, Linear::new() )
      .with_delay( delay );
      tween.update( delay );
      tween
    };

    let _ = ring.scene.borrow().traverse
    (
      &mut | node : Rc< RefCell< Node > > |
      {
        if let Some( name ) = node.borrow().get_name()
        {
          if gems.contains_key( &name.clone().into_string() )
          {
            return Ok( () );
          }
        }

        let Object3D::Mesh( mesh ) = &node.borrow().object
        else
        {
          return Ok( () );
        };

        for primitive in &mesh.borrow().primitives
        {
          let material = &primitive.borrow().material;

          self.animation_state.add_material_animation
          (
            material,
            get_player( get_color( material ) ),
            | player : &dyn AnimatablePlayer, material : &Rc< RefCell< Box< dyn Material > > > |
            {
              if material.borrow().type_name() != "PbrMaterial"
              {
                return;
              }
              let Some( player ) = player.as_any().downcast_ref::< Tween< F32x3 > >()
              else
              {
                return;
              };

              let color = player.value_get();
              let mut material = helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >( material.borrow_mut() );

              for i in 0..3
              {
                material.base_color_factor.0[ i ] = color.0[ i ];
              }
            }
          );
        }

        Ok( () )
      }
    );
  }

  /// Configures renderer state:
  /// - IBL and skybox
  /// - Clear color
  /// - Bloom and exposure
  #[ inline ]
  pub fn setup_renderer( &self )
  {
    let mut renderer_mut = self.renderer.borrow_mut();
    renderer_mut.set_ibl( self.ibl.clone() );

    if let Some( skybox ) = &self.skybox
    {
      renderer_mut.set_skybox( skybox.texture.borrow().source.clone() );
    }
    else
    {
      renderer_mut.set_skybox( None );
      renderer_mut.set_clear_color( CLEAR_COLOR );
    }

    renderer_mut.set_use_emission( true );
    renderer_mut.set_bloom_strength( 2.0 );
    // Exposure 0.0 provides optimal brightness for jewelry visibility
    // (previous -1.0 value made models too dark in studio lighting)
    renderer_mut.set_exposure( -0.5 );
    renderer_mut.set_bloom_radius( 0.1 );
  }
}

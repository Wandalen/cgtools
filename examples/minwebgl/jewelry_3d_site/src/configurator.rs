use std::{ cell::RefCell, rc::Rc };
use renderer::webgl::
{
  shadow::{ ShadowBaker, ShadowMap },
  Texture
};

use mingl::web::canvas;
use minwebgl as gl;
use gl::
{
  math::mat3x3h,
  GL,
  F32x3,
  WebglError
};
use rustc_hash::FxHashMap;
use renderer::webgl::
{
  Camera,
  IBL,
  Material,
  Node,
  Object3D,
  Renderer,
  Scene,
  TextureInfo,
  material::PbrMaterial
};
use animation::{ AnimatablePlayer, Sequencer, Tween, easing::{ Linear, EasingBuilder } };
use crate::
{
  cube_normal_map_generator::CubeNormalMapGenerator, gem::GemMaterial, helpers::*, ui::{ UiState, clear_changed, get_ui_state },
  surface_material::SurfaceMaterial,
};

/// Duration of color transition animation in milliseconds (MS)
const TRANSITION_DURATION_MS : f64 = 1000.0;

pub struct Configurator
{
  pub _cube_normal_map_generator : CubeNormalMapGenerator,
  pub renderer : Rc< RefCell< Renderer > >,
  pub camera : Camera,
  pub ibl : IBL,
  pub skybox : Option< TextureInfo >,
  pub rings : RingsInfo,
  pub ui_state : UiState,
  pub animation_state : AnimationState
}

impl Configurator
{
  pub async fn new( gl : &GL, canvas : &canvas::HtmlCanvasElement ) -> Result< Self, WebglError >
  {
    let mut _cube_normal_map_generator = CubeNormalMapGenerator::new( gl )?;
    _cube_normal_map_generator.set_texture_size( gl, 512, 512 );

    let ibl = renderer::webgl::loaders::ibl::load( gl, "environment_maps/studio", None ).await;

    let env_map = create_empty_texture( &gl ).await;
    renderer::webgl::loaders::hdr_texture::load_to_mip_d2
    (
      gl,
      env_map.as_ref().unwrap().texture.borrow().source.as_ref(),
      0,
      "environment_maps/studio3/env-gem-4.hdr"
    )
    .await;

    let rings = setup_rings( gl, &env_map, &_cube_normal_map_generator ).await?;

    let renderer = Renderer::new( gl, canvas.width(), canvas.height(), 4 )?;
    let renderer = Rc::new( RefCell::new( renderer ) );

    let camera = setup_camera( &canvas );

    let ui_state = get_ui_state().unwrap();

    let skybox = None;

    let mut configurator = Configurator
    {
      _cube_normal_map_generator,
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
    // configurator.setup_light();

    Ok( configurator )
  }

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

  fn _setup_light( &self )
  {
    let light =
    renderer::webgl::Light::Direct
    (
      renderer::webgl::DirectLight
      {
        direction : to_decart( 1.0, 30.0, 65.0 ),
        color : F32x3::splat( 1.0 ),
        strength : 40000.0
      }
    );
    let light_node = Rc::new( RefCell::new( Node::new() ) );
    light_node.borrow_mut().object = Object3D::Light( light );
    for scene in &self.rings.rings
    {
      scene.borrow_mut().add( light_node.clone() );
    }
  }

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

    let current_ring = self.rings.current_ring;
    for ( _, gem ) in &self.rings.gems[ current_ring ]
    {
      let Object3D::Mesh( mesh ) = &gem.borrow().object
      else
      {
        continue;
      };

      mesh.borrow_mut().is_shadow_caster = true;

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
            let mut material = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< GemMaterial >( material.borrow_mut() );
            material.color = color;
            material.needs_update = true;
          }
        );
      }
    }
  }


  pub fn set_metal_color
  (
    &mut self,
    color : F32x3
  )
  {
    let current_ring = self.rings.current_ring;
    let gems = &self.rings.gems[ current_ring ];

    let delay = self.animation_state.animations.time();
    let get_player = | old_color : F32x3 |
    {
      let mut tween = Tween::new( old_color, color, TRANSITION_DURATION_MS, Linear::new() )
      .with_delay( delay );
      tween.update( delay );
      tween
    };

    let _ = self.rings.rings[ current_ring ].borrow().traverse
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

        mesh.borrow_mut().is_shadow_caster = true;

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
              let mut material = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >( material.borrow_mut() );
              material.alpha_mode = renderer::webgl::AlphaMode::Opaque;
              for i in 0..3
              {
                material.base_color_factor.0[ i ] = color.0[ i ];
              }
              material.base_color_factor.0[ 3 ] = 1.0;
              material.roughness_factor = 0.04;
              material.metallic_factor = 1.0;
              material.needs_update = true;
            }
          );
        }

        Ok( () )
      }
    );
  }

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
      renderer_mut.set_clear_color( F32x3::splat( 0.854 ) );
    }

    renderer_mut.set_use_emission( true );
    renderer_mut.set_bloom_strength( 2.0 );
    renderer_mut.set_exposure( -1.0 );
    renderer_mut.set_bloom_radius( 0.1 );
  }
}

/// Controls and updates animations and then applies interpolated values to materials using callbacks
pub struct AnimationState
{
  /// Animation storage, player and state manager
  animations : Sequencer,
  /// Material updated in callbacks
  materials: FxHashMap< String, Rc< RefCell< Box< dyn Material > > > >,
  /// Callbacks triggered when animations are updated in [`AnimationState::animations`].
  /// Callbacks can update values inside material
  material_callbacks : FxHashMap< String, fn ( &dyn AnimatablePlayer, &Rc< RefCell< Box< dyn Material > > > ) >
}

impl AnimationState
{
  /// Creates a new instance
  pub fn new() -> Self
  {
    let mut animations = Sequencer::new();
    animations.resume();

    Self
    {
      animations,
      materials : FxHashMap::default(),
      material_callbacks : FxHashMap::default()
    }
  }

  /// Updates animations, calls callbacks and removes completed animations
  pub fn update( &mut self, delta_time : f64 )
  {
    self.animations.resume();
    self.animations.update( delta_time );

    for name in self.animations.keys()
    {
      let name = name.as_ref();
      let Some( callback ) = self.material_callbacks.get( name )
      else
      {
        continue;
      };

      let Some( material ) = self.materials.get( name )
      else
      {
        continue;
      };

      if let Some( player ) = self.animations.get_dyn_value( name.as_ref() )
      {
        callback( player, material );
      }
    }

    for name in self.animations.keys()
    {
      let completed = if let Some( player ) = self.animations.get_dyn_value( name.as_ref() )
      {
        player.is_completed()
      }
      else
      {
        continue;
      };

      if completed
      {
        self.animations.remove( name.as_ref() );
        self.materials.remove( name.as_ref() );
        self.material_callbacks.remove( name.as_ref() );
      }
    }
  }

  /// Adds new animations with callbacks
  pub fn add_material_animation< P >
  (
    &mut self,
    material : &Rc< RefCell< Box< dyn Material > > >,
    player : P,
    callback : fn ( &dyn AnimatablePlayer, &Rc< RefCell< Box< dyn Material > > > )
  )
  where P : AnimatablePlayer + 'static
  {
    let name = material.borrow().get_id().to_string();

    self.animations.add::< P >( &name, player );
    if self.animations.is_completed()
    {
      self.animations.resume();
    }
    self.materials.insert( name.clone(), material.clone() );
    self.material_callbacks.insert( name, callback );
  }
}

pub struct RingsInfo
{
  pub rings : Vec< Rc< RefCell< Scene > > >,
  pub gems : Vec< FxHashMap< String, Rc< RefCell< Node > > > >,
  pub current_ring : usize
}

fn get_color( material : &Rc< RefCell< Box< dyn Material > > > ) -> F32x3
{
  let type_name =
  {
    let m = material.borrow();
    m.type_name()
  };

  match type_name
  {
    "PbrMaterial" =>
    {
      let material = renderer::webgl::helpers::cast_unchecked_material_to_ref::< PbrMaterial >( material.borrow() );
      material.base_color_factor.truncate()
    },
    "GemMaterial" =>
    {
      let material = renderer::webgl::helpers::cast_unchecked_material_to_ref::< GemMaterial >( material.borrow() );
      material.color
    },
    _ => F32x3::splat( 1.0 )
  }
}

fn remove_numbers( s : &str ) -> String
{
  s.chars().filter( | c | !c.is_ascii_digit() ).collect()
}

async fn setup_rings
(
  gl : &GL,
  environment_texture : &Option< TextureInfo >,
  cube_normal_map_generator : &CubeNormalMapGenerator
)
-> Result< RingsInfo, WebglError >
{
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let mut rings : Vec< Rc< RefCell< Scene > > > = vec![];
  let mut gems : Vec< FxHashMap< String, Rc< RefCell< Node > > > > = vec![];

  let plane_gltf = renderer::webgl::loaders::gltf::load( &document, "gltf/plane.glb", &gl ).await?;
  let plane_template = plane_gltf.scenes[ 0 ].borrow().get_node( "Plane" ).unwrap();

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

  for i in 0..5
  {
    let gltf = renderer::webgl::loaders::gltf::load( &document, format!( "./gltf/{i}.glb" ).as_str(), &gl ).await?;

    for node in &gltf.scenes[ 0 ].borrow().children
    {
      let mut node = node.borrow_mut();
      node.normalize_scale();
      node.compute_local_bounding_box();
      let bb = node.local_bounding_box_hierarchical();
      let t = mat3x3h::translation( [ 0.0, -bb.min.y(), 0.0 ] );
      node.apply_matrix( t );
    }

    let plane_node = plane_template.borrow().clone_tree();
    plane_node.borrow_mut().set_translation( F32x3::from_array( [ 0.0, 0.0, 0.0 ] ) );
    plane_node.borrow_mut().set_scale( F32x3::from_array( [ 3.0, 1.0, 3.0 ] ) );
    gltf.scenes[ 0 ].borrow_mut().add( plane_node.clone() );

    gltf.scenes[ 0 ].borrow_mut().update_world_matrix();

    bake_plane_shadow(gl, lightmap_res, light, &shadowmap, &shadow_baker, &gltf, plane_node)?;

    rings.push( gltf.scenes[ 0 ].clone() );

    let mut ring_gems = FxHashMap::default();
    for substring in [ "gem", "diamond", "crystal" ]
    {
      let nodes = filter_nodes( &gltf.scenes[ 0 ], substring.to_string(), false );
      ring_gems.extend( nodes );
    }

    let mut normal_maps = FxHashMap::< String, TextureInfo >::default();
    for ( name, gem ) in &ring_gems
    {
      let root_name = remove_numbers( name.as_str() );
      let cube_normal_map_texture = if let Some( normal_map ) = normal_maps.get( &root_name )
      {
        normal_map.clone()
      }
      else
      {
        let normal_map = cube_normal_map_generator.generate( gl, &gem ).unwrap();
        normal_maps.insert( name.clone(), normal_map.clone() );
        normal_map
      };
      setup_gem_material( gl, &gem, environment_texture, &Some( cube_normal_map_texture ) );
    }

    gems.push( ring_gems );
  }

  let ui_state = get_ui_state().unwrap();
  clear_changed();
  let current_ring = ui_state.ring as usize;

  Ok
  (
    RingsInfo
    {
      rings,
      gems,
      current_ring
    }
  )
}

fn bake_plane_shadow
(
  gl: &GL,
  lightmap_res: u32,
  light: renderer::webgl::shadow::Light,
  shadowmap: &ShadowMap,
  shadow_baker: &ShadowBaker,
  gltf: &renderer::webgl::loaders::gltf::GLTF,
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
    let primitive = primitives.first().unwrap();

    let mut texture = Texture::new();
    texture.source = shadow_texture;
    let texture_info = TextureInfo
    {
      texture : Rc::new( RefCell::new( texture ) ),
      uv_position : 0,
    };

    let mut surface_material = SurfaceMaterial::new( &gl );
    surface_material.color = F32x3::splat( 0.854 );
    surface_material.texture = Some( texture_info.clone() );
    surface_material.needs_update = false;
    let surface_material_boxed : Rc< RefCell< Box< dyn Material > > > = Rc::new( RefCell::new( Box::new( surface_material ) ) );
    primitive.borrow_mut().material = surface_material_boxed;
  }

  Ok( () )
}

fn create_shadow_texture( gl : &GL, res : u32, mip_levels : i32 ) -> Option< web_sys::WebGlTexture >
{
  let ret = gl.create_texture();
  gl.bind_texture( gl::TEXTURE_2D, ret.as_ref() );
  gl.tex_storage_2d( gl::TEXTURE_2D, mip_levels, gl::R8, res as i32, res as i32 );

  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
  gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32 );
  gl::texture::d2::wrap_clamp( &gl );

  ret
}

fn setup_camera( canvas : &web_sys::HtmlCanvasElement ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let eye = crate::helpers::to_decart( 6.0, 135.0, 65.0 );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.6, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 40.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.get_controls().borrow_mut().use_pan = false;
  camera.get_controls().borrow_mut().use_rotation_easing = true;
  camera.get_controls().borrow_mut().rotation_speed_scale = 50.0;
  camera.bind_controls( &canvas );

  camera
}

fn setup_gem_material
(
  gl : &GL,
  gem_node : &Rc< RefCell< Node > >,
  environment_texture : &Option< TextureInfo >,
  cube_normal_map_texture : &Option< TextureInfo >
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

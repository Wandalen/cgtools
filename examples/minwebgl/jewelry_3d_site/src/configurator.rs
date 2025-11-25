use std::{ cell::RefCell, rc::Rc };
use mingl::web::canvas;
use minwebgl as gl;
use gl::
{
  GL,
  F32x3,
  WebglError
};
use std::collections::HashSet;
use renderer::webgl::
{
  Camera,
  IBL,
  Light,
  Material,
  Node,
  Object3D,
  Renderer,
  Scene,
  SpotLight,
  MinFilterMode,
  MagFilterMode,
  WrappingMode,
  Sampler,
  Texture,
  TextureInfo,
  material::PBRMaterial
};
use crate::
{
  cube_normal_map_generator::CubeNormalMapGenerator,
  gem::GemMaterial, helpers::*,
  ui::{ UiState, clear_changed, get_ui_state }
};

pub struct Configurator
{
  pub _cube_normal_map_generator : CubeNormalMapGenerator,
  pub renderer : Rc< RefCell< Renderer > >,
  pub camera : Camera,
  pub ibl : IBL,
  pub skybox : Option< TextureInfo >,
  pub surface_material : Rc< RefCell< Box< dyn Material > > >,
  pub scene : Rc< RefCell< Scene > >,
  pub rings : RingsInfo,
  pub ui_state : UiState
}

impl Configurator
{
  pub async fn new( gl : &GL, canvas : &canvas::HtmlCanvasElement ) -> Result< Self, WebglError >
  {
    let window = gl::web_sys::window().unwrap();
    let document = window.document().unwrap();

    let gltf = renderer::webgl::loaders::gltf::load( &document, format!( "./gltf/plane.glb" ).as_str(), &gl ).await?;

    let scene = gltf.scenes[ 0 ].clone();

    let mut _cube_normal_map_generator = CubeNormalMapGenerator::new( gl )?;
    _cube_normal_map_generator.set_texture_size( gl, 512, 512 );


    let ibl_ring = renderer::webgl::loaders::ibl::load( gl, "environment_maps/studio", Some( 0..9 ) ).await;
    let gem_env_map = gl.create_texture();
    renderer::webgl::loaders::hdr_texture::load_to_mip_d2( gl, gem_env_map.as_ref(), 0, "environment_maps/studio3/env-gem-4.hdr").await;
    //let ibl_ring = renderer::webgl::loaders::ibl::load( gl, "environment_maps/dancing_hall_4k", None ).await;

    let sampler = Sampler::former()
    .min_filter( MinFilterMode::Linear )
    .mag_filter( MagFilterMode::Linear )
    .wrap_r( WrappingMode::Repeat )
    .wrap_s( WrappingMode::Repeat )
    .wrap_t( WrappingMode::Repeat )
    .end();

    let texture = Texture::former()
    .target( GL::TEXTURE_2D )
    .source( gem_env_map.clone().unwrap() )
    .sampler( sampler )
    .end();

    let env_map = Some
    (
      TextureInfo
      {
        texture : Rc::new( RefCell::new( texture ) ),
        uv_position : 0,
      }
    );

    let rings = setup_rings( gl, &env_map, &_cube_normal_map_generator ).await?;

    scene.borrow_mut().add( rings.current_ring.clone() );
    scene.borrow_mut().update_world_matrix();

    let renderer = Renderer::new( gl, canvas.width(), canvas.height(), 4 )?;
    let renderer = Rc::new( RefCell::new( renderer ) );

    let surface = get_node( &scene, "Plane".to_string() ).unwrap();
    let surface_material = setup_surface( surface );

    let camera = setup_camera( &scene, &canvas );

    let ui_state = get_ui_state().unwrap();

    let skybox = None;

    let mut configurator = Configurator
    {
      _cube_normal_map_generator,
      renderer,
      camera,
      ibl : ibl_ring,
      skybox,
      surface_material,
      scene,
      rings,
      ui_state
    };

    configurator.setup_renderer();
    configurator.setup_light( &gl );

    Ok( configurator )
  }

  pub fn update_gem_color( &self )
  {
    match self.ui_state.gem.as_str()
    {
      "white" => self.set_gem_color( F32x3::from_array( [ 1.0, 1.0, 1.0 ] ) ),
      "black" => self.set_gem_color( F32x3::from_array( [ 0.05, 0.05, 0.05 ] ) ),
      "red" => self.set_gem_color( F32x3::from_array( [ 1.0, 0.05, 0.05 ] ) ),
      "orange" => self.set_gem_color( F32x3::from_array( [ 1.0, 0.3, 0.05 ] ) ),
      "yellow" => self.set_gem_color( F32x3::from_array( [ 1.0, 0.7, 0.05 ] ) ),
      "green" => self.set_gem_color( F32x3::from_array( [ 0.1, 0.4, 0.1 ] ) ),
      "turquoise" => self.set_gem_color( F32x3::from_array( [ 0.2, 0.78, 0.72 ] ) ),
      "light_blue" => self.set_gem_color( F32x3::from_array( [ 0.05, 0.4, 1.0 ] ) ),
      "blue" => self.set_gem_color( F32x3::from_array( [ 0.05, 0.25, 1.0 ] ) ),
      "violet" => self.set_gem_color( F32x3::from_array( [ 0.8, 0.2, 0.8 ] ) ),
      "pink" => self.set_gem_color( F32x3::from_array( [ 1.0, 0.31, 0.71 ] ) ),
      _ => ()
    }
  }

  pub fn update_metal_color( &self )
  {
    match self.ui_state.metal.as_str()
    {
      "silver" => self.set_metal_color( F32x3::from_array( [ 0.753, 0.753, 0.753 ] ) ),
      "copper" => self.set_metal_color( F32x3::from_array( [ 1.0, 0.4, 0.2 ] ) ),
      "gold" => self.set_metal_color( F32x3::from_array( [ 1.0, 0.55, 0.02 ] ) ),
      _ => ()
    }
  }

  pub fn set_gem_color( &self, color : F32x3 )
  {
    let Object3D::Mesh( mesh ) = &self.rings.current_gem.borrow().object
    else
    {
      return;
    };

    mesh.borrow_mut().is_shadow_caster = true;
    mesh.borrow_mut().is_shadow_receiver = true;

    for primitive in &mesh.borrow().primitives
    {
      let material = &primitive.borrow().material;
      {
        if material.borrow().get_type_name() != "GemMaterial"
        {
          continue;
        }
        let mut material = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< GemMaterial >( material.borrow_mut() );
        material.color = color;
        material.need_update = true;
      }
    }
  }

  pub fn set_metal_color
  (
    &self,
    color : F32x3
  )
  {
    let filter = self.rings.filters.get( self.ui_state.ring as usize ).unwrap();
    let _ = self.rings.current_ring.borrow().traverse
    (
      &mut | node : Rc< RefCell< Node > > |
      {
        if let Some( name ) = node.borrow().get_name()
        {
          if filter.contains( &name.clone().into_string() )
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
        mesh.borrow_mut().is_shadow_receiver = true;

        for primitive in &mesh.borrow().primitives
        {
          let material = &primitive.borrow().material;
          {
            if material.borrow().get_type_name() != "PBRMaterial"
            {
              continue;
            }
            let mut material = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< PBRMaterial >( material.borrow_mut() );
            material.base_color_texture = None;
            material.metallic_factor = 1.0;
            material.roughness_factor = 0.0;
            for i in 0..3
            {
              material.base_color_factor.0[ i ] = color.0[ i ];
            }
            material.base_color_factor.0[ 3 ] = 1.0;
            material.need_update = true;
          }
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
      renderer_mut.set_clear_color( F32x3::splat( 4.0 ) );
    }

    renderer_mut.set_use_emission( true ); 
    renderer_mut.set_bloom_strength( 2.0 );
    renderer_mut.set_exposure( 1.5 );
    renderer_mut.set_bloom_radius( 0.1 );
  }

  fn setup_light( &mut self, gl : &GL )
  {
    self.scene.borrow_mut().add( self.rings.current_ring.clone() );
    self.scene.borrow_mut().update_world_matrix();

    let node = Rc::new( RefCell::new( Node::new() ) );
    let spot = SpotLight
    {
      position : F32x3::from_array( [ 20.0, 40.0, 20.0 ] ),
      direction : F32x3::from_array( [ -1.0, -2.0, -1.0 ] ).normalize(),
      color : F32x3::splat( 1.0 ),
      strength : 20000.0,
      range : 200.0,
      inner_cone_angle : 30_f32.to_radians(),
      outer_cone_angle : 50_f32.to_radians(),
      use_light_map : true
    };

    node.borrow_mut().object = Object3D::Light( Light::Spot( spot.clone() ) );
    self.scene.borrow_mut().add( node.clone() );

    let mut shadow_light = renderer::webgl::shadow::Light::new
    (
      spot.position,
      spot.direction,
      gl::math::mat3x3h::perspective_rh_gl( 100.0_f32.to_radians(), 1.0, 0.1, 100.0 ),
      0.01
    );

    let shadowmap_res = 1024; //4096;
    let lightmap_res = 2048; //8192;
    let mut light_maps = vec![];
    let last_ring = self.rings.current_ring.clone();
    let last_gem = self.rings.current_gem.clone();
    for i in ( 0..self.rings.rings.len() ).rev()
    {
      let new_ring = self.rings.rings.get( i ).unwrap();
      let new_gem = self.rings.gems.get( i ).unwrap();
      remove_node_from_scene( &self.scene, &self.rings.current_ring );
      self.rings.current_ring = new_ring.clone();
      self.rings.current_gem = new_gem.clone();
      self.set_gem_color( F32x3::from_array( [ 1.0, 1.0, 1.0 ] ) );
      self.set_metal_color( F32x3::from_array( [ 0.753, 0.753, 0.753 ] ) );
      self.scene.borrow_mut().add( self.rings.current_ring.clone() );
      self.scene.borrow_mut().update_world_matrix();

      renderer::webgl::shadow::bake_shadows( &gl, &*self.scene.borrow(), &mut shadow_light, lightmap_res, shadowmap_res ).unwrap();
      if self.surface_material.borrow().get_type_name() != "PBRMaterial"
      {
        continue;
      }
      {
        let mut material = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< PBRMaterial >( self.surface_material.borrow_mut() );
        material.need_update = true;
        light_maps.push( material.light_map.clone().unwrap() );
      }
    }
    light_maps.reverse();

    remove_node_from_scene( &self.scene, &self.rings.current_ring );
    self.rings.current_ring = last_ring;
    self.rings.current_gem = last_gem;
    self.scene.borrow_mut().add( self.rings.current_ring.clone() );
    self.scene.borrow_mut().update_world_matrix();

    self.rings.light_maps = light_maps;
  }
}

fn setup_surface
(
  surface : Rc< RefCell< Node > >
)
-> Rc< RefCell< Box< dyn Material > > >
{
  surface.borrow_mut().set_translation( F32x3::from_array( [ 0.0, -20.0, 0.0 ] ) );
  surface.borrow_mut().set_scale( F32x3::from_array( [ 1000.0, 0.1, 1000.0 ] ) );

  let Object3D::Mesh( mesh ) = &surface.borrow().object
  else
  {
    unreachable!();
  };

  mesh.borrow_mut().is_shadow_receiver = true;
  mesh.borrow_mut().is_shadow_caster = true;

  let primitives = &mesh.borrow().primitives;
  let primitive = primitives.first().unwrap();
  let primitive = primitive.borrow();
  let surface_material = primitive.material.clone();

  if surface_material.borrow().get_type_name() == "PBRMaterial"
  {
    let mut material = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< PBRMaterial >( surface_material.borrow_mut() );
    material.base_color_texture = None;
    material.roughness_factor = 1.0;
    material.specular_factor = Some( 0.0 );
    material.metallic_factor = 0.0;
    material.need_use_ibl = false;
    material.need_update = true;
  }

  surface_material
}

pub struct RingsInfo
{
  pub rings : Vec< Rc< RefCell< Node > > >,
  pub gems : Vec< Rc< RefCell< Node > > >,
  pub filters : Vec< HashSet< String > >,
  pub light_maps : Vec< TextureInfo >,
  pub current_ring : Rc< RefCell< Node > >,
  pub current_gem : Rc< RefCell< Node > >,
}

const DELTA_Y : f32 = -2.0;

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

  let mut rings : Vec< Rc< RefCell< Node > > > = vec![];
  let mut gems : Vec< Rc< RefCell< Node > > > = vec![];
  let mut filters : Vec< HashSet< String > > = vec![];

  let mut material_3 = None;

  for i in 0..5
  {
    let gltf = renderer::webgl::loaders::gltf::load( &document, format!( "./gltf/{i}.glb" ).as_str(), &gl ).await?;

    match i
    {
      0 =>
      {
        let gem = get_node( &gltf.scenes[ 0 ], "Object_2".to_string() ).unwrap();
        gem.borrow_mut().set_name( "gem0" );

        let gem_clone = gem.borrow().clone_tree();

        gem_clone.borrow_mut().set_center_to_origin();
        let cube_normal_map_texture = Some( cube_normal_map_generator.generate( gl, &gem_clone ).unwrap() );
        setup_gem_material( &gem, environment_texture, &cube_normal_map_texture );
        let ring = get_node( &gltf.scenes[ 0 ], "Sketchfab_model".to_string() ).unwrap();
        ring.borrow_mut().set_name( "ring0" );

        let mut translation = ring.borrow_mut().get_translation();
        translation.0[ 1 ] += DELTA_Y;
        ring.borrow_mut().set_translation( translation );

        gems.push( gem.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "gem0".to_string() ] ) );
      },
      1 =>
      {
        let gem = get_node( &gltf.scenes[ 0 ], "Object_11".to_string() ).unwrap();
        gem.borrow_mut().set_name( "gem1" );

        let gem_clone = gem.borrow().clone_tree();
        gem_clone.borrow_mut().set_center_to_origin();
        let cube_normal_map_texture = Some( cube_normal_map_generator.generate( gl, &gem_clone ).unwrap() );
        setup_gem_material( &gem, environment_texture, &cube_normal_map_texture );
        let ring = get_node( &gltf.scenes[ 0 ], "Empty.001_6".to_string() ).unwrap();
        ring.borrow_mut().set_name( "ring1" );
        let mut translation = ring.borrow_mut().get_translation();
        translation.0[ 1 ] -= 11.0;
        ring.borrow_mut().set_translation( translation );

        let mut translation = ring.borrow_mut().get_translation();
        translation.0[ 1 ] += DELTA_Y;
        ring.borrow_mut().set_translation( translation );

        ring.borrow_mut().set_scale( F32x3::splat( 5.0 ) );
        gems.push( gem.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "gem1".to_string() ] ) );
      },
      2 =>
      {
        let gem = get_node( &gltf.scenes[ 0 ], "Object_2".to_string() ).unwrap();
        gem.borrow_mut().set_name( "gem2" );

        let gem_clone = gem.borrow().clone_tree();
        gem_clone.borrow_mut().set_center_to_origin();
        let cube_normal_map_texture = Some( cube_normal_map_generator.generate( gl, &gem_clone ).unwrap() );
        setup_gem_material( &gem, environment_texture, &cube_normal_map_texture );
        let ring = get_node( &gltf.scenes[ 0 ], "Sketchfab_model".to_string() ).unwrap();
        ring.borrow_mut().set_name( "ring2" );
        let mut translation = ring.borrow_mut().get_translation();
        translation.0[ 1 ] += 11.0;
        ring.borrow_mut().set_translation( translation );

        let mut translation = ring.borrow_mut().get_translation();
        translation.0[ 1 ] += DELTA_Y;
        ring.borrow_mut().set_translation( translation );

        ring.borrow_mut().set_scale( F32x3::splat( 5.0 ) );
        gems.push( gem.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "gem2".to_string() ] ) );

        let metal = get_node( &gltf.scenes[ 0 ], "Object_3".to_string() ).unwrap();
        let metal_ref = metal.borrow();
        if let Object3D::Mesh( mesh ) = &metal_ref.object
        {
          let primitives = &mesh.borrow().primitives;
          let primitive = primitives.first().unwrap();
          let material = primitive.borrow().material.clone();
          material_3 = Some( material );
        }
      },
      3 =>
      {
        let gem = get_node( &gltf.scenes[ 0 ], "Diamond_Round".to_string() ).unwrap();
        gem.borrow_mut().set_name( "gem3" );

        let gem_clone = gem.borrow().clone_tree();
        gem_clone.borrow_mut().set_center_to_origin();
        let cube_normal_map_texture = Some( cube_normal_map_generator.generate( gl, &gem_clone ).unwrap() );

        let ring = get_node( &gltf.scenes[ 0 ], "Sketchfab_model".to_string() ).unwrap();
        ring.borrow_mut().set_name( "ring3" );

        gems.push( gem.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "gem3".to_string() ] ) );

        let _ = ring.borrow().traverse
        (
          &mut
          |
            node : Rc< RefCell< Node > >
          |
          -> Result< (), gl::WebglError >
          {
            if let Object3D::Mesh( mesh ) = &node.borrow().object
            {
              for primitive in &mesh.borrow().primitives
              {
                primitive.borrow_mut().material = Rc::new( RefCell::new( material_3.clone().unwrap().borrow().dyn_clone() ) );
              }
            }

            Ok( () )
          }
        );

        setup_gem_material( &gem, environment_texture, &cube_normal_map_texture );
      },
      4 =>
      {
        let gem = get_node( &gltf.scenes[ 0 ], "Diamond_Cushion_Square".to_string() ).unwrap();
        gem.borrow_mut().set_name( "gem2" );

        let gem_clone = gem.borrow().clone_tree();
        gem_clone.borrow_mut().set_center_to_origin();
        let cube_normal_map_texture = Some( cube_normal_map_generator.generate( gl, &gem_clone ).unwrap() );
        setup_gem_material( &gem, environment_texture, &cube_normal_map_texture );
        let ring = get_node( &gltf.scenes[ 0 ], "Scene_1".to_string() ).unwrap();
        ring.borrow_mut().set_name( "ring2" );
        let mut translation = ring.borrow_mut().get_translation();
        //translation.0[ 1 ] += 11.0;
        ring.borrow_mut().set_translation( translation );

        let mut translation = ring.borrow_mut().get_translation();
        //translation.0[ 1 ] += DELTA_Y;
        ring.borrow_mut().set_translation( translation );

        ring.borrow_mut().set_scale( F32x3::splat( 1.0 ) );
        gems.push( gem.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "gem3".to_string() ] ) );
      },
      _ => ()
    }
  }

  let ui_state = get_ui_state().unwrap();
  clear_changed();

  let current_ring = rings[ ui_state.ring as usize ].clone();
  let current_gem = gems[ ui_state.ring as usize ].clone();

  Ok
  (
    RingsInfo
    {
      rings,
      gems,
      filters,
      light_maps : vec![],
      current_ring,
      current_gem
    }
  )
}

fn setup_camera( scene : &Rc< RefCell< Scene > >, canvas : &web_sys::HtmlCanvasElement ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let scene_bounding_box = scene.borrow().bounding_box();

  let eye = gl::math::F32x3::from( [ 0.0, 15.0, 0.0 ] );
  let up = gl::math::F32x3::from( [ 1.0, 0.0, 0.0 ] );

  let center = scene_bounding_box.center() + gl::math::F32x3::from( [ 0.0, 10.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.get_controls().borrow_mut().block_pan = true;
  camera.bind_controls( &canvas );

  camera
}

fn setup_gem_material
(
  gem_node : &Rc< RefCell< Node > >,
  environment_texture : &Option< TextureInfo >,
  cube_normal_map_texture : &Option< TextureInfo >
)
{
  if let Object3D::Mesh( mesh ) = &gem_node.borrow().object
  {
    let primitives = &mesh.borrow().primitives;
    let mut gem_material = GemMaterial::default();
    gem_material.cube_normal_map_texture = cube_normal_map_texture.clone();
    gem_material.environment_texture = environment_texture.clone();
    for primitive in primitives
    {
      let material = &primitive.borrow().material;
      *material.borrow_mut() = gem_material.dyn_clone();
    }
  }
}

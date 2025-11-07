//! Renders GLTF files using postprocess effects.
#![ doc( html_root_url = "https://docs.rs/gltf_viewer/latest/gltf_viewer/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders GLTF files using postprocess effects" ) ]

#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]

use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::
{
  GL,
  F32x3,
  JsCast,
  web_sys::wasm_bindgen::closure::Closure
};
use std::collections::HashSet;

use renderer::webgl::
{
  Camera, Light, Node, Object3D, Renderer, Scene, SpotLight, post_processing::{ self, Pass, SwapFramebuffer }
};

mod ui;

fn get_node( scene : &Rc< RefCell< Scene > >, name : String ) -> Option< Rc< RefCell< Node > > >
{
  let mut target = None;
  let _ = scene.borrow_mut().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      if target.is_some()
      {
        return Ok( () );
      }
      if let Some( current_name ) = node.borrow().get_name()
      {
        if name == current_name.clone().into_string()
        {
          target = Some( node.clone() );
          return Err( gl::WebglError::Other( "" ) );
        }
      }
      Ok( () )
    }
  );
  target
}

fn set_gem_color
(
  gl : &GL,
  renderer : &Rc< RefCell< Renderer > >,
  gem_node : &Rc< RefCell< Node > >,
  color : F32x3
)
{
  let Object3D::Mesh( mesh ) = &gem_node.borrow().object
  else
  {
    return;
  };

  mesh.borrow_mut().is_shadow_caster = true;

  for primitive in &mesh.borrow().primitives
  {
    let material = &primitive.borrow().material;
    {
      let mut material = material.borrow_mut();
      for i in 0..3
      {
        material.base_color_factor.0[ i ] = color.0[ i ];
      }
      material.base_color_factor.0[ 3 ] = 1.0;
    }
    renderer.borrow().update_material_uniforms( gl, primitive );
  }
}

fn set_metal_color
(
  gl : &GL,
  renderer : &Rc< RefCell< Renderer > >,
  ring_node : &Rc< RefCell< Node > >,
  filter : &HashSet< String >,
  color : F32x3
)
{
  let _ = ring_node.borrow().traverse
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

      for primitive in &mesh.borrow().primitives
      {
        let material = &primitive.borrow().material;
        {
          let mut material = material.borrow_mut();
          material.double_sided = true;
          material.base_color_texture = None;
          material.roughness_factor = 0.0;
          for i in 0..3
          {
            material.base_color_factor.0[ i ] = color.0[ i ];
          }
          material.base_color_factor.0[ 3 ] = 1.0;
        }
        renderer.borrow().update_material_uniforms( gl, primitive );
      }

      Ok( () )
    }
  );

}

fn remove_node_from_scene( root : &Rc< RefCell< Scene > >, node : &Rc< RefCell< Node > > )
{
  let name = node.borrow().get_name().unwrap();

  let remove_child_ids = root.borrow().children
  .iter()
  .enumerate()
  .filter
  (
    | ( _, n ) |
    {
      if let Some( current_name ) = n.borrow().get_name()
      {
        *current_name.clone().into_string() == *name
      }
      else
      {
        false
      }
    }
  )
  .map( | ( i, _ ) | i )
  .collect::< Vec< _ > >();

  for i in remove_child_ids.iter().rev()
  {
    let _ = root.borrow_mut().children.remove( *i );
  }

  let _ = root.borrow_mut().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let remove_child_ids = node.borrow().get_children()
      .iter()
      .enumerate()
      .filter
      (
        | ( _, n ) |
        {
          if let Some( current_name ) = n.borrow().get_name()
          {
            *current_name.clone().into_string() == *name
          }
          else
          {
            false
          }
        }
      )
      .map( | ( i, _ ) | i )
      .collect::< Vec< _ > >();

      for i in remove_child_ids.iter().rev()
      {
        let _ = node.borrow_mut().remove_child( *i );
      }

      Ok( () )
    }
  );
}

fn _remove_node_from_node( root : &Rc< RefCell< Node > >, node : &Rc< RefCell< Node > > )
{
  let name = node.borrow().get_name().unwrap();
  let _ = root.borrow_mut().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let remove_child_ids = node.borrow().get_children()
      .iter()
      .enumerate()
      .filter
      (
        | ( _, n ) |
        {
          if let Some( current_name ) = n.borrow().get_name()
          {
            *current_name.clone().into_string() == *name
          }
          else
          {
            false
          }
        }
      )
      .map( | ( i, _ ) | i )
      .collect::< Vec< _ > >();

      for i in remove_child_ids.iter().rev()
      {
        let _ = node.borrow_mut().remove_child( *i );
      }

      Ok( () )
    }
  );
}

fn add_resize_callback() -> Rc< RefCell< bool > >
{
  let is_resized = Rc::new( RefCell::new( false ) );
  let _is_resized = is_resized.clone();

  let resize_closure =
  Closure::wrap
  (
    Box::new
    (
      move | _ : web_sys::Event |
      {
        *_is_resized.borrow_mut() = true;
      }
    ) as Box< dyn FnMut( _ ) >
  );

  gl::web_sys::window()
  .unwrap()
  .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
  .unwrap();
  resize_closure.forget();

  is_resized
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf = renderer::webgl::loaders::gltf::load( &document, format!( "./gltf/cube.glb" ).as_str(), &gl ).await?;

  let scene = gltf.scenes[ 0 ].clone(); // Rc::new( RefCell::new( Scene::new() ) );

  let node = Rc::new( RefCell::new( Node::new() ) );
  let spot = SpotLight
  {
    position : F32x3::from_array( [ 0.0, 20.0, 20.0 ] ),
    direction : F32x3::from_array( [ 0.0, -1.0, -1.0 ] ).normalize(),
    color : F32x3::splat( 1.0 ),
    strength : 20000.0,
    range : 200.0,
    inner_cone_angle : 30_f32.to_radians(),
    outer_cone_angle : 50_f32.to_radians(),
    use_light_map : true
  };

  node.borrow_mut().object = Object3D::Light( Light::Spot( spot.clone() ) );
  scene.borrow_mut().add( node );

  let surface = get_node( &scene, "Cube".to_string() ).unwrap();
  surface.borrow_mut().set_translation( F32x3::from_array( [ 0.0, -20.0, 0.0 ] ) );
  surface.borrow_mut().set_scale( F32x3::from_array( [ 200.0, 0.1, 200.0 ] ) );

  if let Object3D::Mesh( mesh ) = &surface.borrow().object
  {
    mesh.borrow_mut().is_shadow_receiver = true;
    mesh.borrow_mut().is_shadow_caster = true;
    for primitives in &mesh.borrow().primitives
    {
      let material = &primitives.borrow().material;
      material.borrow_mut().base_color_texture = None;
      material.borrow_mut().base_color_factor.0[ 3 ] = 0.0;
    }
  }

  let mut rings : Vec< Rc< RefCell< Node > > > = vec![];
  let mut gems : Vec< Rc< RefCell< Node > > > = vec![];
  let mut filters : Vec< HashSet< String > > = vec![];

  for i in 0..3
  {
    let gltf = renderer::webgl::loaders::gltf::load( &document, format!( "./gltf/{i}.glb" ).as_str(), &gl ).await?;

    match i
    {
      0 =>
      {
        let gem = get_node( &gltf.scenes[ 0 ], "Object_2".to_string() ).unwrap();
        gem.borrow_mut().set_name( "gem0" );
        let ring = get_node( &gltf.scenes[ 0 ], "Sketchfab_model".to_string() ).unwrap();
        ring.borrow_mut().set_name( "ring0" );
        gems.push( gem.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "gem0".to_string() ] ) );
      },
      1 =>
      {
        let gem = get_node( &gltf.scenes[ 0 ], "Object_11".to_string() ).unwrap();
        gem.borrow_mut().set_name( "gem1" );
        let ring = get_node( &gltf.scenes[ 0 ], "Empty.001_6".to_string() ).unwrap();
        ring.borrow_mut().set_name( "ring1" );
        let mut translation = ring.borrow_mut().get_translation();
        translation.0[ 1 ] -= 11.0;
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
        let ring = get_node( &gltf.scenes[ 0 ], "Sketchfab_model".to_string() ).unwrap();
        ring.borrow_mut().set_name( "ring2" );
        let mut translation = ring.borrow_mut().get_translation();
        translation.0[ 1 ] += 11.0;
        ring.borrow_mut().set_translation( translation );
        ring.borrow_mut().set_scale( F32x3::splat( 5.0 ) );
        gems.push( gem.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "gem2".to_string() ] ) );
      },
      _ => ()
    }
  }

  let ui_state = ui::get_ui_state().unwrap();
  ui::clear_changed();

  let mut current_ring = rings[ ui_state.ring as usize ].clone();
  let mut current_gem = gems[ ui_state.ring as usize ].clone();

  scene.borrow_mut().add( current_ring.clone() );
  scene.borrow_mut().update_world_matrix();

  let scene_bounding_box = scene.borrow().bounding_box();

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 0.0, 35.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.get_controls().borrow_mut().block_pan = true;
  camera.bind_controls( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  let ibl = renderer::webgl::loaders::ibl::load( &gl, "envMap", Some( 0..0 ) ).await;
  renderer.set_ibl( ibl.clone() );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  renderer.borrow_mut().set_clear_color( F32x3::splat( 1.0 ) );
  renderer.borrow_mut().set_exposure( 1.0 );

  match ui_state.gem.as_str()
  {
    "white" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 1.0, 1.0 ] ) ),
    "black" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.0, 0.0, 0.0 ] ) ),
    "red" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 0.0, 0.0 ] ) ),
    "orange" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 0.5, 0.0 ] ) ),
    "yellow" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 1.0, 0.0 ] ) ),
    "green" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.0, 1.0, 0.0 ] ) ),
    "turquoise" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.25, 0.88, 0.82 ] ) ),
    "light_blue" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.53, 0.81, 0.92 ] ) ),
    "blue" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.0, 0.0, 1.0 ] ) ),
    "violet" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.5, 0.0, 0.5 ] ) ),
    "pink" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 0.41, 0.71 ] ) ),
    _ => ()
  }
  match ui_state.metal.as_str()
  {
    "silver" => set_metal_color( &gl, &renderer, &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 0.753, 0.753, 0.753 ] ) ),
    "copper" => set_metal_color( &gl, &renderer, &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 1.0, 0.4, 0.2 ] ) ),
    "gold" => set_metal_color( &gl, &renderer, &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 1.0, 0.65, 0.02 ] ) ),
    _ => ()
  }

  let mut shadow_light = renderer::webgl::shadow::Light::new
  (
    spot.position,
    spot.direction,
    gl::math::mat3x3h::perspective_rh_gl( 100.0_f32.to_radians(), 1.0, 0.1, 100.0 ),
    0.7
  );

  let shadowmap_res = 4096;
  let lightmap_res = 8192;
  renderer::webgl::shadow::bake_shadows( &gl, &*scene.borrow(), &mut shadow_light, lightmap_res, shadowmap_res ).unwrap();

  let is_resized = add_resize_callback();

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      if *is_resized.borrow()
      {
        if let Ok( r ) = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )
        {
          let mut renderer_mut = renderer.borrow_mut();
          *renderer_mut = r;
          renderer_mut.set_ibl( ibl.clone() );
          renderer_mut.set_exposure( 1.0 );

          match ui::get_ui_state().unwrap().light_mode.as_str()
          {
            "light" =>
            {
              renderer_mut.set_clear_color( F32x3::splat( 1.0 ) );
              renderer_mut.set_exposure( 1.0 );
            },
            "dark" =>
            {
              renderer_mut.set_clear_color( F32x3::splat( 0.2 ) );
              renderer_mut.set_exposure( 0.5 );
            }
            _ => ()
          }

          swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

          camera.set_window_size( [ canvas.width() as f32, canvas.height() as f32 ].into() );
          let aspect = canvas.width() as f32 / canvas.height() as f32;
          let perspective = gl::math::d2::mat3x3h::perspective_rh_gl( 70.0f32.to_radians(), aspect, 0.1, 1000.0 );
          camera.set_projection_matrix( perspective );

          *is_resized.borrow_mut() = false;
        }
      }

      if ui::is_changed()
      {
        if let Some( ui_state ) = ui::get_ui_state()
        {
          let ring_changed = ui_state.changed.contains( &"ring".to_string() );

          if ring_changed
          {
            if let Some( new_gem ) = gems.get( ui_state.ring as usize ).cloned()
            {
              current_gem = new_gem;
            }
            if let Some( new_ring ) = rings.get( ui_state.ring as usize ).cloned()
            {
              remove_node_from_scene( &scene, &current_ring );
              current_ring = new_ring;
              scene.borrow_mut().add( current_ring.clone() );
              scene.borrow_mut().update_world_matrix();
            }
          }

          if ui_state.changed.contains( &"lightMode".to_string() )
          {
            match ui::get_ui_state().unwrap().light_mode.as_str()
            {
              "light" =>
              {
                renderer.borrow_mut().set_clear_color( F32x3::splat( 1.0 ) );
                renderer.borrow_mut().set_exposure( 1.0 );
              },
              "dark" =>
              {
                renderer.borrow_mut().set_clear_color( F32x3::splat( 0.2 ) );
                renderer.borrow_mut().set_exposure( 0.5 );
              }
              _ => ()
            }
          }

          if ui_state.changed.contains( &"gem".to_string() ) || ring_changed
          {
            match ui_state.gem.as_str()
            {
              "white" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 1.0, 1.0 ] ) ),
              "black" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.0, 0.0, 0.0 ] ) ),
              "red" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 0.0, 0.0 ] ) ),
              "orange" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 0.5, 0.0 ] ) ),
              "yellow" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 1.0, 0.0 ] ) ),
              "green" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.0, 1.0, 0.0 ] ) ),
              "turquoise" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.25, 0.88, 0.82 ] ) ),
              "light_blue" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.53, 0.81, 0.92 ] ) ),
              "blue" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.0, 0.0, 1.0 ] ) ),
              "violet" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 0.5, 0.0, 0.5 ] ) ),
              "pink" => set_gem_color( &gl, &renderer, &current_gem, F32x3::from_array( [ 1.0, 0.41, 0.71 ] ) ),
              _ => ()
            }
          }

          if ui_state.changed.contains( &"metal".to_string() ) || ring_changed
          {
            match ui_state.metal.as_str()
            {
              "silver" => set_metal_color( &gl, &renderer, &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 0.753, 0.753, 0.753 ] ) ),
              "copper" => set_metal_color( &gl, &renderer, &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 1.0, 0.4, 0.2 ] ) ),
              "gold" => set_metal_color( &gl, &renderer, &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 1.0, 0.65, 0.02 ] ) ),
              _ => ()
            }
          }

          ui::clear_changed();
        }
      }

      // If textures are of different size, gl.view_port needs to be called
      let _time = t as f32 / 1000.0;

      renderer.borrow_mut().render( &gl, &mut scene.borrow_mut(), &camera ).expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( renderer.borrow().get_main_texture() );

      let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.set_output( t );
      swap_buffer.swap();

      let _ = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render ToSrgbPass" );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}

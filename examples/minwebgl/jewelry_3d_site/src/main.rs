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
use gl::F32x3;
use std::collections::HashSet;

use renderer::webgl::
{
  post_processing::{ self, Pass, SwapFramebuffer },
  Camera,
  Renderer,
  Node,
  Object3D,
  Scene
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

fn set_diamond_color( diamond_node : &Rc< RefCell< Node > >, color : F32x3 )
{
  let Object3D::Mesh( mesh ) = &diamond_node.borrow().object
  else
  {
    return;
  };

  for primitive in &mesh.borrow().primitives
  {
    let material = &primitive.borrow().material;
    let mut material = material.borrow_mut();
    for i in 0..3
    {
      material.base_color_factor.0[ i ] = color.0[ i ];
    }
  }
}

fn set_metal_color( ring_node : &Rc< RefCell< Node > >, filter : &HashSet< String >, color : F32x3 )
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

      let Object3D::Mesh( mesh ) = &ring_node.borrow().object
      else
      {
        return Ok( () );
      };

      for primitive in &mesh.borrow().primitives
      {
        let material = &primitive.borrow().material;
        let mut material = material.borrow_mut();
        for i in 0..3
        {
          material.base_color_factor.0[ i ] = color.0[ i ];
        }
      }

      Ok( () )
    }
  );

}

fn remove_node_from_scene( root : &Rc< RefCell< Scene > >, node : &Rc< RefCell< Node > > )
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

  let scene = Rc::new( RefCell::new( Scene::new() ) );
  let mut rings : Vec< Rc< RefCell< Node > > > = vec![];
  let mut diamonds : Vec< Rc< RefCell< Node > > > = vec![];
  let mut filters : Vec< HashSet< String > > = vec![];

  for i in 0..3
  {
    let gltf = renderer::webgl::loaders::gltf::load( &document, format!( "./gltf/{i}.glb" ).as_str(), &gl ).await?;

    match i
    {
      0 =>
      {
        let diamond = get_node( &gltf.scenes[ 0 ], "Object_2".to_string() ).unwrap();
        let ring = get_node( &gltf.scenes[ 0 ], "Sketchfab_model".to_string() ).unwrap();
        diamonds.push( diamond.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "Object_2".to_string() ] ) );
      },
      1 =>
      {
        let diamond = get_node( &gltf.scenes[ 0 ], "object_2_Vien KC Lon_0".to_string() ).unwrap();
        let ring = get_node( &gltf.scenes[ 0 ], "Sketchfab_model".to_string() ).unwrap();
        diamonds.push( diamond.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "object_2_Vien KC Lon_0".to_string() ] ) );
      },
      2 =>
      {
        let diamond = get_node( &gltf.scenes[ 0 ], "Object_2".to_string() ).unwrap();
        let ring = get_node( &gltf.scenes[ 0 ], "Sketchfab_model".to_string() ).unwrap();
        diamonds.push( diamond.clone() );
        rings.push( ring.clone() );
        filters.push( HashSet::from( [ "Object_2".to_string() ] ) );
      },
      _ => ()
    }
  }

  let ui_state = ui::get_ui_state().unwrap();
  ui::clear_changed();

  let mut current_ring = rings[ 2 ].clone();
  let mut current_diamond = diamonds[ 2 ].clone();

  // let mut current_ring = rings[ ui_state.ring as usize ].clone();
  // let mut current_diamond = diamonds[ ui_state.ring as usize ].clone();

  scene.borrow_mut().add( current_ring.clone() );
  scene.borrow_mut().update_world_matrix();

  let scene_bounding_box = scene.borrow().bounding_box();

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.bind_controls( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "envMap" ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  renderer.borrow_mut().set_clear_color( F32x3::splat( 0.8 ) );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      if ui::is_changed()
      {
        if let Some( ui_state ) = ui::get_ui_state()
        {
          let ring_changed = ui_state.changed.contains( &"ring".to_string() );

          if ring_changed
          {
            if let Some( new_diamond ) = diamonds.get( ui_state.ring as usize ).cloned()
            {
              current_diamond = new_diamond;
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
            match ui_state.light_mode.as_str()
            {
              "light" => renderer.borrow_mut().set_clear_color( F32x3::splat( 0.8 ) ),
              "dark" => renderer.borrow_mut().set_clear_color( F32x3::splat( 0.2 ) ),
              _ => ()
            }
          }

          if ui_state.changed.contains( &"diamond".to_string() ) || ring_changed
          {
            match ui_state.diamond.as_str()
            {
              "white" => set_diamond_color( &current_diamond, F32x3::from_array( [ 1.0, 1.0, 1.0 ] ) ),
              "ruby" => set_diamond_color( &current_diamond, F32x3::from_array( [ 1.0, 0.0, 0.0 ] ) ),
              "emerald" => set_diamond_color( &current_diamond, F32x3::from_array( [ 0.0, 1.0, 0.0 ] ) ),
              _ => ()
            }
          }

          if ui_state.changed.contains( &"metal".to_string() ) || ring_changed
          {
            match ui_state.metal.as_str()
            {
              "silver" => set_metal_color( &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 0.753, 0.753, 0.753 ] ) ),
              "copper" => set_metal_color( &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 0.722, 0.451, 0.2 ] ) ),
              "gold" => set_metal_color( &current_ring, &filters[ ui_state.ring as usize ], F32x3::from_array( [ 1.0, 0.843, 0.0 ] ) ),
              _ => ()
            }
          }

          ui::clear_changed();
        }
      }

      // If textures are of different size, gl.view_port needs to be called
      let _time = t as f32 / 1000.0;

      renderer.borrow_mut().render( &gl, &mut scene.borrow_mut(), &camera )
      .expect( "Failed to render" );

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

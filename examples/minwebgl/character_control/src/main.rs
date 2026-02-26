//! Renders animated character that can be controlled with WASD + mouse.
//!
//! This demo showcases the CharacterControls system:
//! - WASD keys for movement (W=forward, S=backward, A=strafe left, D=strafe right)
//! - Mouse movement for rotation (yaw and pitch)
//! - Click on canvas to enable mouse control
//! - ESC to release mouse
#![ doc( html_root_url = "https://docs.rs/character_control/latest/character_control/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders animated character that can be controlled with WASD + mouse" ) ]

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

use core::f32;
use animation::easing::{ EasingBuilder, Linear };
// use renderer::webgl::cast_unchecked_material_to_ref_mut;
// use renderer::webgl::material::PbrMaterial;
use rustc_hash::FxHashMap;
use std::{ cell::RefCell, rc::Rc };
use animation::{ Sequencer, Sequence, Tween };
use mingl::{ F32x3, F64x3, Quat, QuatF32 };
use mingl::controls::{ CharacterControls, CharacterInput };
use minwebgl::{ self as gl, WebglError };
use gl::{ JsCast, GL };
use renderer::webgl::animation::{ Pose, AnimatableComposition, Animation, AnimationGraph, AnimationEdge, Mirror, MirrorPlane };
use renderer::webgl::
{
  post_processing::
  {
    self,
    Pass,
    SwapFramebuffer
  },
  loaders::gltf::GLTF,
  Camera,
  Renderer,
  Scene
};
use primitive_generation::
{
  primitives_data_to_gltf,
  plane_to_geometry
};
use web_sys::HtmlCanvasElement;

/// Add new plane [`renderer::webgl::Node`] to [`Scene`]
fn create_plane( gl : &GL, scene : &Rc< RefCell< Scene > > )
{
  let Some( plane ) = plane_to_geometry()
  else
  {
    return;
  };
  let gltf = primitives_data_to_gltf( gl, vec![ plane ] );
  if let Some( plane ) = gltf.nodes.first()
  {
    // if let Object3D::Mesh( mesh ) = &plane.borrow().object
    // {
    //   let mesh_ref = mesh.borrow();
    //   let primitive_ref = mesh_ref.primitives.first().unwrap().borrow();
    //   let mut material = cast_unchecked_material_to_ref_mut::< PbrMaterial >
    //   (
    //     primitive_ref.material.borrow_mut()
    //   );

    //   material.base_color_texture = Some( create_texture( gl, "textures/chessboard.jpg" ).unwrap() );
    //   material.needs_update = true;
    // };
    plane.borrow_mut().set_name( "Plane" );
    scene.borrow_mut().children.push( plane.clone() );
  }
}

async fn setup_scene( gl : &GL ) -> Result< GLTF, WebglError >
{
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let gltf_path = "gltf/multi_animation_extended.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  gltf.scenes[ 0 ].borrow_mut().update_world_matrix();

  create_plane( &gl, &gltf.scenes[ 0 ] );

  let character = gltf.scenes[ 0 ].borrow().get_nodes_by_substring( "Armature" )[ 0 ].clone();
  let plane = gltf.scenes[ 0 ].borrow().get_nodes_by_substring( "Plane" )[ 0 ].clone();

  character.borrow_mut().set_scale( F32x3::splat( 0.1 ) );
  character.borrow_mut().set_rotation( QuatF32::from_angle_y( 0.0 ) );

  plane.borrow_mut().set_scale( F32x3::splat( 100.0 ) );
  plane.borrow_mut().set_rotation( QuatF32::from_angle_x( f32::consts::PI / 2.0 ) );

  gltf.scenes[ 0 ].borrow_mut().update_world_matrix();

  Ok( gltf )
}

fn setup_camera( width : f32, height : f32 ) -> Camera
{
  // Camera setup - will follow character
  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

  // Initial camera position
  let eye = F32x3::from( [ 0.0, 1.5, 3.0 ] );
  let up = F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );

  camera
}

fn setup_input( canvas : &HtmlCanvasElement ) -> ( Rc< RefCell< CharacterControls > >, Rc< RefCell< CharacterInput > > )
{
  // Character controls setup
  let mut character_controls = CharacterControls::default();

  character_controls.set_position( F64x3::from( [ 0.0, 1.5, 3.0 ] ) );
  character_controls.set_rotation( 0.0, 0.0 );

  character_controls.rotation_sensitivity = 0.003;

  let character_controls = Rc::new( RefCell::new( character_controls ) );
  let character_input = Rc::new( RefCell::new( CharacterInput::new() ) );

  // Bind character controls to input
  mingl::controls::character_controls::bind_controls_to_input
  (
    &canvas,
    &character_controls,
    &character_input
  );

  ( character_controls, character_input )
}

fn setup_graph( animations : Vec< Animation >, input_ : &Rc< RefCell< browser_input::Input > > ) -> AnimationGraph
{
  let mut graph = AnimationGraph::new( &animations[ 0 ].nodes );

  let mut animations = animations.into_iter()
  .filter_map( | a | Some( ( a.name?.into_string(), a.animation.as_any().downcast_ref::< Sequencer >().unwrap().clone() ) ) )
  .collect::< FxHashMap< String, Sequencer > >();

  let leave_global_translation =
  [
    "standing_jump",
  ];

  for ( name, animation ) in &mut animations
  {
    if !leave_global_translation.contains( &name.as_str() )
    {
      animation.remove( "mixamorig:Hips.translation" );
    }
  }

  if let Some( run_jump ) = animations.get_mut( "running_jump" )
  {
    if let Some( sequence ) = run_jump.get_mut::< Sequence< Tween< F64x3 > > >( "mixamorig:Hips.translation" )
    {
      for player in sequence.players_mut()
      {
        player.start_value.0[ 0 ] = - player.start_value.x();
        player.end_value.0[ 0 ] = - player.end_value.x();
        player.start_value.0[ 2 ] = - player.start_value.z();
        player.end_value.0[ 2 ] = - player.end_value.z();
      }
    }
  }

  let instant_tween = Tween::new( 1.0, 1.0, 0.0, Linear::new() );
  let true_condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    true
  };

  graph.node_add( "idle", animations.get( "happy_idle" ).unwrap().clone() );
  graph.node_add( "jump", animations.get( "standing_jump" ).unwrap().clone() );

  let input = input_.clone();
  let tween = Tween::new( 1.0, 1.0, 2.4, Linear::new() );
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::Space )
  };
  graph.edge_add( "idle", "jump", "idle_to_jump", tween, condition );

  graph.edge_add( "jump", "idle", "jump_to_idle", instant_tween.clone(), true_condition.clone() );

  graph.node_add( "walk", animations.get( "female_walk" ).unwrap().clone() );
  graph.node_add( "walk_backward", animations.get( "running_backward" ).unwrap().clone() );
  graph.node_add( "walk_left", animations.get( "walk_strafe_left" ).unwrap().clone() );

  let mut walk_right = animations.get( "walk_strafe_left" ).unwrap().clone();
  walk_right = Mirror::along_plane( &walk_right, MirrorPlane::YZ );
  graph.node_add( "walk_right", walk_right );

  let stop_walk = animations.get( "female_stop_walking" ).unwrap().clone();
  graph.node_add( "stop_walk", stop_walk );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyW )
  };
  graph.edge_add( "idle", "walk", "idle_to_walk", instant_tween.clone(), condition );

  let input = input_.clone();
  let tween = Tween::new( 1.0, 1.0, 1.55, Linear::new() );
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyW )
  };
  graph.edge_add( "walk", "stop_walk", "walk_to_stop_walk", tween, condition );

  graph.edge_add( "stop_walk", "idle", "stop_walk_to_idle", instant_tween.clone(), true_condition.clone() );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyS )
  };
  graph.edge_add( "idle", "walk_backward", "idle_to_walk_backward", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyS )
  };
  graph.edge_add( "walk_backward", "idle", "walk_backward_to_idle", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyA )
  };
  graph.edge_add( "idle", "walk_left", "idle_to_walk_left", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyA )
  };
  graph.edge_add( "walk_left", "idle", "walk_left_to_idle", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyD )
  };
  graph.edge_add( "idle", "walk_right", "idle_to_walk_right", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyD )
  };
  graph.edge_add( "walk_right", "idle", "walk_right_to_idle", instant_tween.clone(), condition );

  graph.node_add( "run", animations.get( "run_forward" ).unwrap().clone() );
  graph.node_add( "run_backward", animations.get( "running_backward" ).unwrap().clone() );
  graph.node_add( "run_jump", animations.get( "running_jump" ).unwrap().clone() );

  let input = input_.clone();
  let tween = Tween::new( 1.0, 1.0, 0.9, Linear::new() );
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyW ) &&
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::ShiftLeft ) &&
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::Space )
  };
  graph.edge_add( "run", "run_jump", "run_to_run_jump", tween, condition );

  graph.edge_add( "run_jump", "run", "run_jump_to_run", instant_tween.clone(), true_condition.clone() );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyW ) && input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::ShiftLeft )
  };
  graph.edge_add( "walk", "run", "walk_to_run", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyW ) && !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::ShiftLeft )
  };
  graph.edge_add( "run", "walk", "run_to_walk", instant_tween.clone(), condition );

  let input = input_.clone();
  let tween = Tween::new( 1.0, 1.0, 1.55, Linear::new() );
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyW )
  };
  graph.edge_add( "run", "stop_walk", "run_to_stop_walk", tween, condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyS ) && input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::ShiftLeft )
  };
  graph.edge_add( "walk_backward", "run_backward", "walk_backward_to_run_backward", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyS ) && !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::ShiftLeft )
  };
  graph.edge_add( "run_backward", "walk_backward", "run_backward_to_walk_backward", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyS )
  };
  graph.edge_add( "run_backward", "idle", "run_backward_to_idle", instant_tween.clone(), condition );

  graph.node_add( "idle_to_fight", animations.get( "standing_idle_to_fight_idle" ).unwrap().clone() );
  graph.node_add( "fight_to_idle", animations.get( "fight_idle_to_standing_idle" ).unwrap().clone() );
  graph.node_add( "arm_kick", animations.get( "punching" ).unwrap().clone() );
  graph.node_add( "leg_kick", animations.get( "mma_kick" ).unwrap().clone() );

  let input = input_.clone();
  let tween = Tween::new( 1.0, 1.0, 1.0, Linear::new() );
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyE ) ||
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyQ )
  };
  graph.edge_add( "idle", "idle_to_fight", "idle_to_idle_to_fight", tween.clone(), condition );

  let tween = Tween::new( 0.0, 0.0, 1.0, Linear::new() );
  graph.edge_add( "fight_to_idle", "idle", "fight_to_idle_to_idle", tween.clone(), true_condition.clone() );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyE )
  };
  graph.edge_add( "idle_to_fight", "arm_kick", "idle_to_fight_to_arm_kick", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyQ )
  };
  graph.edge_add( "idle_to_fight", "leg_kick", "idle_to_fight_to_leg_kick", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyE )
  };
  graph.edge_add( "arm_kick", "fight_to_idle", "arm_kick_to_fight_to_idle", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyQ )
  };
  graph.edge_add( "leg_kick", "fight_to_idle", "leg_kick_to_fight_to_idle", instant_tween.clone(), condition );

  let input = input_.clone();
  let condition = move | _edge : &AnimationEdge, _p1 : &Pose, _p2 : &Pose |
  {
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyE ) &&
    !input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyQ )
  };
  graph.edge_add( "idle_to_fight", "fight_to_idle", "idle_to_fight_to_fight_to_idle", instant_tween.clone(), condition );

  graph
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );
  let _ = gl.get_extension( "EXT_shader_image_load_store" ).expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf = setup_scene( &gl ).await?;
  let scene = gltf.scenes[ 0 ].clone();
  let ( character_controls, character_input ) = setup_input( &canvas );
  let camera = setup_camera( width, height );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "envMap", None ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let last_time = Rc::new( RefCell::new( 0.0 ) );

  let character = scene.borrow().get_nodes_by_substring( "Armature" )[ 0 ].clone();

  let mut initial_center = character.borrow().get_translation();
  initial_center.0[ 1 ] += 1.5;
  camera.get_controls().borrow_mut().center = initial_center;

  character_controls.borrow_mut().set_rotation( 0.0, 0.0 );
  let forward = F32x3::from_array( character_controls.borrow().forward().map( | v | v as f32 ) );
  camera.get_controls().borrow_mut().eye = initial_center - forward * character_controls.borrow().zoom as f32;

  let input = Rc::new( RefCell::new( browser_input::Input::new( Some( canvas.clone().dyn_into().unwrap() ), browser_input::CLIENT ).expect( "Failed to initialize input" ) ) );
  let mut graph = setup_graph( gltf.animations.clone(), &input );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      input.borrow_mut().update_state();

      let time = t / 1000.0;

      let last_time = last_time.clone();

      let delta_time = time - *last_time.borrow();
      *last_time.borrow_mut() = time;

      graph.update( delta_time );
      graph.set( graph.animated_nodes_get() );

      character_controls.borrow_mut().update( &character_input.borrow(), delta_time );

      let mut position = F32x3::from_array( character_controls.borrow().position().map( | v | v as f32 ) );
      position.0[ 1 ] -= 1.5;

      character.borrow_mut().set_translation( position );

      scene.borrow_mut().update_world_matrix();

      let mut center = ( character.borrow().get_world_matrix() * character.borrow().get_translation().to_homogenous() ).truncate();
      center.0[ 1 ] += 1.5;
      camera.get_controls().borrow_mut().center = center;

      if input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyW ) ||
      input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyS ) ||
      input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyA ) ||
      input.borrow().is_key_down( browser_input::keyboard::KeyboardKey::KeyD )
      {
        character.borrow_mut().set_rotation( Quat::from_angle_y( character_controls.borrow().yaw() as f32 / 2.0 ) );
      }

      let forward = F32x3::from_array( character_controls.borrow().forward().map( | v | v as f32 ) );
      camera.get_controls().borrow_mut().eye = center - forward * character_controls.borrow().zoom as f32;

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

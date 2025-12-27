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
use rustc_hash::FxHashMap;
use std::{ cell::RefCell, rc::Rc };
use animation::Sequencer;
use mingl::{ F32x3, F64x3, QuatF32 };
use mingl::controls::{ CharacterControls, CharacterInput };
use minwebgl::{self as gl, WebglError};
use gl::{ JsCast, web_sys::WebGlTexture, GL, wasm_bindgen::closure::Closure };
use renderer::webgl::animation::AnimatableComposition;
use renderer::webgl::
{
  post_processing::
  {
    self,
    Pass,
    SwapFramebuffer
  },
  loaders::gltf::GLTF,
  animation::{ Animation, AnimationGraph },
  Camera,
  Renderer,
  Scene,
  Node,
  TextureInfo,
  Texture,
  WrappingMode,
  MagFilterMode,
  MinFilterMode,
  Sampler
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
    //   mesh.borrow().primitives.first().unwrap().borrow()
    //   .material.borrow_mut()
    //   .base_color_texture = create_texture( gl, "textures/chessboard.jpg" );
    // };
    plane.borrow_mut().set_name( "Plane" );
    scene.borrow_mut().children.push( plane.clone() );
  }
}

/// Finds [`Node`] in [`Scene`] by name
fn find_node( scene : &Rc< RefCell< Scene > >, substring : &str ) -> Option< Rc< RefCell< Node > > >
{
  let mut target_node = None;
  let _ = scene.borrow().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let name = node.borrow().get_name();
      if let Some( name ) = name
      {
        if name.contains( substring )
        {
          target_node = Some( node.clone() );
        }
      }

      Ok( () )
    }
  );

  target_node
}

/// Uploads an image from a URL to a WebGL texture.
///
/// This function creates a new `WebGlTexture` and asynchronously loads an image from the provided URL into it.
/// It uses a `Closure` to handle the `onload` event of an `HtmlImageElement`, ensuring the texture is
/// uploaded only after the image has finished loading.
///
/// # Arguments
///
/// * `gl` - The WebGl2RenderingContext.
/// * `src` - A reference-counted string containing the URL of the image to load.
///
/// # Returns
///
/// A `WebGlTexture` object.
fn _upload_texture( gl : &GL, src : Rc< String > ) -> WebGlTexture
{
  let window = web_sys::window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );

  let texture = gl.create_texture().expect( "Failed to create a texture" );

  let img_element = document.create_element( "img" )
  .expect( "Can't create img" )
  .dyn_into::< gl::web_sys::HtmlImageElement >()
  .expect( "Can't convert to gl::web_sys::HtmlImageElement" );
  img_element.style().set_property( "display", "none" ).expect( "Can't set property" );
  let load_texture : Closure< dyn Fn() > = Closure::new
  (
    {
      let gl = gl.clone();
      let img = img_element.clone();
      let texture = texture.clone();
      move ||
      {
        gl::texture::d2::upload_no_flip( &gl, Some( &texture ), &img );
        gl.generate_mipmap( gl::TEXTURE_2D );
        img.remove();
      }
    }
  );

  img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
  img_element.set_src( &src );
  load_texture.forget();

  texture
}

/// Creates a new `TextureInfo` struct with a texture loaded from a file.
///
/// This function calls `upload_texture` to load an image, sets up a default `Sampler`
/// with linear filtering and repeat wrapping, and then combines them into a `TextureInfo`
/// struct.
///
/// # Arguments
///
/// * `gl` - The WebGl2RenderingContext.
/// * `image_path` - The path to the image file, relative to the `static/` directory.
///
/// # Returns
///
/// An `Option<TextureInfo>` containing the texture data, or `None` if creation fails.
fn _create_texture
(
  gl : &GL,
  image_path : &str
) -> Option< TextureInfo >
{
  let image_path = format!( "static/{image_path}" );
  let texture_id = _upload_texture( gl, Rc::new( image_path ) );

  let sampler = Sampler::former()
  .min_filter( MinFilterMode::Linear )
  .mag_filter( MagFilterMode::Linear )
  .wrap_s( WrappingMode::Repeat )
  .wrap_t( WrappingMode::Repeat )
  .end();

  let texture = Texture::former()
  .target( GL::TEXTURE_2D )
  .source( texture_id )
  .sampler( sampler )
  .end();

  let texture_info = TextureInfo
  {
    texture : Rc::new( RefCell::new( texture ) ),
    uv_position : 0,
  };

  Some( texture_info )
}

async fn setup_scene( gl : &GL ) -> Result< GLTF, WebglError >
{
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let gltf_path = "gltf/multi_animation_extended.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  gltf.scenes[ 0 ].borrow_mut().update_world_matrix();

  create_plane( &gl, &gltf.scenes[ 0 ] );

  let character = find_node( &gltf.scenes[ 0 ], "Armature" ).unwrap();
  let plane = find_node( &gltf.scenes[ 0 ], "Plane" ).unwrap();

  character.borrow_mut().set_scale( F32x3::splat( 0.1 ) );
  character.borrow_mut().set_rotation( QuatF32::from_angle_x( f32::consts::PI / 4.0 ).normalize() );

  plane.borrow_mut().set_scale( F32x3::splat( 100.0 ) );
  plane.borrow_mut().set_rotation( QuatF32::from_angle_x( f32::consts::PI / 2.0 ).normalize() );

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

fn setup_graph( animations : Vec< Animation > ) -> AnimationGraph
{
  let mut graph = AnimationGraph::new( &animations[ 0 ].nodes );

  let animations = animations.into_iter()
  .filter_map( | a | Some( ( a.name?.into_string(), a.animation.as_any().downcast_ref::< Sequencer >().unwrap().clone() ) ) )
  .collect::< FxHashMap< String, Sequencer > >();

  graph.node_add( "idle".into(), animations.get( "happy_idle" ).unwrap().clone() );
  // graph.node_add( "walk",  );

  // graph.edge_add( a, b, name, tween, condition );

  graph
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

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

  let character = find_node( &scene, "Armature" ).unwrap();
  let neck = find_node( &scene, "Neck" ).unwrap();

  let mut initial_center = character.borrow().get_translation();
  initial_center.0[ 1 ] += 1.5;
  camera.get_controls().borrow_mut().center = initial_center;

  character_controls.borrow_mut().set_rotation( 0.0, 0.0 );
  let forward = F32x3::from_array( character_controls.borrow().forward().map( | v | v as f32 ) );
  camera.get_controls().borrow_mut().eye = initial_center - forward * character_controls.borrow().zoom as f32;

  let mut graph = setup_graph( gltf.animations.clone() );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = t / 1000.0;

      let last_time = last_time.clone();

      let delta_time = time - *last_time.borrow();
      *last_time.borrow_mut() = time;

      graph.update( delta_time );
      graph.set( graph.animated_nodes_get() );

      character_controls.borrow_mut().update( &character_input.borrow(), delta_time );

      let mut position = F32x3::from_array( character_controls.borrow().position().map( | v | v as f32 ) );
      let rotation = QuatF32::from_angle_x( -3.8_f32.to_radians() );
      let q_position = QuatF32::from( position.to_homogenous().0 );
      position = ( rotation * q_position * rotation.conjugate() ).0.truncate();
      position.0[ 1 ] -= 1.5;

      character.borrow_mut().set_translation( position );

      scene.borrow_mut().update_world_matrix();

      neck.borrow_mut().set_rotation( QuatF32::from( character_controls.borrow().rotation().0.map( | v | v as f32 ) ) );

      let mut center = ( character.borrow().get_world_matrix() * character.borrow().get_translation().to_homogenous() ).truncate();
      center.0[ 1 ] += 1.5;
      camera.get_controls().borrow_mut().center = center;

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

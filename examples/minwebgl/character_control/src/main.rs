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
use std::{ cell::RefCell, rc::Rc };
use mingl::{ F32x3, F64x3, QuatF32 };
use mingl::controls::{ CharacterControls, CharacterInput };
use minwebgl as gl;
use gl::{ JsCast, web_sys::WebGlTexture, GL, wasm_bindgen::closure::Closure };
use renderer::webgl::
{
  post_processing::
  {
    self,
    Pass,
    SwapFramebuffer
  },
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
fn upload_texture( gl : &GL, src : Rc< String > ) -> WebGlTexture
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
fn create_texture
(
  gl : &GL,
  image_path : &str
) -> Option< TextureInfo >
{
  let image_path = format!( "static/{image_path}" );
  let texture_id = upload_texture( gl, Rc::new( image_path ) );

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

  let gltf_path = "gltf/multi_animation_extended.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;
  let scenes = gltf.scenes;
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  gl::info!( "Scene bounding box: {:?}", scene_bounding_box );
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag();
  // let dist = scene_bounding_box.max.mag();
  let exponent =
  {
    let bits = diagonal.to_bits();
    let exponent_field = ( ( bits >> 23 ) & 0xFF ) as i32;
    exponent_field - 127
  };
  gl::info!( "Exponent: {:?}", exponent );

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

  // Camera setup - will follow character
  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1 * 10.0f32.powi( exponent ).min( 1.0 ) * 2000.0;
  let far = near * 100.0f32.powi( exponent.abs() ) / 100.0;

  // Initial camera position
  let eye = F32x3::from( [ 0.0, 1.5, 3.0 ] );
  let up = F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  // camera.bind_controls( &canvas );

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "envMap", None ).await );

  let renderer = Rc::new( RefCell::new( renderer ) );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let last_time = Rc::new( RefCell::new( 0.0 ) );

  create_plane( &gl, &scenes[ 0 ] );

  let character = find_node( &scenes[ 0 ], "Armature" ).unwrap();
  let neck = find_node( &scenes[ 0 ], "Neck" ).unwrap();
  let plane = find_node( &scenes[ 0 ], "Plane" ).unwrap();

  character.borrow_mut().set_scale( F32x3::splat( 0.1 ) );
  character.borrow_mut().set_rotation( QuatF32::from_angle_x( f32::consts::PI / 4.0 ).normalize() );

  plane.borrow_mut().set_scale( F32x3::splat( 100.0 ) );
  plane.borrow_mut().set_rotation( QuatF32::from_angle_x( f32::consts::PI / 2.0 ).normalize() );

  scenes[ 0 ].borrow_mut().update_world_matrix();

  let mut initial_center = character.borrow().get_translation();
  initial_center.0[ 1 ] += 1.5;
  camera.get_controls().borrow_mut().center = initial_center;

  character_controls.borrow_mut().set_rotation( 0.0, 0.0 );
  let forward = F32x3::from_array( character_controls.borrow().forward().map( | v | v as f32 ) );
  camera.get_controls().borrow_mut().eye = initial_center - forward * character_controls.borrow().zoom as f32;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = t / 1000.0;

      let last_time = last_time.clone();

      let delta_time = time - *last_time.borrow();
      *last_time.borrow_mut() = time;

      character_controls.borrow_mut().update( &character_input.borrow(), delta_time );

      let mut position = F32x3::from_array( character_controls.borrow().position().map( | v | v as f32 ) );
      let rotation = QuatF32::from_angle_x( -3.8_f32.to_radians() );
      let q_position = QuatF32::from( position.to_homogenous().0 );
      position = ( rotation * q_position * rotation.conjugate() ).0.truncate();
      position.0[ 1 ] -= 1.5;

      character.borrow_mut().set_translation( position );

      scenes[ 0 ].borrow_mut().update_world_matrix();

      neck.borrow_mut().set_rotation( QuatF32::from( character_controls.borrow().rotation().0.map( | v | v as f32 ) ) );

      let mut center = ( character.borrow().get_world_matrix() * character.borrow().get_translation().to_homogenous() ).truncate();
      center.0[ 1 ] += 1.5;
      camera.get_controls().borrow_mut().center = center;

      let forward = F32x3::from_array( character_controls.borrow().forward().map( | v | v as f32 ) );
      camera.get_controls().borrow_mut().eye = center - forward * character_controls.borrow().zoom as f32;

      renderer.borrow_mut().render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
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

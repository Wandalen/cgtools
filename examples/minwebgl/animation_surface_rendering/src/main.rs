#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ]

#![ allow( clippy::implicit_return ) ]

use std::cell::RefCell;
use minwebgl as gl;
use gl::
{
  F32x4,
  math::mat4x4::identity,
  JsCast,
  GL,
  WebGl2RenderingContext,
  web_sys::
  {
    HtmlCanvasElement,
    wasm_bindgen::closure::Closure,
    WebGlTexture
  }
};
use renderer::webgl::
{
  loaders::gltf::GLTF,
  post_processing::
  {
    self, Pass, SwapFramebuffer
  },
  MinFilterMode,
  MagFilterMode,
  WrappingMode,
  Camera,
  Object3D,
  Renderer,
  Scene,
  Texture,
  TextureInfo,
  Sampler,
  Material,
  Node
};
use std::rc::Rc;
use canvas_renderer::renderer::CanvasRenderer;
use primitive_generation::text;
use ::mod_interface::mod_interface;

mod animation;
mod primitive_data;
mod primitive;

use crate::animation::{ model, Model, Shape, Layer, Transform, Color, fixed, ease, LINEAR, EASE_IN_OUT_BACK };

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
fn upload_texture< 'a >( gl : &'a WebGl2RenderingContext, src : Rc< String > ) -> WebGlTexture
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
  gl : &WebGl2RenderingContext,
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

/// Initializes the WebGL2 rendering context and canvas.
///
/// This function sets up a new HTML canvas, creates a WebGL2 rendering context with
/// antialiasing disabled, and enables the `EXT_color_buffer_float` extension.
///
/// # Returns
///
/// A tuple containing the `WebGl2RenderingContext` and the `HtmlCanvasElement`.
fn init_context() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()
  .expect( "Can't create canvas" );
  let gl = gl::context::from_canvas_with( &canvas, options )
  .expect( "Can't create WebGL context" );

  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

/// Initializes the camera based on the scene's bounding box and canvas size.
///
/// This function computes the camera's position and orientation to frame the entire scene.
/// It calculates the distance needed to view the scene's bounding box, sets the camera's
/// `eye`, `up`, and `center` vectors, and configures the perspective projection.
///
/// # Arguments
///
/// * `canvas` - The `HtmlCanvasElement` to get the viewport size.
/// * `scenes` - A slice of `Scene` objects to calculate the bounding box from.
///
/// # Returns
///
/// A configured `Camera` object.
fn init_camera( canvas : &HtmlCanvasElement, scenes : &[ Rc< RefCell< Scene > > ] ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  let dist = scene_bounding_box.max.mag();

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 0.0, 1.0 ] );
  eye *= dist;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 10000000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );

  camera.set_window_size( [ width, height ].into() );

  camera
}

/// Clones a `Node` and its entire subtree, adding the new nodes, meshes, and materials to the GLTF structure.
///
/// This function creates a deep clone of a node, including its children and any associated meshes.
/// It registers all new components within the `GLTF` struct and adds the cloned node to the scene.
///
/// # Arguments
///
/// * `gltf` - A mutable reference to the `GLTF` struct.
/// * `node` - A reference to the `Rc<RefCell<Node>>` to be cloned.
///
/// # Returns
///
/// A reference-counted, mutable reference to the newly cloned `Node`.
fn clone( gltf : &mut GLTF, node : &Rc< RefCell< Node > > ) -> Rc< RefCell< Node > >
{
  let clone = node.borrow().clone_tree();
  gltf.nodes.push( clone.clone() );
  if let Object3D::Mesh( ref mesh ) = clone.borrow().object
  {
    let mesh = Rc::new( RefCell::new( mesh.borrow().clone() ) );
    for p in mesh.borrow().primitives.iter()
    {
      gltf.materials.push( p.borrow().material.clone() );
    }
    gltf.meshes.push( mesh );
  }
  gltf.scenes[ 0 ].borrow_mut().add( clone.clone() );

  clone
}

/// Sets the textures for a `Node` and its primitives using a material callback.
///
/// This function iterates through all primitives of a given `Node` (if it's a `Mesh`),
/// and applies a provided callback function to each primitive's material.
///
/// # Arguments
///
/// * `node` - A reference to the `Rc<RefCell<Node>>` to modify.
/// * `material_callback` - A closure that takes a mutable reference to a `Material` and modifies it.
fn set_texture
(
  node : &Rc< RefCell< Node > >,
  mut material_callback : impl FnMut( &mut Material )
)
{
  if let Object3D::Mesh( ref mesh ) = &node.borrow().object
  {
    for p in &mesh.borrow().primitives
    {
      material_callback( &mut p.borrow().material.borrow_mut() );
    }
  }
}

/// Sets up the main 3D scene by loading a GLTF file and configuring objects.
///
/// This asynchronous function loads a sphere from `sphere.glb`, and then clones it to create
/// several objects: an earth, clouds, a moon, and a large environment sphere. It then
/// applies different textures and transformations to each object to create a simple solar system scene.
///
/// # Arguments
///
/// * `gl` - The `WebGl2RenderingContext`.
///
/// # Returns
///
/// A `Result` containing the configured `GLTF` scene, or a `gl::WebglError` if loading fails.
async fn setup_scene( gl : &WebGl2RenderingContext ) -> Result< GLTF, gl::WebglError >
{
  let window = web_sys::window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );
  let mut gltf = renderer::webgl::loaders::gltf::load( &document, "gltf/sphere.glb", &gl ).await?;

  let earth = gltf.scenes[ 0 ].borrow().children.get( 1 )
  .expect( "Scene is empty" ).clone();
  let texture = create_texture( &gl, "textures/earth2.jpg" );
  set_texture( &earth, | m | { m.base_color_texture = texture.clone(); } );
  earth.borrow_mut().update_local_matrix();

  let clouds = clone( &mut gltf, &earth );
  let texture = create_texture( &gl, "textures/clouds2.png" );
  set_texture
  (
    &clouds,
    | m |
    {
      m.base_color_texture = texture.clone();
      m.alpha_mode = renderer::webgl::AlphaMode::Blend;
    }
  );
  let scale = 1.005;
  clouds.borrow_mut().set_translation( [ 0.0, 1.0 - scale, 0.0 ] );
  clouds.borrow_mut().set_scale( [ scale; 3 ] );
  clouds.borrow_mut().set_rotation( gl::Quat::from_angle_y( 90.0 ) );
  clouds.borrow_mut().update_local_matrix();

  let moon = clone( &mut gltf, &earth );
  let texture = create_texture( &gl, "textures/moon2.jpg" );
  set_texture( &moon, | m | { m.base_color_texture = texture.clone(); } );
  let scale = 0.25;
  let distance = 7.0;// 30.0 * 1.0;
  moon.borrow_mut().set_translation( [ distance, ( 1.0 - scale ), 0.0 ] );
  moon.borrow_mut().set_scale( [ scale; 3 ] );
  moon.borrow_mut().update_local_matrix();

  let environment = clone( &mut gltf, &earth );
  let texture = create_texture( &gl, "environment_maps/equirectangular_maps/space3.png" );
  set_texture( &environment, | m | { m.base_color_texture = texture.clone(); } );
  let scale = 100000.0;
  environment.borrow_mut().set_translation( [ 0.0, 1.0 - scale, 0.0 ] );
  environment.borrow_mut().set_scale( [ scale; 3 ] );
  environment.borrow_mut().update_local_matrix();

  Ok( gltf )
}

/// Sets up a 2D scene for text rendering on a canvas.
///
/// This asynchronous function loads a font, generates a mesh for the text "CGTools",
/// and converts it into a `GLTF` format suitable for rendering on a separate canvas.
/// It also returns the colors used for the text primitives.
///
/// # Arguments
///
/// * `gl` - The `WebGl2RenderingContext`.
///
/// # Returns
///
/// A tuple containing the `GLTF` scene for the canvas and a `Vec` of `F32x4` colors.
async fn setup_canvas_scene( gl : &WebGl2RenderingContext ) -> ( GLTF, Vec< F32x4 > )
{
  let font_names = [ "Roboto-Regular" ];
  let fonts = text::ufo::load_fonts( &font_names ).await;

  let colors =
  [
    F32x4::from_array( [ 1.0, 1.0, 1.0, 1.0 ] ),
  ];
  let text = "CGTools".to_string();

  let mut primitives_data = vec![];
  let mut transform = primitive_generation::Transform::default();
  transform.translation.0[ 1 ] += ( font_names.len() as f32 + 1.0 ) / 2.0 + 0.5;
  for font_name in font_names
  {
    transform.translation[ 1 ] -= 1.0;
    let mut text_mesh = text::ufo::text_to_countour_mesh
    (
      &text,
      fonts.get( font_name ).expect( "Can't find font" ),
      &transform,
      5.0
    );
    text_mesh.iter_mut()
    .for_each( | p | p.color = colors[ 0 ].clone() );
    primitives_data.extend( text_mesh );
  }

  let colors = primitives_data.iter()
  .map( | p | p.color )
  .collect::< Vec< _ > >();
  let canvas_gltf = primitive_generation::primitives_data_to_gltf( &gl, primitives_data );

  ( canvas_gltf, colors )
}

/// Calculates the mathematical modulo of two floating-point numbers.
///
/// This function ensures the result is always non-negative, which differs
/// from the standard remainder operator (`%`) for negative dividends.
///
/// # Arguments
///
/// * `dividend` - The number to be divided.
/// * `divisor` - The number to divide by.
///
/// # Returns
///
/// The non-negative remainder of the division.
pub fn modulo( dividend : f64, divisor : f64 ) -> f64
{
  let mut result = dividend % divisor;
  if result < 0.0
  {
    result += divisor.abs();
  }
  result
}

/// Sets up a complex 2D animation using the `animation` module.
///
/// This function creates a hierarchical animation model with several layers,
/// including a base layer, a circles layer, and multiple repeated rectangle layers.
/// It defines transformations and colors for each layer, resulting in a series of
/// spinning rectangles.
///
/// # Arguments
///
/// * `gl` - The `GL` context.
/// * `width` - The width of the viewport.
/// * `height` - The height of the viewport.
///
/// # Returns
///
/// An `animation::Animation` struct.
fn setup_animation( gl : &GL, width : usize, height : usize ) -> animation::Animation
{
  let points : Vec< [ f32; 2 ] > = vec!
  [
    [ -0.5, -0.5 ],
    [ -0.5, 0.5 ],
    [ 0.5, 0.5 ],
    [ 0.5, -0.5 ],
  ];

  let rect_geo = Shape::Geometry( points );

  let base = Layer::former()
  .frames( 0.0..10.0 )
  .form();

  let transform = Transform::former()
  .form();

  let circles = Layer::former()
  .parent( 0_isize )
  .frames( 0.0..10.0 )
  .transform( interpoli::Transform::Animated( transform.into() ) )
  .form();

  let mut model = Model::former()
  .width( width )
  .height( height )
  .frames( 0.0..10.0 )
  .layers()
  .add( base )
  .add( circles )
  .end()
  .form();

  let mut add_circle =
  | circle_transform : Transform, rect_transform : Transform, color : F32x4, repeats : usize |
  {
    let circle = Layer::former()
    .parent( 1_isize )
    .frames( 0.0..10.0 )
    .transform( interpoli::Transform::Animated( circle_transform.into() ) )
    .form();

    model.layers.push( circle );
    let circle_id = model.layers.len() as isize - 1;

    let offset_rect_transform = Transform::former()
    .rotation
    (
      ease
      (
        ( 0.0, 10.0 ),
        ( 0.0, 360.0 ),
        LINEAR
      )
    )
    .form();

    let offset_rect = Layer::former()
    .parent( circle_id )
    .frames( 0.0..10.0 )
    .transform( interpoli::Transform::Animated( offset_rect_transform.clone().into() ) )
    .form();

    let rect = Layer::former()
    .parent( 3_isize )
    .frames( 0.0..10.0 )
    .transform( interpoli::Transform::Animated( rect_transform.into() ) )
    .content()
    .add( Shape::Color( Color::Fixed( *color ) ) )
    .add( rect_geo.clone() )
    .end()
    .form();

    let diff = 360.0 / repeats as f64;
    for i in 0..repeats
    {
      let mut rect = rect.clone();
      let mut offset_rect = offset_rect.clone();

      let mut transform = offset_rect_transform.clone();
      transform.rotation = fixed( diff * i as f64 );
      offset_rect.transform = interpoli::Transform::Animated( transform.into() );
      model.layers.push( offset_rect );

      rect.parent = model.layers.len() as isize - 1;
      model.layers.push( rect );
    }
  };

  let circle_transform = Transform::former()
  .rotation
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( -10.0, 350.0 ),
      LINEAR
    )
  )
  .scale
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( kurbo::Vec2::new( 100.0, 100.0 ), kurbo::Vec2::new( 200.0, 200.0 ) ),
      EASE_IN_OUT_BACK
    )
  )
  .form();

  let rect_transform = Transform::former()
  .position( fixed( kurbo::Point::new( 1.6, 0.0 ) ) )
  .rotation( fixed( -20.0 ) )
  .scale( fixed( kurbo::Vec2::new( 60.0, 60.0 ) ) )
  .form();

  add_circle( circle_transform.clone(), rect_transform, F32x4::from_array( [ 1.0, 1.0, 1.0, 1.0 ] ), 11 );

  let circle_transform = Transform::former()
  .rotation
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( 0.0, 360.0 ),
      LINEAR
    )
  )
  .scale
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( kurbo::Vec2::new( 100.0, 100.0 ), kurbo::Vec2::new( 200.0, 200.0 ) ),
      EASE_IN_OUT_BACK
    )
  )
  .form();

  let rect_transform = Transform::former()
  .position( fixed( kurbo::Point::new( 2.7, 0.0 ) ) )
  .rotation( fixed( -20.0 ) )
  .scale( fixed( kurbo::Vec2::new( 80.0, 80.0 ) ) )
  .form();

  add_circle( circle_transform.clone(), rect_transform, F32x4::from_array( [ 1.0, 0.75, 0.75, 1.0 ] ), 15 );

  let circle_transform = Transform::former()
  .rotation
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( 10.0, 370.0 ),
      LINEAR
    )
  )
  .scale
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( kurbo::Vec2::new( 100.0, 100.0 ), kurbo::Vec2::new( 200.0, 200.0 ) ),
      EASE_IN_OUT_BACK
    )
  )
  .form();

  let rect_transform = Transform::former()
  .position( fixed( kurbo::Point::new( 4.2, 0.0 ) ) )
  .rotation( fixed( -20.0 ) )
  .form();

  add_circle( circle_transform.clone(), rect_transform, F32x4::from_array( [ 1.0, 0.5, 0.5, 1.0 ] ), 17 );

  let circle_transform = Transform::former()
  .rotation
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( 20.0, 380.0 ),
      LINEAR
    )
  )
  .scale
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( kurbo::Vec2::new( 100.0, 100.0 ), kurbo::Vec2::new( 200.0, 200.0 ) ),
      EASE_IN_OUT_BACK
    )
  )
  .form();

  let rect_transform = Transform::former()
  .position( fixed( kurbo::Point::new( 5.7, 0.0 ) ) )
  .rotation( fixed( -20.0 ) )
  .scale( fixed( kurbo::Vec2::new( 120.0, 120.0 ) ) )
  .form();

  add_circle( circle_transform, rect_transform, F32x4::from_array( [ 1.0, 0.25, 0.25, 1.0 ] ), 19 );

  let circle_transform = Transform::former()
  .rotation
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( 30.0, 390.0 ),
      LINEAR
    )
  )
  .scale
  (
    ease
    (
      ( 0.0, 10.0 ),
      ( kurbo::Vec2::new( 100.0, 100.0 ), kurbo::Vec2::new( 200.0, 200.0 ) ),
      EASE_IN_OUT_BACK
    )
  )
  .form();

  let rect_transform = Transform::former()
  .position( fixed( kurbo::Point::new( 7.4, 0.0 ) ) )
  .rotation( fixed( -20.0 ) )
  .scale( fixed( kurbo::Vec2::new( 140.0, 140.0 ) ) )
  .form();

  add_circle( circle_transform, rect_transform, F32x4::from_array( [ 0.8, 0.0, 0.0, 1.0 ] ), 21 );

  animation::Animation::new( gl, model )
}

/// The main asynchronous function to set up and run the rendering loop.
///
/// This function orchestrates the entire application flow:
/// 1. Initializes the WebGL context and canvas.
/// 2. Sets up a 3D scene (a rotating solar system).
/// 3. Sets up a 2D canvas scene with animated text.
/// 4. Creates a `CanvasRenderer` to render the 2D scene to a texture.
/// 5. Adds a sphere to the 3D scene that uses the rendered 2D canvas texture.
/// 6. Configures the cameras for both scenes.
/// 7. Initializes the main `Renderer` and a post-processing pipeline.
/// 8. Starts the main render loop, which updates the animations and renders the scenes each frame.
async fn run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = init_context();

  let mut gltf = setup_scene( &gl ).await?;

  let ( canvas_gltf, _ ) = setup_canvas_scene( &gl ).await;
  canvas_gltf.scenes[ 0 ].borrow_mut().update_world_matrix();
  let animation = setup_animation( &gl, canvas.height() as usize, canvas.width() as usize );
  animation.set_world_matrix( identity() );

  let canvas_camera = init_camera( &canvas, &canvas_gltf.scenes );
  canvas_camera.bind_controls( &canvas );
  canvas_camera.get_controls().borrow_mut().window_size = [ ( canvas.width() * 4 ) as f32, ( canvas.height() * 4 ) as f32 ].into();
  canvas_camera.get_controls().borrow_mut().eye = [ 0.0, 0.0, 150.0 ].into();
  {
    let controls = canvas_camera.get_controls();
    let mut controls_ref = controls.borrow_mut();
    let center = controls_ref.center.as_mut();
    center[ 1 ] += 45.0;
    center[ 0 ] -= 25.0;
  }

  let canvas_renderer = CanvasRenderer::new( &gl, canvas.width() * 4, canvas.height() * 4 )?;
  let canvas_texture = canvas_renderer.get_texture();

  let earth = gltf.scenes[ 0 ].borrow().children.get( 1 )
  .expect( "Scene is empty" ).clone();
  let canvas_sphere = clone( &mut gltf, &earth );
  set_texture
  (
    &canvas_sphere,
    | m |
    {
      m.base_color_texture.as_mut()
      .map
      (
        | t |
        {
          let texture = t.texture.borrow().clone();
          t.texture = Rc::new( RefCell::new( texture ) );
          t.texture.borrow_mut().source = Some( canvas_texture.clone() );
        }
      );
      m.alpha_mode = renderer::webgl::AlphaMode::Blend;
    }
  );
  let scale = 1.01;
  canvas_sphere.borrow_mut().set_translation( [ 0.0, 1.0 - scale, 0.0 ] );
  canvas_sphere.borrow_mut().set_scale( [ scale; 3 ] );

  let scenes = gltf.scenes.clone();

  let camera = init_camera( &canvas, &scenes );
  camera.bind_controls( &canvas );
  let eye = gl::math::mat3x3h::rot( 0.0, - 73.0_f32.to_radians(), - 15.0_f32.to_radians() )
  * F32x4::from_array([ 0.0, 1.7, 1.7, 1.0 ] );
  camera.get_controls().borrow_mut().eye = [ eye.x(), eye.y(), eye.z() ].into();

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "environment_maps/gltf_viewer_ibl_unreal" ).await );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      let time = t as f32 / 1000.0;

      if let Some( ( mut scene, colors ) ) = animation.frame( modulo( time as f64 * 1.0, 10.0 ) )
      {
        canvas_renderer.render( &gl, &mut scene, &canvas_camera, &colors ).expect( "Failed to render frame" );
      }

      renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( renderer.get_main_texture() );
      //swap_buffer.set_input( Some( canvas_renderer.get_texture() ) );

      let t = tonemapping.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.set_output( t );
      swap_buffer.swap();

      let _t = to_srgb.render( &gl, swap_buffer.get_input(), swap_buffer.get_output() )
      .expect( "Failed to render to srgb pass" );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

/// The main entry point of the application.
///
/// This function calls `gl::spawn_local` to execute the asynchronous `run` function,
/// which sets up and runs the entire WebGL application.
fn main()
{
  gl::spawn_local
  (
    async move
    {
      run().await.expect( "Program finish work with errors" )
    }
  );
}

#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ]

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

mod camera_controls;
mod loaders;
mod animation;
mod primitive_data;
mod primitive;

use animation::load_animation;

/// Uploads an image from a URL to a WebGL texture.
fn upload_texture( gl : &WebGl2RenderingContext, src : Rc< String > ) -> WebGlTexture
{
  let window = web_sys::window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );

  let texture = gl.create_texture().expect( "Failed to create a texture" );

  let img_element = document.create_element( "img" )
  .expect( "Can't create img" )
  .dyn_into::< gl::web_sys::HtmlImageElement >()
  .expect( "Can't convert to gl::web_sys::HtmlImageElement" );
  img_element.style().set_property( "display", "none" )
  .expect( "Can't set property" );
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

/// Creates a new texture from a given image path and returns its metadata.
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

/// Initializes the WebGL2 rendering context and an HTML canvas.
fn init_context() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()
  .expect( "Can't create canvas" );
  let gl = gl::context::from_canvas_with( &canvas, options )
  .expect( "Can't create WebGL context" );

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

/// Initializes a camera based on the scene's bounding box and canvas dimensions.
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

/// Clones a node and its children, adding them to the GLTF scene and internal lists.
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

/// Sets the texture on a node's materials using a callback.
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

/// Asynchronously sets up the initial GLTF scene with multiple textured objects.
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

/// The main asynchronous function that sets up the scene, camera, and render loop.
async fn run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = init_context();

  let mut gltf = setup_scene( &gl ).await?; 

  let lottie_path = "lottie/google.json";
  let animation = load_animation( &gl, lottie_path ).await;
  animation.set_world_matrix( identity() );

  let ( s, _ ) = animation.frame( 0.0 ).expect( "Can't get scene at start frame" );
  let canvas_camera = init_camera( &canvas, &[ Rc::new( RefCell::new( s ) ) ] ); 
  camera_controls::bind_controls_to_input( &canvas, &canvas_camera.get_controls() );
  canvas_camera.get_controls().borrow_mut().window_size = [ ( canvas.width() * 4 ) as f32, ( canvas.height() * 4 ) as f32 ].into();
  {
    let controls = canvas_camera.get_controls();
    let mut controls_ref = controls.borrow_mut();
    {
      let center = controls_ref.center.as_mut();
      center[ 1 ] -= 250.0;
      center[ 0 ] -= 110.0;
      center[ 1 ] += 175.0;
    }
    {
      let eye = controls_ref.eye.as_mut();
      eye[ 0 ] += 125.0;
      eye[ 1 ] -= 125.0;
      eye[ 0 ] -= 110.0;
      eye[ 1 ] += 175.0;
      eye[ 2 ] += 300.0;
    }
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
  canvas_sphere.borrow_mut().update_local_matrix();

  let scenes = gltf.scenes.clone();
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let camera = init_camera( &canvas, &scenes );
  camera_controls::bind_controls_to_input( &canvas, &camera.get_controls() );
  let eye = gl::math::mat3x3h::rot( 0.0, - 73.0_f32.to_radians(), - 15.0_f32.to_radians() ) 
  * F32x4::from_array( [ 0.0, 1.7, 1.7, 1.0 ] );
  camera.get_controls().borrow_mut().eye = [ eye.x(), eye.y(), eye.z() ].into();

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( loaders::ibl::load( &gl, "environment_maps/gltf_viewer_ibl_unreal" ).await );

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

      let frame = modulo( time as f64 * 75.0, 125.0 );
      if let Some( ( mut scene, colors ) ) = animation.frame( frame )
      {
        canvas_renderer.render( &gl, &mut scene, &canvas_camera, &colors )
        .expect( "Failed to render frame" );
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
fn main()
{
  gl::spawn_local( async move { run().await.expect( "Program finished with errors" ) } );
}

#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ]

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::assign_op_pattern ) ]
#![ allow( clippy::semicolon_if_nothing_returned ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::redundant_field_names ) ]
#![ allow( clippy::useless_format ) ]
#![ allow( clippy::let_unit_value ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::needless_continue ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::collapsible_if ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::needless_borrows_for_generic_args ) ]
#![ allow( clippy::manual_midpoint ) ]
#![ allow( clippy::needless_for_each ) ]
#![ allow( clippy::clone_on_copy ) ]
#![ allow( clippy::option_map_unit_fn ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::expect_fun_call ) ]
#![ allow( clippy::assigning_clones ) ]

use std::cell::RefCell;
use minwebgl as gl;
use gl::
{
  texture::d2::upload_image_from_path,
  F32x4,
  math::mat4x4::identity,
  GL,
  WebGl2RenderingContext,
  web_sys::HtmlCanvasElement
};
use renderer::webgl::
{
  Camera, MagFilterMode, Material, MinFilterMode, Node, Object3D, Renderer, Sampler, Scene, Texture, TextureInfo, WrappingMode, cast_unchecked_material_to_ref_mut, loaders::gltf::GLTF, material::PbrMaterial, post_processing::
  {
    self, Pass, SwapFramebuffer
  }
};
use std::rc::Rc;
use canvas_renderer::renderer::CanvasRenderer;

mod animation;

use animation::load_animation;

/// Creates a new texture from a given image path and returns its metadata.
fn create_texture
(
  gl : &WebGl2RenderingContext,
  image_path : &str
) -> TextureInfo
{
  let image_path = format!( "static/{image_path}" );
  let texture_id = upload_image_from_path( gl, &image_path, false );

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

  TextureInfo
  {
    texture : Rc::new( RefCell::new( texture ) ),
    uv_position : 0,
  }
}

/// Initializes the WebGL2 rendering context and an HTML canvas.
fn init_context() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

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

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 0.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.1;
  let far = 1000.0;

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
  mut material_callback : impl FnMut( Rc< RefCell< Box< dyn Material> > > )
)
{
  if let Object3D::Mesh( ref mesh ) = &node.borrow().object
  {
    for p in &mesh.borrow().primitives
    {
      material_callback( p.borrow().material.clone() );
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
  set_texture
  (
    &earth,
    | m |
    {
      let mut m = cast_unchecked_material_to_ref_mut::< PbrMaterial >( m.borrow_mut() );
      m.base_color_texture = Some( texture.clone() );
    }
  );
  earth.borrow_mut().update_local_matrix();

  let clouds = clone( &mut gltf, &earth );
  let texture = create_texture( &gl, "textures/clouds2.png" );
  set_texture
  (
    &clouds,
    | m |
    {
      let mut m = cast_unchecked_material_to_ref_mut::< PbrMaterial >( m.borrow_mut() );
      m.base_color_texture = Some( texture.clone() );
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
  set_texture
  (
    &moon,
    | m |
    {
      let mut m = cast_unchecked_material_to_ref_mut::< PbrMaterial >( m.borrow_mut() );
      m.base_color_texture = Some( texture.clone() );
    }
  );
  let scale = 0.25;
  let distance = 7.0;// 30.0 * 1.0;
  moon.borrow_mut().set_translation( [ distance, ( 1.0 - scale ), 0.0 ] );
  moon.borrow_mut().set_scale( [ scale; 3 ] );
  moon.borrow_mut().update_local_matrix();

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
#[ must_use ]
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
  canvas_camera.get_controls().borrow_mut().window_size = [ ( canvas.width() * 4 ) as f32, ( canvas.height() * 4 ) as f32 ].into();
  {
    let controls = canvas_camera.get_controls();
    let mut controls_ref = controls.borrow_mut();
    {
      controls_ref.center = [ 7.671358, 105.80746, 61.174854 ].into();

    }
    {
      controls_ref.eye = [ -43.71087, -343.4742, 744.99524 ].into();
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
      let mut m = cast_unchecked_material_to_ref_mut::< PbrMaterial >( m.borrow_mut() );
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
  gltf.scenes[ 0 ].borrow_mut().children.push( canvas_sphere.clone() );

  let scenes = gltf.scenes.clone();
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let camera = init_camera( &canvas, &scenes );
  camera.bind_controls( &canvas );
  let eye = gl::math::mat3x3h::rot( 0.0, - 73.0_f32.to_radians(), - 15.0_f32.to_radians() )
  * F32x4::from_array( [ 0.0, 1.7, 1.7, 1.0 ] );
  camera.get_controls().borrow_mut().eye = [ eye.x(), eye.y(), eye.z() ].into();
  camera.get_controls().borrow_mut().center = [ 0.0, 1.0, 0.0 ].into();

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.set_ibl( renderer::webgl::loaders::ibl::load( &gl, "environment_maps/gltf_viewer_ibl_unreal", None ).await );
  let skybox = create_texture( &gl, "environment_maps/equirectangular_maps/space3.png" );
  renderer.set_skybox( skybox.texture.borrow().source.clone() );

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

      let frame = modulo( f64::from( time ) * 75.0, 125.0 );
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

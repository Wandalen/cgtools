#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ]

use std::cell::RefCell;
use minwebgl as gl;
use gl::
{
  texture::d2::image_upload_from_path,
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

use animation::animation_load;

/// Creates a new texture from a given image path and returns its metadata.
fn texture_create
(
  gl : &WebGl2RenderingContext,
  image_path : &str
) -> TextureInfo
{
  let image_path = format!( "static/{image_path}" );
  let texture_id = image_upload_from_path( gl, &image_path, false );

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
fn context_init() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make()
  .expect( "Can't create canvas" );
  let gl = gl::context::from_canvas_with( &canvas, options )
  .expect( "Can't create WebGL context" );

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

/// Initializes a camera based on the scene's bounding box and canvas dimensions.
fn camera_init( canvas : &HtmlCanvasElement, scenes : &[ Rc< RefCell< Scene > > ] ) -> Camera
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

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far ).expect( "camera parameters are valid" );

  camera.window_size_set( [ width, height ].into() );

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
  let clone = node.borrow().tree_clone();
  gltf.nodes.push( clone.clone() );
  if let Object3D::Mesh( ref mesh ) = clone.borrow().object
  {
    let mesh = Rc::new( RefCell::new( mesh.borrow().clone() ) );
    for p in &mesh.borrow().primitives
    {
      gltf.materials.push( p.borrow().material.clone() );
    }
    gltf.meshes.push( mesh );
  }
  gltf.scenes[ 0 ].borrow_mut().add( clone.clone() );

  clone
}

/// Applies a material modification callback to all primitives of a `Node`.
///
/// This function iterates through all primitives of a given `Node` (if it's a `Mesh`),
/// and applies a provided callback function to each primitive's material, allowing
/// modification of textures, alpha modes, and other material properties.
///
/// # Arguments
///
/// * `node` - A reference to the `Rc<RefCell<Node>>` to modify.
/// * `material_callback` - A closure that takes a material reference and modifies it.
fn function_to_node_materials_apply
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
async fn scene_setup( gl : &WebGl2RenderingContext ) -> Result< GLTF, gl::WebglError >
{
  let window = web_sys::window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );
  let mut gltf = renderer::webgl::loaders::gltf::load( &document, "static/gltf/sphere.glb", gl ).await?;

  let earth = gltf.scenes[ 0 ].borrow().children.get( 1 )
  .expect( "Scene is empty" ).clone();
  let texture = texture_create( gl, "textures/earth2.jpg" );
  function_to_node_materials_apply
  (
    &earth,
    | m |
    {
      let mut m = cast_unchecked_material_to_ref_mut::< PbrMaterial >( m.borrow_mut() );
      m.base_color_texture_set( Some( texture.clone() ) );
    }
  );
  earth.borrow_mut().local_matrix_update();

  let clouds = clone( &mut gltf, &earth );
  let texture = texture_create( gl, "textures/clouds2.png" );
  function_to_node_materials_apply
  (
    &clouds,
    | m |
    {
      let mut m = cast_unchecked_material_to_ref_mut::< PbrMaterial >( m.borrow_mut() );
      m.base_color_texture_set( Some( texture.clone() ) );
      m.alpha_mode_set( renderer::webgl::AlphaMode::Blend );
    }
  );
  let scale = 1.005;
  clouds.borrow_mut().translation_set( [ 0.0, 1.0 - scale, 0.0 ] );
  clouds.borrow_mut().scale_set( [ scale; 3 ] );
  clouds.borrow_mut().rotation_set( gl::Quat::from_angle_y( 90.0 ) );
  clouds.borrow_mut().local_matrix_update();

  let moon = clone( &mut gltf, &earth );
  let texture = texture_create( gl, "textures/moon2.jpg" );
  function_to_node_materials_apply
  (
    &moon,
    | m |
    {
      let mut m = cast_unchecked_material_to_ref_mut::< PbrMaterial >( m.borrow_mut() );
      m.base_color_texture_set( Some( texture.clone() ) );
    }
  );
  let scale = 0.25;
  let distance = 7.0;// 30.0 * 1.0;
  moon.borrow_mut().translation_set( [ distance, ( 1.0 - scale ), 0.0 ] );
  moon.borrow_mut().scale_set( [ scale; 3 ] );
  moon.borrow_mut().local_matrix_update();

  Ok( gltf )
}

/// The main asynchronous function that sets up the scene, camera, and render loop.
async fn app_run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = context_init();

  let mut gltf = scene_setup( &gl ).await?;

  let lottie_path = "static/lottie/google.json";
  let animation = animation_load( &gl, lottie_path ).await;
  animation.world_matrix_set( identity() );

  let ( s, _ ) = animation.frame( 0.0 ).expect( "Can't get scene at start frame" );
  let canvas_camera = camera_init( &canvas, &[ Rc::new( RefCell::new( s ) ) ] );
  canvas_camera.controls_get().borrow_mut().window_size = [ ( canvas.width() * 4 ) as f32, ( canvas.height() * 4 ) as f32 ].into();
  {
    let controls = canvas_camera.controls_get();
    let mut controls_ref = controls.borrow_mut();
    controls_ref.center = [ 7.671_358, 105.807_46, 61.174_854 ].into();
    controls_ref.eye = [ -43.71087, -343.4742, 744.99524 ].into();
  }

  let canvas_renderer = CanvasRenderer::new( &gl, canvas.width() * 4, canvas.height() * 4 )?;
  let canvas_texture = canvas_renderer.texture_get();

  let earth = gltf.scenes[ 0 ].borrow().children.get( 1 )
  .expect( "Scene is empty" ).clone();
  let canvas_sphere = clone( &mut gltf, &earth );
  function_to_node_materials_apply
  (
    &canvas_sphere,
    | m |
    {
      let mut m = cast_unchecked_material_to_ref_mut::< PbrMaterial >( m.borrow_mut() );
      let uv_position = m.base_color_texture().map_or( 0, | t | t.uv_position );
      let texture = Texture::former().source( canvas_texture.clone() ).form();
      let texture_info = TextureInfo { texture : Rc::new( RefCell::new( texture ) ), uv_position };
      m.base_color_texture_set( Some( texture_info ) );
      m.alpha_mode_set( renderer::webgl::AlphaMode::Blend );
    }
  );
  let scale = 1.01;
  canvas_sphere.borrow_mut().translation_set( [ 0.0, 1.0 - scale, 0.0 ] );
  canvas_sphere.borrow_mut().scale_set( [ scale; 3 ] );
  canvas_sphere.borrow_mut().local_matrix_update();
  gltf.scenes[ 0 ].borrow_mut().children.push( canvas_sphere.clone() );

  let scenes = gltf.scenes.clone();
  scenes[ 0 ].borrow_mut().world_matrix_update();

  let camera = camera_init( &canvas, &scenes );
  camera.controls_bind( &canvas );
  let eye = gl::math::mat3x3h::rot( 0.0, - 73.0_f32.to_radians(), - 15.0_f32.to_radians() )
  * F32x4::from_array( [ 0.0, 1.7, 1.7, 1.0 ] );
  camera.controls_get().borrow_mut().eye = eye.truncate();
  camera.controls_get().borrow_mut().center = [ 0.0, 1.0, 0.0 ].into();

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.ibl_set( renderer::webgl::loaders::ibl::load( &gl, "static/environment_maps/gltf_viewer_ibl_unreal", None ).await );
  let skybox = texture_create( &gl, "environment_maps/equirectangular_maps/space3.png" );
  renderer.skybox_set( skybox.texture.borrow().source.clone() );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      let time = t / 1000.0;

      // Scales time to speed or slowdown the animation
      let speed = 75.0;
      // Total duration of the lottie animation in milliseconds
      let animation_duration = 125.0;
      let frame = time * speed % animation_duration;
      // [`Animation::frame`] receives as input time moment from animation start in milliseconds
      if let Some( ( mut scene, colors ) ) = animation.frame( frame )
      {
        canvas_renderer.render( &gl, &mut scene, &canvas_camera, &colors )
        .expect( "Failed to render frame" );
      }

      renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.input_set( renderer.main_texture() );
      //swap_buffer.input_set( Some( canvas_renderer.texture_get() ) );

      let t = tonemapping.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
      .expect( "Failed to render tonemapping pass" );

      swap_buffer.output_set( t );
      swap_buffer.swap();

      let _t = to_srgb.render( &gl, swap_buffer.input_get(), swap_buffer.output_get() )
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
  gl::spawn_local( async move { app_run().await.expect( "Program finished with errors" ) } );
}

//! Renders 2D curves on surface of 3D object.
#![ doc( html_root_url = "https://docs.rs/curve_surface_rendering/latest/curve_surface_rendering/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renders 2D curves on surface of 3D object" ) ]

use std::cell::RefCell;
use mingl::F32x4;
use minwebgl as gl;
use gl::
{
  WebGl2RenderingContext,
  web_sys::HtmlCanvasElement,
};
use renderer::webgl::
{
  loaders::gltf::GLTF,
  post_processing::
  {
    self, Pass, SwapFramebuffer
  },
  Camera,
  Object3D,
  Renderer,
  Scene,
  Texture,
  TextureInfo,
  material::PbrMaterial,
  Node
};
use std::rc::Rc;
use canvas_renderer::renderer::CanvasRenderer;
use primitive_generation::{text, Transform, primitives_data_to_gltf};

fn texture_create(
  gl : &WebGl2RenderingContext,
  image_path : &str
) -> TextureInfo
{
  let image_path = format!( "static/{image_path}" );
  let texture = Texture::load_from_path( gl, &image_path, false );

  TextureInfo
  {
    texture : Rc::new( RefCell::new( texture ) ),
    uv_position : 0,
  }
}

fn context_init() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make().unwrap();
  let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

fn camera_init( canvas : &HtmlCanvasElement, scenes : &[ Rc< RefCell< Scene > > ] ) -> Camera
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
  let far = 10_000_000.0;

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
  mut material_callback : impl FnMut( &mut PbrMaterial )
)
{
  if let Object3D::Mesh( ref mesh ) = &node.borrow().object
  {
    for p in &mesh.borrow().primitives
    {
      let p = p.borrow();
      let mut mat = renderer::webgl::helpers::cast_unchecked_material_to_ref_mut::< PbrMaterial >
      (
        p.material.borrow_mut()
      );
      material_callback( &mut mat );
    }
  }
}

async fn scene_setup( gl : &WebGl2RenderingContext ) -> Result< GLTF, gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document =  window.document().unwrap();
  let mut gltf = renderer::webgl::loaders::gltf::load( &document, "static/gltf/sphere.glb", gl ).await?;

  let earth = gltf.scenes[ 0 ].borrow().children.get( 1 ).unwrap().clone();
  let texture = texture_create( gl, "textures/earth2.jpg" );
  function_to_node_materials_apply( &earth, | m | { m.base_color_texture_set( Some( texture.clone() ) ); } );
  earth.borrow_mut().local_matrix_update();

  let clouds = clone( &mut gltf, &earth );
  let texture = texture_create( gl, "textures/clouds2.png" );
  function_to_node_materials_apply( &clouds,
    | m |
    {
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
  function_to_node_materials_apply( &moon, | m | { m.base_color_texture_set( Some( texture.clone() ) ); } );
  let scale = 0.25;
  let distance = 7.0;// 30.0 * 1.0;
  moon.borrow_mut().translation_set( [ distance, ( 1.0 - scale ), 0.0 ] );
  moon.borrow_mut().scale_set( [ scale; 3 ] );
  moon.borrow_mut().local_matrix_update();

  Ok( gltf )
}

async fn canvas_scene_setup( gl : &WebGl2RenderingContext ) -> ( GLTF, Vec< F32x4 > )
{
  let font_names = [ "Roboto-Regular" ];
  let fonts = text::ufo::fonts_load( &font_names ).await;

  let colors =
  [
    F32x4::from_array( [ 1.0, 0.0, 0.0, 1.0 ] ),
    F32x4::from_array( [ 1.0, 1.0, 1.0, 1.0 ] ),
    F32x4::from_array( [ 0.0, 1.0, 0.0, 1.0 ] ),
  ];
  let text = "CGTools".to_string();

  let mut primitives_data = vec![];
  let mut transform = Transform::default();
  transform.translation.0[ 1 ] += f32::midpoint(font_names.len() as f32, 1.0) + 0.5;
  for font_name in font_names
  {
    transform.translation[ 1 ] -= 1.0;
    let mut text_mesh = text::ufo::text_to_countour_mesh( &text, fonts.get( font_name ).unwrap(), &transform, 5.0 );
    for p in &mut text_mesh
    {
      p.color = colors[ 0 ];
    }
    primitives_data.extend( text_mesh );
  }

  let colors = primitives_data.iter()
  .map( | p | p.color )
  .collect::< Vec< _ > >();
  let canvas_gltf = primitives_data_to_gltf( gl, &primitives_data );

  ( canvas_gltf, colors )
}

async fn app_run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = context_init();

  let mut gltf = scene_setup( &gl ).await?;

  let ( canvas_gltf, colors ) = canvas_scene_setup( &gl ).await;

  let canvas_camera = camera_init( &canvas, &canvas_gltf.scenes );
  canvas_camera.controls_get().borrow_mut().window_size = [ ( canvas.width() * 4 ) as f32, ( canvas.height() * 4 ) as f32 ].into();
  canvas_camera.controls_get().borrow_mut().eye = [ 0.0, 0.0, 8.0 ].into();
  {
    let controls = canvas_camera.controls_get();
    let mut controls_ref = controls.borrow_mut();
    let center = controls_ref.center.as_mut();
    center[ 1 ] += 3.0;
    center[ 0 ] -= 1.0;
  }

  let canvas_renderer = CanvasRenderer::new( &gl, canvas.width() * 4, canvas.height() * 4 )?;
  let canvas_texture = canvas_renderer.texture_get();

  let earth = gltf.scenes[ 0 ].borrow().children.get( 1 ).unwrap().clone();
  let canvas_sphere = clone( &mut gltf, &earth );
  function_to_node_materials_apply
  (
    &canvas_sphere,
    | m |
    {
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

  let scenes = gltf.scenes.clone();
  scenes[ 0 ].borrow_mut().world_matrix_update();

  let camera = camera_init( &canvas, &scenes );
  camera.controls_bind( &canvas );
  let eye = gl::math::mat3x3h::rot( 0.0, - 76.0_f32.to_radians(), - 20.0_f32.to_radians() )
  * F32x4::from_array([ 0.0, 1.7, 1.7, 1.0 ] );
  camera.controls_get().borrow_mut().eye = [ eye.x(), eye.y(), eye.z() ].into();
  camera.controls_get().borrow_mut().center = [ 0.0, 1.0, 0.0 ].into();

  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.ibl_set( renderer::webgl::loaders::ibl::load( &gl, "static/environment_maps/gltf_viewer_ibl_unreal/", None ).await );
  let skybox = texture_create( &gl, "environment_maps/equirectangular_maps/space3.png" );
  renderer.skybox_set( skybox.texture.borrow().source.clone() );

  let mut swap_buffer = SwapFramebuffer::new( &gl, canvas.width(), canvas.height() );

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | _ : f64 |
    {
      // If textures are of different size, gl.view_port needs to be called
      canvas_renderer.render( &gl, &mut canvas_gltf.scenes[ 0 ].borrow_mut(), &canvas_camera, &colors ).unwrap();

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

fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

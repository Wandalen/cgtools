use std::cell::RefCell;
use minwebgl as gl;
use gl::
{
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

mod camera_controls;
mod loaders;
mod canvas_renderer;
mod primitive;
mod geometry_generation;

use canvas_renderer::*;
use primitive::*;
use geometry_generation::*;

fn upload_texture( gl : &WebGl2RenderingContext, src : Rc< String > ) -> WebGlTexture
{
  let window = web_sys::window().unwrap();
  let document =  window.document().unwrap();

  let texture = gl.create_texture().expect( "Failed to create a texture" );

  let img_element = document.create_element( "img" ).unwrap()
  .dyn_into::< gl::web_sys::HtmlImageElement >().unwrap();
  img_element.style().set_property( "display", "none" ).unwrap();
  let load_texture : Closure< dyn Fn() > = Closure::new
  (
    {
      let gl = gl.clone();
      let img = img_element.clone();
      let texture = texture.clone();
      let src = src.clone();
      move ||
      {
        gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
        gl.tex_image_2d_with_u32_and_u32_and_html_image_element
        (
          gl::TEXTURE_2D,
          0,
          gl::RGBA as i32,
          gl::RGBA,
          gl::UNSIGNED_BYTE,
          &img
        ).expect( "Failed to upload data to texture" );

        gl.generate_mipmap( gl::TEXTURE_2D );

        //match
        gl::web_sys::Url::revoke_object_url( &src ).unwrap();
        img.remove();
      }
    }
  );

  img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
  img_element.set_src( &src );
  load_texture.forget();

  texture
}

async fn create_texture( 
  gl : &WebGl2RenderingContext,
  image_path : &str
) -> Option< TextureInfo >
{
  let texture_id = upload_texture( gl, Rc::new( image_path.to_string() ) );

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

fn init_context() -> ( WebGl2RenderingContext, HtmlCanvasElement )
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make().unwrap();
  let gl = gl::context::from_canvas_with( &canvas, options ).unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  ( gl, canvas )
}

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

fn clone( gltf : &mut GLTF, node : &Rc< RefCell< Node > > ) -> Rc< RefCell< Node > > 
{
  let clone = Rc::new( RefCell::new( node.borrow().clone() ) );
  gltf.nodes.push( clone.clone() );
  if let Object3D::Mesh( ref mesh ) = clone.borrow().object
  {
    gltf.meshes.push( mesh.clone() );
    for p in mesh.borrow().primitives.iter()
    {
      gltf.materials.push( p.borrow().material.clone() );
    }
  }
  gltf.scenes[ 0 ].borrow_mut().add( clone.clone() );

  clone
}

fn set_texture( 
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

async fn setup_scene( gl : &WebGl2RenderingContext ) -> Result< GLTF, gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document =  window.document().unwrap();
  let mut gltf = renderer::webgl::loaders::gltf::load( &document, "curve_surface_rendering/sphere.glb", &gl ).await?;

  let earth = gltf.scenes[ 0 ].borrow().children.get( 1 ).unwrap().clone();
  let image_path = "static/curve_surface_rendering/earth2.jpg";
  let texture = create_texture( &gl, image_path ).await;
  set_texture( &earth, | m | { m.base_color_texture = texture.clone(); } );
  earth.borrow_mut().update_local_matrix();

  let clouds = clone( &mut gltf, &earth );
  let image_path = "static/curve_surface_rendering/clouds2.png";
  let texture = create_texture( &gl, image_path ).await;
  set_texture( &clouds, 
    | m | 
    { 
      m.base_color_texture = texture.clone(); 
      m.alpha_mode = renderer::webgl::AlphaMode::Blend;
    } 
  );
  let scale = 1.005;
  clouds.borrow_mut().set_translation( [ 0.0, 1.0 - scale, 0.0 ] );
  clouds.borrow_mut().set_scale( [ scale; 3 ] );
  clouds.borrow_mut().update_local_matrix();

  let moon = clone( &mut gltf, &earth );
  let image_path = "static/curve_surface_rendering/moon2.jpg";
  let texture = create_texture( &gl, image_path ).await;
  set_texture( &moon, | m | { m.base_color_texture = texture.clone(); } );
  let scale = 0.25;
  let distance = 7.0;// 30.0 * 1.0;
  moon.borrow_mut().set_translation( [ distance, ( 1.0 - scale ), 0.0 ] );
  moon.borrow_mut().set_scale( [ scale; 3 ] );
  moon.borrow_mut().update_local_matrix();

  let environment = clone( &mut gltf, &earth );
  let image_path = "static/curve_surface_rendering/space3.png";
  let texture = create_texture( &gl, image_path ).await;
  set_texture( &environment, | m | { m.base_color_texture = texture.clone(); } );
  let scale = 100000.0;
  environment.borrow_mut().set_translation( [ 0.0, 1.0 - scale, 0.0 ] );
  environment.borrow_mut().set_scale( [ scale; 3 ] );
  environment.borrow_mut().update_local_matrix();

  Ok( gltf )
}

fn setup_canvas_scene( gl : &WebGl2RenderingContext ) -> GLTF
{
  let mut primitives_data = vec![];



  let canvas_gltf = primitives_data_to_gltf( &gl, primitives_data );

  canvas_gltf
}

async fn run() -> Result< (), gl::WebglError >
{
  let ( gl, canvas ) = init_context();

  let gltf = setup_scene( &gl ).await?; 
  
  let canvas_gltf = setup_canvas_scene( &gl );

  let canvas_camera = init_camera( &canvas, &canvas_gltf.scenes );
  camera_controls::bind_controls_to_input( &canvas, &canvas_camera.get_controls() );
  //canvas_camera.get_controls()

  let mut canvas_renderer = CanvasRenderer::new( &gl, canvas.width(), canvas.height() )?;
  let canvas_texture = canvas_renderer.get_texture();

  let earth = gltf.scenes[ 0 ].borrow().children.get( 1 ).unwrap().clone();
  set_texture( 
    &earth, 
    | m | 
    { 
      m.base_color_texture.iter_mut()
      .for_each
      (
        | t |
        {
          t.texture.borrow_mut().source = Some( canvas_texture.clone() );
        }
      );
    } 
  );

  let scenes = gltf.scenes.clone();
  scenes[ 0 ].borrow_mut().update_world_matrix();

  let camera = init_camera( &canvas, &scenes );
  // camera_controls::bind_controls_to_input( &canvas, &camera.get_controls() );
  camera.get_controls().borrow_mut().eye = [ 0.0, 3.0, 3.0 ].into();

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
      let _time = t as f32 / 1000.0;

      // renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )
      // .expect( "Failed to render" );

      canvas_renderer.render( &gl, &mut canvas_gltf.scenes[ 0 ].borrow_mut(), &canvas_camera ).unwrap();

      swap_buffer.reset();
      swap_buffer.bind( &gl );
      swap_buffer.set_input( Some( canvas_renderer.get_texture() ) );

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

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
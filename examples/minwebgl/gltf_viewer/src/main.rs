use std::
{
  cell::RefCell, collections::HashMap, rc::Rc 
};

use buffer::Buffer;
use camera::Camera;
use gltf::Gltf;
use material::Material;
use mesh::Mesh;
use minwebgl::{ self as gl, JsCast };
use node::{ Node, Object3D };
use renderer::Renderer;
use scene::Scene;
use texture::Texture;
use web_sys::wasm_bindgen::prelude::Closure;

mod mesh;
mod camera_controls;
mod material;
mod scene;
mod texture;
mod sampler;
mod primitive;
mod buffer;
mod node;
mod renderer;
mod camera;
mod program;
mod loaders;
mod ibl;
mod post_processing;

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas, Default::default() )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let _ = gl.get_extension( "EXT_color_buffer_float" ).expect( "Failed to enable EXT_color_buffer_float extension" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  // Camera setup
  let mut eye = gl::math::F32x3::from( [ 0.0, 20.0, 20.0 ] );
  eye /= 500.0;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.001;
  let far = 100.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );

  camera_controls::setup_controls( &canvas, &camera.get_controls() );

  let gltf_file_path = "dodge-challenger/gltf";

  let gltf_slice= gl::file::load( &format!( "{}/scene.gltf", gltf_file_path ) ).await.expect( "Failed to load gltf file" );
  let mut gltf_file = Gltf::from_slice( &gltf_slice ).unwrap();

  let mut buffers : Vec< gl::js_sys::Uint8Array > = Vec::new();

  // Move the GLB bin into buffers
  if let Some( blob ) = gltf_file.blob.as_mut()
  {
    let blob = std::mem::take( blob );
    gl::log::info!( "The gltf binary payload is present: {}", blob.len() );
    buffers.push( blob.as_slice().into() );
  }

  for gltf_buffer in gltf_file.buffers()
  {
    match gltf_buffer.source()
    {
      gltf::buffer::Source::Uri( uri ) =>
      {
        let path = format!( "{}/{}", gltf_file_path, uri );
        let buffer = gl::file::load( &path ).await
        .expect( "Failed to load a buffer" );

        gl::log::info!
        (
          "Buffer path: {}\n
          \tBuffer length: {}", 
          path,
          buffer.len()
        );

        buffers.push( buffer.as_slice().into() );
      },
      _ => {}
    }
  }

  gl::info!( "Bufffers: {}", buffers.len() );

  // Upload images 
  let images = Rc::new( RefCell::new( Vec::new() ) );

  // Creates an <img> html elements, and sets its src property to 'src' parameter
  // When the image is loaded, createa a texture and adds it to the 'images' array
  let upload_texture = | src : Rc< String > | {
    let texture = gl.create_texture();
    images.borrow_mut().push( texture.clone() );

    let img_element = document.create_element( "img" ).unwrap().dyn_into::< gl::web_sys::HtmlImageElement >().unwrap();
    img_element.style().set_property( "display", "none" ).unwrap();
    let load_texture : Closure< dyn Fn() > = Closure::new
    ( 
      {
        //let images = images.clone();
        let gl = gl.clone();
        let img = img_element.clone();
        let src = src.clone();
        move || 
        {
          gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
          //gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );
          gl.tex_image_2d_with_u32_and_u32_and_html_image_element
          (
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &img
          ).expect( "Failed to upload data to texture" );
          //gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

          gl.generate_mipmap( gl::TEXTURE_2D );

          //match 
          gl::web_sys::Url::revoke_object_url( &src ).unwrap();
          // {
          //   Ok( _ ) => { gl::info!( "Remove object url: {}", &src ) },
          //   Err( _ ) => { gl::info!( "Not an object url: {}", &src ) }
          // }

          img.remove();
        }
      }
    );

    img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
    img_element.set_src( &src );
    load_texture.forget();
  };

  // If a source of an image is Uri - load the file
  // If a source of an image is View - create a blob from buffer, then turn it into an Object Url,
  // then load an image from the url
  for gltf_image in gltf_file.images()
  {
    match  gltf_image.source() 
    {
      gltf::image::Source::Uri { uri, mime_type: _ } => 
      {
        upload_texture( Rc::new( format!( "static/{}/{}", gltf_file_path, uri ) ) );
      },
      gltf::image::Source::View { view, mime_type } => 
      {
        let buffer = buffers[ view.buffer().index() ].clone();
        let blob = {
          let options = gl::web_sys::BlobPropertyBag::new();
          options.set_type( mime_type );

          let mut blob_parts = Vec::new();
          blob_parts.push( buffer );

          gl::web_sys::Blob::new_with_u8_array_sequence_and_options( &( blob_parts.into() ), &options )
        }.expect( "Failed to create a Blob" );

        let url = gl::web_sys::Url::create_object_url_with_blob( &blob ).expect( "Failed to create object url" );  
        upload_texture( Rc::new( url ) );
      }
    }
  }

  gl::info!( "Images: {}", images.borrow().len() );

  // Upload buffer to the GPU
  let mut gl_buffers = HashMap::new();
  // The target option may not be set for the attributes/indices buffers
  // This scenario should be checked
  for ( i, view ) in gltf_file.views().enumerate()
  {
    if view.target().is_some()
    {
      let buffer = Buffer::new( &gl, &view, &buffers )?;
      gl_buffers.insert( i, buffer );
    }
  }

  gl::info!( "GL Buffers: {}", gl_buffers.len() );

  // Create textures
  let mut textures = Vec::new();
  for gltf_t in gltf_file.textures()
  {
    let t = Texture::new( &images.borrow(), gltf_t );
    textures.push( t );
  }
  let textures : &'static Vec< Texture > = Box::leak( Box::new( textures ) );

  // Create materials
  let mut materials = Vec::new();
  for gltf_mat in gltf_file.materials()
  {
    let m = Material::new( gltf_mat, &textures );
    materials.push( m );
  }

  gl::log::info!( "Materials: {}",materials.len() );

  let mut meshes = Vec::new();
  for gltf_mesh in gltf_file.meshes()
  {
    let m = Mesh::new( &gl, &gltf_mesh, &gl_buffers )?;
    meshes.push( m );
  }

  gl::log::info!( "Meshes: {}",meshes.len() );

  let mut nodes = Vec::new();
  for gltf_node in gltf_file.nodes()
  {
    let node = Rc::new( RefCell::new( Node::new( &gltf_node ) ) );
    nodes.push( node );
  }

  gl::log::info!( "Nodes: {}", nodes.len() );

  let mut scenes = Vec::new();
  for gltf_scene in gltf_file.scenes()
  {
    let scene = Scene::new( &gltf_scene, &nodes );
    scenes.push( scene );
  }

  gl::log::info!( "Scenes: {}", scenes.len() );

  let mut renderer = Renderer::new( materials, meshes );
  renderer.load_ibl( &gl, "envMap" ).await;
  renderer.compile( &gl )?;

  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::BLEND );

  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );

  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.clear_depth( 1.0 );

  scenes[ 0 ].update_world_matrix();
  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      renderer.render( &gl, &mut scenes[ 0 ], &camera )
      .expect( "Failed to render" );

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

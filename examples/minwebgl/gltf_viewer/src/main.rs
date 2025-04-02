//! Just draw a large point in the middle of the screen.

use std::
{
  cell::RefCell, collections::{ HashMap, HashSet }, rc::Rc 
};

use buffer::Buffer;
use gltf::Gltf;
use material::Material;
// use material::{ GLMaterial, TextureType };
// use mesh::GLMesh;
use mingl::CameraOrbitControls;
use minwebgl::{ self as gl, JsCast };
use texture::Texture;
use web_sys::wasm_bindgen::prelude::Closure;

mod mesh;
mod camera_controls;
mod material;
//mod scene;
//mod node;
mod texture;
mod sampler;
mod primitive;
mod buffer;

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let frag = include_str!( "../shaders/test/shader.frag" );
  let vert = include_str!( "../shaders/test/shader.vert" );

  let program = gl::ProgramFromSources::new( vert, frag ).compile_and_link( &gl )?;
  gl.use_program( Some( &program ) );

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 20.0, 20.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl
  (
    fov,  
    aspect_ratio, 
    0.1, 
    10000.0
  );

  let camera = CameraOrbitControls
  {
    eye : eye,
    up : up,
    center : center,
    window_size : [ width, height ].into(),
    fov,
    rotation_speed_scale : 200.0,
    ..Default::default()
  };
  let camera = Rc::new( RefCell::new( camera ) );

  camera_controls::setup_controls( &canvas, &camera );

  let gltf_file_path = "dodge-challenger/gltf";

  let gltf_slice= gl::file::load( &format!( "{}/scene.gltf", gltf_file_path ) ).await.expect( "Failed to load gltf file" );
  let mut gltf_file = Gltf::from_slice( &gltf_slice ).unwrap();

  let mut buffers : Vec< gl::js_sys::Uint8Array > = Vec::new();

  // Move the GLB bin into buffers
  if let Some( blob ) = gltf_file.blob.as_mut()
  {
    let blob = std::mem::take( blob );
    buffers.push( blob.as_slice().into() );
  }

  for gltf_buffer in gltf_file.buffers()
  {
    match gltf_buffer.source()
    {
      gltf::buffer::Source::Uri( uri ) =>
      {
        let buffer = gl::file::load( &format!( "{}/{}", gltf_file_path, uri ) ).await
        .expect( "Failed to load a buffer" );

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
    let img_element = document.create_element( "img" ).unwrap().dyn_into::< gl::web_sys::HtmlImageElement >().unwrap();
    img_element.style().set_property( "display", "none" ).unwrap();
    let load_texture : Closure< dyn Fn() > = Closure::new
    ( 
      {
        let images = images.clone();
        let gl = gl.clone();
        let img = img_element.clone();
        let src = src.clone();
        move || 
        {
          let texture = gl::texture::d2::upload( &gl, &img );

          if texture.is_some()
          {
            images.borrow_mut().push( texture.unwrap() );
          }

          match gl::web_sys::Url::revoke_object_url( &src )
          {
            Ok( _ ) => { gl::info!( "Remove object url: {}", &src ) },
            Err( _ ) => { gl::info!( "Not an object url: {}", &src ) }
          }

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
        upload_texture( Rc::new( format!( "{}/{}", gltf_file_path, uri ) ) );
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

  // Create materials
  let mut materials = Vec::new();
  for gltf_m in gltf_file.materials()
  {
    let m = Material::new( gltf_m, &textures );
    materials.push( m );
  }

  gl::log::info!( "{:?}", gltf_file.materials().len() );

  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::BLEND );

  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.clear_depth( 1.0 );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      let view_matrix = camera.borrow().view().to_array();
      let eye = camera.borrow().eye().to_array();

      gl::uniform::upload
      (
        &gl, 
        gl.get_uniform_location( &program, "cameraPosition" ), 
        &eye[ .. ]
      ).unwrap();

      gl::uniform::matrix_upload
      ( 
        &gl, 
        gl.get_uniform_location( &program, "viewMatrix" ), 
        &view_matrix[ .. ], 
        true 
      ).unwrap();

      gl::uniform::matrix_upload
      ( 
        &gl, 
        gl.get_uniform_location( &program, "projectionMatrix" ), 
        projection_matrix.to_array().as_slice(), 
        true 
      ).unwrap();

      gl.draw_arrays( gl::TRIANGLE_STRIP, 0, 4 );
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

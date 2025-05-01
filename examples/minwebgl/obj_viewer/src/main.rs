//! Just draw a large point in the middle of the screen.

use std::
{
  collections::{ HashMap, HashSet },
  sync::{ Arc, Mutex }
};

use material::{ GLMaterial, TextureType };
use mesh::GLMesh;
use mingl::CameraOrbitControls;
use minwebgl::{ self as gl, JsCast };
use web_sys::wasm_bindgen::prelude::Closure;

mod mesh;
mod camera_controls;
mod material;

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, Default::default() )?;

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 20.0, 20.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let perspective_matrix = gl::math::mat3x3h::perspective_rh_gl
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
    ..Default::default()
  };
  let camera = Arc::new( Mutex::new( camera ) );

  camera_controls::setup_controls( &canvas, &camera );

  // You need to provide the full path to the object, and paths to folder that contain textures and mtl
  // Path is relative to "assets", and you cannot move up, so all of your file should be located in "assets" folder
  // mtl and texture's paths are specified inside obj and mtl files respectively
  let mtl_path = "lost-empire";
  let texture_path = "lost-empire";
  let obl_path = "lost-empire/lost_empire.obj";

  gl::console::time_with_label( "Load" );
  let model_buffer = gl::file::load( obl_path ).await.expect( "Failed to fetch the model" );
  gl::console::time_end_with_label( "Load" );

  // This is for loading the model in a form as close as possible to the form specified in obj file, which might not me suitable for render
  // But you can get some useful diagnostic information

  // let ( models, materials ) = load_model_from_slice( &model_buffer, mtl_path, &tobj::LoadOptions::default() ).await.expect( "Failed to load OBJ file" );
  // let materials = materials.expect( "Failed to load materials" );
  // diagnostic::make_reports( &models, &materials ).iter().for_each( | v | println!("{}", v));

  // Load model
  gl::console::time_with_label( "Parse" );
  let ( models, materials ) = gl::model::obj::load_model_from_slice( &model_buffer, mtl_path, &tobj::GPU_LOAD_OPTIONS ).await.expect( "Failed to load OBJ file" );
  let materials = materials.expect( "Failed to load materials" );
  gl::console::time_end_with_label( "Parse" );

  // Provides detailed info about the model
  gl::diagnostics::obj::make_reports( &models, &materials ).iter().for_each( | v | gl::log::info!( "{}", v ) );
  gl::console::time_with_label( "Create gl objects" );

  // Here we generate texture programs for each material( compile shaders for each one )
  // We store unique texture names inside a HashSet to then load them separately in the next step
  let mut texture_names = HashSet::new();
  let mut gl_materials = Vec::with_capacity( materials.len() );
  for mat in materials.iter()
  {
    let gl_material = GLMaterial::from_tobj_material( &gl, &mat, &mut texture_names )?;
    gl_material.init_uniforms( &gl );
    gl_materials.push(  gl_material );
  }

  let textures = Arc::new( Mutex::new( HashMap::new() ) );
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();
  for ( name, t_type ) in texture_names.into_iter()
  {
    let path = format!( "static/{}/{}", texture_path, name );
    gl::info!( "{}", path );

    let img_element = document.create_element( "img" ).unwrap().dyn_into::< gl::web_sys::HtmlImageElement >().unwrap();
    img_element.style().set_property( "display", "none" ).unwrap();
    let load_texture : Closure< dyn Fn() > = Closure::new
    (
      {
        let textures = textures.clone();
        let gl = gl.clone();
        let img = img_element.clone();
        move ||
        {
          let texture = gl.create_texture();
          gl::texture::d2::upload( &gl, texture.as_ref(), &img );

          if texture.is_some()
          {
            gl::texture::d2::default_parameters( &gl );
            // We generate mipmaps for the color textures, and ignore the others
            match t_type
            {
              TextureType::Ambient | TextureType::Diffuse =>
              {
                gl.generate_mipmap( gl::TEXTURE_2D );
                gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32 );
              },
              _ => {}
            }

            textures.lock().unwrap().insert( name.to_string(), texture.unwrap() );
          }
          img.remove();
        }
      }
    );

    img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
    img_element.set_src( &path );
    load_texture.forget();
  }

  // Here we generate Vertex Array Objects for each mesh and then bind
  // Each mesh with its material in a single struct
  let mut gl_meshes_opaque =  Vec::with_capacity( models.len() );
  let mut gl_meshes_transparent =  Vec::with_capacity( models.len() );
  for model in models.iter()
  {
    let gl_mesh = GLMesh::from_tobj_model( &gl, model, &gl_materials )?;
    gl_mesh.set_perpsective( &gl, &perspective_matrix );

    match gl_mesh.material().mtl
    {
      Some( ref mtl )
      if  mtl.dissolve.is_some() || mtl.dissolve_texture.is_some() =>
      {
        gl_meshes_transparent.push( gl_mesh );
      },
      _ =>
      {
        gl_meshes_opaque.push( gl_mesh );
      }
    }
  }

  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::BLEND );

  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.clear_depth( 1.0 );

  gl::console::time_end_with_label( "Create gl objects" );

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let _time = t as f32 / 1000.0;

      let view_matrix = camera.lock().unwrap().view().to_array();
      let eye = camera.lock().unwrap().eye().to_array();

      for m in gl_meshes_opaque.iter()
      {
        m.update( &gl, &eye, &view_matrix );
      }

      for m in gl_meshes_transparent.iter()
      {
        m.update( &gl, &eye, &view_matrix );
      }

      gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
      for m in gl_meshes_opaque.iter()
      {
        m.render( &gl, &textures );
      }

      for m in gl_meshes_transparent.iter()
      {
        m.render( &gl, &textures );
      }

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

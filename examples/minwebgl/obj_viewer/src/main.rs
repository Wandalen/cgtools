//! Interactive OBJ model viewer in WebGL2, with orbit-camera rotation and
//! zoom controls over a loaded Wavefront OBJ scene. Starts with a bundled
//! "lost empire" scene; upload your own `.obj` via the file picker to swap
//! it in. A standalone upload has no server-side folder to resolve a
//! companion `.mtl`/textures from, so it renders with the shader's default
//! (untextured) look — geometry and full diagnostics still work the same.

use std::
{
  collections::{ HashMap, HashSet },
  rc::Rc,
  cell::RefCell,
  sync::{ Arc, Mutex }
};

use material::{ GLMaterial, TextureType };
use mesh::GLMesh;
use mingl::
{
  CameraOrbitControls,
  controls::camera_orbit_controls::controls_bind_to_input
};
use minwebgl::{ self as gl, JsCast };
use web_sys::wasm_bindgen::prelude::Closure;

mod mesh;
mod material;

/// Everything the render loop needs per-frame; swapped wholesale on upload.
struct SceneState
{
  meshes_opaque : Vec< GLMesh >,
  meshes_transparent : Vec< GLMesh >,
  textures : Arc< Mutex< HashMap< String, gl::web_sys::WebGlTexture > > >,
}

/// Builds the orbit camera and its fixed projection matrix from the canvas size.
fn camera_setup( width : f32, height : f32 ) -> ( Rc< RefCell< CameraOrbitControls > >, gl::math::F32x4x4 )
{
  let eye = gl::math::F32x3::from( [ 0.0, 20.0, 20.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );
  let fov = 70.0f32.to_radians();
  let perspective_matrix = gl::math::mat3x3h::perspective_rh_gl( fov, width / height, 0.1, 10000.0 );

  let camera = CameraOrbitControls
  {
    eye,
    up,
    center,
    fov,
    window_size : [ width, height ].into(),
    ..Default::default()
  };

  ( Rc::new( RefCell::new( camera ) ), perspective_matrix )
}

/// Parses `obj_bytes`, logs full diagnostics (reused from task 097's reporting
/// machinery), and builds GPU-side materials/textures/meshes for it.
/// `mtl_path`/`texture_path` are the server-relative folders to resolve a
/// companion `.mtl`/textures from — pass `""` for a standalone uploaded file
/// with nothing to resolve against; materials/textures simply won't load and
/// geometry renders with the shader's defaults instead.
async fn scene_state_build
(
  gl : &gl::WebGl2RenderingContext,
  obj_bytes : &[ u8 ],
  mtl_path : &str,
  texture_path : &str,
  perspective_matrix : &gl::math::F32x4x4,
) -> Result< SceneState, gl::WebglError >
{
  gl::console::time_with_label( "Parse" );
  let ( models, materials ) = gl::model::obj::model_load_from_slice( obj_bytes, mtl_path, &tobj::GPU_LOAD_OPTIONS ).await.expect( "Failed to load OBJ file" );
  let materials = materials.unwrap_or_default();
  gl::console::time_end_with_label( "Parse" );

  // Provides detailed info about the model
  for report in &gl::diagnostics::obj::reports_make( &models, &materials )
  {
    gl::log::info!( "{report}" );
  }

  gl::console::time_with_label( "Create gl objects" );

  // Here we generate texture programs for each material( compile shaders for each one )
  // We store unique texture names inside a HashSet to then load them separately in the next step
  let mut texture_names = HashSet::new();
  let mut gl_materials = Vec::with_capacity( materials.len() );
  for mat in &materials
  {
    let gl_material = GLMaterial::from_tobj_material( gl, mat, &mut texture_names )?;
    gl_material.uniforms_init( gl );
    gl_materials.push( gl_material );
  }

  let textures = textures_load( gl, texture_names, texture_path );
  let ( meshes_opaque, meshes_transparent ) = meshes_build( gl, &models, &gl_materials, perspective_matrix )?;

  gl::console::time_end_with_label( "Create gl objects" );

  Ok( SceneState { meshes_opaque, meshes_transparent, textures } )
}

/// Wires the `#file-input` element's `change` event to `on_file_selected`.
fn file_input_setup< F : Fn( web_sys::File ) + 'static >( input_id : &str, on_file_selected : F )
{
  let document = web_sys::window().unwrap().document().unwrap();
  let file_input = document.get_element_by_id( input_id ).unwrap()
  .dyn_into::< web_sys::HtmlInputElement >().unwrap();

  let onchange : Closure< dyn FnMut( web_sys::Event ) > = Closure::new( move | event : web_sys::Event |
  {
    let input = event.target().unwrap().dyn_into::< web_sys::HtmlInputElement >().unwrap();
    if let Some( files ) = input.files()
    {
      if let Some( file ) = files.get( 0 )
      {
        on_file_selected( file );
      }
    }
  });
  file_input.set_onchange( Some( onchange.as_ref().unchecked_ref() ) );
  onchange.forget();
}

/// Reads `file`'s full contents into memory and invokes `on_bytes_read` with the bytes.
fn file_bytes_read< F : Fn( Vec< u8 > ) + 'static >( file : &web_sys::File, on_bytes_read : F )
{
  let file_reader = web_sys::FileReader::new().expect( "Should be able to create FileReader" );
  let fr = file_reader.clone();

  let onload : Closure< dyn FnMut() > = Closure::new( move ||
  {
    let result = fr.result().expect( "FileReader should have a result on load" );
    let array_buffer = result.dyn_into::< gl::js_sys::ArrayBuffer >()
    .expect( "FileReader result should be an ArrayBuffer" );
    on_bytes_read( gl::js_sys::Uint8Array::new( &array_buffer ).to_vec() );
  });
  file_reader.set_onload( Some( onload.as_ref().unchecked_ref() ) );
  onload.forget();

  file_reader.read_as_array_buffer( file ).expect( "Should be able to start reading file" );
}

/// Wires the file picker so each newly-selected file replaces the rendered scene.
fn upload_wire( gl : &gl::WebGl2RenderingContext, scene : &Rc< RefCell< SceneState > >, perspective_matrix : &gl::math::F32x4x4 )
{
  let gl_for_load = gl.clone();
  let scene_for_load = scene.clone();
  let perspective_matrix = *perspective_matrix;
  file_input_setup( "file-input", move | file : web_sys::File |
  {
    let gl_for_load = gl_for_load.clone();
    let scene_for_load = scene_for_load.clone();
    let perspective_matrix = perspective_matrix;
    file_bytes_read( &file, move | bytes : Vec< u8 > |
    {
      let gl_for_load = gl_for_load.clone();
      let scene_for_load = scene_for_load.clone();
      let perspective_matrix = perspective_matrix;
      gl::spawn_local( async move
      {
        match scene_state_build( &gl_for_load, &bytes, "", "", &perspective_matrix ).await
        {
          Ok( new_scene ) => *scene_for_load.borrow_mut() = new_scene,
          Err( e ) => gl::log::info!( "Failed to load uploaded model: {e:?}" ),
        }
      });
    });
  });
}

/// Starts the render loop: updates every mesh's camera-dependent uniforms,
/// then draws opaque meshes before transparent ones, reading whichever scene
/// (bundled or uploaded) is current each frame.
fn render_loop_start( gl : gl::WebGl2RenderingContext, camera : Rc< RefCell< CameraOrbitControls > >, scene : Rc< RefCell< SceneState > > )
{
  let update_and_draw = move | _t : f64 |
  {
    let view_matrix = camera.borrow().view().to_array();
    let eye = camera.borrow().eye().to_array();
    let scene_ref = scene.borrow();

    for m in &scene_ref.meshes_opaque
    {
      m.update( &gl, &eye, &view_matrix );
    }
    for m in &scene_ref.meshes_transparent
    {
      m.update( &gl, &eye, &view_matrix );
    }

    gl.clear( gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT );
    for m in &scene_ref.meshes_opaque
    {
      m.render( &gl, &scene_ref.textures );
    }
    for m in &scene_ref.meshes_transparent
    {
      m.render( &gl, &scene_ref.textures );
    }

    true
  };

  gl::exec_loop::run( update_and_draw );
}

async fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;
  let ( camera, perspective_matrix ) = camera_setup( width, height );
  controls_bind_to_input( &canvas, &camera );

  gl.enable( gl::DEPTH_TEST );
  gl.enable( gl::BLEND );
  gl.blend_func( gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.clear_depth( 1.0 );

  // You need to provide the full path to the object, and paths to folder that contain textures and mtl
  // Path is relative to "assets", and you cannot move up, so all of your file should be located in "assets" folder
  // mtl and texture's paths are specified inside obj and mtl files respectively
  gl::console::time_with_label( "Load" );
  let model_buffer = gl::file::load( "static/lost-empire/lost_empire.obj" ).await
  .map_err( | e | gl::dom::Error::BindgenError( "Failed to fetch the model", format!( "{e:?}" ) ) )?;
  gl::console::time_end_with_label( "Load" );

  let initial_scene = scene_state_build( &gl, &model_buffer, "static/lost-empire", "lost-empire", &perspective_matrix ).await?;
  let scene = Rc::new( RefCell::new( initial_scene ) );

  upload_wire( &gl, &scene, &perspective_matrix );
  render_loop_start( gl, camera, scene );

  Ok( () )
}

/// Loads every texture named in `texture_names` from `static/{texture_path}/`,
/// uploading each into a WebGl texture once its hidden image element finishes loading.
fn textures_load
(
  gl : &gl::WebGl2RenderingContext,
  texture_names : HashSet< ( String, TextureType ) >,
  texture_path : &str
) -> Arc< Mutex< HashMap< String, gl::web_sys::WebGlTexture > > >
{
  let textures = Arc::new( Mutex::new( HashMap::new() ) );
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();
  for ( name, t_type ) in texture_names
  {
    let path = format!( "static/{texture_path}/{name}" );
    gl::info!( "{path}" );

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

          if let Some( texture ) = texture
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

            textures.lock().unwrap().insert( name.clone(), texture );
          }
          img.remove();
        }
      }
    );

    img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
    img_element.set_src( &path );
    load_texture.forget();
  }
  textures
}

/// Builds a VAO-backed mesh for every model, binding each with its material,
/// and splits the result into opaque and transparent groups.
fn meshes_build
(
  gl : &gl::WebGl2RenderingContext,
  models : &[ tobj::Model ],
  gl_materials : &[ GLMaterial ],
  perspective_matrix : &gl::math::F32x4x4
) -> Result< ( Vec< GLMesh >, Vec< GLMesh > ), gl::WebglError >
{
  // Here we generate Vertex Array Objects for each mesh and then bind
  // Each mesh with its material in a single struct
  let mut gl_meshes_opaque = Vec::with_capacity( models.len() );
  let mut gl_meshes_transparent = Vec::with_capacity( models.len() );
  for model in models
  {
    let gl_mesh = GLMesh::from_tobj_model( gl, model, gl_materials )?;
    gl_mesh.perpsective_set( gl, perspective_matrix );

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
  Ok( ( gl_meshes_opaque, gl_meshes_transparent ) )
}

fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

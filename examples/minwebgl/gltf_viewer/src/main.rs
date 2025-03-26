//! Just draw a large point in the middle of the screen.

use std::
{
  cell::RefCell, collections::{ HashMap, HashSet }, rc::Rc 
};

// use material::{ GLMaterial, TextureType };
// use mesh::GLMesh;
use mingl::CameraOrbitControls;
use minwebgl::{ self as gl, JsCast };
use web_sys::wasm_bindgen::prelude::Closure;

// mod mesh;
mod camera_controls;
// mod material;
mod scene;
mod node;

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

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

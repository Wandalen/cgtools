//! 3d line demo
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::match_wildcard_for_single_variants ) ]
#![ allow( clippy::single_match ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::redundant_field_names ) ]
#![ allow( clippy::std_instead_of_core ) ]

use mingl::CameraOrbitControls;
use minwebgl as gl;
use std::
{
  cell::RefCell,
  rc::Rc,
};
use gl::wasm_bindgen::prelude::*;
use serde::{ Deserialize, Serialize };

mod camera_controls;
mod lil_gui;

#[ derive( Default, Serialize, Deserialize ) ]
struct Settings
{
  width : f32
}


fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  #[ allow( clippy::cast_precision_loss ) ]
  let width = canvas.width() as f32;
  #[ allow( clippy::cast_precision_loss ) ]
  let height = canvas.height() as f32;

  let main_frag = include_str!( "../shaders/main.frag" );

  let background_frag = include_str!( "../shaders/background.frag" );
  let background_vert = include_str!( "../shaders/background.vert" );

  let background_program = gl::ProgramFromSources::new( background_vert, background_frag ).compile_and_link( &gl )?;

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 2.0, 2.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::default();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.0001f32;
  let far = 100.0f32;

  let camera = CameraOrbitControls
  {
    eye : eye,
    up : up,
    center : center,
    window_size : [ width, height ].into(),
    fov,
    ..Default::default()
  };
  let camera = Rc::new( RefCell::new( camera ) );
  camera_controls::setup_controls( &canvas, &camera );

  let world_matrix = gl::math::mat4x4::identity();
  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl( fov, aspect_ratio, near, far );

  let line_width = 10.0;
  let radius = 1.0;

  let mut line = line_tools::d3::Line::default();

  line.create_mesh( &gl, 16, main_frag )?;
  let mesh = line.get_mesh();
  mesh.upload( &gl, "u_width", &line_width )?;
  mesh.upload( &gl, "u_resolution", &[ width as f32, height as f32 ] )?;
  mesh.upload_matrix( &gl, "u_projection_matrix", &projection_matrix.to_array() )?;
  mesh.upload_matrix( &gl, "u_world_matrix", &world_matrix.to_array() ).unwrap();

  let line = Rc::new( RefCell::new( line ) );

  let settings = Settings
  {
    width : line_width
  };

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = lil_gui::new_gui();

  let prop = lil_gui::add_slider( &gui, &object, "width", 0.0, 100.0, 0.01 );
  let callback = Closure::new
  (
    {
      let line = line.clone();
      let gl = gl.clone();
      move | value : f32 |
      {
        line.borrow().get_mesh().upload( &gl, "u_width", &value ).unwrap();
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  // Define the update and draw logic
  let update_and_draw =
  {
    let add_interval = 0.2;
    let mut elapsed_time = 0.0;
    #[ allow( clippy::min_ident_chars ) ]
    move | time_ms : f64 |
    {
      #[ allow( clippy::cast_possible_truncation ) ]
      let time = time_ms as f32 / 1000.0;

      let x_freq = ( time / 10.0 ).sin() * 3.0;
      let y_freq = ( time / 10.0 ).cos() * 3.0;
      let z_freq = ( time / 20.0 ).sin() * 3.0;

      let x_offset = 0.0;
      let y_offset = 0.0;
      let z_offset = 0.0;

      let x = ( x_freq * time + x_offset ).cos() * radius;
      let y = ( y_freq * time + y_offset ).sin() * radius;
      let z = ( z_freq * time + z_offset ).sin() * radius;

      if elapsed_time > add_interval
      {
        line.borrow_mut().add_point( gl::F32x3::new( x, y, z ) );
        elapsed_time -= add_interval;
      }

      line.borrow().get_mesh().upload_matrix( &gl, "u_view_matrix", &camera.borrow().view().to_array() ).unwrap();

      gl.use_program( Some( &background_program ) );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );
      line.borrow_mut().draw( &gl ).unwrap();

      elapsed_time += time;

      return true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );
  return Ok( () )
}

fn main()
{
  run().unwrap();
}

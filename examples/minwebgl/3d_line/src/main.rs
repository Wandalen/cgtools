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

use mingl::
{ 
  CameraOrbitControls, 
  camera_orbit_controls::bind_controls_to_input 
};
use minwebgl as gl;
use std::
{
  cell::RefCell,
  rc::Rc,
};
use gl::wasm_bindgen::prelude::*;
use serde::{ Deserialize, Serialize };

use crate::simulation::Simulation;

mod lil_gui;
mod simulation;

#[ derive( Default, Serialize, Deserialize ) ]
struct Settings
{
  width : f32,
  #[ serde( rename = "Alpha to coverage" ) ]
  alpha_to_coverage : bool
}

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;
  //let gl = gl::context::from_canvas_with( &canvas, gl::context::ContexOptions::default().antialias( true ) )?;

  gl::info!("{:?}", gl.get_context_attributes().unwrap().get_antialias() );

  #[ allow( clippy::cast_precision_loss ) ]
  let width = canvas.width() as f32;
  #[ allow( clippy::cast_precision_loss ) ]
  let height = canvas.height() as f32;

  let background_frag = include_str!( "../shaders/background.frag" );
  let background_vert = include_str!( "../shaders/background.vert" );

  let background_program = gl::ProgramFromSources::new( background_vert, background_frag ).compile_and_link( &gl )?;

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] ) * 0.75;
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
  bind_controls_to_input( &canvas, &camera );

  let world_matrix = gl::math::mat4x4::identity();
  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl( fov, aspect_ratio, near, far );

  let line_width = 0.004;
  let num_bodies = 10;

  let settings = Settings
  {
    width : line_width,
    alpha_to_coverage : false
  };

  let mut simulation = Simulation::new( num_bodies );
  let mut lines = Vec::with_capacity( num_bodies );
  let mut base_colors = Vec::with_capacity( num_bodies );

  for _ in 0..num_bodies
  {
    //let color = base_color * i as f32 / num_bodies as f32;
    let color = gl::F32x3::new( fastrand::f32(), fastrand::f32(), fastrand::f32() );

    base_colors.push( color );

    let mut line = line_tools::d3::Line::default();
    line.use_vertex_color( true );
    line.use_alpha_to_coverage( settings.alpha_to_coverage );
    line.mesh_create( &gl, None )?;

    let mesh = line.mesh_get()?;
    mesh.upload( &gl, "u_width", &line_width )?;
    mesh.upload( &gl, "u_color", &color.to_array() )?;
    mesh.upload( &gl, "u_resolution", &[ width as f32, height as f32 ] )?;
    mesh.upload_matrix( &gl, "u_projection_matrix", &projection_matrix.to_array() )?;
    mesh.upload_matrix( &gl, "u_world_matrix", &world_matrix.to_array() ).unwrap();

    lines.push( line );
  }

  // lines[ 0 ].point_add( [ 0.0, 0.0, 0.0 ] );
  // lines[ 0 ].point_add( [ 1.0, 1.0, 0.0 ] );

  let lines = Rc::new( RefCell::new( lines ) );

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = lil_gui::new_gui();

  let prop = lil_gui::add_slider( &gui, &object, "width", 0.0, 0.05, 0.001 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let gl = gl.clone();
      move | value : f32 |
      {
        let lines = lines.borrow();
        for i in 0..lines.len()
        {
          lines[ i ].mesh_get().unwrap().upload( &gl, "u_width", &value ).unwrap();
        }
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_boolean( &gui, &object, "Alpha to coverage" );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let gl = gl.clone();
      move | value : bool |
      {
        let mut lines = lines.borrow_mut();
        for i in 0..lines.len()
        {
          lines[ i ].use_alpha_to_coverage( value );
        }

        if value
        {
          gl.enable( gl::SAMPLE_ALPHA_TO_COVERAGE );
        }
        else 
        {    
          gl.disable( gl::SAMPLE_ALPHA_TO_COVERAGE );
        }
      }
    }
  );
  lil_gui::on_change_bool( &prop, &callback );
  callback.forget();

  gl.enable( gl::DEPTH_TEST );
  gl.depth_func( gl::LEQUAL );

  // Define the update and draw logic
  let update_and_draw =
  {
    let add_interval = 0.02;
    let mut elapsed_time = 0.0;
    let mut last_time = 0.0;
    #[ allow( clippy::min_ident_chars ) ]
    move | time_ms : f64 |
    {
      #[ allow( clippy::cast_possible_truncation ) ]
      let time = time_ms as f32 / 1000.0;
      let delta_time = last_time - time;

      gl.clear( gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT );

      simulation.simulate( delta_time );
      //if elapsed_time > add_interval
      {
        for i in 0..num_bodies
        {
          let pos = simulation.bodies[ i ].position;
          let color = base_colors[ i ] * ( pos.mag() * 5.0 ).powf( 2.0 ).min( 1.0 );
          lines.borrow_mut()[ i ].point_add( pos );
          lines.borrow_mut()[ i ].color_add( color );
        }
        elapsed_time -= add_interval;
      }

      for i in 0..num_bodies
      {
        lines.borrow()[ i ].mesh_get().unwrap().upload_matrix( &gl, "u_view_matrix", &camera.borrow().view().to_array() ).unwrap();
      }

      gl.use_program( Some( &background_program ) );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      for i in 0..num_bodies
      {
        lines.borrow_mut()[ i ].draw( &gl ).unwrap();
      }

      elapsed_time += time;
      last_time = time;

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

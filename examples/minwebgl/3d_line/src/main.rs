//! # 3D Line Demo
//!
//! Demonstrates 3D line rendering with dashed patterns using an N-body gravitational simulation.
//! Each body leaves a colored trail drawn as a 3D line with configurable dash patterns,
//! width units (screen-space or world-space), and alpha-to-coverage anti-aliasing.
//! A lil-gui panel exposes all settings at runtime.
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::match_wildcard_for_single_variants ) ]
#![ allow( clippy::single_match ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::redundant_field_names ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::needless_range_loop ) ]

use mingl::
{
  CameraOrbitControls,
  controls::camera_orbit_controls::bind_controls_to_input
};
use minwebgl as gl;
use web_sys::js_sys;
use std::
{
  cell::RefCell,
  rc::Rc,
};

use crate::simulation::Simulation;

mod lil_gui;
mod simulation;
mod settings;

/// Sets up the WebGL context, simulation, line meshes, GUI, and starts the render loop.
fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  // Seed the random number generator with the current time for varied body positions each run.
  fastrand::seed( js_sys::Date::now() as u64 );

  #[ allow( clippy::cast_precision_loss ) ]
  let width = canvas.width() as f32;
  #[ allow( clippy::cast_precision_loss ) ]
  let height = canvas.height() as f32;

  // Compile the full-screen background gradient shader.
  let background_frag = include_str!( "../shaders/background.frag" );
  let background_vert = include_str!( "../shaders/background.vert" );
  let background_program = gl::ProgramFromSources::new( background_vert, background_frag ).compile_and_link( &gl )?;

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 0.0, 1.0 ] ) * 0.6;
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::default();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.0001f32;
  let far = 100.0f32;

  let mut camera = CameraOrbitControls::default();
  camera.eye = eye;
  camera.up = up;
  camera.center = center;
  camera.fov = fov;
  camera.window_size = [ width, height ].into();

  let camera = Rc::new( RefCell::new( camera ) );
  bind_controls_to_input( &canvas, &camera );

  let world_matrix = gl::math::mat4x4::identity();
  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl( fov, aspect_ratio, near, far );

  let settings = settings::init();
  let num_bodies = 20;

  let mut simulation = Simulation::new( num_bodies );
  let mut lines = Vec::with_capacity( num_bodies );
  let mut base_colors = Vec::with_capacity( num_bodies );

  // Pick line width depending on the unit mode (world-space vs screen-space).
  let line_width = if settings.world_units { settings.world_width } else { settings.screen_width };

  // Create one line mesh per body, each with a random base color.
  for _ in 0..num_bodies
  {
    let color = gl::F32x3::new( fastrand::f32(), fastrand::f32(), fastrand::f32() );

    base_colors.push( color );

    let mut line = line_tools::d3::Line::default();
    line.use_vertex_color( true );
    line.use_alpha_to_coverage( settings.alpha_to_coverage );
    line.use_world_units( settings.world_units );
    line.use_dash( settings.dashes );
    line.mesh_create( &gl, None )?;

    // Upload initial uniforms shared by every line.
    let mesh = line.mesh_get_mut()?;
    mesh.upload( &gl, "u_width", &line_width )?;
    mesh.upload( &gl, "u_color", &color )?;
    mesh.upload( &gl, "u_resolution", &gl::F32x2::from( [ width as f32, height as f32 ] ) )?;
    mesh.upload( &gl, "u_projection_matrix", &projection_matrix )?;
    mesh.upload( &gl, "u_world_matrix", &world_matrix ).unwrap();
    mesh.upload( &gl, "u_dash_offset", &settings.dash_offset ).unwrap();

    lines.push( line );
  }
 
  // Wrap lines in Rc<RefCell> so they can be shared with the GUI callbacks and render loop.
  let lines = Rc::new( RefCell::new( lines ) );
  settings::upload_dash_pattern( lines.clone(), &settings );
  // Build the lil-gui panel and get a JsValue handle to the settings object
  // so the render loop can read live UI values each frame.
  let settings_jsvalue = settings::bind_to_ui( &gl, &settings, lines.clone() );


  gl.enable( gl::DEPTH_TEST );
  gl.depth_func( gl::LEQUAL );

  if settings.alpha_to_coverage
  {
    gl.enable( gl::SAMPLE_ALPHA_TO_COVERAGE );
  }

  // Define the update and draw logic
  let update_and_draw =
  {
    #[ allow( clippy::min_ident_chars ) ]
    move | _ : f64 |
    {
      gl.clear( gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT );

      let settings : settings::Settings = serde_wasm_bindgen::from_value( settings_jsvalue.clone() ).unwrap();

      // Advance the N-body simulation and append each body's new position to its trail line.
      if settings.simulation_speed > 0.0
      {
        simulation.simulate( settings.simulation_speed );

        for i in 0..num_bodies
        {
          let pos = simulation.bodies[ i ].position;
          // Modulate the base color by distance from the origin for a pulsing glow effect.
          let color = base_colors[ i ] * ( pos.mag() * 4.0 ).powf( 2.0 ).min( 1.0 );
          lines.borrow_mut()[ i ].point_add_back( &pos );
          lines.borrow_mut()[ i ].color_add_back( color );

          // Trim the trail to the maximum allowed length so it doesn't grow indefinitely.
          let num_points = lines.borrow()[ i ].num_points();
          let max_point = settings.trail_length as usize;

          if num_points > max_point
          {
            lines.borrow_mut()[ i ].points_remove_front( num_points - max_point );
            lines.borrow_mut()[ i ].colors_remove_front( num_points - max_point );
          }
        }
      }
      

      // Upload the latest view matrix from the orbit camera.
      for i in 0..num_bodies
      {
        lines.borrow_mut()[ i ].mesh_get_mut().unwrap().upload( &gl, "u_view_matrix", &camera.borrow().view() ).unwrap();
      }

      // Draw the full-screen background first, then each trail line on top.
      gl.use_program( Some( &background_program ) );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      for i in 0..num_bodies
      {
        lines.borrow_mut()[ i ].draw( &gl ).unwrap();
      }

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

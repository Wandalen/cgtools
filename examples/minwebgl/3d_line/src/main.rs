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
use web_sys::js_sys;
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
  #[ serde( rename = "World width" ) ]
  world_width : f32,
  #[ serde( rename = "Screen width" ) ]
  screen_width : f32,
  #[ serde( rename = "Alpha to coverage" ) ]
  alpha_to_coverage : bool,
  #[ serde( rename = "World units" ) ]
  world_units : bool,
  #[ serde( rename = "Dashes" ) ]
  dashes : bool,
  #[ serde( rename = "Trail length" ) ]
  trail_length : f32,
  #[ serde( rename = "Simulation speed" ) ]
  simulation_speed : f32,
  #[ serde( rename = "Dash Version" ) ]
  dash_version : String,
  #[ serde( rename = "Dash offset" ) ]
  dash_offset : f32,
  #[ serde( rename = "Dash size 1" ) ]
  dash_size1 : f32,
  #[ serde( rename = "Dash gap 1" ) ]
  dash_gap1 : f32,
  #[ serde( rename = "Dash size 2" ) ]
  dash_size2 : f32,
  #[ serde( rename = "Dash gap 2" ) ]
  dash_gap2 : f32
}

fn upload_dash_pattern( lines : Rc< RefCell< Vec< line_tools::d3::Line > > >, settings : &Settings )
{
  let mut lines = lines.borrow_mut();
  gl::info!( "{}", settings.dash_version );
  match settings.dash_version.as_str()
  {
    "V1" => 
    {
      let dash_settings = line_tools::d3::DashPattern::V1( settings.dash_size1 );
      for i in 0..lines.len()
      {
        lines[ i ].dash_pattern_set( dash_settings );
      }
    },
    "V2" => 
    {
      let dash_settings = line_tools::d3::DashPattern::V2( [ settings.dash_size1, settings.dash_gap1 ] );
      for i in 0..lines.len()
      {
        lines[ i ].dash_pattern_set( dash_settings );
      }
    },
    "V3" => 
    {
      let dash_settings = line_tools::d3::DashPattern::V3( [ settings.dash_size1, settings.dash_gap1, settings.dash_size2 ] );
      for i in 0..lines.len()
      {
        lines[ i ].dash_pattern_set( dash_settings );
      }
    },
    "V4" => 
    {
      let dash_settings = line_tools::d3::DashPattern::V4( [ settings.dash_size1, settings.dash_gap1, settings.dash_size2, settings.dash_gap2 ] );
      for i in 0..lines.len()
      {
        lines[ i ].dash_pattern_set( dash_settings );
      }
    },
    _ => {}
  };
}

fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas( &canvas )?;

  fastrand::seed( js_sys::Date::now() as u64 );

  #[ allow( clippy::cast_precision_loss ) ]
  let width = canvas.width() as f32;
  #[ allow( clippy::cast_precision_loss ) ]
  let height = canvas.height() as f32;

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

  let screen_width = 5.0;
  let world_width = 0.01;
  let num_bodies = 20;

  let settings = Settings
  {
    world_width : world_width,
    screen_width : screen_width,
    alpha_to_coverage : true,
    world_units : true,
    dashes : true,
    trail_length : 300.0,
    simulation_speed : 0.003,
    dash_version : "V2".into(),
    dash_offset : 0.0,
    dash_size1 : 0.1,
    dash_gap1 : 0.1,
    dash_size2 : 0.1,
    dash_gap2 : 0.1,
  };

  let trail_length = Rc::new( RefCell::new( settings.trail_length ) );
  let simulation_speed = Rc::new( RefCell::new( settings.simulation_speed ) );

  let mut simulation = Simulation::new( num_bodies );
  let mut lines = Vec::with_capacity( num_bodies );
  let mut base_colors = Vec::with_capacity( num_bodies );

  let line_width = if settings.world_units { settings.world_width } else { settings.screen_width };

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

    let mesh = line.mesh_get_mut()?;
    mesh.upload( &gl, "u_width", &line_width )?;
    mesh.upload( &gl, "u_color", &color )?;
    mesh.upload( &gl, "u_resolution", &gl::F32x2::from( [ width as f32, height as f32 ] ) )?;
    mesh.upload( &gl, "u_projection_matrix", &projection_matrix )?;
    mesh.upload( &gl, "u_world_matrix", &world_matrix ).unwrap();
    mesh.upload( &gl, "u_dash_offset", &settings.dash_offset ).unwrap();
    // mesh.upload( &gl, "u_dash_size", &settings.dash_size ).unwrap();
    // mesh.upload( &gl, "u_dash_gap", &settings.dash_gap ).unwrap();

    lines.push( line );
  }

  lines[ 0 ].point_add_back( &[ 0.0, 0.0, 0.0 ] );
  lines[ 0 ].point_add_back( &[ 1.0, 0.0, 0.0 ] );
  lines[ 0 ].point_add_back( &[ 1.0, 1.0, 0.0 ] );
  lines[ 0 ].point_add_back( &[ 1.0, 1.0, 1.0 ] );
 
  let lines = Rc::new( RefCell::new( lines ) );

  upload_dash_pattern( lines.clone(), &settings );

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = lil_gui::new_gui();

  let prop = lil_gui::add_slider( &gui, &object, "World width", 0.0, 0.05, 0.001 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let gl = gl.clone();
      let object = object.clone();
      move | value : f32 |
      {
        let settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        if settings.world_units
        {
          let mut lines = lines.borrow_mut();
          for i in 0..lines.len()
          {
            lines[ i ].mesh_get_mut().unwrap().upload( &gl, "u_width", &value ).unwrap();
          }
        }
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_slider( &gui, &object, "Screen width", 0.0, 100.0, 1.0 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      let gl = gl.clone();
      move | value : f32 |
      {
        let settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        if !settings.world_units
        {
          let mut lines = lines.borrow_mut();
          for i in 0..lines.len()
          {
            lines[ i ].mesh_get_mut().unwrap().upload( &gl, "u_width", &value ).unwrap();
          }
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

  let prop = lil_gui::add_boolean( &gui, &object, "World units" );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let gl = gl.clone();
      let object = object.clone();
      move | value : bool |
      {
        let settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        let mut lines = lines.borrow_mut();
        for i in 0..lines.len()
        {
          lines[ i ].use_world_units( value );
        }

        if value
        {
          for i in 0..lines.len()
          {
            lines[ i ].mesh_get_mut().unwrap().upload( &gl, "u_width", &settings.world_width ).unwrap();
          }
        }
        else 
        {
          for i in 0..lines.len()
          {
            lines[ i ].mesh_get_mut().unwrap().upload( &gl, "u_width", &settings.screen_width ).unwrap();
          }
        }
      }
    }
  );
  lil_gui::on_change_bool( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_boolean( &gui, &object, "Dashes" );
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
          lines[ i ].use_dash( value );
        }
      }
    }
  );
  lil_gui::on_change_bool( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_slider( &gui, &object, "Trail length", 2.0, 500.0, 1.0 );
  let callback = Closure::new
  (
    {
      let trail_length = trail_length.clone();
      move | value : f32 |
      {
        *trail_length.borrow_mut() = value;
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_slider( &gui, &object, "Simulation speed", 0.0, 0.01, 0.00001 );
  let callback = Closure::new
  (
    {
      let simulation_speed = simulation_speed.clone();
      move | value : f32 |
      {
        *simulation_speed.borrow_mut() = value;
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  let gui = lil_gui::add_folder( &gui, "Dash settings" );

  let prop = lil_gui::add_dropdown( &gui, &object, "Dash Version", &serde_wasm_bindgen::to_value( &[ "V1", "V2", "V3", "V4" ] ).unwrap() );
  let callback = Closure::new
  (
    {
      let object = object.clone();
      let lines = lines.clone();
      move | value : String |
      {
        let mut settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        settings.dash_version = value;
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change_string( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_slider( &gui, &object, "Dash offset", 0.0, 1.0, 0.0001 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let gl = gl.clone();
      move | value : f32 |
      {
        let mut lines = lines.borrow_mut();
        for i in 0..lines.len()
        {
          lines[ i ].mesh_get_mut().unwrap().upload( &gl, "u_dash_offset", &value ).unwrap();
        }
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_slider( &gui, &object, "Dash size 1", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      move | value : f32 |
      {
        let mut settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        settings.dash_size1 = value;
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_slider( &gui, &object, "Dash gap 1", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      move | value : f32 |
      {
        let mut settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        settings.dash_gap1 = value;
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

   let prop = lil_gui::add_slider( &gui, &object, "Dash size 2", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      move | value : f32 |
      {
        let mut settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        settings.dash_size2 = value;
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  let prop = lil_gui::add_slider( &gui, &object, "Dash gap 2", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      move | value : f32 |
      {
        let mut settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        settings.dash_gap2 = value;
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();


  gl.enable( gl::DEPTH_TEST );
  gl.depth_func( gl::LEQUAL );

  if settings.alpha_to_coverage
  {
    gl.enable( gl::SAMPLE_ALPHA_TO_COVERAGE );
  }

  // Define the update and draw logic
  let update_and_draw =
  {
    let mut last_time = 0.0;
    #[ allow( clippy::min_ident_chars ) ]
    move | time_ms : f64 |
    {
      #[ allow( clippy::cast_possible_truncation ) ]
      let time = time_ms as f32 / 1000.0;
      let _delta_time = last_time - time;

      gl.clear( gl::DEPTH_BUFFER_BIT | gl::COLOR_BUFFER_BIT );

      // simulation.simulate( *simulation_speed.borrow() );
      
      // for i in 0..num_bodies
      // {
      //   let pos = simulation.bodies[ i ].position;
      //   let color = base_colors[ i ] * ( pos.mag() * 4.0 ).powf( 2.0 ).min( 1.0 );
      //   lines.borrow_mut()[ i ].point_add_back( &pos );
      //   lines.borrow_mut()[ i ].color_add_back( color );

      //   let num_points = lines.borrow()[ i ].num_points();

      //   let max_point = *trail_length.borrow() as usize;

      //   if num_points > max_point
      //   {
      //     lines.borrow_mut()[ i ].points_remove_front( num_points - max_point );
      //     lines.borrow_mut()[ i ].colors_remove_front( num_points - max_point );
      //   }
      // }
      

      for i in 0..num_bodies
      {
        lines.borrow_mut()[ i ].mesh_get_mut().unwrap().upload( &gl, "u_view_matrix", &camera.borrow().view() ).unwrap();
      }

      gl.use_program( Some( &background_program ) );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      for i in 0..num_bodies
      {
        lines.borrow_mut()[ i ].draw( &gl ).unwrap();
      }

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

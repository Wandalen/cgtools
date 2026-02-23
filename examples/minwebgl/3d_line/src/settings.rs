//! Runtime settings and lil-gui bindings for the 3D line demo.
//!
//! [`Settings`] is serialized to a JS object via `serde_wasm_bindgen` so that
//! lil-gui can read and mutate it in the browser. Each GUI control is wired to
//! a closure that pushes the new value into the line meshes immediately.

use serde::{ Deserialize, Serialize };
use minwebgl as gl;
use gl::wasm_bindgen::prelude::*;
use crate::lil_gui;
use std::
{
  cell::RefCell,
  rc::Rc,
};
use line_tools::d3::{ Line, DashPattern };

/// Mirrors the lil-gui panel state.
///
/// Field names are renamed via `serde` to match the labels displayed in the UI.
#[ derive( Default, Serialize, Deserialize ) ]
pub struct Settings
{
  /// Line width when using world-space units.
  #[ serde( rename = "World width" ) ]
  pub world_width : f32,
  /// Line width when using screen-space (pixel) units.
  #[ serde( rename = "Screen width" ) ]
  pub screen_width : f32,
  /// Whether to use alpha-to-coverage for smoother line edges.
  #[ serde( rename = "Alpha to coverage" ) ]
  pub alpha_to_coverage : bool,
  /// If `true`, line width is measured in world units; otherwise in screen pixels.
  #[ serde( rename = "World units" ) ]
  pub world_units : bool,
  /// Enable dashed line rendering.
  #[ serde( rename = "Dashes" ) ]
  pub dashes : bool,
  /// Maximum number of points per trail line.
  #[ serde( rename = "Trail length" ) ]
  pub trail_length : f32,
  /// Speed multiplier passed to the N-body simulation each frame.
  #[ serde( rename = "Simulation speed" ) ]
  pub simulation_speed : f32,
  /// Which [`DashPattern`] variant to use (`"V1"` through `"V4"`).
  #[ serde( rename = "Dash Version" ) ]
  pub dash_version : String,
  /// Phase offset applied to the dash pattern along the line.
  #[ serde( rename = "Dash offset" ) ]
  pub dash_offset : f32,
  /// Length of the first dash segment.
  #[ serde( rename = "Dash size 1" ) ]
  pub dash_size1 : f32,
  /// Length of the first gap segment.
  #[ serde( rename = "Dash gap 1" ) ]
  pub dash_gap1 : f32,
  /// Length of the second dash segment (used by V3 and V4).
  #[ serde( rename = "Dash size 2" ) ]
  pub dash_size2 : f32,
  /// Length of the second gap segment (used by V4).
  #[ serde( rename = "Dash gap 2" ) ]
  pub dash_gap2 : f32
}

/// Returns the default settings used at startup.
pub fn init() -> Settings
{
  let screen_width = 5.0;
  let world_width = 0.01;

  let settings = Settings
  {
    world_width : world_width,
    screen_width : screen_width,
    alpha_to_coverage : true,
    world_units : true,
    dashes : true,
    trail_length : 300.0,
    simulation_speed : 0.0003,
    dash_version : "V2".into(),
    dash_offset : 0.0,
    dash_size1 : 0.1,
    dash_gap1 : 0.1,
    dash_size2 : 0.1,
    dash_gap2 : 0.1,
  };

  settings
}

/// Builds the lil-gui panel and wires every control to a closure that updates the line meshes.
///
/// Returns the JS settings object so the render loop can read live values each frame.
pub fn bind_to_ui
(
  gl : &gl::WebGl2RenderingContext,
  settings : &Settings,
  lines : Rc< RefCell< Vec< Line > > >
) -> JsValue
{
  // Serialize the Rust settings into a JS object that lil-gui can bind to.
  let object = serde_wasm_bindgen::to_value( settings ).unwrap();
  let gui = lil_gui::new_gui();

  // Line width in world coordinates
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

  // Line width in screen coordinates
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

  // Enable/disable alpha to coverage
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

  // Switch between world and screen space units
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

  // Enable/disable dashes
  let prop = lil_gui::add_boolean( &gui, &object, "Dashes" );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
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

  // Trail length and simulation speed are read each frame from the JS object,
  // so they only need a slider — no onChange callback required.
  let _ = lil_gui::add_slider( &gui, &object, "Trail length", 2.0, 500.0, 1.0 );
  let _ = lil_gui::add_slider( &gui, &object, "Simulation speed", 0.0, 0.001, 0.00001 );

  let gui = lil_gui::add_folder( &gui, "Dash settings" );

  // Switch the dash style
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

  // Change the dash offset
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

  // Dash parameter 1
  let prop = lil_gui::add_slider( &gui, &object, "Dash size 1", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      move | _ : f32 |
      {
        let settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  // Dash parameter 2
  let prop = lil_gui::add_slider( &gui, &object, "Dash gap 1", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      move | _ : f32 |
      {
        let settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  // Dash parameter 3
  let prop = lil_gui::add_slider( &gui, &object, "Dash size 2", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      move | _ : f32 |
      {
        let settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  // Dash parameter 4
  let prop = lil_gui::add_slider( &gui, &object, "Dash gap 2", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let lines = lines.clone();
      let object = object.clone();
      move | _ : f32 |
      {
        let settings : Settings = serde_wasm_bindgen::from_value( object.clone() ).unwrap();
        upload_dash_pattern( lines.clone(), &settings );
      }
    }
  );
  lil_gui::on_change( &prop, &callback );
  callback.forget();

  object
}

/// Constructs a [`DashPattern`] from the current settings and applies it to every line.
///
/// The variant is chosen by `settings.dash_version` (`"V1"` .. `"V4"`), and the
/// segment lengths come from `dash_size1`, `dash_gap1`, `dash_size2`, `dash_gap2`.
pub fn upload_dash_pattern( lines : Rc< RefCell< Vec< Line > > >, settings : &Settings )
{
  let mut lines = lines.borrow_mut();
  match settings.dash_version.as_str()
  {
    "V1" => 
    {
      let dash_settings = DashPattern::V1( settings.dash_size1 );
      for i in 0..lines.len()
      {
        lines[ i ].dash_pattern_set( dash_settings );
      }
    },
    "V2" => 
    {
      let dash_settings = DashPattern::V2( [ settings.dash_size1, settings.dash_gap1 ] );
      for i in 0..lines.len()
      {
        lines[ i ].dash_pattern_set( dash_settings );
      }
    },
    "V3" => 
    {
      let dash_settings = DashPattern::V3( [ settings.dash_size1, settings.dash_gap1, settings.dash_size2 ] );
      for i in 0..lines.len()
      {
        lines[ i ].dash_pattern_set( dash_settings );
      }
    },
    "V4" => 
    {
      let dash_settings = DashPattern::V4( [ settings.dash_size1, settings.dash_gap1, settings.dash_size2, settings.dash_gap2 ] );
      for i in 0..lines.len()
      {
        lines[ i ].dash_pattern_set( dash_settings );
      }
    },
    _ => {}
  };
}
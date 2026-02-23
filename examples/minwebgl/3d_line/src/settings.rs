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


#[ derive( Default, Serialize, Deserialize ) ]
pub struct Settings
{
  #[ serde( rename = "World width" ) ]
  pub world_width : f32,
  #[ serde( rename = "Screen width" ) ]
  pub screen_width : f32,
  #[ serde( rename = "Alpha to coverage" ) ]
  pub alpha_to_coverage : bool,
  #[ serde( rename = "World units" ) ]
  pub world_units : bool,
  #[ serde( rename = "Dashes" ) ]
  pub dashes : bool,
  #[ serde( rename = "Trail length" ) ]
  pub trail_length : f32,
  #[ serde( rename = "Simulation speed" ) ]
  pub simulation_speed : f32,
  #[ serde( rename = "Dash Version" ) ]
  pub dash_version : String,
  #[ serde( rename = "Dash offset" ) ]
  pub dash_offset : f32,
  #[ serde( rename = "Dash size 1" ) ]
  pub dash_size1 : f32,
  #[ serde( rename = "Dash gap 1" ) ]
  pub dash_gap1 : f32,
  #[ serde( rename = "Dash size 2" ) ]
  pub dash_size2 : f32,
  #[ serde( rename = "Dash gap 2" ) ]
  pub dash_gap2 : f32
}

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

pub fn bind_to_ui
(
  gl : &gl::WebGl2RenderingContext,
  settings : &Settings,
  lines : Rc< RefCell< Vec< Line > > > 
) -> JsValue
{
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

  // Change the maximum amount of points a line can have
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
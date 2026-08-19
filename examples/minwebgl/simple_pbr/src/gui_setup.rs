use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;
use crate::lil_gui::{ gui_new, slider_add, color_add, on_change, on_finish_change, show };

#[ derive( Serialize, Deserialize ) ]
pub struct Settings
{
  #[ serde( rename = "baseColor" ) ]
  pub base_color : [ f32; 3 ],
  #[ serde( rename = "lightIntensity" ) ]
  pub light_intensity : f32,
  #[ serde( rename = "ambientIntensity" ) ]
  pub ambient_intensity : f32,
  pub exposure : f32,
}

impl Default for Settings
{
  fn default() -> Self
  {
    Self
    {
      base_color : [ 1.0, 0.766, 0.336 ],
      light_intensity : 3.0,
      ambient_intensity : 0.35,
      exposure : 1.0,
    }
  }
}

/// Builds the lil-gui panel and returns the live settings, updated in place as the user
/// interacts with the controls; the render loop reads it once per frame.
pub fn setup() -> Rc< RefCell< Settings > >
{
  let settings = Rc::new( RefCell::new( Settings::default() ) );
  let object = serde_wasm_bindgen::to_value( &*settings.borrow() ).unwrap();
  let gui = gui_new();

  let prop = color_add( &gui, &object, "baseColor" );
  let callback = Closure::new
  (
    {
      let settings = settings.clone();
      move | value : JsValue |
      {
        if let Ok( color ) = serde_wasm_bindgen::from_value::< [ f32; 3 ] >( value )
        {
          settings.borrow_mut().base_color = color;
        }
      }
    }
  );
  on_finish_change( &prop, &callback );
  callback.forget();

  let prop = slider_add( &gui, &object, "lightIntensity", 0.0, 8.0, 0.1 );
  let callback = Closure::new
  (
    {
      let settings = settings.clone();
      move | value : f32 | { settings.borrow_mut().light_intensity = value; }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = slider_add( &gui, &object, "ambientIntensity", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let settings = settings.clone();
      move | value : f32 | { settings.borrow_mut().ambient_intensity = value; }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = slider_add( &gui, &object, "exposure", 0.1, 3.0, 0.01 );
  let callback = Closure::new
  (
    {
      let settings = settings.clone();
      move | value : f32 | { settings.borrow_mut().exposure = value; }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  std::mem::forget( object );

  show( &gui );

  settings
}

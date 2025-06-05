use std::{cell::RefCell, rc::Rc};

use minwebgl as gl;
use renderer::webgl::Renderer;
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;

use crate::lil_gui::{add_slider, new_gui, on_change, show};


#[ derive( Serialize, Deserialize ) ]
pub struct Settings
{
  #[ serde( rename = "bloomRadius" ) ]
  bloom_radius : f32,
  #[ serde( rename = "bloomStrength" ) ]
  bloom_strength : f32,
}

impl Default for Settings
{
  fn default() -> Self 
  {
    let bloom_radius = 0.5;
    let bloom_strength = 1.0;

    Self
    {
      bloom_radius,
      bloom_strength,
    }
  }    
}


pub fn setup( renderer : Rc< RefCell< Renderer > > )
{
  let mut settings = Settings::default();
  settings.bloom_radius = renderer.borrow().get_bloom_radius();
  settings.bloom_strength = renderer.borrow().get_bloom_strength();

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

  let prop = add_slider( &gui, &object, "bloomRadius", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let renderer = renderer.clone();
      move | value |
      {
        renderer.borrow_mut().set_bloom_radius( value );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  let prop = add_slider( &gui, &object, "bloomStrength", 0.0, 10.0, 0.1 );
  let callback = Closure::new
  (
    {
      let renderer = renderer.clone();
      move | value |
      {
        renderer.borrow_mut().set_bloom_strength( value );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  std::mem::forget( object );

  show( &gui );
}
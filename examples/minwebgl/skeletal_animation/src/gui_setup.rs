
use std::{cell::RefCell, rc::Rc};

use minwebgl as gl;
use renderer::webgl::animation::Animation;
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;
use rustc_hash::FxHashMap;

use crate::lil_gui::{ on_change_string, gui_new, dropdown_add, show };

#[ derive( Default, Serialize, Deserialize ) ]
pub struct Settings
{
  animation : String,
}

pub fn setup
(
  animations : Vec< Animation >,
  current_animation : &Rc< RefCell< Animation > >
)
{
  if animations.is_empty()
  {
    return;
  }

  let mut settings = Settings::default();
  if let Some( name ) = &animations[ 0 ].name
  {
    settings.animation = name.clone().into_string();
    *current_animation.borrow_mut() = animations[ 0 ].clone();
  }

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = gui_new();

  let animations = animations.into_iter()
  .filter_map
  (
    | a |
    {
      a.name.clone()
      .map
      (
        | n |
        {
          ( n.into_string(), a )
        }
      )
    }
  )
  .collect::< FxHashMap< _, _ > >();

  let animation_names = animations.keys()
  .cloned()
  .collect::< Vec< _ > >();

  // Choose animation
  let prop = dropdown_add
  (
    &gui,
    &object,
    "animation",
    &serde_wasm_bindgen::to_value( animation_names.as_slice() ).unwrap()
  );

  let callback = Closure::new
  (
    {
      let current_animation = current_animation.clone();
      move | value : String |
      {
        if let Some( animation ) = animations.get( value.as_str() )
        {
          let mut current_animation = current_animation.borrow_mut();
          *current_animation = animation.clone();
        }
      }
    }
  );
  on_change_string( &prop, &callback );
  callback.forget();

  std::mem::forget( object );

  show( &gui );
}

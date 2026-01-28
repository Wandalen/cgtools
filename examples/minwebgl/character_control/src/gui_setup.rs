// setup_gui.rs

#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::field_reassign_with_default ) ]

use std::{ cell::RefCell, collections::HashMap, rc::Rc };

use animation::Sequencer;
use minwebgl as gl;
use renderer::webgl::animation::{ Animation, Scaler };
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;
use crate::lil_gui::{ add_dropdown, add_slider, new_gui, on_change, on_change_string, show };

const PART_NAMES : [ &str; 4 ] =
[
  "head",
  "hands",
  "body",
  "legs"
];

/// Dynamic settings object for lil-gui
#[ derive( Serialize, Deserialize ) ]
pub struct Settings
{
  animation : String,
  head : f64,
  hands : f64,
  body : f64,
  legs : f64,
}

impl Default for Settings
{
  fn default() -> Self
  {
    Self
    {
      animation : Default::default(),
      head : 1.0,
      hands : 1.0,
      body : 1.0,
      legs : 1.0
    }
  }
}

pub fn setup
(
  animations : Vec< Animation >
)
-> Rc< RefCell< Option< Scaler > > >
{
  let animations = animations
  .into_iter()
  .filter( | a | a.name.is_some() )
  .collect::< Vec< _ > >();

  if animations.is_empty()
  {
    return Rc::new( RefCell::new( None ) );
  }

  let mut settings = Settings::default();

  let scaler = Scaler::new( animations[ 0 ].inner_get::< Sequencer >().unwrap().clone() );
  let scaler = Rc::new( RefCell::new( Some( scaler ) ) );
  settings.animation = animations[ 0 ].name.clone().unwrap().to_string();

  if let Some( scaler ) = scaler.borrow_mut().as_mut()
  {
    for part in PART_NAMES
    {
      scaler.add( part, vec![], gl::F64x4::splat( 1.0 ) )
    }
  }

  let animation_names = animations
  .iter()
  .filter_map( | a | a.name.clone() )
  .collect::< Vec< _ > >();

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

  let prop = add_dropdown
  (
    &gui,
    &object,
    "animation",
    &serde_wasm_bindgen::to_value( animation_names.as_slice() ).unwrap()
  );

  let animations = animations
  .into_iter()
  .filter_map( | a | a.name.clone().map( | n | ( n, a ) ) )
  .collect::< HashMap< _, _ > >();

  let callback = Closure::new
  (
    {
      let scaler = scaler.clone();
      move | value : String |
      {
        if let Some( animation ) = animations.get( &value.into_boxed_str() )
        {
          if let Some( scaler_mut ) = scaler.borrow_mut().as_mut()
          {
            scaler_mut.animation = animation.inner_get::< Sequencer >().unwrap().clone();
          }
        }
      }
    }
  );
  on_change_string( &prop, &callback );
  callback.forget();

  for part in PART_NAMES
  {
    let prop = add_slider( &gui, &object, part, 0.0, 3.0, 0.01 );
    let scaler_ref = Rc::clone( &scaler );

    let callback = Closure::new
    (
      move | value : f32 |
      {
        let Ok( mut scaler_ref ) = scaler_ref.try_borrow_mut()
        else
        {
          return;
        };

        scaler_ref.as_mut()
        .map
        (
          | s |
          {
            if let Some( scale ) = s.scale_get_mut( part )
            {
              *scale = gl::F64x4::splat( value as f64 );
            }
            s.animation.reset();
          }
        );
      }
    );

    on_change( &prop, &callback );
    callback.forget();
  }

  std::mem::forget( object );

  show( &gui );

  scaler
}

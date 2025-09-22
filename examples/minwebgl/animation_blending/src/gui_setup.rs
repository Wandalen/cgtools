// setup_gui.rs

#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::field_reassign_with_default ) ]

use std::{ cell::RefCell, collections::HashMap, rc::Rc };

use animation::Sequencer;
use minwebgl as gl;
use renderer::webgl::animation::{ Animation, Blender };
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;
use crate::lil_gui::{ add_dropdown, add_slider, new_gui, on_change, on_change_string, show };

const MAX_BLENDED_ANIMATIONS : usize = 4;

/// Dynamic settings object for lil-gui.
/// Fields: `animation_0`, `weight_0`, ...
#[ derive( Default, Serialize, Deserialize ) ]
pub struct Settings
{
  animation_0 : String,
  animation_1 : String,
  animation_2 : String,
  animation_3 : String,
  weight_0 : f64,
  weight_1 : f64,
  weight_2 : f64,
  weight_3 : f64,
}

impl Settings
{
  fn set_animation( &mut self, i : usize, name : String )
  {
    match i
    {
      0 => { self.animation_0 = name; },
      1 => { self.animation_1 = name; },
      2 => { self.animation_2 = name; },
      3 => { self.animation_3 = name; },
      _ => ()
    }
  }

  fn set_weight( &mut self, i : usize, weight : f64 )
  {
    match i
    {
      0 => { self.weight_0 = weight; },
      1 => { self.weight_1 = weight; },
      2 => { self.weight_2 = weight; },
      3 => { self.weight_3 = weight; },
      _ => ()
    }
  }
}

pub fn setup
(
  animations : Vec< Animation >
)
-> Rc< RefCell< Option< Animation > > >
{
  let animations = animations
    .into_iter()
    .filter( | a | a.name.is_some() )
    .collect::< Vec< _ > >();

  if animations.len() < MAX_BLENDED_ANIMATIONS || animations.is_empty()
  {
    return Rc::new( RefCell::new( None ) );
  }

  let mut settings = Settings::default();

  let animation_names = animations
    .iter()
    .filter_map( | a | a.name.clone() )
    .collect::< Vec< _ > >();

  let blender = Rc::new
  (
    RefCell::new
    (
      Some
      (
        Animation::new
        (
          Some( "blender".to_string().into_boxed_str() ),
          Box::new( Blender::new() ),
          animations[ 0 ].nodes.clone()
        )
      )
    )
  );
  blender.borrow_mut().as_mut().map
  (
    | a | a.get_inner_mut::< Blender >().unwrap().normalize = true
  );

  let current_assignments : Rc< RefCell< HashMap< usize, String > > > = Rc::new( RefCell::new( HashMap::new() ) );

  for i in 0..MAX_BLENDED_ANIMATIONS
  {
    let anim_name = animation_names[ i ].to_string();
    let weight = 1.0 / MAX_BLENDED_ANIMATIONS as f64;

    settings.set_animation( i, anim_name.clone() );
    settings.set_weight( i, weight );

    if let Some( seq ) = animations[ i ].get_inner::< Sequencer >().cloned()
    {
      let slot_key = format!( "slot_{}", i ).into_boxed_str();
      blender.borrow_mut().as_mut().map
      (
        | a |
        {
          a.get_inner_mut::< Blender >()
          .unwrap()
          .add( slot_key, seq, gl::F64x3::splat( weight ) );
        }
      );
    }

    current_assignments.borrow_mut().insert( i, anim_name );
  }

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

  for i in 0..MAX_BLENDED_ANIMATIONS
  {
    let prop = add_dropdown
    (
      &gui,
      &object,
      &format!( "animation_{}", i ),
      &serde_wasm_bindgen::to_value( &animation_names ).unwrap()
    );

    let blender_ref = Rc::clone( &blender );
    let all_anims = animations.clone();
    let idx = i;
    let assignments_ref = Rc::clone( &current_assignments );

    let callback = Closure::new
    (
      move | value : String |
      {
        let mut option_b = blender_ref.borrow_mut();
        let b = option_b.as_mut()
        .map( | a | a.get_inner_mut::< Blender >() )
        .flatten()
        .unwrap();

        let slot_key = format!( "slot_{}", idx ).into_boxed_str();

        let weight = b.get_weights( slot_key.clone() ).unwrap_or_default();

        if let Some( anim ) = all_anims.iter().find( | a | a.name == Some( value.clone().into_boxed_str() ) )
        {
          if let Some( seq ) = anim.get_inner::< Sequencer >()
          .cloned()
          {
            b.remove( slot_key.clone() );
            b.add( slot_key.clone(), seq, weight );
            assignments_ref.borrow_mut().insert( idx, value );
            b.reset();
          }
        }
      }
    );

    on_change_string( &prop, &callback );
    callback.forget();
  }

  for i in 0..MAX_BLENDED_ANIMATIONS
  {
    let prop = add_slider( &gui, &object, &format!( "weight_{}", i ), 0.0, 1.0, 0.01 );
    let blender_ref = Rc::clone( &blender );
    let idx = i;

    let callback = Closure::new
    (
      move | value : f32 |
      {
        let slot_key = format!( "slot_{}", idx ).into_boxed_str();
        blender_ref.borrow_mut().as_mut()
        .map
        (
          | a |
          {
            a.get_inner_mut::< Blender >()
            .unwrap()
            .get_weights_mut( slot_key )
            .map( | w | { *w = gl::F64x3::splat( value as f64 ); } );
          }
        );
      }
    );

    on_change( &prop, &callback );
    callback.forget();
  }

  std::mem::forget( object );

  show( &gui );

  blender
}

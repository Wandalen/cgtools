
use std::{ cell::RefCell, rc::Rc };

use minwebgl as gl;
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;
use renderer::webgl::animation::Animation;
use std::collections::HashMap;

use crate::lil_gui::{ on_change_string, new_gui, dropdown_add, slider_add, on_change, show };

#[ derive( Default, Serialize, Deserialize ) ]
pub struct Settings
{
  animation : String,
  w0 : f32,
  w1 : f32,
  w2 : f32,
  w3 : f32,
  w4 : f32,
  w5 : f32,
  w6 : f32,
  w7 : f32,
  w8 : f32,
  w9 : f32,
  w10 : f32,
  w11 : f32,
  w12 : f32,
  w13 : f32,
  w14 : f32,
  w15 : f32,
  w16 : f32,
  w17 : f32,
  w18 : f32,
  w19 : f32,
  w20 : f32,
  w21 : f32,
  w22 : f32,
  w23 : f32,
  w24 : f32,
  w25 : f32,
  w26 : f32,
  w27 : f32,
  w28 : f32,
  w29 : f32,
  w30 : f32,
  w31 : f32,
  w32 : f32,
  w33 : f32,
  w34 : f32,
  w35 : f32,
  w36 : f32,
  w37 : f32,
  w38 : f32,
  w39 : f32,
  w40 : f32,
  w41 : f32,
  w42 : f32,
  w43 : f32,
  w44 : f32,
  w45 : f32,
  w46 : f32,
  w47 : f32,
  w48 : f32,
  w49 : f32,
  w50 : f32,
  w51 : f32,
  w52 : f32,
  w53 : f32,
  w54 : f32,
  w55 : f32,
  w56 : f32,
  w57 : f32,
  w58 : f32,
  w59 : f32,
}

/// Copies the first 60 morph weights into the numbered settings fields, defaulting missing entries to zero.
fn weight_settings_init( settings : &mut Settings, weights : &[ f32 ] )
{
  let mut weights_iter = weights.iter();
  settings.w0 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w1 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w2 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w3 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w4 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w5 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w6 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w7 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w8 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w9 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w10 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w11 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w12 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w13 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w14 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w15 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w16 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w17 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w18 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w19 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w20 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w21 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w22 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w23 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w24 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w25 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w26 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w27 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w28 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w29 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w30 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w31 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w32 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w33 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w34 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w35 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w36 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w37 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w38 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w39 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w40 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w41 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w42 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w43 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w44 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w45 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w46 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w47 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w48 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w49 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w50 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w51 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w52 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w53 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w54 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w55 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w56 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w57 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w58 = *weights_iter.next().unwrap_or( &0.0 );
  settings.w59 = *weights_iter.next().unwrap_or( &0.0 );
}

/// Builds the animation dropdown and wires selection changes to `current_animation`.
fn animation_dropdown_bind
(
  gui : &JsValue,
  object : &JsValue,
  animations : HashMap< String, Animation >,
  current_animation : &Rc< RefCell< Option< Animation > > >
)
{
  let mut animation_names = animations.keys()
  .cloned()
  .collect::< Vec< _ > >();

  animation_names.insert( 0, "<none>".to_string() );

  // Choose animation
  let prop = dropdown_add
  (
    gui,
    object,
    "animation",
    &serde_wasm_bindgen::to_value( animation_names.as_slice() ).unwrap()
  );

  let callback = Closure::new
  (
    {
      let current_animation = current_animation.clone();
      move | value : String |
      {
        let mut current_animation = current_animation.borrow_mut();
        if !animations.contains_key( value.as_str() )
        {
          if let Some( a ) = current_animation.as_mut()
          {
            if let Some( s ) = a.inner_get_mut::< animation::Sequencer >().as_mut()
            {
              s.reset();
            }
            a.set();
          }
        }
        *current_animation = animations.get( value.as_str() ).cloned();
      }
    }
  );
  on_change_string( &prop, &callback );
  callback.forget();
}

/// Creates the 60 weight sliders and wires each to its morph weight slot.
fn weight_sliders_bind( gui : &JsValue, object : &JsValue, weights : &Rc< RefCell< Vec< f32 > > > )
{
  for i in 0..60
  {
    let prop = slider_add( gui, object, &format!( "w{i}" ), 0.0, 1.0, 0.01 );
    let weights_rc = Rc::clone( weights );

    let callback = Closure::new
    (
      move | value : f32 |
      {
        let Ok( mut weights_ref ) = weights_rc.try_borrow_mut()
        else
        {
          return;
        };

        if let Some( w ) = weights_ref.get_mut( i )
        {
          *w = value;
        }
      }
    );

    on_change( &prop, &callback );
    callback.forget();
  }
}

/// Builds the lil-gui panel : an animation dropdown plus 60 morph weight sliders.
///
/// `initial_weights` seeds each slider's *displayed* starting value ( the current,
/// animation-driven weight ); `gui_weights` is the separate override-tracking buffer
/// ( BUG-330's `f32::NAN`-sentinel array ) that slider drags write into.
// Fix(BUG-462): split the single `weights` parameter into `initial_weights : &[ f32 ]`
// ( read once, for the sliders' initial displayed value ) and `gui_weights` ( the
// write-target for user overrides ), instead of one `Rc<RefCell<Vec<f32>>>` serving
// both roles.
// Root cause: the caller passed `gui_weights` -- an all-`f32::NAN` sentinel buffer --
// as this single parameter, so `weight_settings_init` read NaN for every slider's
// initial value. Passing the real `weights` buffer instead would have "fixed" the
// display but broken `weight_sliders_bind`'s write-back: `weights` is overwritten
// every frame by the animation system ( see `main.rs`'s per-frame `animation.set()`/
// `fill( 0.0 )` ), so a slider drag would be silently stomped on the very next frame
// instead of persisting as an override.
// Pitfall: one buffer used for two different roles ( "current displayed value" vs.
// "user override storage" ) can only ever be correct for one of them at a time --
// the fix is to give each role its own parameter, not to swap which single buffer
// is threaded through.
pub fn setup
(
  animations : Vec< Animation >,
  current_animation : &Rc< RefCell< Option< Animation > > >,
  initial_weights : &[ f32 ],
  gui_weights : &Rc< RefCell< Vec< f32 > > >
)
{
  let mut settings = Settings::default();

  if let Some( name ) = &animations[ 0 ].name
  {
    settings.animation = name.clone().into_string();
    *current_animation.borrow_mut() = Some( animations[ 0 ].clone() );
  }
  else
  {
    settings.animation = "<none>".to_string();
    *current_animation.borrow_mut() = None;
  }

  weight_settings_init( &mut settings, initial_weights );

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

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
  .collect::< HashMap< _, _ > >();

  animation_dropdown_bind( &gui, &object, animations, current_animation );
  weight_sliders_bind( &gui, &object, gui_weights );

  std::mem::forget( object );

  show( &gui );
}

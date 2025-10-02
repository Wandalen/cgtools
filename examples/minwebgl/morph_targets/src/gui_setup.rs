#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::field_reassign_with_default ) ]

use std::{cell::RefCell, rc::Rc};

use minwebgl as gl;
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;

use crate::lil_gui::{ new_gui, add_slider, on_change, show };


#[ derive( Default, Serialize, Deserialize ) ]
pub struct Settings
{
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

pub fn setup
(
  weights : Rc< RefCell< Vec< f32 > > >
)
{
  let mut settings = Settings::default();
  {
    let weights_ref = weights.borrow();
    let mut weights_iter = weights_ref.iter();
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

  let object = serde_wasm_bindgen::to_value( &settings ).unwrap();
  let gui = new_gui();

  for i in 0..60
  {
    let prop = add_slider( &gui, &object, &format!( "w{i}" ), 0.0, 1.0, 0.01 );
    let weights_rc = Rc::clone( &weights );

    let callback = Closure::new
    (
      move | value : f32 |
      {
        let Ok( mut weights_ref ) = weights_rc.try_borrow_mut()
        else
        {
          return;
        };

        weights_ref.get_mut( i )
        .map( | w | { *w = value; } );
      }
    );

    on_change( &prop, &callback );
    callback.forget();
  }

  std::mem::forget( object );

  show( &gui );
}

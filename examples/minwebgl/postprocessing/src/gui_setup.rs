#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::field_reassign_with_default ) ]
#![ allow( clippy::too_many_lines ) ]

use minwebgl as gl;
use std::rc::Rc;
use core::cell::RefCell;
use renderer::webgl::{ Renderer, post_processing::ColorGradingPass };
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;
use crate::lil_gui::{ add_slider, add_folder, new_gui, on_change, show };

#[ derive( Default, Serialize, Deserialize ) ]
pub struct RendererSettings
{
  #[ serde( rename = "bloomRadius" ) ]
  bloom_radius : f32,
  #[ serde( rename = "bloomStrength" ) ]
  bloom_strength : f32,
  exposure : f32
}

#[ derive( Default, Serialize, Deserialize ) ]
pub struct ColorGradingSettings
{
  temperature : f32,
  tint : f32,
  exposure : f32,
  shadows : f32,
  highlights : f32,
  contrast : f32,
  vibrance : f32,
  saturation : f32,
}


pub fn setup( renderer : Rc< RefCell< Renderer > >, color_grading : Rc< RefCell< ColorGradingPass > > )
{
  let gui = new_gui();

  // === Renderer Settings ===
  let mut renderer_settings = RendererSettings::default();
  renderer_settings.bloom_radius = renderer.borrow().get_bloom_radius();
  renderer_settings.bloom_strength = renderer.borrow().get_bloom_strength();
  renderer_settings.exposure = renderer.borrow().get_exposure();

  let renderer_object = serde_wasm_bindgen::to_value( &renderer_settings ).unwrap();
  let renderer_folder = add_folder( &gui, "Renderer" );

  // Exposure
  let prop = add_slider( &renderer_folder, &renderer_object, "exposure", -10.0, 10.0, 0.1 );
  let callback = Closure::new
  (
    {
      let renderer = renderer.clone();
      move | value |
      {
        renderer.borrow_mut().set_exposure( value );
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Bloom Radius
  let prop = add_slider( &renderer_folder, &renderer_object, "bloomRadius", 0.0, 1.0, 0.01 );
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

  // Bloom Strength
  let prop = add_slider( &renderer_folder, &renderer_object, "bloomStrength", 0.0, 10.0, 0.1 );
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

  // === Color Grading Settings ===
  let params = color_grading.borrow();
  let params = params.get_params();
  let mut cg_settings = ColorGradingSettings::default();
  cg_settings.temperature = params.temperature;
  cg_settings.tint = params.tint;
  cg_settings.exposure = params.exposure;
  cg_settings.shadows = params.shadows;
  cg_settings.highlights = params.highlights;
  cg_settings.contrast = params.contrast;
  cg_settings.vibrance = params.vibrance;
  cg_settings.saturation = params.saturation;

  let cg_object = serde_wasm_bindgen::to_value( &cg_settings ).unwrap();
  let cg_folder = add_folder( &gui, "Color Grading" );

  // White Balance folder
  let wb_folder = add_folder( &cg_folder, "White Balance" );

  // Temperature
  let prop = add_slider( &wb_folder, &cg_object, "temperature", -1.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        color_grading.borrow_mut().get_params_mut().temperature = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Tint
  let prop = add_slider( &wb_folder, &cg_object, "tint", -1.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        color_grading.borrow_mut().get_params_mut().tint = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Tone Controls folder
  let tone_folder = add_folder( &cg_folder, "Tone Controls" );

  // Exposure
  let prop = add_slider( &tone_folder, &cg_object, "exposure", -1.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        color_grading.borrow_mut().get_params_mut().exposure = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Shadows
  let prop = add_slider( &tone_folder, &cg_object, "shadows", -1.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        color_grading.borrow_mut().get_params_mut().shadows = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Highlights
  let prop = add_slider( &tone_folder, &cg_object, "highlights", -1.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        color_grading.borrow_mut().get_params_mut().highlights = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Color Adjustments folder
  let color_folder = add_folder( &cg_folder, "Color Adjustments" );

  // Contrast
  let prop = add_slider( &color_folder, &cg_object, "contrast", 0.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        color_grading.borrow_mut().get_params_mut().contrast = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Vibrance
  let prop = add_slider( &color_folder, &cg_object, "vibrance", -1.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        color_grading.borrow_mut().get_params_mut().vibrance = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  // Saturation
  let prop = add_slider( &color_folder, &cg_object, "saturation", -1.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        color_grading.borrow_mut().get_params_mut().saturation = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();

  core::mem::forget( renderer_object );
  core::mem::forget( cg_object );

  show( &gui );
}


use minwebgl as gl;
use std::rc::Rc;
use core::cell::RefCell;
use renderer::webgl::{ Renderer, post_processing::{ ColorGradingPass, ColorGradingParams } };
use serde::{ Deserialize, Serialize };
use gl::wasm_bindgen::prelude::*;
use crate::lil_gui::{ slider_add, folder_add, gui_new, on_change, show };

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

/// Adds one color grading slider wired to a single `ColorGradingParams` field.
fn grading_slider_add
(
  folder : &JsValue,
  cg_object : &JsValue,
  name : &str,
  color_grading : &Rc< RefCell< ColorGradingPass > >,
  field : fn( &mut ColorGradingParams ) -> &mut f32,
)
{
  let prop = slider_add( folder, cg_object, name, -1.0, 1.0, 0.01 );
  let callback = Closure::new
  (
    {
      let color_grading = color_grading.clone();
      move | value |
      {
        *field( color_grading.borrow_mut().get_params_mut() ) = value;
      }
    }
  );
  on_change( &prop, &callback );
  callback.forget();
}

pub fn setup( renderer : &Rc< RefCell< Renderer > >, color_grading : &Rc< RefCell< ColorGradingPass > > )
{
  let gui = gui_new();

  // === Renderer Settings ===
  let mut renderer_settings = RendererSettings::default();
  renderer_settings.bloom_radius = renderer.borrow().bloom_radius();
  renderer_settings.bloom_strength = renderer.borrow().bloom_strength();
  renderer_settings.exposure = renderer.borrow().exposure();

  let renderer_object = serde_wasm_bindgen::to_value( &renderer_settings ).unwrap();
  let renderer_folder = folder_add( &gui, "Renderer" );

  // Exposure
  let prop = slider_add( &renderer_folder, &renderer_object, "exposure", -10.0, 10.0, 0.1 );
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
  let prop = slider_add( &renderer_folder, &renderer_object, "bloomRadius", 0.0, 1.0, 0.01 );
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
  let prop = slider_add( &renderer_folder, &renderer_object, "bloomStrength", 0.0, 10.0, 0.1 );
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
  let cg_folder = folder_add( &gui, "Color Grading" );

  // White Balance folder
  let wb_folder = folder_add( &cg_folder, "White Balance" );

  grading_slider_add( &wb_folder, &cg_object, "temperature", color_grading, | p | &mut p.temperature );

  grading_slider_add( &wb_folder, &cg_object, "tint", color_grading, | p | &mut p.tint );

  // Tone Controls folder
  let tone_folder = folder_add( &cg_folder, "Tone Controls" );

  grading_slider_add( &tone_folder, &cg_object, "exposure", color_grading, | p | &mut p.exposure );

  grading_slider_add( &tone_folder, &cg_object, "shadows", color_grading, | p | &mut p.shadows );

  grading_slider_add( &tone_folder, &cg_object, "highlights", color_grading, | p | &mut p.highlights );

  // Color Adjustments folder
  let color_folder = folder_add( &cg_folder, "Color Adjustments" );

  grading_slider_add( &color_folder, &cg_object, "contrast", color_grading, | p | &mut p.contrast );

  grading_slider_add( &color_folder, &cg_object, "vibrance", color_grading, | p | &mut p.vibrance );

  grading_slider_add( &color_folder, &cg_object, "saturation", color_grading, | p | &mut p.saturation );

  core::mem::forget( renderer_object );
  core::mem::forget( cg_object );

  show( &gui );
}

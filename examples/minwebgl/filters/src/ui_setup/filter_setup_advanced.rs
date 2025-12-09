//! Setup for filters that require UI controls (sliders, dropdowns, etc.)

use crate::*;
use filters::*;
use wasm_bindgen::{ JsCast, JsValue, prelude::Closure };
use std::{ cell::RefCell, rc::Rc };
use web_sys::HtmlElement;
use super::{ filter_setup_helpers, event_handlers };

/// Macro to reduce repetition in filter setup
macro_rules! setup_filter_with_sliders {
  (
    $filter_renderer:expr,
    $current_filter:expr,
    $card_id:expr,
    $filter_type:ty,
    $initial_value:expr,
    [ $( ($label:expr, $prop:expr, $min:expr, $max:expr, $step:expr) ),+ ]
  ) => {{
    let filter_renderer_clone = $filter_renderer.clone();
    let current_filter_clone = $current_filter.clone();
    let onclick : Closure< dyn Fn() > = Closure::new( move ||
    {
      *current_filter_clone.borrow_mut() = String::from( $card_id );
      filter_renderer_clone.borrow_mut().save_previous_texture();

      // Clear and setup controls
      controls::clear_controls();

      // Add sliders
      $(
        let initial_val = serde_wasm_bindgen::to_value( &$initial_value ).unwrap();
        let obj_map = initial_val.dyn_into::< web_sys::js_sys::Object >().unwrap();
        let val = web_sys::js_sys::Reflect::get( &obj_map, &JsValue::from_str( $prop ) ).unwrap();
        let num_val = val.as_f64().unwrap_or( $min );
        controls::add_slider( $label, $prop, num_val, $min, $max, $step );
      )+

      // Apply initial filter
      filter_renderer_clone.borrow_mut().apply_filter( &$initial_value );

      // Setup onChange callback
      let fr = filter_renderer_clone.clone();
      let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
      {
        let filter : $filter_type = serde_wasm_bindgen::from_value( values ).unwrap();
        fr.borrow_mut().apply_filter( &filter );
      });
      controls::on_change( callback.as_ref().unchecked_ref() );
      callback.forget();

      controls::show();
    });

    let card = utils::get_element_by_id_unchecked::< HtmlElement >( $card_id );
    card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
    onclick.forget();
  }};
}

/// Sets up all filters that have UI controls
pub fn setup_filters_with_controls
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >
)
{
  // Blur filters need manual setup due to generic type parameters
  filter_setup_helpers::setup_blur_filter( filter_renderer, current_filter, "box-blur", "Box Blur", blur::Box, 80.0 );
  filter_setup_helpers::setup_blur_filter( filter_renderer, current_filter, "gaussian-blur", "Gaussian Blur", blur::Gaussian, 50.0 );
  filter_setup_helpers::setup_blur_filter( filter_renderer, current_filter, "stack-blur", "Stack Blur", blur::Stack, 80.0 );

  // Binarize
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "binarize",
    binarize::Binarize,
    binarize::Binarize { threshold: 0.5 },
    [ ("Threshold", "threshold", 0.0, 1.0, 0.001) ]
  );

  // Rescale
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "rescale",
    rescale::Rescale,
    rescale::Rescale { scale: 1.0 },
    [ ("Scale", "scale", 0.1, 10.0, 0.01) ]
  );

  // Resize filters need manual setup due to generic type parameters
  filter_setup_helpers::setup_resize_filter( filter_renderer, current_filter, "resize-nn", "Resize (NN)", resize::Nearest );
  filter_setup_helpers::setup_resize_filter( filter_renderer, current_filter, "resize-bilinear", "Resize (Bilinear)", resize::Bilinear );

  // Sharpen
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "sharpen",
    sharpen::Sharpen,
    sharpen::Sharpen { factor: 1.0 },
    [ ("Factor", "factor", 1.0, 10.0, 0.1) ]
  );

  // Dithering
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "dither",
    dithering::Dithering,
    dithering::Dithering { levels: 4 },
    [ ("Levels", "levels", 2.0, 20.0, 1.0) ]
  );

  // Posterize
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "posterize",
    posterize::Posterize,
    posterize::Posterize { levels: 4 },
    [ ("Levels", "levels", 2.0, 20.0, 1.0) ]
  );

  // Gamma
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "gamma",
    gamma::Gamma,
    gamma::Gamma { gamma: 1.0 },
    [ ("Gamma", "gamma", 0.1, 5.0, 0.01) ]
  );

  // Mosaic
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "mosaic",
    mosaic::Mosaic,
    mosaic::Mosaic { scale: 10 },
    [ ("Scale", "scale", 1.0, 100.0, 1.0) ]
  );

  // BrightnessContrast filters need manual setup due to generic type parameters
  filter_setup_helpers::setup_brightness_contrast_filter( filter_renderer, current_filter, "bcgimp", "BC (GIMP)", brightness_contrast::GIMP, -100.0, 100.0, 1.0 );
  filter_setup_helpers::setup_brightness_contrast_filter( filter_renderer, current_filter, "bcph", "BC (PS)", brightness_contrast::Photoshop, -1.0, 1.0, 0.01 );

  // Color Transform
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "color-transform",
    color_transform::ColorTransform,
    color_transform::ColorTransform
    {
      red_multiplier: 1.0,
      green_multiplier: 1.0,
      blue_multiplier: 1.0,
      red_offset: 0.0,
      green_offset: 0.0,
      blue_offset: 0.0,
    },
    [
      ("Red Mult", "redMultiplier", 0.0, 2.0, 0.01),
      ("Green Mult", "greenMultiplier", 0.0, 2.0, 0.01),
      ("Blue Mult", "blueMultiplier", 0.0, 2.0, 0.01),
      ("Red Offset", "redOffset", -1.0, 1.0, 0.01),
      ("Green Offset", "greenOffset", -1.0, 1.0, 0.01),
      ("Blue Offset", "blueOffset", -1.0, 1.0, 0.01)
    ]
  );

  // HSL Adjust
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "hsl-adjust",
    hsl_adjustment::HSLAdjustment,
    hsl_adjustment::HSLAdjustment
    {
      hue: 0.0,
      saturation: 0.0,
      lightness: 0.0,
    },
    [
      ("Hue", "hue", -1.0, 1.0, 0.01),
      ("Saturation", "saturation", -1.0, 1.0, 0.01),
      ("Lightness", "lightness", -1.0, 1.0, 0.01)
    ]
  );

  // Oil
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "oil",
    oil::Oil,
    oil::Oil { levels: 10, range: 3 },
    [
      ("Levels", "levels", 2.0, 50.0, 1.0),
      ("Range", "range", 1.0, 15.0, 1.0)
    ]
  );

  // Twirl
  setup_filter_with_sliders!(
    filter_renderer,
    current_filter,
    "twirl",
    twirl::Twirl,
    twirl::Twirl
    {
      center_x: 0.5,
      center_y: 0.5,
      radius: 0.3,
      strength: 10.0,
    },
    [
      ("Center X", "centerX", 0.0, 1.0, 0.01),
      ("Center Y", "centerY", 0.0, 1.0, 0.01),
      ("Radius", "radius", 0.0, 1.0, 0.01),
      ("Strength", "strength", -200.0, 200.0, 1.0)
    ]
  );

  // Channels (dropdown)
  event_handlers::setup_channels_filter( filter_renderer, current_filter );

  // Flip (dropdown)
  event_handlers::setup_flip_filter( filter_renderer, current_filter );
}

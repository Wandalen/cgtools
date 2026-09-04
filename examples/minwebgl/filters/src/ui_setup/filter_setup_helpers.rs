//! Helper functions for setting up filters with generic type parameters

use crate::
{
  utils,
  filters,
  wasm_bindgen,
  Renderer,
  controls,
};
use utils::element_by_id_unchecked_get;
use filters::{ blur, Filter, resize, brightness_contrast };
use wasm_bindgen::{ JsCast, JsValue, prelude::Closure };
use std::{ cell::RefCell, rc::Rc };
use web_sys::HtmlElement;

// Fix(UX/DX-14): dropped the `_label : &str` parameter -- it was never read by this
// function's body (the "Size" slider label is hardcoded, and the card's own visible
// name is generated independently by `filter_buttons.rs`'s own id/name table), so every
// call site was hand-duplicating a string that already exists canonically there.
// Root cause: leftover parameter from an earlier version of this helper, never wired up
// or removed after the card-label responsibility moved to `filter_buttons.rs`.
// Pitfall: an `_`-prefixed parameter silences the unused-variable lint but does not mean
// the value is actually used elsewhere -- verify against the function body, not the lint.
/// Helper for blur filters (they have generic type parameters)
pub fn blur_filter_setup< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  blur_type : T,
  max : f64
)
where
blur::Blur< T > : Filter
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let card_id_str = card_id.to_string();
  let blur_type_init = blur_type.clone();

  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    filter_renderer_clone.borrow_mut().previous_texture_restore();
    ( *current_filter_clone.borrow_mut() ).clone_from( &card_id_str );
    filter_renderer_clone.borrow_mut().previous_texture_save();

    controls::controls_clear();
    controls::slider_add( "Size", "size", 5.0, 1.0, max, 1.0 );

    let initial = blur::Blur::new( 5, blur_type_init.clone() );
    filter_renderer_clone.borrow_mut().filter_apply( &initial );

    let fr = filter_renderer_clone.clone();
    let blur_type_change = blur_type_init.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "size" ) ).unwrap();
      let size = val.as_f64().unwrap() as i32;

      let filter = blur::Blur::new( size, blur_type_change.clone() );
      fr.borrow_mut().filter_apply( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    controls::show();
  });

  let card = element_by_id_unchecked_get::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

// Fix(UX/DX-14): dropped the dead `_label : &str` parameter -- see `blur_filter_setup`'s
// comment for the shared root cause/pitfall across all 3 helpers in this file.
/// Helper for resize filters (they have generic type parameters)
pub fn resize_filter_setup< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  resize_type : T
)
where
resize::Resize< T > : Filter
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let card_id_str = card_id.to_string();
  let resize_type_init = resize_type.clone();

  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    filter_renderer_clone.borrow_mut().previous_texture_restore();
    ( *current_filter_clone.borrow_mut() ).clone_from( &card_id_str );
    filter_renderer_clone.borrow_mut().previous_texture_save();

    controls::controls_clear();
    controls::slider_add( "Scale", "scale", 1.0, 0.1, 10.0, 0.01 );

    let initial = resize::Resize::new( 1.0_f32, resize_type_init.clone() );
    filter_renderer_clone.borrow_mut().filter_apply( &initial );

    let fr = filter_renderer_clone.clone();
    let resize_type_change = resize_type_init.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "scale" ) ).unwrap();
      let scale = val.as_f64().unwrap() as f32;

      let filter = resize::Resize::new( scale, resize_type_change.clone() );
      fr.borrow_mut().filter_apply( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    controls::show();
  });

  let card = element_by_id_unchecked_get::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

/// Shared min/max/step bounds for the brightness and contrast sliders.
pub struct SliderRange
{
  pub min : f64,
  pub max : f64,
  pub step : f64
}

/// Helper for brightness/contrast filters (they have generic type parameters)
// `range` must be owned, not borrowed: it is moved into the `'static` `Closure::new` below,
// so a `&SliderRange` parameter would fail to outlive the closure (E0521).
// Fix(UX/DX-14): dropped the dead `_label : &str` parameter -- see `blur_filter_setup`'s
// comment for the shared root cause/pitfall across all 3 helpers in this file.
#[ allow( clippy::needless_pass_by_value, reason = "range must be owned to be moved into the 'static onclick closure" ) ]
pub fn brightness_contrast_filter_setup< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  bc_type : T,
  range : SliderRange
)
where
brightness_contrast::BrightnessContrast< T > : Filter
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let card_id_str = card_id.to_string();
  let bc_type_init = bc_type.clone();

  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    filter_renderer_clone.borrow_mut().previous_texture_restore();
    ( *current_filter_clone.borrow_mut() ).clone_from( &card_id_str );
    filter_renderer_clone.borrow_mut().previous_texture_save();

    controls::controls_clear();
    controls::slider_add( "Brightness", "brightness", 0.0, range.min, range.max, range.step );
    controls::slider_add( "Contrast", "contrast", 0.0, range.min, range.max, range.step );

    let initial = brightness_contrast::BrightnessContrast::new( 0.0, 0.0, bc_type_init.clone() );
    filter_renderer_clone.borrow_mut().filter_apply( &initial );

    let fr = filter_renderer_clone.clone();
    let bc_type_change = bc_type_init.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let brightness_val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "brightness" ) ).unwrap();
      let contrast_val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "contrast" ) ).unwrap();
      let brightness = brightness_val.as_f64().unwrap();
      let contrast = contrast_val.as_f64().unwrap();

      let filter = brightness_contrast::BrightnessContrast::new( brightness as f32, contrast as f32, bc_type_change.clone() );
      fr.borrow_mut().filter_apply( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    controls::show();
  });

  let card = element_by_id_unchecked_get::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

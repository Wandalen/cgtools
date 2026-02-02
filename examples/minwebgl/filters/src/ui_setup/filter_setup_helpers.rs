//! Helper functions for setting up filters with generic type parameters

use crate::*;
use utils::*;
use filters::*;
use wasm_bindgen::{ JsCast, JsValue, prelude::Closure };
use std::{ cell::RefCell, rc::Rc };
use web_sys::HtmlElement;

/// Helper for blur filters (they have generic type parameters)
pub fn setup_blur_filter< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  _label : &str,
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
    *current_filter_clone.borrow_mut() = card_id_str.clone();
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();
    controls::add_slider( "Size", "size", 5.0, 1.0, max, 1.0 );

    let initial = blur::Blur::new( 5, blur_type_init.clone() );
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let blur_type_change = blur_type_init.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "size" ) ).unwrap();
      let size = val.as_f64().unwrap() as i32;

      let filter = blur::Blur::new( size, blur_type_change.clone() );
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    controls::show();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

/// Helper for resize filters (they have generic type parameters)
pub fn setup_resize_filter< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  _label : &str,
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
    *current_filter_clone.borrow_mut() = card_id_str.clone();
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();
    controls::add_slider( "Scale", "scale", 1.0, 0.1, 10.0, 0.01 );

    let initial = resize::Resize::new( 1.0_f32, resize_type_init.clone() );
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let resize_type_change = resize_type_init.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "scale" ) ).unwrap();
      let scale = val.as_f64().unwrap() as f32;

      let filter = resize::Resize::new( scale, resize_type_change.clone() );
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    controls::show();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

/// Helper for brightness/contrast filters (they have generic type parameters)
pub fn setup_brightness_contrast_filter< T : 'static + Clone >
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >,
  card_id : &str,
  _label : &str,
  bc_type : T,
  min : f64,
  max : f64,
  step : f64
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
    *current_filter_clone.borrow_mut() = card_id_str.clone();
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();
    controls::add_slider( "Brightness", "brightness", 0.0, min, max, step );
    controls::add_slider( "Contrast", "contrast", 0.0, min, max, step );

    let initial = brightness_contrast::BrightnessContrast::new( 0.0, 0.0, bc_type_init.clone() );
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

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
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    controls::show();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( card_id );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

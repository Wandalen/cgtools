//! Special event handlers for filters with dropdown controls

use crate::*;
use utils::*;
use filters::*;
use wasm_bindgen::{ JsCast, JsValue, prelude::Closure };
use std::{ cell::RefCell, rc::Rc };
use web_sys::HtmlElement;

/// Sets up the channels filter with dropdown control
pub fn setup_channels_filter
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >
)
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    *current_filter_clone.borrow_mut() = String::from( "channels" );
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();

    let options = web_sys::js_sys::Array::of3( &"Red".into(), &"Green".into(), &"Blue".into() );
    controls::add_dropdown( "Channel", "channel", "Red", &options.into() );

    let initial = channels::Channels { channel: channels::Channel::Red };
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      // Get the channel value from the values object
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "channel" ) ).unwrap();
      let channel_str = val.as_string().unwrap();

      // Parse string to enum
      let channel = match channel_str.as_str()
      {
        "Red" => channels::Channel::Red,
        "Green" => channels::Channel::Green,
        "Blue" => channels::Channel::Blue,
        _ => channels::Channel::Red,
      };

      let filter = channels::Channels { channel };
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    controls::show();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( "channels" );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

/// Sets up the flip filter with dropdown control
pub fn setup_flip_filter
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >
)
{
  let filter_renderer_clone = filter_renderer.clone();
  let current_filter_clone = current_filter.clone();
  let onclick : Closure< dyn Fn() > = Closure::new( move ||
  {
    *current_filter_clone.borrow_mut() = String::from( "flip" );
    filter_renderer_clone.borrow_mut().save_previous_texture();

    controls::clear_controls();

    let options = web_sys::js_sys::Array::of3( &"FlipX".into(), &"FlipY".into(), &"FlipXY".into() );
    controls::add_dropdown( "Direction", "flip", "FlipX", &options.into() );

    let initial = flip::Flip { flip: flip::FlipDirection::FlipX };
    filter_renderer_clone.borrow_mut().apply_filter( &initial );

    let fr = filter_renderer_clone.clone();
    let callback : Closure< dyn Fn( JsValue ) > = Closure::new( move | values : JsValue |
    {
      // Get the flip value from the values object
      let obj = values.dyn_into::< web_sys::js_sys::Object >().unwrap();
      let val = web_sys::js_sys::Reflect::get( &obj, &JsValue::from_str( "flip" ) ).unwrap();
      let flip_str = val.as_string().unwrap();

      // Parse string to enum
      let flip = match flip_str.as_str()
      {
        "FlipX" => flip::FlipDirection::FlipX,
        "FlipY" => flip::FlipDirection::FlipY,
        "FlipXY" => flip::FlipDirection::FlipXY,
        _ => flip::FlipDirection::FlipX,
      };

      let filter = flip::Flip { flip };
      fr.borrow_mut().apply_filter( &filter );
    });
    controls::on_change( callback.as_ref().unchecked_ref() );
    callback.forget();

    controls::show();
  });

  let card = get_element_by_id_unchecked::< HtmlElement >( "flip" );
  card.add_event_listener_with_callback( "click", onclick.as_ref().unchecked_ref() ).unwrap();
  onclick.forget();
}

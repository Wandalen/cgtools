//! Setup for filters that don't require UI controls

use crate::*;
use utils::*;
use filters::*;
use wasm_bindgen::{ JsCast, prelude::Closure };
use std::{ cell::RefCell, rc::Rc };
use web_sys::HtmlElement;

/// Sets up filters that don't have parameters
pub fn setup_filters_without_controls
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >
)
{
  let filters =
  [
    ( "desaturate",  make_closure_with_filter_tracking( filter_renderer, desaturate::Desaturate, "desaturate", current_filter ) ),
    ( "edge",        make_closure_with_filter_tracking( filter_renderer, edge::Edge, "edge", current_filter ) ),
    ( "emboss",      make_closure_with_filter_tracking( filter_renderer, emboss::Emboss, "emboss", current_filter ) ),
    ( "enrich",      make_closure_with_filter_tracking( filter_renderer, enrich::Enrich, "enrich", current_filter ) ),
    ( "grayscale",   make_closure_with_filter_tracking( filter_renderer, gray_scale::GrayScale, "grayscale", current_filter ) ),
    ( "invert",      make_closure_with_filter_tracking( filter_renderer, invert::Invert, "invert", current_filter ) ),
    ( "sepia",       make_closure_with_filter_tracking( filter_renderer, sepia::Sepia, "sepia", current_filter ) ),
    ( "solarize",    make_closure_with_filter_tracking( filter_renderer, solarize::Solarize, "solarize", current_filter ) ),
    ( "transpose",   make_closure_with_filter_tracking( filter_renderer, transpose::Transpose, "transpose", current_filter ) ),
  ];

  for ( card_id, closure ) in filters
  {
    let card = get_element_by_id_unchecked::< HtmlElement >( card_id );
    card.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
}

/// Creates a closure that applies a filter and tracks the current filter
pub fn make_closure_with_filter_tracking
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  filter : impl Filter + 'static,
  filter_name : &str,
  current_filter : &Rc< RefCell< String > >
)
-> Closure< dyn Fn() >
{
  let filter_renderer = filter_renderer.clone();
  let current_filter = current_filter.clone();
  let filter_name = filter_name.to_string();
  Closure::new( Box::new( move ||
  {
    *current_filter.borrow_mut() = filter_name.clone();
    filter_renderer.borrow_mut().save_previous_texture();
    controls::clear_controls();
    filter_renderer.borrow_mut().apply_filter( &filter );
    controls::show();
  }))
}

//! Setup for filters that don't require UI controls

use crate::
{
  utils,
  filters,
  wasm_bindgen,
  Renderer,
  controls,
};
use utils::element_by_id_unchecked_get;
use filters::
{
  desaturate,
  edge,
  emboss,
  enrich,
  gray_scale,
  invert,
  sepia,
  solarize,
  transpose,
  Filter,
};
use wasm_bindgen::{ JsCast, prelude::Closure };
use std::{ cell::RefCell, rc::Rc };
use web_sys::HtmlElement;

/// Sets up filters that don't have parameters
pub fn filters_without_controls_setup
(
  filter_renderer : &Rc< RefCell< Renderer > >,
  current_filter : &Rc< RefCell< String > >
)
{
  let filters =
  [
    ( "desaturate",  closure_with_filter_tracking_make( filter_renderer, desaturate::Desaturate, "desaturate", current_filter ) ),
    ( "edge",        closure_with_filter_tracking_make( filter_renderer, edge::Edge, "edge", current_filter ) ),
    ( "emboss",      closure_with_filter_tracking_make( filter_renderer, emboss::Emboss, "emboss", current_filter ) ),
    ( "enrich",      closure_with_filter_tracking_make( filter_renderer, enrich::Enrich, "enrich", current_filter ) ),
    ( "grayscale",   closure_with_filter_tracking_make( filter_renderer, gray_scale::GrayScale, "grayscale", current_filter ) ),
    ( "invert",      closure_with_filter_tracking_make( filter_renderer, invert::Invert, "invert", current_filter ) ),
    ( "sepia",       closure_with_filter_tracking_make( filter_renderer, sepia::Sepia, "sepia", current_filter ) ),
    ( "solarize",    closure_with_filter_tracking_make( filter_renderer, solarize::Solarize, "solarize", current_filter ) ),
    ( "transpose",   closure_with_filter_tracking_make( filter_renderer, transpose::Transpose, "transpose", current_filter ) ),
  ];

  for ( card_id, closure ) in filters
  {
    let card = element_by_id_unchecked_get::< HtmlElement >( card_id );
    card.add_event_listener_with_callback( "click", closure.as_ref().unchecked_ref() ).unwrap();
    closure.forget();
  }
}

/// Creates a closure that applies a filter and tracks the current filter
pub fn closure_with_filter_tracking_make
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
    // Skip if this filter is already active
    if *current_filter.borrow() == filter_name
    {
      return;
    }
    // Restore previous state if switching from another unapplied filter
    filter_renderer.borrow_mut().previous_texture_restore();
    ( *current_filter.borrow_mut() ).clone_from( &filter_name );
    filter_renderer.borrow_mut().previous_texture_save();
    controls::controls_clear();
    filter_renderer.borrow_mut().filter_apply( &filter );
    controls::show();
  }))
}

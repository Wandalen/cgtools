//! UI setup for filter controls
//!
//! This module coordinates the setup of the filter UI by delegating to specialized submodules.
//! The implementation has been modularized for better maintainability:
//! - Button generation in `ui_setup::filter_buttons`
//! - Simple filter setup in `ui_setup::filter_setup_simple`
//! - Helper functions in `ui_setup::filter_setup_helpers`
//! - Advanced filter setup in `ui_setup::filter_setup_advanced`
//! - Event handlers in `ui_setup::event_handlers`

#![ allow( clippy::if_not_else ) ]

mod filter_buttons;
mod filter_setup_simple;
mod filter_setup_helpers;
mod filter_setup_advanced;
mod event_handlers;

use crate::*;
use std::{ cell::RefCell, rc::Rc };

/// Hides the controls bar and clears all controls
pub fn hide_controls_bar()
{
  controls::hide();
  controls::clear_controls();
}

/// Sets up the complete UI for filters
///
/// This function coordinates:
/// 1. Generation of filter buttons in the grid
/// 2. Setup of filters without UI controls
/// 3. Setup of filters with UI controls (sliders, dropdowns)
///
/// Returns a reference to the current filter name
pub fn setup_ui( filter_renderer : &Rc< RefCell< Renderer > > ) -> Rc< RefCell< String > >
{
  let current_filter = Rc::new( RefCell::new( String::from( "none" ) ) );

  // Generate filter buttons in the UI grid
  filter_buttons::generate_filter_buttons();

  // Setup filters that don't require controls
  filter_setup_simple::setup_filters_without_controls( filter_renderer, &current_filter );

  // Setup filters that require controls (sliders, dropdowns, etc.)
  filter_setup_advanced::setup_filters_with_controls( filter_renderer, &current_filter );

  current_filter
}

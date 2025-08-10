//! Backend adapter implementations.
//!
//! This module contains all backend adapter implementations following the
//! Ports & Adapters architecture pattern. Each adapter implements the core
//! rendering traits to provide output to specific backends.

mod_interface::mod_interface!
{
  /// SVG backend adapter for static vector graphics output.
  #[ cfg( any( feature = "adapter-svg-basic", feature = "adapter-svg" ) ) ]
  exposed use super::private::svg::SvgRenderer;

  /// Terminal backend adapter for ASCII art output.
  #[ cfg( any( feature = "adapter-terminal-basic", feature = "adapter-terminal" ) ) ]
  exposed use super::private::terminal::TerminalRenderer;
}

mod private
{
  #[ cfg( any( feature = "adapter-svg-basic", feature = "adapter-svg" ) ) ]
  pub mod svg;

  #[ cfg( any( feature = "adapter-terminal-basic", feature = "adapter-terminal" ) ) ]
  pub mod terminal;
}
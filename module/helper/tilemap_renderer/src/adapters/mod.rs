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

  /// Interactive SVG-in-browser backend adapter with JavaScript interactivity.
  #[ cfg( any( feature = "adapter-svg-browser", feature = "adapter-svg" ) ) ]
  exposed use super::private::svg_browser::SvgBrowserRenderer;

  /// Terminal backend adapter for ASCII art output.
  #[ cfg( any( feature = "adapter-terminal-basic", feature = "adapter-terminal" ) ) ]
  exposed use super::private::terminal::TerminalRenderer;

  /// WebGL backend adapter for hardware-accelerated web rendering.
  #[ cfg( any( feature = "adapter-webgl-context", feature = "adapter-webgl" ) ) ]
  exposed use super::private::webgl::WebGLRenderer;

  /// WebGPU backend adapter for next-generation GPU computing and rendering.
  #[ cfg( any( feature = "adapter-webgpu-device", feature = "adapter-webgpu" ) ) ]
  exposed use super::private::webgpu::WebGPURenderer;
}

mod private
{
  #[ cfg( any( feature = "adapter-svg-basic", feature = "adapter-svg" ) ) ]
  pub mod svg;

  #[ cfg( any( feature = "adapter-svg-browser", feature = "adapter-svg" ) ) ]
  pub mod svg_browser;

  #[ cfg( any( feature = "adapter-terminal-basic", feature = "adapter-terminal" ) ) ]
  pub mod terminal;

  #[ cfg( any( feature = "adapter-webgl-context", feature = "adapter-webgl" ) ) ]
  pub mod webgl;

  #[ cfg( any( feature = "adapter-webgpu-device", feature = "adapter-webgpu" ) ) ]
  pub mod webgpu;
}
//! # `browser_log::log`
//!
//! Logging utility for Rust applications compiled to WebAssembly (`wasm32-unknown-unknown`). It integrates with JavaScript's `console` API to output log messages with varying levels of severity, enhancing the visibility and management of log data in web environments.
//!
//! ## Features
//!
//! - **Configurable Logging Levels**: Supports multiple log levels (Trace, Debug, Info, Warn, Error) with customizable styles.
//! - **Target Filtering**: Allows filtering logs based on module paths, enabling focused logging for specific parts of your application.
//!
//! ## Usage
//!
//! ### Setup Logger
//!
//! Initialize the logger with a configuration. If initialization fails, an error message is logged to the browser console.
//!
//! ``` no_run
//! // Default setup
//! browser_log::log::setup::setup( Default::default() );
//!
//! // Setup with target filtering
//! browser_log::log::setup::setup( browser_log::log::setup::Config::default().target_filter( "lib_name" ) );
//! ```
//!
//! ## Configuration
//!
//! - **Config**: Use the `Config` struct to specify the maximum log level and optional target filtering.
//! - **Predefined**: Log messages are styled with CSS for better readability in the console.
//!
//! This module leverages the `web_sys` crate to interact with the browser's console, ensuring that log messages are displayed with appropriate styling and context.

/// Internal namespace.
mod private
{
}

crate::mod_interface!
{

  layer debug_log;
  layer setup;

  exposed use ::web_sys::console;
  orphan use ::log::*;

}

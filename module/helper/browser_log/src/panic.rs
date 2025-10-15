//! # `browser_log::panic`
//!
//! Debugging utility for Rust applications compiled to WebAssembly (`wasm32-unknown-unknown`). It redirects panic messages to JavaScript's `console.error`, enhancing error visibility in web browsers and Node.js environments.
//!
//! ## Manual Setup
//!
//! Set the panic hook manually in your initialization code:
//!
//! ```rust
//! use std::panic;
//!
//! fn setup()
//! {
//!   let config = browser_log::panic::Config::default();
//!   std::panic::set_hook( Box::new( move | info | browser_log::panic::hook( info, &config ) ) );
//!   // Your code...
//! }
//! ```
//!
//! ## Automatic Setup
//!
//! Use the `setup` function to ensure the hook is set once, leveraging Rust's `std::sync::Once` for thread safety:
//!
//! ```rust
//!
//! struct MyApp;
//!
//! impl MyApp
//! {
//!   pub fn new() -> Self
//!   {
//!     browser_log::panic::setup( Default::default() );
//!     Self
//!   }
//! }
//! ```
//!
//! ## Advanced Configuration
//!
//! ### Increasing Stack Trace Depth
//! By default, browsers limit stack traces to 10 frames. To capture more frames, adjust the `Error.stackTraceLimit` property. Refer to the [MDN Web Docs](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Microsoft_Extensions/Error.stackTraceLimit) for more information.

/// Internal namespace.
mod private
{
  // use crate::*;

  use std::panic;

  /// A panic hook designed for use with
  /// [`std::panic::set_hook`](https://doc.rust-lang.org/nightly/std/panic/fn.set_hook.html).
  /// It logs panic messages to `console.error` in WebAssembly environments.
  /// For non-WASM targets, it outputs the panic to standard error.
  pub fn hook( info : &panic::PanicHookInfo< '_ >, config : &Config )
  {
    hook_impl( info, config );
  }

  /// Configures the panic hook to use `console.error` for logging. This function
  /// ensures the hook is set only once, regardless of how many times it is called.
  pub fn setup( config : Config )
  {
    use std::sync::Once;
    static INIT_HOOK : Once = Once::new();
    INIT_HOOK.call_once( ||
    {
      panic::set_hook( Box::new( move | info | hook( info, &config ) ) );
    });
  }

  /// Specify how to handle panic.
  #[ derive( Debug ) ]
  pub struct Config
  {
    // qqq : cover by test
    /// Print location.
    pub with_location : bool,
    // qqq : cover by test
    /// Print stack trace.
    pub with_stack_trace : bool,
  }

  impl Default for Config
  {
    fn default() -> Self
    {
      Self
      {
        with_location : true,
        with_stack_trace : true,
      }
    }
  }

  #[ cfg( target_arch = "wasm32" ) ]
  mod imp
  {
    use super::Config;
    use std::panic;

    // extern crate wasm_bindgen;
    use wasm_bindgen::prelude::*;

    #[ wasm_bindgen ]
    extern
    {
      type Error;

      #[wasm_bindgen( js_namespace = console )]
      fn error( msg : String );

      #[wasm_bindgen( constructor )]
      fn new() -> Error;

      #[wasm_bindgen( structural, method, getter )]
      fn stack( error : &Error ) -> String;
    }

    pub fn hook_impl( info : &panic::PanicHookInfo< '_ >, config : &Config )
    {
      use std::fmt::Write;

      let mut message = "=== Error\n\n".to_string();

      message.push_str( &info.to_string() );

      if config.with_location
      {
        let location = info.location();
        if let Some( location ) = location
        {
          // message.push_str( "\n\n = Location:\n\n {}:{}", location.file(), location.line() );
          let _ = write!( message, "\n\n = Location:\n\n {}:{}", location.file(), location.line() );
        }
      }

      if config.with_stack_trace
      {
        // Add the error stack to the message to ensure it is visible.
        message.push_str( "\n\n = Stack Trace:\n\n" );
        let error_instance = Error::new();
        let stack_trace = error_instance.stack();
        message.push_str( &stack_trace );
        message.push_str( "\n\n" );
      }

      // Log the complete panic message using `console.error`.
      error( message );
    }
  }

  #[ cfg( not( target_arch = "wasm32" ) ) ]
  mod imp
  {
    use super::Config;
    use std::io::{ self, Write };

    pub fn hook_impl( info : &std::panic::PanicHookInfo< '_ >, _config : &Config )
    {
      let _ = writeln!( io::stderr(), "{info}" );
    }
  }

  pub use imp::*;

}

crate::mod_interface!
{

  own use
  {
    Config,
    hook,
    setup,
  };

}

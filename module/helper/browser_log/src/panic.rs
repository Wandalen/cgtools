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
  #[ inline ]
  pub fn hook( info : &panic::PanicHookInfo< '_ >, config : &Config )
  {
    hook_impl( info, config );
  }

  /// Configures the panic hook to use `console.error` for logging. This function
  /// ensures the hook is set only once, regardless of how many times it is called.
  #[ inline ]
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
  ///
  /// The two flags gate message sections on the wasm32 target only, where the
  /// hook assembles the `console.error` payload; the native fallback prints the
  /// panic info as-is and ignores them. Defaults and field contract are pinned
  /// by `tests/panic_hook_test.rs`.
  // Both fields are plain flags with no invariant between them — direct struct-literal
  // construction is the deliberate public contract (pinned by `tests/panic_hook_test.rs`'s
  // `config_fields_construct_independently`), so `#[non_exhaustive]` would break that contract.
  #[ derive( Debug ) ]
  pub struct Config
  {
    /// Print location.
    pub with_location : bool,
    /// Print stack trace.
    pub with_stack_trace : bool,
  }

  impl Default for Config
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        with_location : true,
        with_stack_trace : true,
      }
    }
  }

  // Fix(BUG-168): `hook_impl` used to build the message body from `info.to_string()`
  // unconditionally, then only ever *append* a second, redundant location block when
  // `with_location` was true -- `with_location : false` never suppressed anything, since the
  // location was already embedded by `to_string()` before the flag was even checked.
  // Root cause: `PanicHookInfo`'s `Display` impl unconditionally writes
  // `"panicked at {file}:{line}:{col}:\n{message}"` -- there is no `Display` mode that omits
  // the location, so gating on `with_location` requires bypassing `Display` entirely and
  // reading the payload directly.
  // Pitfall: a boolean config flag whose doc says "Print location" must be checked *before*
  // the very first point the location could enter the output, not only at the point a second,
  // additive block is appended -- a type's `Display` impl is not a neutral, location-free
  // starting point to build on top of just because the visible code only references it once.
  /// Builds the panic message body, honoring `with_location`.
  ///
  /// Split out of the wasm-only hook implementation so this is unit-testable on native
  /// targets, matching this crate's `tests/panic_hook_test.rs` testing convention (the rest of
  /// `hook_impl` stays wasm-only because it also touches `console.error`/`Error.stack`).
  #[ doc( hidden ) ]
  #[ must_use ]
  pub fn panic_message( info : &panic::PanicHookInfo< '_ >, with_location : bool ) -> String
  {
    if with_location
    {
      return info.to_string();
    }
    if let Some( payload ) = info.payload().downcast_ref::< &str >()
    {
      ( *payload ).to_string()
    }
    else if let Some( payload ) = info.payload().downcast_ref::< String >()
    {
      payload.clone()
    }
    else
    {
      "<non-string panic payload>".to_string()
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

      message.push_str( &super::panic_message( info, config.with_location ) );

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
    panic_message,
  };

}

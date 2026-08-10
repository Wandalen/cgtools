//! Native coverage for `browser_log::panic` — the `Config` field contract and
//! the native hook path, exercised by a REAL panic ( no mocks ).
//!
//! Placement decision ( task 077 ) : the behavior the `Config` flags gate —
//! appending the `= Location:` and `= Stack Trace:` sections — lives inside the
//! `#[ cfg( target_arch = "wasm32" ) ]` implementation, interleaved with the JS
//! `console.error` / `Error.stack` bindings, so it is browser-only and cannot
//! be observed natively without faking a console. What IS natively observable —
//! the field defaults, independent field construction, and the native fallback
//! running to completion on a genuine panic — is pinned here. This test lives
//! in its own file on purpose : the panic-hook swap is process-global, and a
//! separate integration-test binary cannot race `basic_test.rs`'s own hook
//! installs.

use browser_log::panic::Config;

#[ test ]
fn config_default_enables_location_and_stack_trace()
{
  let config = Config::default();
  assert!( config.with_location, "with_location must default to true" );
  assert!( config.with_stack_trace, "with_stack_trace must default to true" );
}

#[ test ]
fn config_fields_construct_independently()
{
  let config = Config { with_location : false, with_stack_trace : true };
  assert!( !config.with_location );
  assert!( config.with_stack_trace );

  let config = Config { with_location : true, with_stack_trace : false };
  assert!( config.with_location );
  assert!( !config.with_stack_trace );
}

#[ test ]
fn native_hook_runs_on_real_panic()
{
  use std::panic;
  use std::sync::atomic::{ AtomicBool, Ordering };

  static HOOK_RAN : AtomicBool = AtomicBool::new( false );

  let previous = panic::take_hook();
  let config = Config::default();
  panic::set_hook( Box::new( move | info |
  {
    browser_log::panic::hook( info, &config );
    HOOK_RAN.store( true, Ordering::SeqCst );
  }));

  let result = panic::catch_unwind( || panic!( "deliberate panic to exercise the native hook" ) );

  drop( panic::take_hook() );
  panic::set_hook( previous );

  assert!( result.is_err(), "the panic must unwind into catch_unwind" );
  assert!( HOOK_RAN.load( Ordering::SeqCst ), "the installed hook must have run during the panic" );
}

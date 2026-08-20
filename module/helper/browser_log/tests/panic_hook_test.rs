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

use browser_log::panic::{ Config, panic_message };

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

/// Serializes every test in this file that swaps the process-global panic hook.
/// `cargo test` runs tests in this file concurrently on separate threads by default; without
/// this lock, two tests taking/setting/restoring `std::panic`'s single global hook at the same
/// time can steal each other's hook (test A's panic firing test B's hook), corrupting both
/// results. Held only around the hook-swap + trigger + restore sequence in
/// [`with_panic_hook_locked`], never across a test's own `assert!` calls, so a failing
/// assertion can never poison it.
static PANIC_HOOK_LOCK : std::sync::Mutex< () > = std::sync::Mutex::new( () );

/// Installs a hook that runs `probe` on the real `PanicHookInfo` delivered by `trigger`,
/// captures `probe`'s return value, restores the previous hook, and returns the captured value
/// -- shared by every hook-touching test in this file so each stays a single real
/// `catch_unwind` against a real panic, no fabricated `PanicHookInfo`, and all serialize
/// through [`PANIC_HOOK_LOCK`].
fn with_panic_hook_locked< R >(
  trigger : impl FnOnce() + std::panic::UnwindSafe,
  probe : impl Fn( &std::panic::PanicHookInfo< '_ > ) -> R + Send + Sync + 'static,
) -> R
where
  R : Send + 'static,
{
  use std::panic;
  use std::sync::Mutex;

  static CAPTURED : Mutex< Option< Box< dyn std::any::Any + Send > > > = Mutex::new( None );

  let _guard = PANIC_HOOK_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );

  let previous = panic::take_hook();
  panic::set_hook( Box::new( move | info |
  {
    let result = probe( info );
    *CAPTURED.lock().unwrap() = Some( Box::new( result ) );
  }));

  let _ = panic::catch_unwind( trigger );

  drop( panic::take_hook() );
  panic::set_hook( previous );

  let boxed = CAPTURED.lock().unwrap().take().expect( "hook must have run and stashed a result" );
  *boxed.downcast::< R >().expect( "stashed result must be of type R" )
}

/// `bug_reproducer(BUG-168)`
///
/// Root cause: `panic_message` used to delegate straight to `PanicHookInfo`'s `Display` impl
/// regardless of `with_location`, and `Display` unconditionally embeds `"panicked at
/// {file}:{line}:{col}:"` ahead of the message -- so `with_location : false` never actually
/// suppressed the location, it only skipped a second, redundant "= Location:" block appended
/// on top of the same, already-present location.
///
/// This pins the fix: with `with_location : false`, the returned string must contain the real
/// panic message but must never contain a source-location marker (`.rs:` or this file's own
/// name), proving the location was never embedded in the first place -- not merely hidden by a
/// second suppressed block.
#[ test ]
fn panic_message_with_location_false_omits_file_and_line()
{
  let message = with_panic_hook_locked(
    || panic!( "bug 168 regression message" ),
    | info | panic_message( info, false ),
  );

  assert!( message.contains( "bug 168 regression message" ), "the real panic message must still be present: {message:?}" );
  assert!( !message.contains( ".rs:" ), "no source-location marker may appear when with_location is false: {message:?}" );
  assert!( !message.contains( "panic_hook_test" ), "the test file's own name must not leak into a location-suppressed message: {message:?}" );
}

/// Control case for [`panic_message_with_location_false_omits_file_and_line`]: with
/// `with_location : true`, the pre-existing `Display`-based behavior is preserved byte-for-byte
/// -- the location must still be present, proving the fix only changed the `false` branch.
#[ test ]
fn panic_message_with_location_true_includes_file_and_line()
{
  let message = with_panic_hook_locked(
    || panic!( "bug 168 control message" ),
    | info | panic_message( info, true ),
  );

  assert!( message.contains( "bug 168 control message" ), "the real panic message must be present: {message:?}" );
  assert!( message.contains( ".rs:" ), "with_location true must still include a source-location marker: {message:?}" );
}

/// Covers `panic_message`'s non-string-payload fallback (reached via [`std::panic::panic_any`]
/// with a payload that is neither `&str` nor `String`), which the two tests above cannot
/// exercise since both panic through the `panic!` macro's string-payload path.
#[ test ]
fn panic_message_with_location_false_handles_non_string_payload()
{
  let message = with_panic_hook_locked(
    || std::panic::panic_any( 42_i32 ),
    | info | panic_message( info, false ),
  );

  assert!( !message.contains( ".rs:" ), "no source-location marker may appear when with_location is false: {message:?}" );
  assert!( !message.is_empty(), "a non-string payload must still produce a non-empty fallback message: {message:?}" );
}

#[ test ]
fn native_hook_runs_on_real_panic()
{
  let config = Config::default();
  let ran = with_panic_hook_locked(
    || panic!( "deliberate panic to exercise the native hook" ),
    move | info |
    {
      browser_log::panic::hook( info, &config );
      true
    },
  );

  assert!( ran, "the installed hook must have run during the panic" );
}

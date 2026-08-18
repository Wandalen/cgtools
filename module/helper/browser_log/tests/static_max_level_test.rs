//! Pins BUG-354: the `log` dependency's `release_max_level_info` Cargo feature (present in
//! `Cargo.toml` pre-fix, removed by this bug's fix) caps `log::STATIC_MAX_LEVEL` at `Info` at
//! COMPILE TIME in every build with `cfg(not(debug_assertions))` (release profile by default)
//! -- for the ENTIRE dependency graph via Cargo feature unification, not just browser_log's own
//! internal logging. `log::debug!`/`log::trace!` check their level against `STATIC_MAX_LEVEL`
//! (see `log-0.4.33/src/macros.rs:137`, `lvl <= $crate::STATIC_MAX_LEVEL && lvl <=
//! $crate::max_level()`) BEFORE the runtime level set by `log::set_max_level` is even
//! consulted -- so with the cap in place, a call site's `log::debug!()` never reaches any
//! installed `Log` implementation, no matter what `browser_log::log::setup::Config`/`Level`
//! requests at runtime. This silently contradicts the crate's own documented contract
//! (`Config::new`'s doc comment, readme.md's "Configurable log levels for deployment").
//!
//! `STATIC_MAX_LEVEL`'s value is selected by a `match cfg!(debug_assertions) { false if
//! cfg!(feature = "release_max_level_info") => LevelFilter::Info, ... }` (`log-0.4.33/src/
//! lib.rs:1637-1650`), so this file's tests only actually exercise the bug when compiled with
//! `debug_assertions` OFF. `cargo test`'s default `test` profile inherits `dev` profile
//! settings (`debug_assertions = true`) -- this workspace's `Cargo.toml` carries no
//! `[profile.*]` override, confirmed by reading it -- so a PLAIN `cargo test -p browser_log`
//! cannot observe the release-only symptom no matter which state `Cargo.toml`'s `log` feature
//! list is in.
//!
//! **To actually witness fail -> pass, run**: `cargo test -p browser_log --release
//! static_max_level -- --nocapture`. Pre-fix (with `release_max_level_info` present) that
//! command's `debug_records_reach_the_logger_at_current_build_profile` test fails (0 records
//! captured) and `static_max_level_is_not_capped_in_release_profile` fails
//! (`STATIC_MAX_LEVEL` is `Info`, not `Trace`); post-fix both pass. The always-on
//! `debug_records_reach_the_logger_at_current_build_profile` test also runs under plain
//! `cargo test` (dev profile) in every fix state -- it is expected to pass there regardless,
//! since `release_max_level_info`'s own `cfg` gate never applies under `debug_assertions =
//! true`; that branch is ordinary coverage of the runtime path, not a BUG-354 witness. The
//! `cfg(not(debug_assertions))`-gated `static_max_level_is_not_capped_in_release_profile` test
//! is compiled OUT entirely under plain `cargo test` (0 tests reported from it) -- this is
//! expected and not a false pass; see the release-mode command above for the run that actually
//! exercises it.
//!
//! Lives in its own file for the same reason as `debug_log_test.rs`/`panic_hook_test.rs`:
//! `log::set_logger` is process-global and callable once per process, so this installs its own
//! captor without racing the other two logger-installing test files.

// BUG-354 task/bug/354_browser_log_release_max_level_info_silently_caps_logging.md — reproduces
// the compile-time STATIC_MAX_LEVEL cap this bug's fix removes from Cargo.toml.

use log::{ Level, Log, Metadata, Record };
use std::sync::Mutex;

/// Captures every record it receives -- installed once, process-global, so this file's single
/// `#[ test ]` fn is the only test in this binary that may call `log::set_logger`.
struct CapturingLogger;

static CAPTURED : Mutex< Vec< ( Level, String ) > > = Mutex::new( Vec::new() );

impl Log for CapturingLogger
{
  fn enabled( &self, _metadata : &Metadata< '_ > ) -> bool
  {
    true
  }

  fn log( &self, record : &Record< '_ > )
  {
    CAPTURED.lock().unwrap().push( ( record.level(), record.args().to_string() ) );
  }

  fn flush( &self ) {}
}

static LOGGER : CapturingLogger = CapturingLogger;

/// `bug_reproducer(BUG-354)`
///
/// Root cause: `Cargo.toml`'s `log` dependency previously enabled the `release_max_level_info`
/// feature, which sets `log::STATIC_MAX_LEVEL = LevelFilter::Info` at COMPILE TIME whenever
/// `cfg(not(debug_assertions))` holds -- for the ENTIRE dependency graph via Cargo feature
/// unification, not just browser_log's own internal logging. `log::debug!`/`log::trace!` check
/// `lvl <= STATIC_MAX_LEVEL` before the runtime level ever matters, so with the cap in place a
/// `log::debug!()` call compiles to a permanently-false branch and NEVER reaches any installed
/// `Log` implementation, regardless of what `log::set_max_level` requested at runtime.
///
/// This exercises the exact consumer-facing path the crate's readme documents -- install a
/// logger, request a Debug-or-lower level via the crate's own runtime contract
/// (`log::set_max_level`, the mechanism `browser_log::log::setup::setup` itself calls), then
/// call `log::debug!()` -- and asserts the message actually reached the logger, proving the
/// RUNTIME level request is honored rather than silently discarded by a COMPILE-TIME cap.
///
/// Only genuinely distinguishes pre-fix from post-fix under `cfg(not(debug_assertions))`
/// (release profile) -- see this file's module doc for the exact command and for why the
/// `debug_assertions = true` branch below passes in every fix state.
#[ test ]
fn debug_records_reach_the_logger_at_current_build_profile()
{
  log::set_logger( &LOGGER ).expect( "set_logger must succeed -- the only test in this binary installing a logger" );
  log::set_max_level( log::LevelFilter::Trace );

  log::debug!( "BUG-354 static_max_level_test probe" );

  let captured = CAPTURED.lock().unwrap();

  if cfg!( debug_assertions )
  {
    // dev/test profile: release_max_level_info's cfg gate never applies here, so the runtime
    // Trace level requested above must be honored regardless of Cargo.toml's feature list.
    assert_eq!(
      captured.len(), 1,
      "log::debug!() must reach the logger under debug_assertions=true regardless of \
       BUG-354's fix state -- got {captured:?}"
    );
  }
  else
  {
    // release profile: this is the one branch that actually distinguishes pre-fix from
    // post-fix. Pre-fix (release_max_level_info present): STATIC_MAX_LEVEL == Info, so
    // log::debug!() compiles to a permanently-false branch and captured stays empty. Post-fix:
    // STATIC_MAX_LEVEL == Trace, so the call reaches CapturingLogger exactly like the
    // debug_assertions branch above.
    assert_eq!(
      captured.len(), 1,
      "BUG-354: log::debug!() did not reach the logger under a release-profile build \
       (debug_assertions=false) -- log::STATIC_MAX_LEVEL is {:?} (expected Trace). This is \
       exactly BUG-354: Cargo.toml's `log` dependency features cap the compile-time level \
       below what browser_log's own runtime Config/Level contract promises. captured={captured:?}",
      log::STATIC_MAX_LEVEL
    );
  }
}

/// Companion, always-on assertion on the raw compile-time constant itself (no logger
/// involved) -- `cfg(not(debug_assertions))`-gated so it exists ONLY in release-profile
/// builds, where `release_max_level_info`'s own `cfg` gate can actually apply. Under plain
/// `cargo test` this function is compiled out entirely (0 tests from it is the correct,
/// expected result, not a false pass) -- see this file's module doc for the release-mode
/// command that actually runs it.
#[ cfg( not( debug_assertions ) ) ]
#[ test ]
fn static_max_level_is_not_capped_in_release_profile()
{
  assert_eq!(
    log::STATIC_MAX_LEVEL,
    log::LevelFilter::Trace,
    "BUG-354: release-profile log::STATIC_MAX_LEVEL must be Trace (uncapped) -- browser_log's \
     only documented level control is the runtime Config/Level mechanism in \
     log::setup::setup, not a compile-time Cargo feature cap. Found {:?}.",
    log::STATIC_MAX_LEVEL
  );
}

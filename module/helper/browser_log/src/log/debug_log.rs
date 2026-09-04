//! This module introduces the `DebugLog` trait, which provides convenient shortcut methods
//! for logging the debug representation of any type that implements `fmt::Debug`.
//! It integrates with the `log` crate to offer logging at various levels.

/// Internal namespace for implementation details.
mod private
{
  // use crate::*;
  use ::log::{ Level, Record };
  use core::fmt;

  /// A trait that provides convenience methods for logging the debug output of a struct.
  ///
  /// This trait is automatically implemented for any type that implements `fmt::Debug`,
  /// allowing for quick and easy logging without boilerplate code.
  pub trait DebugLog : fmt::Debug
  {
    // Fix(BUG-167): the 4 convenience methods used to call `log::trace!`/`log::info!`/etc.
    // directly, and `debug_log` called `log::log!` directly -- every one of these macros
    // expands `file!()`/`line!()` at ITS OWN call site (here, inside this trait's default
    // body), so every logged record always reported `debug_log.rs`'s own location, never the
    // real external caller's, regardless of `#[inline]`.
    // Root cause: `file!()`/`line!()` are lexical (resolved where the macro is written), not
    // dynamic -- the only mechanism that captures the true runtime caller is `#[track_caller]`
    // + `Location::caller()`, which requires bypassing the `log!` convenience macros entirely
    // (they give no way to inject a caller-supplied file/line) and building the `Record`
    // manually instead.
    // Pitfall: `#[inline]` affects codegen, not macro hygiene -- it has no effect on where
    // `file!()`/`line!()` resolve, so it can never fix a lexical-location bug no matter how
    // deep the inlining goes.
    // Fix(BUG-229): the BUG-167 fix left `module_path!()` itself unaddressed -- it's lexical
    // exactly like `file!()`/`line!()`, but unlike them has no `#[track_caller]`-equivalent in
    // stable Rust, so every call from any external crate still tagged its `Record` with this
    // trait's OWN module path (`browser_log::log::debug_log::private`), silently defeating
    // `Config::target_filter` (which matches `metadata.target().starts_with(prefix)`) for every
    // consumer that set one -- exactly the crate's own advertised, documented use case.
    // Root cause: no stable API captures a macro's true dynamic call-site module path (only
    // `file!()`/`line!()` gained a caller-tracking mechanism, via `Location`).
    // Pitfall: when a lexical macro's value cannot be recovered dynamically, the only correct
    // fix is to require the caller to supply it explicitly (mirroring `log`'s own
    // `log!(target: "...", ...)` escape hatch) -- there is no way to make the trait's default
    // method body "just work" without changing its signature.
    /// Logs the debug representation of the object at a specified log level, reporting the
    /// real external caller's `file:line` (via `#[track_caller]`) and the given `target` (the
    /// caller's own `module_path!()` -- this trait cannot resolve that dynamically; see
    /// `Fix(BUG-229)`) instead of this method's own.
    #[ track_caller ]
    #[inline]
    fn debug_log( &self, level : Level, target : &str )
    {
      // Mirrors the `log!` macro's own pre-filter so a disabled level still skips the
      // `{self:#?}` formatting entirely, matching the original macro-based laziness.
      if !log::log_enabled!( target: target, level )
      {
        return;
      }
      let location = std::panic::Location::caller();
      log::logger().log(
        &Record::builder()
          .level( level )
          .file( Some( location.file() ) )
          .line( Some( location.line() ) )
          .module_path( Some( target ) )
          .target( target )
          .args( format_args!( "{self:#?}" ) )
          .build()
      );
    }

    /// Logs the debug representation of the object at the `trace` level. `target` should be
    /// the caller's own `module_path!()` (see `debug_log`'s doc comment for why).
    #[ track_caller ]
    #[inline]
    fn debug_trace( &self, target : &str )
    {
      self.debug_log( Level::Trace, target );
    }

    /// Logs the debug representation of the object at the `info` level. `target` should be
    /// the caller's own `module_path!()` (see `debug_log`'s doc comment for why).
    #[ track_caller ]
    #[inline]
    fn debug_info( &self, target : &str )
    {
      self.debug_log( Level::Info, target );
    }

    /// Logs the debug representation of the object at the `warn` level. `target` should be
    /// the caller's own `module_path!()` (see `debug_log`'s doc comment for why).
    #[ track_caller ]
    #[inline]
    fn debug_warn( &self, target : &str )
    {
      self.debug_log( Level::Warn, target );
    }

    /// Logs the debug representation of the object at the `error` level. `target` should be
    /// the caller's own `module_path!()` (see `debug_log`'s doc comment for why).
    #[ track_caller ]
    #[inline]
    fn debug_error( &self, target : &str )
    {
      self.debug_log( Level::Error, target );
    }
  }

  impl< T > DebugLog for T
  where
    T : fmt::Debug,
  {
  }
}

// This macro exposes the public interface of the module.
crate::mod_interface!
{
  /// Makes the `DebugLog` trait available for use.
  prelude use DebugLog;
}

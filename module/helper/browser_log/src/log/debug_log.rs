//! This module introduces the `DebugLog` trait, which provides convenient shortcut methods
//! for logging the debug representation of any type that implements `fmt::Debug`.
//! It integrates with the `log` crate to offer logging at various levels.

/// Internal namespace for implementation details.
mod private
{
  // use crate::*;
  use ::log::Level;
  use core::fmt;

  /// A trait that provides convenience methods for logging the debug output of a struct.
  ///
  /// This trait is automatically implemented for any type that implements `fmt::Debug`,
  /// allowing for quick and easy logging without boilerplate code.
  pub trait DebugLog : fmt::Debug
  {
    /// Logs the debug representation of the object at a specified log level.
    fn debug_log( &self, level : Level )
    {
      log::log!( level, "{:#?}", self );
    }

    /// Logs the debug representation of the object at the `trace` level.
    fn debug_trace( &self )
    {
      log::trace!( "{:#?}", self );
    }

    /// Logs the debug representation of the object at the `info` level.
    fn debug_info( &self )
    {
      log::info!( "{:#?}", self );
    }

    /// Logs the debug representation of the object at the `warn` level.
    fn debug_warn( &self )
    {
      log::warn!( "{:#?}", self );
    }

    /// Logs the debug representation of the object at the `error` level.
    fn debug_error( &self )
    {
      log::error!( "{:#?}", self );
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

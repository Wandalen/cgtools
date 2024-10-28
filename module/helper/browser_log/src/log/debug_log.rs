/// Internal namespace.
mod private
{
  // use crate::*;
  use ::log::Level;
  use core::fmt;

  pub trait DebugLog : fmt::Debug
  {

    fn debug_log( &self, level : Level )
    {
      log::log!( level, "{:#?}", self );
    }

    fn debug_trace( &self )
    {
      log::trace!( "{:#?}", self );
    }

    fn debug_info( &self )
    {
      log::info!( "{:#?}", self );
    }

    fn debug_warn( &self )
    {
      log::warn!( "{:#?}", self );
    }

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

crate::mod_interface!
{

  prelude use DebugLog;

}

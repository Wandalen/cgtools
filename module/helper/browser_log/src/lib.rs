//! Browser logging and panic handling utilities.
#![ doc( html_root_url = "https://docs.rs/browser_log/latest/browser_log/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Browser logging and panic handling utilities" ) ]

#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

mod private
{
  /// Config of both logging and panic
  #[ derive( Debug, Default ) ]
  #[non_exhaustive]
  pub struct Config
  {
    /// Logging config.
    pub log : crate::log::setup::Config,
    /// Panic config.
    pub panic : crate::panic::Config,
  }

  /// Setup both logging and panic.
  #[inline]
  pub fn setup( config : Config )
  {
    crate::panic::setup( config.panic );
    crate::log::setup::setup( config.log );
  }

}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{

  own use
  {
    Config,
    setup,
  };

  /// Logger in browser.
  layer log;
  /// Panic hook handling in Browser.
  layer panic;

}

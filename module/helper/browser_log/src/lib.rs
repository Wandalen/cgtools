//! Browser logging and panic handling utilities.
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

mod private
{
  /// Config of both logging and panic
  #[ derive( Debug, Default ) ]
  pub struct Config
  {
    /// Logging config.
    pub log : crate::log::setup::Config,
    /// Panic config.
    pub panic : crate::panic::Config,
  }

  /// Setup both logging and panic.
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

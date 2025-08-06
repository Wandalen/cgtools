#![ doc = include_str!( "../readme.md" ) ]

use ::mod_interface::mod_interface;

mod private
{
  use super::*;

  /// Config of both logging and panic
  #[ derive( Debug, Default ) ]
  pub struct Config
  {
    /// Logging config.
    pub log : log::setup::Config,
    /// Panic config.
    pub panic : panic::Config,
  }

  /// Setup both logging and panic.
  pub fn setup( config : Config )
  {
    panic::setup( config.panic );
    log::setup::setup( config.log );
  }

}

mod_interface!
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

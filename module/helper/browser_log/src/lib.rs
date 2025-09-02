//! Browser logging and panic handling utilities.
#![ doc( html_root_url = "https://docs.rs/browser_log/latest/browser_log/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Browser logging and panic handling utilities" ) ]

#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::str_to_string)]
#![allow(clippy::pattern_type_mismatch)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::unused_trait_names)]
#![allow(clippy::let_underscore_must_use)]
#![allow(clippy::missing_trait_methods)]

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

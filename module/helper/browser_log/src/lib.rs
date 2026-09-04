//! Browser logging and panic handling utilities.
#![ doc( html_root_url = "https://docs.rs/browser_log/latest/browser_log/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Browser logging and panic handling utilities" ) ]

#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

// Fix(BUG-169): `mod private` was unconditional while every dependency it needs
// (`crate::log::setup::Config`, `crate::panic::Config`) is only ever declared as a real
// module by the `mod_interface!` invocation below, itself already `#[cfg(feature = "enabled")]`
// -- building with `--no-default-features` compiled `mod private` anyway and failed with 4
// `E0433` "cannot find `log`/`panic` in `crate`" errors.
// Root cause: `enabled` gates every optional dependency (`log`, `mod_interface`, `wasm-bindgen`,
// `web-sys`) AND the `mod_interface!` invocation that declares the `log`/`panic` submodules --
// but `mod private`, which references both submodules' types, was never given the same gate.
// Pitfall: when a feature gates a macro invocation that declares submodules, every other item
// referencing those submodules needs the identical `#[cfg(...)]` -- gating the macro call alone
// doesn't retroactively gate unconditional code elsewhere that depends on its expansion.
#[ cfg( feature = "enabled" ) ]
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

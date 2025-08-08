//! Logging utility for Rust applications compiled to WebAssembly (`wasm32-unknown-unknown`)

/// Internal namespace.
mod private
{
  // use crate::*;

  use ::log::{ Level, Log, Metadata, Record };
  use wasm_bindgen::prelude::*;
  pub use web_sys::console;

  /// Specify what to be logged
  #[ derive( Debug ) ]
  pub struct Config
  {
    level : Level,
    target_filter : Option< String >,
  }

  impl Default for Config
  {
    fn default() -> Self
    {
      Self
      {
        level : Level::Debug,
        target_filter : None,
      }
    }
  }

  impl Config
  {
    /// Specify the maximum level you want to log
    pub fn new( level : Level ) -> Self
    {
      Self
      {
        level,
        target_filter : None,
      }
    }

    /// Configure the `target` of the logger. If specified, the logger
    /// only output for `log`s in module that its path starts with
    /// `target_filter`. this logger only supports single prefix. Only
    /// the last call to `target_filter` has effect if you call it multiple times.
    pub fn target_filter( mut self, target_filter : &str ) -> Self
    {
      self.target_filter = Some( target_filter.to_string() );
      self
    }

  }

  /// The log styles
  struct Predefined
  {
    lvl_trace : String,
    lvl_debug : String,
    lvl_info : String,
    lvl_warn : String,
    lvl_error : String,
    tgt : String,
    args : String,
  }

  impl Predefined
  {
    fn new() -> Predefined
    {
      let base = String::from( "color: white; padding: 0 3px; background:" );
      Predefined
      {
        lvl_trace : format!( "{base} gray;" ),
        lvl_debug : format!( "{base} blue;" ),
        lvl_info : format!( "{base} green;" ),
        lvl_warn : format!( "{base} orange;" ),
        lvl_error : format!( "{base} darkred;" ),
        tgt : String::from( "font-weight: bold; color: inherit" ),
        args : String::from( "background: inherit; color: inherit" ),
      }
    }
  }

  /// The logger
  struct BrowserLogger
  {
    config : Config,
    style : Predefined,
  }

  impl Log for BrowserLogger
  {
    fn enabled( &self, metadata: &Metadata<'_> ) -> bool
    {
      if let Some( prefix ) = &self.config.target_filter
      {
        metadata.target().starts_with( prefix )
      }
      else
      {
        true
      }
    }

    fn log( &self, record : &Record< '_ > )
    {
      if self.enabled( record.metadata() )
      {
        let style = &self.style;
        let s = format!
        (
          "%c{}%c {}:{}%c\n{}",
          record.level(),
          record.file().unwrap_or_else( || record.target() ),
          record
          .line()
          .map_or_else( || "[ Unknown ]".to_string(), |line| line.to_string() ),
          record.args(),
        );
        let s = JsValue::from_str( &s );
        let tgt_style = JsValue::from_str( &style.tgt );
        let args_style = JsValue::from_str( &style.args );
        match record.level()
        {
          Level::Trace => console::debug_4
          (
            &s,
            &JsValue::from( &style.lvl_trace ),
            &tgt_style,
            &args_style,
          ),
          Level::Debug => console::log_4
          (
            &s,
            &JsValue::from( &style.lvl_debug ),
            &tgt_style,
            &args_style,
          ),
          Level::Info =>
          {
            console::info_4( &s, &JsValue::from( &style.lvl_info ), &tgt_style, &args_style )
          }
          Level::Warn =>
          {
            console::warn_4( &s, &JsValue::from( &style.lvl_warn ), &tgt_style, &args_style )
          }
          Level::Error => console::error_4(
            &s,
            &JsValue::from( &style.lvl_error ),
            &tgt_style,
            &args_style,
          ),
        }
      }
    }

    fn flush( &self ) {}
  }

  /// Initialize the logger which the given config. If failed, it will log a message to the the browser console.
  ///
  /// ## Examples
  ///
  /// browser_log::log::setup::setup( Default::default() );
  ///
  /// or
  ///
  /// browser_log::log::setup::setup( browser_log::log::setup::Config::default().target_filter( "lib_name" ) );
  ///
  ///
  pub fn setup( config : Config )
  {
    let max_level = config.level;
    let wl = BrowserLogger
    {
      config,
      style : Predefined::new(),
    };
    match ::log::set_boxed_logger( Box::new( wl ) )
    {
      Ok( _ ) => log::set_max_level( max_level.to_level_filter() ),
      Err( e ) => console::error_1( &JsValue::from( e.to_string() ) ),
    }
  }

}

crate::mod_interface!
{


  orphan use
  {
    Config,
    setup,
  };

}

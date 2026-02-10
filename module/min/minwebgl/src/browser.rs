/// Internal namespace.
mod private
{
  use crate::web_sys;

  /// Detects if the browser is Firefox by checking the user agent string.
  ///
  /// Returns `true` if Firefox is detected, `false` otherwise.
  /// This is useful for working around Firefox-specific WebGL deprecations.
  #[ inline ]
  pub fn is_firefox() -> bool
  {
    web_sys::window()
      .and_then( | w | w.navigator().user_agent().ok() )
      .map( | ua | ua.contains( "Firefox" ) )
      .unwrap_or( false )
  }
}

crate::mod_interface!
{

  // xxx : investigate
  reuse ::browser_log;
  exposed use ::wasm_bindgen::
  {
    JsCast,
  };

  own use is_firefox;

}

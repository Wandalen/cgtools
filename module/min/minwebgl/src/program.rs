/// Internal namespace.
mod private
{
  #[ allow( clippy::wildcard_imports, reason = "crate-root prelude from mod_interface!; enumerating would break on every layer change" ) ]
  use crate::*;
  pub use shader::Error;
  pub use web_sys::WebGlProgram;

  pub use crate::shader::
  {
    // WebGlProgram,
    ProgramFromSources,
    ProgramShaders,
  };

}

crate::mod_interface!
{

  own use
  {
    Error,
  };

  orphan use
  {
    WebGlProgram,
    ProgramFromSources,
    ProgramShaders,
  };

}

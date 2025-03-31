/// Internal namespace.
mod private
{
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

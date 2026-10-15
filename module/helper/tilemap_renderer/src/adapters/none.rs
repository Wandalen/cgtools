//! No-op backend adapter.
//!
//! Accepts assets and commands and performs no GPU or document work at
//! all -- first-class support for "math-only simulation, no rendering"
//! (see `docs/adr/003_d2_stack_hal_adoption.md` Decision #2).

mod private
{
  use crate::assets::Assets;
  use crate::backend::{ Backend, Capabilities, Output, RenderError };
  use crate::commands::RenderCommand;
  use crate::types::RenderConfig;

  /// A backend that performs no rendering work at all.
  ///
  /// Every `Backend` method is a no-op: assets and commands are accepted
  /// and discarded, `output()` always reports `Output::Presented`, and
  /// `capabilities()` always reports `Capabilities::default()`.
  pub struct NoneBackend;

  impl NoneBackend
  {
    /// Creates a new no-op backend. `config` is accepted for shape
    /// symmetry with the other adapters' constructors but is otherwise
    /// unused -- this backend has no rendering state to configure.
    #[ inline ]
    #[ must_use ]
    pub fn new( _config : RenderConfig ) -> Self
    {
      Self
    }
  }

  impl Backend for NoneBackend
  {
    #[ inline ]
    fn assets_load( &mut self, _assets : &Assets ) -> Result< (), RenderError >
    {
      Ok( () )
    }

    #[ inline ]
    fn submit( &mut self, _commands : &[ RenderCommand ] ) -> Result< (), RenderError >
    {
      Ok( () )
    }

    #[ inline ]
    fn output( &self ) -> Result< Output, RenderError >
    {
      Ok( Output::Presented )
    }

    #[ inline ]
    fn resize( &mut self, _width : u32, _height : u32 )
    {
    }

    #[ inline ]
    fn capabilities( &self ) -> Capabilities
    {
      Capabilities::default()
    }
  }
}

mod_interface::mod_interface!
{
  own use NoneBackend;
}

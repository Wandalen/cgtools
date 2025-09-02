/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuProgrammableStage`.
  #[ derive( Clone ) ]
  pub struct ProgrammableStage< 'a >
  {
    /// The required shader module containing the shader code for this stage.
    ///
    /// This is the compiled representation of the WGSL shader.
    module : &'a web_sys::GpuShaderModule,
    /// An optional name of the entry point function within the shader module.
    ///
    /// If not specified, the WebGPU implementation will typically look for a
    /// default entry point, like "main".
    ///
    /// Defaults to `None`.
    entry_point : Option< &'a str >,
  }

  impl< 'a > ProgrammableStage< 'a > 
  {
    /// Creates a new `ProgrammableStage` builder with a required shader module.
    pub fn new( module : &'a web_sys::GpuShaderModule ) -> Self
    {
      let entry_point = None;

      ProgrammableStage
      {
        module,
        entry_point
      }
    }

    /// Sets the entry point function name for the shader stage.
    pub fn entry_point( mut self, entry : &'a str ) -> Self
    {
      self.entry_point = Some( entry );
      self
    }
  }

  impl From< ProgrammableStage< '_ > > for web_sys::GpuProgrammableStage
  {
    fn from( value: ProgrammableStage< '_ > ) -> Self 
    {
      let state = web_sys::GpuProgrammableStage::new( &value.module );

      if let Some( v ) = value.entry_point { state.set_entry_point( v ); }

      state
    }   
  }
}

crate::mod_interface!
{
  exposed use
  {
    ProgrammableStage
  };
}

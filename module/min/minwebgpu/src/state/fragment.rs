/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuFragmentState`.
  #[ derive( Clone ) ]
  pub struct FragmentState< 'a >
  {
    /// The required shader module containing the fragment shader code.
    ///
    /// This is the compiled representation of the WGSL fragment shader.
    module : &'a web_sys::GpuShaderModule,
    /// An optional name of the entry point function within the shader module.
    ///
    /// If not specified, the WebGPU implementation will typically look for a
    /// default entry point, like "main".
    ///
    /// Defaults to `None`.
    entry_point : Option< &'a str >,
    /// A list of color targets that the fragment shader will output to.
    ///
    /// Each `ColorTargetState` corresponds to a single output attachment and
    /// configures its format, blending, and write mask.
    targets : Vec< ColorTargetState >
  }

  impl< 'a > FragmentState< 'a > 
  {
    /// Creates a new `FragmentState` builder with a required shader module.
    pub fn new( module :  &'a web_sys::GpuShaderModule ) -> Self
    {
      let entry_point = None;
      let targets = Vec::with_capacity( 1 );

      FragmentState
      {
        module,
        entry_point,
        targets
      }
    }

    /// Sets the entry point function name for the fragment shader.
    pub fn entry_point( mut self, entry : &'a str ) -> Self
    {
      self.entry_point = Some( entry );
      self
    }

    /// Adds a color target to the list of targets.
    pub fn target( mut self, target : ColorTargetState ) -> Self
    {
      self.targets.push( target );
      self
    } 
  }

  impl From< FragmentState< '_ > > for web_sys::GpuFragmentState 
  {
    fn from( value: FragmentState< '_ > ) -> Self 
    {
      let targets : Vec< web_sys::GpuColorTargetState > = value.targets.into_iter().map( | t | t.into() ).collect();
      let state = web_sys::GpuFragmentState::new( &value.module, &targets.into() );

      if let Some( v ) = value.entry_point { state.set_entry_point( v ); }

      state
    }   
  }

}

crate::mod_interface!
{
  exposed use
  {
    FragmentState
  };
}

/// Internal namespace.
mod private
{
  use crate::web_sys;

  /// A builder-style struct for creating a `GpuShaderModule`.
  pub struct ShaderModule< 'a >
  {
    /// The source code of the shader, typically written in WGSL.
    code : &'a str,
    /// An optional label for debugging and identification purposes.
    label : Option< &'a str >
  }

  impl< 'a > ShaderModule< 'a > 
  {
    /// Creates a new `ShaderModule` instance with a given shader source code.
    #[ inline ]
    #[ must_use ]
    pub fn new( code : &'a str ) -> Self
    {
      let label = None;

      ShaderModule
      {
        code,
        label
      }
    } 

    /// Sets an optional label for the shader module.
    #[ inline ]
    #[ must_use ]
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Creates the `GpuShaderModule` using the configured properties.
    #[ inline ]
    #[ must_use ]
    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuShaderModule
    {
      let desc = web_sys::GpuShaderModuleDescriptor::new( self.code );

      if let Some( v ) = self.label { desc.set_label( v ); }

      device.create_shader_module( &desc )
    }
  }

  /// A convenience function to create a `GpuShaderModule` with just the code.
  #[ inline ]
  #[ must_use ]
  pub fn create( device : &web_sys::GpuDevice, code : &str ) -> web_sys::GpuShaderModule
  {
    ShaderModule::new( code ).create( device )
  }
}

crate::mod_interface!
{
  own use
  {
    create
  };
  exposed use
  {
    ShaderModule
  };
}

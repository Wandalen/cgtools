/// Internal namespace.
mod private
{
  use crate::*;

  pub struct ShaderModule< 'a >
  {
    code : &'a str,
    label : Option< &'a str >
  }

  impl< 'a > ShaderModule< 'a > 
  {
    pub fn new( code : &'a str ) -> Self
    {
      let label = None;

      ShaderModule
      {
        code,
        label
      }
    } 

    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuShaderModule
    {
      let desc = web_sys::GpuShaderModuleDescriptor::new( &self.code );

      if let Some( v ) = self.label { desc.set_label( v ); }

      let shader = device.create_shader_module( &desc );
      shader
    }
  }

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

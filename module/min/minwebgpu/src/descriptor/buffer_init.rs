/// Internal namespace.
mod private
{
  use crate::*;

  pub struct BufferInitDescriptor< 'a, T : AsBytes >
  {
    pub data : &'a T,
    pub usage : u32,
    /// Defaults to `None`
    pub label : Option< &'a str >
  }

  impl< 'a, T : AsBytes > BufferInitDescriptor< 'a, T >
  {
    pub fn new( data : &'a T, usage : u32 ) -> Self
    {
      let label = None;

      BufferInitDescriptor
      {
        data,
        usage,
        label
      }
    }

    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    pub fn create( &self, device : &web_sys::GpuDevice ) -> Result< web_sys::GpuBuffer, WebGPUError >
    {
      buffer::init( device, &self )
    }
  }
}

crate::mod_interface!
{
  exposed use 
  {
    BufferInitDescriptor
  };
}

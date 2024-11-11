/// Internal namespace.
mod private
{
  use crate::*;

  pub struct BufferDescriptor< 'a >
  {
    usage : u32,
    /// Defaults to `0`
    size : f64,
    /// Defaults to `false`
    mapped_at_creation : Option< bool >,
    /// Defaults to `None`
    label : Option< &'a str >
  }

  impl< 'a > BufferDescriptor< 'a >
  {
    pub fn new( usage : u32 ) -> Self
    {
      let label = None;
      let size = 0.0;
      let mapped_at_creation = None;

      BufferDescriptor
      {
        usage,
        label,
        size,
        mapped_at_creation
      }
    }

    /// Set size from the provided type
    pub fn size< T >( mut self ) -> Self
    {
      self.size = std::mem::size_of::< T >() as f64;
      self
    }

    /// Set size from the provided variable, i.e. use std::mem::size_of_val
    pub fn size_from_var< T >( mut self, var : &T ) -> Self
    {
      self.size = std::mem::size_of_val( var ) as f64;
      self
    }

    /// Set size from the provided value
    pub fn size_from_value( mut self, val : f64 ) -> Self
    {
      self.size = val;
      self
    }

    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    pub fn mapped( mut self ) -> Self
    {
      self.mapped_at_creation = Some( true );
      self
    }

    pub fn create( self, device : &web_sys::GpuDevice ) -> Result< web_sys::GpuBuffer, WebGPUError >
    {
      buffer::create( device, &self.into() )
    }
  }

  impl From< BufferDescriptor< '_ > > for web_sys::GpuBufferDescriptor 
  {
    fn from( value: BufferDescriptor< '_ > ) -> Self 
    {
      let desc = web_sys::GpuBufferDescriptor::new( value.size, value.usage );

      if let Some( v ) = value.label { desc.set_label( v ); }
      if let Some( v ) = value.mapped_at_creation { desc.set_mapped_at_creation( v ); }

      desc
    }   
  }
}

crate::mod_interface!
{
  exposed use 
  {
    BufferDescriptor
  };
}

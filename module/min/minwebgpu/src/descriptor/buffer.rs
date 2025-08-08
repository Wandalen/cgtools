/// Internal namespace.
mod private
{
  use crate::*;

  /// Describes the configuration for creating a WebGPU buffer.
  pub struct BufferDescriptor< 'a >
  {
    /// A bitmask that defines how the buffer will be used (e.g., as vertex data,
    /// index data, or uniform data). This is a required field.
    usage : u32,
    /// The size of the buffer in bytes.
    /// Defaults to `0` and should be set to a non-zero value before creation.
    size : f64,
    /// A boolean flag indicating whether the buffer should be mapped for writing
    /// immediately upon creation. This is useful for buffers that need to be
    /// populated with data right away.
    /// Defaults to `false`.
    mapped_at_creation : Option< bool >,
    /// An optional label for the buffer, useful for debugging and performance tools.
    /// Defaults to `None`.
    label : Option< &'a str >
  }

  impl< 'a > BufferDescriptor< 'a >
  {
    /// Creates a new `BufferDescriptor` with a specified usage.
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
    pub fn size_from_var< T : ?Sized >( mut self, var : &T ) -> Self
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

    /// Sets an optional label for the buffer.
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Sets the `mapped_at_creation` property to `true`.
    pub fn mapped( mut self ) -> Self
    {
      self.mapped_at_creation = Some( true );
      self
    }

    /// Creates a `web_sys::GpuBuffer` from this descriptor.
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

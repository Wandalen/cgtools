/// Internal namespace.
mod private
{
  use crate::*;

  /// Represents a binding to a WebGPU buffer.
  pub struct BufferBinding< 'a >
  {
    /// A reference to the underlying WebGPU buffer object.
    buffer : &'a web_sys::GpuBuffer,
    /// Defaults to `0`
    offset : Option< u32 >,
    /// Defaults to the size of the buffer, if offset is omitted aswell
    size : Option< u32 >
  }

  impl< 'a > BufferBinding< 'a >  
  {
     /// Creates a new `BufferBinding` with a given buffer and default offset and size.
    pub fn new( buffer : &'a web_sys::GpuBuffer ) -> Self
    {
      let offset = None;
      let size = None;

      BufferBinding
      {
        buffer,
        offset,
        size
      }
    }

    /// Sets the offset for the buffer binding.
    pub fn offset( mut self, offset : u32 ) -> Self
    {
      self.offset = Some( offset );
      self
    }

    /// Sets the size of the buffer binding.
    pub fn size( mut self, size : u32 ) -> Self
    {
      self.size = Some( size );
      self
    }
  }

  impl From< &BufferBinding< '_ > > for web_sys::GpuBufferBinding 
  {
    fn from( value: &BufferBinding< '_ > ) -> Self {
      let binding = web_sys::GpuBufferBinding::new( value.buffer );

      if let Some( v ) = value.size { binding.set_size( v ); }
      if let Some( v ) = value.offset { binding.set_offset( v ); }

      binding
    }
  }

  impl From< BufferBinding< '_ > > for web_sys::GpuBufferBinding 
  {
    fn from( value: BufferBinding< '_ > ) -> Self {
      ( &value ).into()
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    BufferBinding
  };
}

/// Internal namespace.
mod private
{
  use crate::*;

  pub struct BufferBinding< 'a >
  {
    buffer : &'a web_sys::GpuBuffer,
    /// Defaults to `0.0`
    offset : Option< f64 >,
    /// Defaults to the size of the buffer, if offset is omitted aswell
    size : Option< f64 >
  }

  impl< 'a > BufferBinding< 'a >  
  {
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

    pub fn offset( mut self, offset : f64 ) -> Self
    {
      self.offset = Some( offset );
      self
    }    

    pub fn size( mut self, size : f64 ) -> Self
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

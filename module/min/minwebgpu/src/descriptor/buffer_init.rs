/// Internal namespace.
mod private
{
  use crate::*;

  /// Describes a buffer to be created and initialized with data.
  pub struct BufferInitDescriptor< 'a, T : AsBytes >
  {
    /// A reference to the data that will be copied into the buffer.
    pub data : &'a T,
    /// A bitmask that specifies how the buffer will be used, such as for vertex data,
    /// index data, or uniform data. This is a crucial hint to the GPU driver for
    /// optimizing memory layout.
    pub usage : u32,
    /// An optional label for the buffer. This can be helpful for debugging and
    /// performance tracing in tools like the browser's GPU debugger.
    /// Defaults to `None`.
    pub label : Option< &'a str >
  }

  impl< 'a, T : AsBytes > BufferInitDescriptor< 'a, T >
  {
    /// Creates a new `BufferInitDescriptor` with a given data reference and usage.
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

    /// Sets an optional label for the buffer.
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Creates a `web_sys::GpuBuffer` from this descriptor.
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

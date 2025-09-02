/// Internal namespace.
mod private
{
  use crate::*;

  /// Submits a command buffer to the WebGPU queue.
  pub fn submit( queue : &web_sys::GpuQueue, buffer : web_sys::GpuCommandBuffer )
  {
    queue.submit( &Vec::from( [ buffer ] ).into() );
  }

  /// Writes data to a WebGPU buffer.
  pub fn write_buffer< T : mem::Pod >
  ( 
    queue : &web_sys::GpuQueue,
    buffer : &web_sys::GpuBuffer, 
    data : &[ T ] 
  ) -> Result< (), WebGPUError >
  {
    queue.write_buffer_with_f64_and_u8_slice( buffer, 0.0, mem::cast_slice( data ) )
    .map_err( | e | BufferError::FailedWriteToBuffer( format!( "{:?}", e ) ))?;

    Ok( () )
  }
}

crate::mod_interface!
{
  own use
  {
    submit,
    write_buffer
  };
}

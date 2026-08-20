/// Internal namespace.
mod private
{
  use crate::{ web_sys, mem, WebGPUError, BufferError, TextureError };

  /// Submits a command buffer to the WebGPU queue.
  #[ inline ]
  pub fn submit( queue : &web_sys::GpuQueue, buffer : web_sys::GpuCommandBuffer )
  {
    queue.submit( &[ buffer ] );
  }

  /// Writes data to a WebGPU buffer.
  ///
  /// # Errors
  /// Returns `error::BufferError::FailedWriteToBuffer` if the underlying
  /// `GPUQueue.writeBuffer` call throws.
  #[ inline ]
  pub fn buffer_write< T : mem::Pod >
  (
    queue : &web_sys::GpuQueue,
    buffer : &web_sys::GpuBuffer,
    data : &[ T ]
  ) -> Result< (), WebGPUError >
  {
    queue.write_buffer_with_f64_and_u8_slice( buffer, 0.0, mem::cast_slice( data ) )
    .map_err( | e | BufferError::FailedWriteToBuffer( format!( "{e:?}" ) ))?;

    Ok( () )
  }

  /// Writes pixel data to a WebGPU texture (whole-texture, base mip level only).
  ///
  /// # Errors
  /// Returns `error::TextureError::FailedWriteToTexture` if the underlying
  /// `GPUQueue.writeTexture` call throws.
  #[ inline ]
  pub fn texture_write
  (
    queue : &web_sys::GpuQueue,
    destination : &web_sys::GpuTexelCopyTextureInfo,
    data : &[ u8 ],
    data_layout : &web_sys::GpuTexelCopyBufferLayout,
    size : &web_sys::GpuExtent3dDict
  ) -> Result< (), WebGPUError >
  {
    queue.write_texture_with_u8_slice_and_gpu_extent_3d_dict( destination, data, data_layout, size )
    .map_err( | e | TextureError::FailedWriteToTexture( format!( "{e:?}" ) ))?;

    Ok( () )
  }
}

crate::mod_interface!
{
  own use
  {
    submit,
    buffer_write,
    texture_write
  };
}

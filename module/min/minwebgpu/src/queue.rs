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

  /// Writes data to a WebGPU buffer at a given byte offset.
  ///
  /// # Errors
  /// Returns `error::BufferError::FailedWriteToBuffer` if the underlying
  /// `GPUQueue.writeBuffer` call throws.
  //
  // Fix(UX-008): `buffer_write` hardcoded its write offset to `0.0`, with no way for a caller
  // to write into the middle of a buffer -- unlike this crate's own `BufferBinding`, which
  // already supports an arbitrary `offset`. Added as a sibling ( `buffer_write` now delegates
  // here with `0.0` ) rather than adding a parameter to `buffer_write` itself, since that
  // function is called directly from several `examples/` crates and other workspace crates
  // outside this fix's edit scope -- a new required parameter would break all of them.
  #[ inline ]
  pub fn buffer_write_at< T : mem::Pod >
  (
    queue : &web_sys::GpuQueue,
    buffer : &web_sys::GpuBuffer,
    buffer_offset : f64,
    data : &[ T ]
  ) -> Result< (), WebGPUError >
  {
    queue.write_buffer_with_f64_and_u8_slice( buffer, buffer_offset, mem::cast_slice( data ) )
    .map_err( | e | BufferError::FailedWriteToBuffer( format!( "{e:?}" ) ))?;

    Ok( () )
  }

  /// Writes data to a WebGPU buffer, starting at byte offset `0.0`.
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
    buffer_write_at( queue, buffer, 0.0, data )
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
    buffer_write_at,
    texture_write
  };
}

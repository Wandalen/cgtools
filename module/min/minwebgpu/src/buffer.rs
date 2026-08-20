/// Internal namespace.
mod private
{
  use crate::{ web_sys, WebGPUError, DeviceError, AsBytes, BufferInitDescriptor, COPY_BUFFER_ALIGNMENT, BufferError, js_sys };

  /// Creates a GPU buffer from a given descriptor.
  ///
  /// # Errors
  /// Returns `error::DeviceError::FailedToCreateBuffer` if the underlying
  /// `GPUDevice.createBuffer` call throws.
  #[ inline ]
  pub fn create
  (
    device : &web_sys::GpuDevice,
    desc : &web_sys::GpuBufferDescriptor
  ) -> Result< web_sys::GpuBuffer, WebGPUError >
  {
    let buffer = device.create_buffer( desc )
    .map_err( | e | DeviceError::FailedToCreateBuffer( format!( "{e:?}" ) ) )?;

    Ok( buffer )
  }

  /// Creates a new GPU buffer and initializes it with data.
  ///
  /// # Errors
  /// Returns `error::DeviceError::FailedToCreateBuffer` if the underlying
  /// `GPUDevice.createBuffer` call throws, or `error::BufferError::FailedToGetMappedRange`
  /// if the buffer's mapped range cannot be retrieved to copy the initial data into.
  #[ inline ]
  pub fn init< T : AsBytes >
  (
    device : &web_sys::GpuDevice,
    init_desc : &BufferInitDescriptor< '_, T >
  ) -> Result< web_sys::GpuBuffer, WebGPUError >
  {
    // Round up the size to be aligned with `COPY_BUFFER_ALIGNMENT`
    let unpadded_size = init_desc.data.byte_size() as u64;

    if unpadded_size == 0
    {
      let desc = web_sys::GpuBufferDescriptor::new_with_f64( 0.0, init_desc.usage );
      if let Some( v ) = init_desc.label { desc.set_label( v ); }

      let buffer = device.create_buffer( &desc )
      .map_err( | e | DeviceError::FailedToCreateBuffer( format!( "{e:?}" ) ) )?;

      return Ok( buffer );
    }

    let align_mask = COPY_BUFFER_ALIGNMENT - 1;
    let padded_size = ( ( unpadded_size + align_mask ) & !align_mask ).max( COPY_BUFFER_ALIGNMENT );

    // Create buffer descriptor
    // GPU buffer sizes stay far below f64's 2^52 exact-integer limit (4 petabytes) for any
    // realistic allocation, so this precision loss is unreachable.
    let padded_size_f64 = padded_size as f64;
    let desc = web_sys::GpuBufferDescriptor::new_with_f64( padded_size_f64, init_desc.usage );
    desc.set_mapped_at_creation( true );

    if let Some( label ) = init_desc.label { desc.set_label( label ); }

    let buffer = device.create_buffer( &desc )
    .map_err( | e | DeviceError::FailedToCreateBuffer( format!( "{e:?}" ) ) )?;

    // Copy data to the buffer using mapped range
    let mapped_range = buffer.get_mapped_range()
    .map_err( | e | BufferError::FailedToGetMappedRange( format!( "{e:?}" ) ) )?;
    
    let array = js_sys::Uint8Array::new( &mapped_range );
    array.copy_from( init_desc.data.as_bytes() );

    buffer.unmap();

    Ok( buffer )
  }

}

crate::mod_interface!
{
  own use 
  {
    init,
    create
  };
}

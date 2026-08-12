//! This module provides GPU-to-host readback of rendered images, handling the
//! `wgpu` row-alignment requirement ( `bytes_per_row` padded to 256 ) that a naive
//! `width * 4` copy silently violates for most widths.

use mingl::mod_interface;

mod private
{
  /// Reads mip level 0, layer 0 of `texture` back to host memory as tightly packed
  /// RGBA8 pixels, returning them together with the `( width, height )` size.
  ///
  /// The copy pads `bytes_per_row` up to [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`] as
  /// `wgpu` requires, then strips the padding from the returned pixels, so any width
  /// works — not only widths whose `width * 4` happens to be 256-aligned. The texture
  /// must have `COPY_SRC` usage and a 4-byte-per-pixel format ( e.g. `Rgba8Unorm`,
  /// `Rgba8UnormSrgb`, `Bgra8UnormSrgb` ). Blocks until the GPU work completes.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture format is not 4 bytes per pixel, when polling
  /// the device fails, or when mapping the readback buffer fails.
  pub fn rgba8( device : &wgpu::Device, queue : &wgpu::Queue, texture : &wgpu::Texture )
  -> Result< ( Vec< u8 >, ( u32, u32 ) ), crate::Error >
  {
    let format = texture.format();
    if format.block_copy_size( None ) != Some( 4 )
    {
      return Err
      (
        crate::Error::UnsupportedTextureFormat( format, "readback::rgba8 requires a 4-byte-per-pixel format" )
      );
    }

    let size = texture.size();
    let ( width, height ) = ( size.width, size.height );
    let padded = padded_bytes_per_row( width * 4 );
    let buffer_size = u64::from( padded ) * u64::from( height );
    let output_buffer = crate::buffer::buffer( wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ )
    .label( "readback_buffer" )
    .size_from_value( buffer_size )
    .build( device );

    let mut encoder = device.create_command_encoder
    (
      &wgpu::CommandEncoderDescriptor { label : Some( "readback_encoder" ) }
    );
    encoder.copy_texture_to_buffer
    (
      texture.as_image_copy(),
      wgpu::TexelCopyBufferInfo
      {
        buffer : &output_buffer,
        layout : wgpu::TexelCopyBufferLayout
        {
          offset : 0,
          bytes_per_row : Some( padded ),
          rows_per_image : None
        }
      },
      wgpu::Extent3d { width, height, depth_or_array_layers : 1 }
    );
    queue.submit( Some( encoder.finish() ) );

    let buffer_slice = output_buffer.slice( .. );
    let ( sender, receiver ) = std::sync::mpsc::channel();
    buffer_slice.map_async( wgpu::MapMode::Read, move | result | { let _ = sender.send( result ); } );
    device.poll( wgpu::PollType::Wait { submission_index : None, timeout : None } )?;
    receiver.recv().unwrap_or( Err( wgpu::BufferAsyncError ) )?;

    let pixels =
    {
      let data = buffer_slice.get_mapped_range();
      rows_unpad( &data, ( width, height ) )
    };
    output_buffer.unmap();

    Ok( ( pixels, ( width, height ) ) )
  }

  /// Rounds a row byte count up to the next multiple of
  /// [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`].
  ///
  /// Exposed for tests : this is the pure math behind [`rgba8`]'s row padding.
  #[ doc( hidden ) ]
  #[ must_use ]
  pub const fn padded_bytes_per_row( unpadded : u32 ) -> u32
  {
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    unpadded.div_ceil( align ) * align
  }

  /// Copies pixels out of a row-padded readback buffer, dropping the per-row padding.
  ///
  /// `data` is laid out as `height` rows of `padded_bytes_per_row( width * 4 )` bytes,
  /// of which only the first `width * 4` bytes per row are pixels.
  ///
  /// Exposed for tests : this is the pure strip logic behind [`rgba8`].
  ///
  /// # Panics
  ///
  /// Panics when `data` is shorter than the padded row layout implies.
  #[ doc( hidden ) ]
  #[ must_use ]
  pub fn rows_unpad( data : &[ u8 ], size : ( u32, u32 ) ) -> Vec< u8 >
  {
    let ( width, height ) = size;
    let unpadded = ( width * 4 ) as usize;
    let padded = padded_bytes_per_row( width * 4 ) as usize;
    if unpadded == padded
    {
      return data[ .. padded * height as usize ].to_vec();
    }
    let mut pixels = Vec::with_capacity( unpadded * height as usize );
    for row in 0 .. height as usize
    {
      let start = row * padded;
      pixels.extend_from_slice( &data[ start .. start + unpadded ] );
    }
    pixels
  }
}

mod_interface!
{
  own use rgba8;
  own use padded_bytes_per_row;
  own use rows_unpad;
}

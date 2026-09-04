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
  /// must have `COPY_SRC` usage and be one of `Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`,
  /// or `Bgra8UnormSrgb` — the latter two are red/blue-swapped on the way out, since their
  /// native GPU byte layout is BGRA, not RGBA. Blocks until the GPU work completes.
  ///
  /// # Errors
  ///
  /// Returns an error when the texture format is not one of the four formats above, when
  /// polling the device fails, or when mapping the readback buffer fails.
  pub fn rgba8( device : &wgpu::Device, queue : &wgpu::Queue, texture : &wgpu::Texture )
  -> Result< ( Vec< u8 >, ( u32, u32 ) ), crate::Error >
  {
    let format = texture.format();
    // Fix(BUG-166): the format check used to be `block_copy_size( None ) != Some( 4 )`, a
    // byte-size-only test that let every 4-byte-per-pixel format through, including several
    // that aren't RGBA8-shaped at all (`Rg16*`, `R32*`, `Rgb9e5Ufloat`, `Rgb10a2*`,
    // `Rg11b10Ufloat`), and silently mislabeled `Bgra8Unorm`/`Bgra8UnormSrgb` -- explicitly
    // listed in this very function's own doc as accepted input -- as "RGBA8 pixels" without
    // ever swapping their native blue/red byte order.
    // Root cause: `wgpu::Surface::get_default_config`'s own format pick (surfaced in this
    // crate via `surface::preferred_format`, see `surface_test.rs`'s
    // `preferred_format_picks_first_srgb_when_present`) routinely selects `Bgra8UnormSrgb` --
    // the common swapchain format on several real backends -- so a caller reading back a
    // surface-configured render target hit this silently-wrong output, not a rare edge case.
    // Pitfall: a byte-size check is not a format check -- "4 bytes per pixel" says nothing
    // about channel count, order, or bit layout; only an explicit format allowlist can assert
    // the caller-visible "RGBA8 pixels" contract this function actually promises.
    let is_bgra = matches!( format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb );
    let is_rgba = matches!( format, wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb );
    if !is_rgba && !is_bgra
    {
      return Err
      (
        crate::Error::UnsupportedTextureFormat
        (
          format,
          "readback::rgba8 requires Rgba8Unorm, Rgba8UnormSrgb, Bgra8Unorm, or Bgra8UnormSrgb",
        )
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

    let mut pixels =
    {
      let data = buffer_slice.get_mapped_range()?;
      rows_unpad( &data, ( width, height ) )
    };
    output_buffer.unmap();
    if is_bgra
    {
      bgra_to_rgba_swizzle( &mut pixels );
    }

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

  /// Swaps the red and blue byte of every tightly packed pixel in place, converting a BGRA8
  /// buffer to RGBA8.
  ///
  /// Exposed for tests : this is the pure swizzle logic behind [`rgba8`]'s BGRA handling.
  ///
  /// # Panics
  ///
  /// Panics when `pixels`'s length is not a multiple of 4.
  #[ doc( hidden ) ]
  pub fn bgra_to_rgba_swizzle( pixels : &mut [ u8 ] )
  {
    assert!( pixels.len().is_multiple_of( 4 ), "pixels must be a whole number of 4-byte RGBA/BGRA pixels" );
    for pixel in pixels.chunks_exact_mut( 4 )
    {
      pixel.swap( 0, 2 );
    }
  }
}

mod_interface!
{
  own use rgba8;
  own use padded_bytes_per_row;
  own use rows_unpad;
  own use bgra_to_rgba_swizzle;
}

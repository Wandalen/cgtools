mod private
{
  use crate::
  {
    Error,
    BufferUsage,
    TextureUsage,
    TextureFormat,
    VertexFormat,
    IndexFormat,
    ShaderStages,
    BindingType,
    FilterMode,
    AddressMode
  };

  // The HAL's usage and stage flags carry WebGPU bit values, which wgpu's
  // own bitflags share — from_bits_truncate is a bit-identical mapping, not
  // a lossy approximation.

  impl From< BufferUsage > for wgpu::BufferUsages
  {
    /// The equivalent raw wgpu usage flags.
    fn from( value : BufferUsage ) -> Self
    {
      wgpu::BufferUsages::from_bits_truncate( value.bits() )
    }
  }

  impl From< TextureUsage > for wgpu::TextureUsages
  {
    /// The equivalent raw wgpu usage flags.
    fn from( value : TextureUsage ) -> Self
    {
      wgpu::TextureUsages::from_bits_truncate( value.bits() )
    }
  }

  impl From< ShaderStages > for wgpu::ShaderStages
  {
    /// The equivalent raw wgpu stage flags.
    fn from( value : ShaderStages ) -> Self
    {
      wgpu::ShaderStages::from_bits_truncate( value.bits() )
    }
  }

  impl From< TextureFormat > for wgpu::TextureFormat
  {
    /// The equivalent raw wgpu format.
    fn from( value : TextureFormat ) -> Self
    {
      match value
      {
        TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
        TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        TextureFormat::Depth24Plus => wgpu::TextureFormat::Depth24Plus
      }
    }
  }

  impl From< VertexFormat > for wgpu::VertexFormat
  {
    /// The equivalent raw wgpu format.
    fn from( value : VertexFormat ) -> Self
    {
      match value
      {
        VertexFormat::Float32x2 => wgpu::VertexFormat::Float32x2,
        VertexFormat::Float32x3 => wgpu::VertexFormat::Float32x3,
        VertexFormat::Float32x4 => wgpu::VertexFormat::Float32x4
      }
    }
  }

  impl From< IndexFormat > for wgpu::IndexFormat
  {
    /// The equivalent raw wgpu format.
    fn from( value : IndexFormat ) -> Self
    {
      match value
      {
        IndexFormat::Uint32 => wgpu::IndexFormat::Uint32
      }
    }
  }

  impl From< BindingType > for wgpu::BindingType
  {
    /// The equivalent raw wgpu binding type — the v0 fixed set: uniform
    /// buffers, filterable 2d float textures, filtering samplers.
    fn from( value : BindingType ) -> Self
    {
      match value
      {
        BindingType::UniformBuffer => wgpu::BindingType::Buffer
        {
          ty : wgpu::BufferBindingType::Uniform,
          has_dynamic_offset : false,
          min_binding_size : None
        },
        BindingType::Texture => wgpu::BindingType::Texture
        {
          sample_type : wgpu::TextureSampleType::Float { filterable : true },
          view_dimension : wgpu::TextureViewDimension::D2,
          multisampled : false
        },
        BindingType::Sampler => wgpu::BindingType::Sampler( wgpu::SamplerBindingType::Filtering )
      }
    }
  }

  impl From< FilterMode > for wgpu::FilterMode
  {
    /// The equivalent raw wgpu filter mode.
    fn from( value : FilterMode ) -> Self
    {
      match value
      {
        FilterMode::Nearest => wgpu::FilterMode::Nearest,
        FilterMode::Linear => wgpu::FilterMode::Linear
      }
    }
  }

  impl From< AddressMode > for wgpu::AddressMode
  {
    /// The equivalent raw wgpu address mode.
    fn from( value : AddressMode ) -> Self
    {
      match value
      {
        AddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        AddressMode::Repeat => wgpu::AddressMode::Repeat
      }
    }
  }

  /// Copies `texture` ( rgba8unorm, COPY_SRC ) into a staging buffer and
  /// returns its tightly-packed rgba bytes, top row first.
  ///
  /// Synchronous : submits the copy and blocks on the map via device poll —
  /// readback is a verification path, not a frame path.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Unsupported`] if `texture`'s format is not
  /// `Rgba8Unorm`. Returns [`Error::Native`] if the device poll, the
  /// readback map callback, or the GPU-side buffer mapping fails.
  pub fn texture_rgba8_read
  (
    device : &wgpu::Device,
    queue : &wgpu::Queue,
    texture : &wgpu::Texture
  ) -> Result< Vec< u8 >, Error >
  {
    if texture.format() != wgpu::TextureFormat::Rgba8Unorm
    {
      return Err( Error::Unsupported
      (
        format!( "texture_rgba8_read reads rgba8unorm only, not {:?}", texture.format() )
      ) );
    }
    let width = texture.width();
    let height = texture.height();
    let bytes_per_row = width * 4;
    // Buffer copies of textures require 256-byte row alignment; rows are
    // padded on copy and re-packed tightly below.
    let padded_bytes_per_row = bytes_per_row.div_ceil( wgpu::COPY_BYTES_PER_ROW_ALIGNMENT )
    * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let staging = device.create_buffer( &wgpu::BufferDescriptor
    {
      label : Some( "gpu_hal readback staging" ),
      size : u64::from( padded_bytes_per_row ) * u64::from( height ),
      usage : wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      mapped_at_creation : false
    } );

    let mut encoder = device.create_command_encoder( &wgpu::CommandEncoderDescriptor::default() );
    encoder.copy_texture_to_buffer
    (
      wgpu::TexelCopyTextureInfo
      {
        texture,
        mip_level : 0,
        origin : wgpu::Origin3d::ZERO,
        aspect : wgpu::TextureAspect::All
      },
      wgpu::TexelCopyBufferInfo
      {
        buffer : &staging,
        layout : wgpu::TexelCopyBufferLayout
        {
          offset : 0,
          bytes_per_row : Some( padded_bytes_per_row ),
          rows_per_image : None
        }
      },
      wgpu::Extent3d { width, height, depth_or_array_layers : 1 }
    );
    queue.submit( core::iter::once( encoder.finish() ) );

    let slice = staging.slice( .. );
    let ( sender, receiver ) = std::sync::mpsc::channel();
    slice.map_async( wgpu::MapMode::Read, move | result | { let _ = sender.send( result ); } );
    device.poll( wgpu::PollType::wait_indefinitely() )
    .map_err( | e | Error::Native( format!( "device poll failed : {e:?}" ) ) )?;
    receiver.recv()
    .map_err( | _ | Error::Native( "readback map callback never fired".to_string() ) )?
    .map_err( | e | Error::Native( format!( "readback map failed : {e:?}" ) ) )?;

    let mapped = slice.get_mapped_range()
    .map_err( | e | Error::Native( format!( "readback map range failed : {e:?}" ) ) )?;
    let mut pixels = Vec::with_capacity( bytes_per_row as usize * height as usize );
    for row in 0..height as usize
    {
      let start = row * padded_bytes_per_row as usize;
      pixels.extend_from_slice( &mapped[ start..start + bytes_per_row as usize ] );
    }
    drop( mapped );
    staging.unmap();
    Ok( pixels )
  }
}

crate::mod_interface!
{
  own use texture_rgba8_read;
}

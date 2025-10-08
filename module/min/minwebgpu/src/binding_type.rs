/// Internal namespace.
mod private
{
  use crate::*;
  use binding_type::*;

  /// A custom enum to represent the different types of binding resources in WebGPU.
  #[ derive( Clone ) ]
  pub enum BindingType
  {
    /// Represents a buffer binding, used for uniform, storage, or read-only storage buffers.
    Buffer( web_sys::GpuBufferBindingLayout ),
    /// Represents a sampler binding, used for sampling textures.
    Sampler( web_sys::GpuSamplerBindingLayout ),
    /// Represents a sampled texture binding.
    Texture( web_sys::GpuTextureBindingLayout ),
    /// Represents a storage texture binding, used for reading and/or writing to a texture in a shader.
    StorageTexture( web_sys::GpuStorageTextureBindingLayout ),
    /// Represents an external texture binding, used for binding video frames.
    ExternalTexture( web_sys::GpuExternalTextureBindingLayout ),
    /// A placeholder for other or unhandled binding types.
    Other
  }

  macro_rules! impl_into_binding_ty
  {
    ( $s_name:ty, $t_name:ident ) => 
    {
      impl From< $s_name > for BindingType
      {
        fn from( value: $s_name ) -> Self 
        {
            BindingType::$t_name( value.into() )
        }   
      }
    };
  }

  impl_into_binding_ty!( BufferBindingLayout, Buffer );
  impl_into_binding_ty!( web_sys::GpuBufferBindingLayout, Buffer );

  impl_into_binding_ty!( SamplerBindingLayout, Sampler );
  impl_into_binding_ty!( web_sys::GpuSamplerBindingLayout, Sampler );

  impl_into_binding_ty!( TextureBindingLayout, Texture );
  impl_into_binding_ty!( web_sys::GpuTextureBindingLayout, Texture );

  impl_into_binding_ty!( StorageTextureBindingLayout, StorageTexture );
  impl_into_binding_ty!( web_sys::GpuStorageTextureBindingLayout, StorageTexture );

  impl_into_binding_ty!( ExternalTextureBindingLayout, ExternalTexture );
  impl_into_binding_ty!( web_sys::GpuExternalTextureBindingLayout, ExternalTexture );


  /// Creates a default `BufferBindingLayout`.
  pub fn buffer_type() -> BufferBindingLayout
  {
    BufferBindingLayout::new()
  }

  /// Creates a default `TextureBindingLayout`.
  pub fn texture_type() -> TextureBindingLayout
  {
    TextureBindingLayout::new()
  }

  /// Creates a default `SamplerBindingLayout`.
  pub fn sampler_type() -> SamplerBindingLayout
  {
    SamplerBindingLayout::new()
  }
  
  /// Creates a default `StorageTextureBindingLayout`.
  pub fn storage_texture_type() -> StorageTextureBindingLayout
  {
    StorageTextureBindingLayout::new()
  }

  /// Creates a default `ExternalTextureBindingLayout`.
  pub fn external_texture_type() -> ExternalTextureBindingLayout
  {
    ExternalTextureBindingLayout
  }
}

crate::mod_interface!
{
  /// Buffer binding
  layer buffer;
  /// Sampler binding
  layer sampler;
  /// Texture binding
  layer texture;
  /// Storage texture binding
  layer storage_texture;
  /// External texture binding
  layer external_texture;

  own use
  {
    buffer_type,
    texture_type,
    sampler_type,
    storage_texture_type,
    external_texture_type
  };

  exposed use
  {
    BindingType
  };
}

/// Internal namespace.
mod private
{
  use crate::*;
  use binding_type::*;

  #[ derive( Clone ) ]
  pub enum BindingType
  {
    Buffer( web_sys::GpuBufferBindingLayout ),
    Sampler( web_sys::GpuSamplerBindingLayout ),
    Texture( web_sys::GpuTextureBindingLayout ),
    StorageTexture( web_sys::GpuStorageTextureBindingLayout ),
    ExternalTexture( web_sys::GpuExternalTextureBindingLayout ),
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


  pub fn buffer() -> BufferBindingLayout
  {
    BufferBindingLayout::new()
  }

  pub fn texture() -> TextureBindingLayout
  {
    TextureBindingLayout::new()
  }

  pub fn sampler() -> SamplerBindingLayout
  {
    SamplerBindingLayout::new()
  }
  
  pub fn storage_texture() -> StorageTextureBindingLayout
  {
    StorageTextureBindingLayout::new()
  }

  pub fn external_texture() -> ExternalTextureBindingLayout
  {
    ExternalTextureBindingLayout
  }
}

crate::mod_interface!
{
  layer buffer;
  layer sampler;
  layer texture;
  layer storage_texture;
  layer external_texture;

  own use
  {
    buffer,
    texture,
    sampler,
    storage_texture,
    external_texture
  };

  exposed use
  {
    BindingType
  };
}

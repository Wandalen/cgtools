/// Internal namespace.
mod private
{
  use crate::*;
  use binding_type::*;

  pub enum BindingType
  {
    Buffer( web_sys::GpuBufferBindingLayout ),
    Sampler( web_sys::GpuSamplerBindingLayout ),
    Texture( web_sys::GpuTextureBindingLayout ),
    StorageTexture( web_sys::GpuStorageTextureBindingLayout ),
    ExternalTexture( web_sys::GpuExternalTextureBindingLayout ),
    Other
  }

  impl From< BufferBindingLayout > for BindingType 
  {
    fn from( value: BufferBindingLayout ) -> Self 
    {
        BindingType::Buffer( value.into() )
    }   
  }

  impl From< SamplerBindingLayout > for BindingType 
  {
    fn from( value: SamplerBindingLayout ) -> Self 
    {
        BindingType::Sampler( value.into() )
    }   
  }

  impl From< TextureBindingLayout > for BindingType 
  {
    fn from( value: TextureBindingLayout ) -> Self 
    {
        BindingType::Texture( value.into() )
    }   
  }

  impl From< StorageTextureBindingLayout > for BindingType 
  {
    fn from( value: StorageTextureBindingLayout ) -> Self 
    {
        BindingType::StorageTexture( value.into() )
    }   
  }

  impl From< ExternalTextureBindingLayout > for BindingType 
  {
    fn from( value: ExternalTextureBindingLayout ) -> Self 
    {
        BindingType::ExternalTexture( value.into() )
    }   
  }

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

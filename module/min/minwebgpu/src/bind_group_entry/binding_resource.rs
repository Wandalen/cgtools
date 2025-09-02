/// Internal namespace.
mod private
{
  use crate::*;

  /// A trait for types that can be used as a WebGPU binding resource.
  pub trait BindingResource
  {
    /// Converts the resource into a `JsValue`.
    fn as_resource( &self ) -> JsValue;
  }

  impl BindingResource for web_sys::GpuBufferBinding 
  {
    fn as_resource( &self ) -> JsValue 
    {
      self.into()
    }
  }

  impl BindingResource for web_sys::GpuTextureView 
  {
    fn as_resource( &self ) -> JsValue 
    {
      self.into()
    }
  }

  impl BindingResource for web_sys::GpuSampler 
  {
    fn as_resource( &self ) -> JsValue 
    {
      self.into()
    }
  }

  impl BindingResource for web_sys::GpuExternalTexture 
  {
    fn as_resource( &self ) -> JsValue 
    {
      self.into()
    }
  }

  impl BindingResource for BufferBinding< '_ > 
  {
    fn as_resource( &self ) -> JsValue {
      Into::< web_sys::GpuBufferBinding >::into( self ).into()
    }    
  }

}

crate::mod_interface!
{
  exposed use
  {
    BindingResource
  };
}

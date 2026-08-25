/// Internal namespace.
mod private
{
  use crate::{ JsValue, web_sys, Into, BufferBinding };

  /// A trait for types that can be used as a WebGPU binding resource.
  pub trait BindingResource
  {
    /// Converts the resource into a `JsValue`.
    fn as_resource( &self ) -> JsValue;
  }

  impl BindingResource for web_sys::GpuBufferBinding 
  {
    #[ inline ]
    fn as_resource( &self ) -> JsValue 
    {
      self.into()
    }
  }

  impl BindingResource for web_sys::GpuTextureView 
  {
    #[ inline ]
    fn as_resource( &self ) -> JsValue 
    {
      self.into()
    }
  }

  impl BindingResource for web_sys::GpuSampler 
  {
    #[ inline ]
    fn as_resource( &self ) -> JsValue 
    {
      self.into()
    }
  }

  impl BindingResource for web_sys::GpuExternalTexture 
  {
    #[ inline ]
    fn as_resource( &self ) -> JsValue 
    {
      self.into()
    }
  }

  impl BindingResource for BufferBinding< '_ > 
  {
    #[ inline ]
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

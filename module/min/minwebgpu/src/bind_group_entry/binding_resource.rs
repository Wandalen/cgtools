/// Internal namespace.
mod private
{
  use crate::*;

  /// A WebGPU bind-group resource: the typed union of everything that can be bound to a
  /// single binding point.
  ///
  /// Modeling this as an enum (rather than a type-erased `JsValue`) lets the conversion to
  /// `web_sys::GpuBindGroupEntry` dispatch to the matching typed `new_with_*` constructor,
  /// preserving compile-time type checking and avoiding a `js_sys::Reflect::set` hop.
  pub enum BindingResource
  {
    /// A whole buffer bound directly.
    Buffer( web_sys::GpuBuffer ),
    /// A buffer binding, i.e. a (sub)range of a buffer.
    BufferBinding( web_sys::GpuBufferBinding ),
    /// A sampler.
    Sampler( web_sys::GpuSampler ),
    /// A texture view.
    TextureView( web_sys::GpuTextureView ),
    /// An external texture (e.g. imported video frame).
    ExternalTexture( web_sys::GpuExternalTexture ),
  }

  /// A trait for types that can be bound as a WebGPU binding resource.
  pub trait AsBindingResource
  {
    /// Converts the resource into the typed [ `BindingResource` ] enum.
    fn as_binding_resource( &self ) -> BindingResource;
  }

  impl AsBindingResource for web_sys::GpuBuffer
  {
    fn as_binding_resource( &self ) -> BindingResource
    {
      BindingResource::Buffer( self.clone() )
    }
  }

  impl AsBindingResource for web_sys::GpuBufferBinding
  {
    fn as_binding_resource( &self ) -> BindingResource
    {
      BindingResource::BufferBinding( self.clone() )
    }
  }

  impl AsBindingResource for web_sys::GpuTextureView
  {
    fn as_binding_resource( &self ) -> BindingResource
    {
      BindingResource::TextureView( self.clone() )
    }
  }

  impl AsBindingResource for web_sys::GpuSampler
  {
    fn as_binding_resource( &self ) -> BindingResource
    {
      BindingResource::Sampler( self.clone() )
    }
  }

  impl AsBindingResource for web_sys::GpuExternalTexture
  {
    fn as_binding_resource( &self ) -> BindingResource
    {
      BindingResource::ExternalTexture( self.clone() )
    }
  }

  impl AsBindingResource for BufferBinding< '_ >
  {
    fn as_binding_resource( &self ) -> BindingResource
    {
      BindingResource::BufferBinding( self.into() )
    }
  }

}

crate::mod_interface!
{
  exposed use
  {
    BindingResource,
    AsBindingResource
  };
}

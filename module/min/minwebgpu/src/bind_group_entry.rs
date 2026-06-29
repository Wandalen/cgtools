/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuBindGroupEntry`.
  pub struct BindGroupEntry
  {
    /// The index of the binding point in the shader.
    ///
    /// This corresponds to the `@group` and `@binding` attributes in the WGSL
    /// shader code.
    binding : u32,
    /// The GPU resource to bind.
    ///
    /// A typed union over buffer / buffer-binding / sampler / texture-view /
    /// external-texture (see [ `BindingResource` ]).
    resource : BindingResource
  }

  impl BindGroupEntry
  {
    /// Creates a new `BindGroupEntry` builder with a given resource.
    pub fn new< T : AsBindingResource >( resource : &T ) -> Self
    {
      let binding = 0;
      let resource = resource.as_binding_resource();
      BindGroupEntry
      {
        binding,
        resource
      }
    }

    /// Sets the binding index for the entry.
    pub fn binding( mut self, binding : u32 ) -> Self
    {
      self.binding = binding;
      self
    }
  }

  impl From< BindGroupEntry > for web_sys::GpuBindGroupEntry
  {
    fn from( value: BindGroupEntry ) -> Self
    {
      // Dispatch to the matching typed constructor so the resource keeps its concrete
      // type all the way to web-sys — no `JsValue` erasure or reflection.
      let binding = value.binding;
      match value.resource
      {
        BindingResource::Buffer( v ) =>
          web_sys::GpuBindGroupEntry::new_with_gpu_buffer( binding, &v ),
        BindingResource::BufferBinding( v ) =>
          web_sys::GpuBindGroupEntry::new_with_gpu_buffer_binding( binding, &v ),
        BindingResource::Sampler( v ) =>
          web_sys::GpuBindGroupEntry::new( binding, &v ),
        BindingResource::TextureView( v ) =>
          web_sys::GpuBindGroupEntry::new_with_gpu_texture_view( binding, &v ),
        BindingResource::ExternalTexture( v ) =>
          web_sys::GpuBindGroupEntry::new_with_gpu_external_texture( binding, &v ),
      }
    }
  }
}

crate::mod_interface!
{
  /// Module for binding resources
  layer binding_resource;
  /// Module for buffer binding
  layer buffer_binding;

  exposed use
  {
    BindGroupEntry
  };
}

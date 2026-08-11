/// Internal namespace.
mod private
{
  use crate::{ JsValue, BindingResource, web_sys, JsCast };

  /// A builder for creating a `web_sys::GpuBindGroupEntry`.
  #[ derive( Default ) ]
  pub struct BindGroupEntry
  {
    // The index of the binding point in the shader.
    ///
    /// This corresponds to the `@group` and `@binding` attributes in the WGSL
    /// shader code.
    binding : u32,
    /// The GPU resource to bind.
    ///
    /// This can be a `GpuBuffer`, `GpuTextureView`, or a `GpuSampler`.
    resource : JsValue
  }

  impl BindGroupEntry 
  {
    /// Creates a new `BindGroupEntry` builder with a given resource.
    #[ inline ]
    pub fn new< T : BindingResource >( resource : &T ) -> Self
    {
      let binding = 0;
      let resource = resource.as_resource();
      BindGroupEntry
      {
        binding,
        resource
      }
    }

    /// Sets the binding index for the entry.
    #[ inline ]
    #[ must_use ]
    pub fn binding( mut self, binding : u32 ) -> Self
    {
      self.binding = binding;
      self
    }   
  }

  impl From< BindGroupEntry > for web_sys::GpuBindGroupEntry 
  {
    #[ inline ]
    fn from( value: BindGroupEntry ) -> Self 
    {
      // `resource` is a dynamically-typed GPU resource (buffer, texture view, or sampler);
      // web-sys only generates a `&GpuSampler`-typed `new()`, so reinterpret the JsValue -
      // the actual JS object passed to WebGPU is unaffected by this Rust-side static type.
      let entry = web_sys::GpuBindGroupEntry::new( value.binding, value.resource.unchecked_ref() );
      entry
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

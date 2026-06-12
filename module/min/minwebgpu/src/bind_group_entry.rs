/// Internal namespace.
mod private
{
  use crate::*;

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
      // `resource` is a type-erased union (buffer binding / sampler / texture view), so the
      // entry is assembled as a plain JS object: web-sys' typed `new`/`set_resource` only
      // accept a single concrete member.
      let entry : web_sys::GpuBindGroupEntry = wasm_bindgen::JsCast::unchecked_into( js_sys::Object::new() );
      entry.set_binding( value.binding );
      let _ = js_sys::Reflect::set( entry.as_ref(), &"resource".into(), &value.resource );
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

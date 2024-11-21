/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Default ) ]
  pub struct BindGroupEntry
  {
    binding : u32,
    resource : JsValue
  }

  impl BindGroupEntry 
  {
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
      let entry = web_sys::GpuBindGroupEntry::new( value.binding, &value.resource );
      entry
    }   
  }
}

crate::mod_interface!
{
  layer binding_resource;
  layer buffer_binding;

  exposed use
  {
    BindGroupEntry
  };
}

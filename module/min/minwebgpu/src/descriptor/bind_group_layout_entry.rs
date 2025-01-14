/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Clone ) ]
  pub struct BindGroupLayoutEntry
  {
    binding : u32,
    visibility : u32,
    ty : BindingType
  }

  impl BindGroupLayoutEntry
  {
    pub fn new() -> Self
    {
      let binding = 0;
      let visibility = 0;
      let ty = BindingType::Other;

      BindGroupLayoutEntry
      {
        binding,
        visibility,
        ty
      }
    }

    /// Sets the `visibility` to `All`
    pub fn all( self ) -> Self
    {
      self.fragment().compute().vertex()
    }

    /// Sets the visibility of the entry to VERTEX
    pub fn vertex( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::VERTEX;
      self
    }

    /// Sets the visibility of the entry to FRAGMENT
    pub fn fragment( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::FRAGMENT;
      self
    }

    /// Sets the visibility of the entry to COMPUTE
    pub fn compute( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::COMPUTE;
      self
    }
    
    /// Sets the binding value of the entry
    pub fn binding( mut self, binding : u32 ) -> Self
    {
      self.binding = binding;
      self
    }
    
    /// Sets the type of the entry
    pub fn ty( mut self, ty : impl Into< BindingType > ) -> Self
    {
      self.ty = ty.into();
      self
    }
  }

  impl From< BindGroupLayoutEntry > for web_sys::GpuBindGroupLayoutEntry
  {
    fn from( value: BindGroupLayoutEntry ) -> Self 
    {
      let layout = web_sys::GpuBindGroupLayoutEntry::new( value.binding, value.visibility );

      match &value.ty
      {
        BindingType::Buffer( buffer ) => layout.set_buffer( &buffer ),
        BindingType::Sampler( sampler ) => layout.set_sampler( &sampler ),
        BindingType::Texture( texture ) => layout.set_texture( &texture ),
        BindingType::StorageTexture( texture ) => layout.set_storage_texture( &texture ),
        BindingType::ExternalTexture( texture ) => layout.set_external_texture( &texture ),
        BindingType::Other => panic!( "The type of the binding entry was not set" ) 
      }

      layout
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    BindGroupLayoutEntry
  };
}

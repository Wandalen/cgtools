/// Internal namespace.
mod private
{

  use crate::*;

  /// Describes the layout for a WebGPU bind group.
  #[ derive( Clone ) ]
  pub struct  BindGroupLayoutDescriptor
  {
    /// Auto compute `binding` value of the entries. Defaults to `false`
    auto_bindings : bool,
    /// Visibility, that is shared between the entries. Defaults to `0`
    visibility : u32,
    /// Layouts of the entries of the bindgroup
    entries: Vec< web_sys::GpuBindGroupLayoutEntry >
  }

  impl BindGroupLayoutDescriptor
  {
    /// Creates a new `BindGroupLayoutDescriptor` with default values.
    pub fn new() -> Self
    {
      let auto_bindings = false;
      let visibility = 0;
      let entries = Vec::new();
      BindGroupLayoutDescriptor
      {
        auto_bindings,
        visibility,
        entries
      }
    }

    /// Set the `auto_bindings` property to `true`
    pub fn auto_bindings( mut self ) -> Self
    {
      self.auto_bindings = true;
      self
    }

    /// Sets the `visibility` to `All`
    pub fn all( self ) -> Self
    {
      self.fragment().compute().vertex()
    }

    /// Add `FRAGMENT` stage to the `visibility`
    pub fn fragment( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::FRAGMENT;
      self
    }

    /// Add `VERTEX` stage to the `visibility`
    pub fn vertex( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::VERTEX;
      self
    }

    /// Add `COMPUTE` stage to the `visibility`
    pub fn compute( mut self ) -> Self
    {
      self.visibility |= web_sys::gpu_shader_stage::COMPUTE;
      self
    }

    /// Adds an entry to the layout
    pub fn entry( mut self, entry : impl Into< web_sys::GpuBindGroupLayoutEntry > ) -> Self
    {
      self.entries.push( entry.into() );
      self
    }

    /// Adds an entry to the layout
    pub fn entry_from_ty( mut self, ty : impl Into< BindingType > ) -> Self
    {
      let entry = BindGroupLayoutEntry::new().ty( ty );
      self.entries.push( entry.into() );
      self
    }

    /// Creates a `web_sys::GpuBindGroupLayout` from this descriptor.
    pub fn create( self, device : &web_sys::GpuDevice ) -> Result< web_sys::GpuBindGroupLayout, WebGPUError >
    {
      layout::bind_group::create( device, &self.into() )
    } 
  }

  impl From< BindGroupLayoutDescriptor > for web_sys::GpuBindGroupLayoutDescriptor 
  {
    fn from( mut value: BindGroupLayoutDescriptor ) -> Self 
    {
      let mut binding : u32 = 0;
      for entry in value.entries.iter_mut()
      {
        if value.auto_bindings 
        { 
          entry.set_binding( binding ); 
          binding += 1;
        }

        entry.set_visibility( entry.get_visibility() | value.visibility );
      }

      let layout = web_sys::GpuBindGroupLayoutDescriptor::new( &value.entries.into() );
      layout
    }    
  }

}

crate::mod_interface!
{

  exposed use
  {
    BindGroupLayoutDescriptor
  };

}

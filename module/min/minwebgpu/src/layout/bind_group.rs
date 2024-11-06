/// Internal namespace.
mod private
{

  use crate::*;

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

  pub fn create
  ( 
    device : &web_sys::GpuDevice,
    desc : impl Into< web_sys::GpuBindGroupLayoutDescriptor >
  ) -> Result< web_sys::GpuBindGroupLayout, WebGPUError >
  {
    let layout = device.create_bind_group_layout( &desc.into() )
    .map_err( | e | DeviceError::FailedToCreateBindGroupLayout( format!( "{:?}", e ) ) )?;
    Ok( layout ) 
  }

  pub fn desc() -> BindGroupLayoutDescriptor
  {
    BindGroupLayoutDescriptor::new()
  }

  pub fn entry() -> BindGroupLayoutEntry
  {
    BindGroupLayoutEntry::new()
  }
}

crate::mod_interface!
{

  own use
  {
    create,
    desc,
    entry
  };

  exposed use
  {
    BindGroupLayoutDescriptor
  };

}

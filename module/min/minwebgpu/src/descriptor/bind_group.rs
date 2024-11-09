/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Clone ) ]
  pub struct BindGroupDescriptor< 'a >
  {
    layout : &'a web_sys::GpuBindGroupLayout,
    entries : Vec< web_sys::GpuBindGroupEntry >,
    label : Option< &'a str >,
    auto_bindings : bool
  }

  impl< 'a > BindGroupDescriptor< 'a >
  {
    pub fn new( layout : &'a web_sys::GpuBindGroupLayout ) -> Self
    {
      let entries = Vec::new();
      let label = None;
      let auto_bindings = false;
      BindGroupDescriptor
      {
        layout,
        entries,
        label,
        auto_bindings
      }
    }

    pub fn auto_bindings( mut self ) -> Self
    {
      self.auto_bindings = true;
      self
    }

    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    pub fn entry( mut self, entry : impl Into< web_sys::GpuBindGroupEntry > ) -> Self
    {
      self.entries.push( entry.into() );
      self
    }

    pub fn entry_from_resource< T : BindingResource>( self, resource : &T ) -> Self
    {
      let entry = BindGroupEntry::new( resource );
      self.entry( entry )
    }

    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuBindGroup
    {
      device.create_bind_group( &self.into() )
    }
  }

  impl From< BindGroupDescriptor< '_ > > for web_sys::GpuBindGroupDescriptor 
  {
    fn from( mut value: BindGroupDescriptor< '_ > ) -> Self {
      if value.auto_bindings
      {
        let mut binding = 0;
        for e in value.entries.iter_mut()
        {
          e.set_binding( binding );
          binding += 1;
        }
      }

      let desc = web_sys::GpuBindGroupDescriptor::new( &value.entries.into() , value.layout );

      if let Some( v ) = value.label { desc.set_label( v ); }

      desc
    }    
  }
}

crate::mod_interface!
{
  exposed use
  {
    BindGroupDescriptor
  };
}

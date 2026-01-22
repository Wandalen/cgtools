/// Internal namespace.
mod private
{
  use crate::*;

  /// Describes the configuration for creating a WebGPU bind group.
  #[ derive( Clone ) ]
  pub struct BindGroupDescriptor< 'a >
  {
    /// A reference to the `GpuBindGroupLayout` that this bind group must conform to.
    /// The layout defines the structure and types of the resources in the group.
    layout : &'a web_sys::GpuBindGroupLayout,
    /// A list of `GpuBindGroupEntry` objects, each representing a single resource
    /// binding within the group.
    entries : Vec< web_sys::GpuBindGroupEntry >,
    /// An optional label for the bind group, which can be useful for debugging and
    /// performance tracing tools.
    label : Option< &'a str >,
    /// A flag that, if `true`, automatically assigns the `binding` value for each entry
    /// sequentially, starting from `0`. This simplifies the creation process.
    auto_bindings : bool
  }

  impl< 'a > BindGroupDescriptor< 'a >
  {
    /// Creates a new `BindGroupDescriptor` for a given bind group layout.
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

    /// Sets the `auto_bindings` property to `true`.
    pub fn auto_bindings( mut self ) -> Self
    {
      self.auto_bindings = true;
      self
    }

    /// Sets an optional label for the bind group.
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Adds a `web_sys::GpuBindGroupEntry` to the descriptor.
    pub fn entry( mut self, entry : impl Into< web_sys::GpuBindGroupEntry > ) -> Self
    {
      self.entries.push( entry.into() );
      self
    }

    /// Creates a `GpuBindGroupEntry` from a resource and adds it to the descriptor.
    pub fn entry_from_resource< T : BindingResource >( self, resource : &T ) -> Self
    {
      let entry = BindGroupEntry::new( resource );
      self.entry( entry )
    }

    /// Creates a `web_sys::GpuBindGroup` from this descriptor.
    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuBindGroup
    {
      bind_group::create( device, &self.into() )
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

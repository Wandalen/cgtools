/// Internal namespace.
mod private
{
  use crate::{ web_sys, BindGroupDescriptor };

  /// Creates a new bind group descriptor builder.
  #[ inline ]
  #[ must_use ]
  pub fn desc( layout : &web_sys::GpuBindGroupLayout ) -> BindGroupDescriptor< '_ >
  {
    BindGroupDescriptor::new( layout )
  }

  /// Creates a new GPU bind group.
  #[ inline ]
  #[ must_use ]
  pub fn create( device : &web_sys::GpuDevice, desc : &web_sys::GpuBindGroupDescriptor ) -> web_sys::GpuBindGroup
  {
    device.create_bind_group( desc )
  }
}

crate::mod_interface!
{

  own use
  {
    desc,
    create
  };

}

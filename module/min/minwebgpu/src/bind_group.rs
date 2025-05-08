/// Internal namespace.
mod private
{
  use crate::*;

  pub fn desc< 'a >( layout : &'a web_sys::GpuBindGroupLayout ) -> BindGroupDescriptor< 'a >
  {
    BindGroupDescriptor::new( layout )
  }

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

/// Internal namespace.
mod private
{

  use crate::*;

  pub fn create
  ( 
    device : &web_sys::GpuDevice,
    desc : &web_sys::GpuBindGroupLayoutDescriptor
  ) -> Result< web_sys::GpuBindGroupLayout, WebGPUError >
  {
    let layout = device.create_bind_group_layout( desc )
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

}

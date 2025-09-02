/// Internal namespace.
mod private
{

  use crate::*;

  /// Creates a new GPU bind group layout.
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

  /// Creates a new, empty bind group layout descriptor builder.
  pub fn desc() -> BindGroupLayoutDescriptor
  {
    BindGroupLayoutDescriptor::new()
  }

  /// Creates a new, empty bind group layout entry builder.
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

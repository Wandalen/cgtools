/// Internal namespace.
mod private
{

  use crate::{ web_sys, WebGPUError, DeviceError, BindGroupLayoutDescriptor, BindGroupLayoutEntry };

  /// Creates a new GPU bind group layout.
  ///
  /// # Errors
  /// Returns `error::DeviceError::FailedToCreateBindGroupLayout` if the underlying
  /// `GPUDevice.createBindGroupLayout` call throws.
  #[ inline ]
  pub fn create
  ( 
    device : &web_sys::GpuDevice,
    desc : &web_sys::GpuBindGroupLayoutDescriptor
  ) -> Result< web_sys::GpuBindGroupLayout, WebGPUError >
  {
    let layout = device.create_bind_group_layout( desc )
    .map_err( | e | DeviceError::FailedToCreateBindGroupLayout( format!( "{e:?}" ) ) )?;
    Ok( layout ) 
  }

  /// Creates a new, empty bind group layout descriptor builder.
  #[ inline ]
  #[ must_use ]
  pub fn desc() -> BindGroupLayoutDescriptor
  {
    BindGroupLayoutDescriptor::new()
  }

  /// Creates a new, empty bind group layout entry builder.
  #[ inline ]
  #[ must_use ]
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

/// Internal namespace.
mod private
{
  use crate::web_sys;

  /// Represents the layout for a WebGPU external texture binding.
  #[ derive( Clone ) ]
  #[ non_exhaustive ]
  pub struct ExternalTextureBindingLayout;

  impl From< ExternalTextureBindingLayout > for web_sys::GpuExternalTextureBindingLayout
  {
    #[ inline ]
    fn from( _value: ExternalTextureBindingLayout ) -> Self 
    {
      web_sys::GpuExternalTextureBindingLayout::new()
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ExternalTextureBindingLayout
  };
}

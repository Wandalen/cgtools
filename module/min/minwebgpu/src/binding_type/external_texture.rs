/// Internal namespace.
mod private
{
  use crate::*;

  pub struct ExternalTextureBindingLayout;

  impl From< ExternalTextureBindingLayout > for web_sys::GpuExternalTextureBindingLayout
  {
    fn from( value: ExternalTextureBindingLayout ) -> Self 
    {
      ( &value ).into()
    }
  }

  impl From< &ExternalTextureBindingLayout > for web_sys::GpuExternalTextureBindingLayout
  {
    fn from( _value: &ExternalTextureBindingLayout ) -> Self 
    {
      let layout = web_sys::GpuExternalTextureBindingLayout::new();
      layout
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

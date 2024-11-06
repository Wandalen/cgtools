/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Default ) ]
  pub struct SamplerBindingLayout
  {
    /// Defaults to `Filtering`
    s_type : Option< GpuSamplerBindingType >,
  }

  impl SamplerBindingLayout
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the type of the sampler from the provided type
    pub fn set_type( mut self, s_type : web_sys::GpuSamplerBindingType ) -> Self
    {
      self.s_type = Some( s_type );
      self
    }

    /// Sets the type of the sampler to `Filtering`
    pub fn filtering( mut self ) -> Self
    {
      self.s_type = Some( GpuSamplerBindingType::Filtering );
      self
    }

    /// Sets the type of the sampler to `NonFiltering`
    pub fn non_filtering( mut self ) -> Self
    {
      self.s_type = Some( GpuSamplerBindingType::NonFiltering );
      self
    }

    /// Sets the type of the sampler to `Comparison`
    pub fn comparison( mut self ) -> Self
    {
      self.s_type = Some( GpuSamplerBindingType::Comparison );
      self
    }
  }

  impl From< SamplerBindingLayout > for web_sys::GpuSamplerBindingLayout
  {
    fn from( value: SamplerBindingLayout ) -> Self 
    {
      ( &value ).into()
    }
  }

  impl From< &SamplerBindingLayout > for web_sys::GpuSamplerBindingLayout
  {
    fn from( value: &SamplerBindingLayout ) -> Self 
    {
      let layout = web_sys::GpuSamplerBindingLayout::new();

      if let Some( value ) = value.s_type { layout.set_type( value ); }

      layout
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    SamplerBindingLayout
  };
}

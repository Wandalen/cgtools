/// Internal namespace.
mod private
{
  use crate::{ GpuSamplerBindingType, web_sys };

  /// Represents the layout for a WebGPU sampler binding.
  #[ derive( Default, Clone ) ]
  pub struct SamplerBindingLayout
  {
    /// The type of sampler binding.
    /// 
    /// Defaults to `Filtering`
    s_type : Option< GpuSamplerBindingType >,
  }

  impl SamplerBindingLayout
  {
    /// Creates a new `SamplerBindingLayout` with default values.
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the type of the sampler from the provided type
    #[ inline ]
    #[ must_use ]
    pub fn type_set( mut self, s_type : web_sys::GpuSamplerBindingType ) -> Self
    {
      self.s_type = Some( s_type );
      self
    }

    /// Sets the type of the sampler to `Filtering`
    #[ inline ]
    #[ must_use ]
    pub fn filtering( mut self ) -> Self
    {
      self.s_type = Some( GpuSamplerBindingType::Filtering );
      self
    }

    /// Sets the type of the sampler to `NonFiltering`
    #[ inline ]
    #[ must_use ]
    pub fn non_filtering( mut self ) -> Self
    {
      self.s_type = Some( GpuSamplerBindingType::NonFiltering );
      self
    }

    /// Sets the type of the sampler to `Comparison`
    #[ inline ]
    #[ must_use ]
    pub fn comparison( mut self ) -> Self
    {
      self.s_type = Some( GpuSamplerBindingType::Comparison );
      self
    }
  }

  impl From< SamplerBindingLayout > for web_sys::GpuSamplerBindingLayout
  {
    #[ inline ]
    fn from( value: SamplerBindingLayout ) -> Self 
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

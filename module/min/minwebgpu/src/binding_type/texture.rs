/// Internal namespace.
mod private
{
  use crate::{ GpuTextureSampleType, GpuTextureViewDimension, web_sys };

  /// Represents the layout for a WebGPU texture binding.
  #[ derive( Default, Clone ) ]
  pub struct TextureBindingLayout
  {
    /// Indicates whether the texture is multisampled.
    /// 
    /// Defaults to `false`
    multisampled : Option< bool >,
    /// The type of data stored in the texture samples.
    /// 
    /// Defaults to `float`
    sample_type : Option< GpuTextureSampleType >,
    /// The dimension of the texture view.
    ///
    /// Defaults to `2d`
    view_dimension : Option< GpuTextureViewDimension >
  }

  impl TextureBindingLayout {
    /// Creates a new `TextureBindingLayout` with default values.
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the `multisampled` property to `true`
    #[ inline ]
    #[ must_use ]
    pub fn multisampled( mut self ) -> Self
    {
      self.multisampled = Some( true );
      self
    }

    /// Sets the `sample_type` of the texture to the provided type
    #[ inline ]
    #[ must_use ]
    pub fn sample_type( mut self, s_type : GpuTextureSampleType ) -> Self
    {
      self.sample_type = Some( s_type );
      self
    }

    /// Sets the `sample_type` of the texture to `Float`
    #[ inline ]
    #[ must_use ]
    pub fn sample_float( mut self ) -> Self
    {
      self.sample_type = Some( GpuTextureSampleType::Float );
      self
    }

    /// Sets the `sample_type` of the texture to `UnfilterableFloat`
    #[ inline ]
    #[ must_use ]
    pub fn sample_unfilterable_float( mut self ) -> Self
    {
      self.sample_type = Some( GpuTextureSampleType::UnfilterableFloat );
      self
    }

    /// Sets the `sample_type` of the texture to `Depth`
    #[ inline ]
    #[ must_use ]
    pub fn sample_depth( mut self ) -> Self
    {
      self.sample_type = Some( GpuTextureSampleType::Depth );
      self
    }

    /// Sets the `sample_type` of the texture to `Sint`
    #[ inline ]
    #[ must_use ]
    pub fn sample_sint( mut self ) -> Self
    {
      self.sample_type = Some( GpuTextureSampleType::Sint );
      self
    }

    /// Sets the `sample_type` of the texture to `Uint`
    #[ inline ]
    #[ must_use ]
    pub fn sample_uint( mut self ) -> Self
    {
      self.sample_type = Some( GpuTextureSampleType::Uint );
      self
    }

    /// Sets the `view_dimension` of the texture to the provided type
    #[ inline ]
    #[ must_use ]
    pub fn view_dimension( mut self, dimension : GpuTextureViewDimension ) -> Self
    {
      self.view_dimension = Some( dimension );
      self
    }

    /// Sets the `view_dimension` of the texture to `N1d`
    #[ inline ]
    #[ must_use ]
    pub fn view_1d( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::N1d );
      self
    }

    /// Sets the `view_dimension` of the texture to `N2d`
    #[ inline ]
    #[ must_use ]
    pub fn view_2d( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::N2d );
      self
    }

    /// Sets the `view_dimension` of the texture to `N2dArray`
    #[ inline ]
    #[ must_use ]
    pub fn view_2d_array( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::N2dArray );
      self
    }

    /// Sets the `view_dimension` of the texture to `Cube`
    #[ inline ]
    #[ must_use ]
    pub fn view_cube( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::Cube );
      self
    }

    /// Sets the `view_dimension` of the texture to `CubeArray`
    #[ inline ]
    #[ must_use ]
    pub fn view_cube_array( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::CubeArray );
      self
    }

    /// Sets the `view_dimension` of the texture to `N3d`
    #[ inline ]
    #[ must_use ]
    pub fn view_3d( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::N3d );
      self
    }
  }

  impl From< TextureBindingLayout > for web_sys::GpuTextureBindingLayout
  {
    #[ inline ]
    fn from( value: TextureBindingLayout ) -> Self 
    {
      let layout = web_sys::GpuTextureBindingLayout::new();

      if let Some( v ) = value.multisampled { layout.set_multisampled( v ); }
      if let Some( v ) = value.sample_type { layout.set_sample_type( v ); }
      if let Some( v ) = value.view_dimension { layout.set_view_dimension( v ); }

      layout
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    TextureBindingLayout
  };
}

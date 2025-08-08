/// Internal namespace.
mod private
{
  use crate::*;

  /// Represents the layout for a WebGPU storage texture binding.
  #[ derive( Clone ) ]
  pub struct StorageTextureBindingLayout
  {
    /// The texture format that the storage texture must have.
    ///
    /// Defaults to `Rgba8unormSrgb`
    format : GpuTextureFormat,
    /// The access mode for the storage texture.
    ///
    /// Defaults to `WriteOnly`
    access : Option< GpuStorageTextureAccess >,
    /// The dimension of the texture view.
    /// 
    /// Defaults to `2d`
    view_dimension : Option< GpuTextureViewDimension >
  }

  impl StorageTextureBindingLayout {
    /// Creates a new `StorageTextureBindingLayout` with default values.
    pub fn new() -> Self
    {
      let format = GpuTextureFormat::Rgba8unormSrgb;
      let access = None;
      let view_dimension = None;

      StorageTextureBindingLayout
      {
        format,
        access,
        view_dimension
      }
    }

    /// Sets the `access` property to `ReadOnly`
    pub fn read_only( mut self ) -> Self
    {
      self.access = Some( GpuStorageTextureAccess::ReadOnly );
      self
    }

    /// Sets the `access` property to `ReadOnly`
    pub fn write_only( mut self ) -> Self
    {
      self.access = Some( GpuStorageTextureAccess::WriteOnly );
      self
    }

    /// Sets the `access` property to `ReadWrite`
    pub fn read_write( mut self ) -> Self
    {
      self.access = Some( GpuStorageTextureAccess::ReadWrite );
      self
    }

    /// Sets the `format` of the texture to the provided format
    pub fn format( mut self, format : GpuTextureFormat ) -> Self
    {
      self.format = format;
      self
    }

    /// Sets the `view_dimension` of the texture to the provided type
    pub fn view_dimension( mut self, dimension : GpuTextureViewDimension ) -> Self
    {
      self.view_dimension = Some( dimension );
      self
    }

    /// Sets the `view_dimension` of the texture to `N1d`
    pub fn view_1d( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::N1d );
      self
    }

    /// Sets the `view_dimension` of the texture to `N2d`
    pub fn view_2d( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::N2d );
      self
    }

    /// Sets the `view_dimension` of the texture to `N2dArray`
    pub fn view_2d_array( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::N2dArray );
      self
    }

    /// Sets the `view_dimension` of the texture to `Cube`
    pub fn view_cube( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::Cube );
      self
    }

    /// Sets the `view_dimension` of the texture to `CubeArray`
    pub fn view_cube_array( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::CubeArray );
      self
    }

    /// Sets the `view_dimension` of the texture to `N3d`
    pub fn view_3d( mut self ) -> Self
    {
      self.view_dimension = Some( GpuTextureViewDimension::N3d );
      self
    }
  }

  impl From< StorageTextureBindingLayout > for web_sys::GpuStorageTextureBindingLayout
  {
    fn from( value: StorageTextureBindingLayout ) -> Self 
    {
      let layout = web_sys::GpuStorageTextureBindingLayout::new( value.format );

      if let Some( v ) = value.access { layout.set_access( v ); }
      if let Some( v ) = value.view_dimension { layout.set_view_dimension( v ); }

      layout
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    StorageTextureBindingLayout
  };
}

/// Internal namespace.
mod private
{
  use  crate::*;

  #[ derive( Clone ) ]
  /// Builder struct for the GpuTextureDescriptor.
  pub struct TextureDescriptor< 'a >
  {
    /// The way texture is going to be used.
    usage : u32,
    /// Size of the texture in the form [ width, height, depth_or_array_layers ]
    size : [ u32; 3 ],
    /// Texture's format. Default: Rgba8unormSrgb
    format : GpuTextureFormat,
    /// Label for the texture. Used when an error occurs.
    label : Option< &'a str >,
    /// Dimension of the texture: 1d, 2d, 3d. Default: 2d
    dimension : Option< GpuTextureDimension>,
    /// Mip map levels of the texture. Default: 1
    mip_level : Option< u32 >,
    /// Amount of samples of the texture. Default: 1
    sample_count : Option< u32 >,
    /// Texture format's that are allowed to be used when calling create_view(). Default: []
    view_formats : Vec< GpuTextureFormat >
  }

  impl< 'a > TextureDescriptor< 'a > {
    pub fn new() -> Self
    {
      let format = web_sys::GpuTextureFormat::Rgba8unormSrgb;
      let usage = 0;
      let mip_level = None;
      let sample_count = None;
      let dimension = None;
      let label = None;
      let view_formats = Vec::new();
      let size = [ 0, 0, 0 ];

      TextureDescriptor
      {
        usage,
        size,
        format,
        mip_level,
        sample_count,
        view_formats,
        dimension,
        label
      }
    }

    /// Sets the size of the texture
    pub fn size( mut self, size : [ u32; 3 ] ) -> Self
    {
      self.size = size;
      self
    }

    /// Sets the format of the texture
    pub fn format( mut self, format : GpuTextureFormat ) -> Self
    {
      self.format = format;
      self
    }

    /// Sets the label for the texture
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Sets the mip map level
    pub fn mip_level( mut self, mip_level : u32 ) -> Self
    {
      self.mip_level = Some( mip_level );
      self
    }

    /// Sets the sample count
    pub fn sample_count( mut self, sample_count : u32 ) -> Self
    {
      self.sample_count = Some( sample_count );
      self
    }

    /// Sets the dimension of the texture
    pub fn dimension( mut self, dimension : GpuTextureDimension ) -> Self
    {
      self.dimension = Some( dimension );
      self
    }

    /// Adds view formats
    pub fn view_formats( mut self, formats : &[ web_sys::GpuTextureFormat ] ) -> Self
    {
      self.view_formats.extend_from_slice( &formats );
      self
    }

    /// Sets the usage flag to COPY_DST
    pub fn copy_dst( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::COPY_DST;
      self
    }

    /// Sets the usage flag to COPY_SRC
    pub fn copy_src( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::COPY_SRC;
      self
    }

    /// Sets the usage flag to RENDER_ATTACHMENT
    pub fn render_attachment( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::RENDER_ATTACHMENT;
      self
    }

    /// Sets the usage flag to STORAGE_BINDING
    pub fn storage_binding( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::STORAGE_BINDING;
      self
    }

    /// Sets the usage flag to TEXTURE_BINDING
    pub fn texture_binding( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::TEXTURE_BINDING;
      self
    } 
  }

  impl From< TextureDescriptor< '_ > > for web_sys::GpuTextureDescriptor 
  {
    fn from( value: TextureDescriptor< '_ > ) -> Self 
    {
      let desc = web_sys::GpuTextureDescriptor::new
      (
        value.format, 
        &Vec::from( value.size ).into(), 
        value.usage
      );

      if let Some( v ) = value.mip_level { desc.set_mip_level_count( v ); }
      if let Some( v ) = value.sample_count { desc.set_sample_count( v ); }
      if let Some( v ) = value.dimension { desc.set_dimension( v ); }
      if let Some( v ) = value.label { desc.set_label( v ); }

      if value.view_formats.len() > 0
      {
        let view_formats : Vec< u32 > = value.view_formats.into_iter().map( | f | f as u32 ).collect();
        desc.set_view_formats( &view_formats.into() );
      }

      desc
    }    
  }

  impl From< &TextureDescriptor< '_ > > for web_sys::GpuTextureDescriptor 
  {
    fn from( value: &TextureDescriptor< '_ > ) -> Self 
    {
      let desc = web_sys::GpuTextureDescriptor::new
      (
        value.format, 
        &Vec::from( value.size ).into(), 
        value.usage
      );

      if let Some( v ) = value.mip_level { desc.set_mip_level_count( v ); }
      if let Some( v ) = value.sample_count { desc.set_sample_count( v ); }
      if let Some( v ) = value.dimension { desc.set_dimension( v ); }
      if let Some( v ) = value.label { desc.set_label( v ); }

      if value.view_formats.len() > 0
      {
        let view_formats : Vec< u32 > = value.view_formats.iter().copied().map( | f | f as u32 ).collect();
        desc.set_view_formats( &view_formats.into() );
      }

      desc
    }    
  }
}

crate::mod_interface!
{
  exposed use
  {
    TextureDescriptor
  };
}
  

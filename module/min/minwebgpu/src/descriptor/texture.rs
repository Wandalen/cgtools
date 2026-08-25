/// Internal namespace.
mod private
{
  use  crate::{ GpuTextureFormat, GpuTextureDimension, web_sys, WebGPUError, texture, Into, js_sys, IntoIterator, JsCast, wasm_bindgen };

  #[ derive( Clone ) ]
  /// Builder struct for the GpuTextureDescriptor.
  pub struct TextureDescriptor< 'a >
  {
    /// The way texture is going to be used.
    usage : u32,
    /// Size of the texture in the form [ width, height, depth_or_array_layers ]
    size : [ u32; 3 ],
    /// Texture's format. Default: Rgba8unorm
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

  impl Default for TextureDescriptor< '_ >
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl< 'a > TextureDescriptor< 'a >
  {
    /// Creates a new `TextureDescriptor` with default values.
    #[ inline ]
    #[ must_use ]
    // BUG-300 task/bug/300_texture_descriptor_default_format_not_storage_capable.md -- was
    // `Rgba8unormSrgb`, incompatible with `.storage_binding()` usage.
    // Fix(BUG-300): default `format` changed from `web_sys::GpuTextureFormat::Rgba8unormSrgb` to
    // `web_sys::GpuTextureFormat::Rgba8unorm`.
    // Root cause: this builder's `format` default must stay valid across every usage flag it can
    // produce -- including `.storage_binding()` -- but per the WebGPU spec's texture format
    // capability table, no `-srgb` format supports `STORAGE_BINDING` usage. A caller chaining
    // `.storage_binding()` without an explicit `.format(..)` override got a format/usage
    // combination `GPUDevice.createTexture` rejects only via an async device error-scope event,
    // never a synchronous throw, so `texture::create`'s `.map_err(..)` (which only catches
    // synchronous throws) silently returned `Ok` for an unusable texture.
    // Pitfall: a single default shared across every usage flag a builder can produce must be
    // valid for the narrowest usage class among them, not just the most common one.
    pub fn new() -> Self
    {
      let format = web_sys::GpuTextureFormat::Rgba8unorm;
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
        label,
        dimension,
        mip_level,
        sample_count,
        view_formats
      }
    }

    /// Sets the size of the texture
    #[ inline ]
    #[ must_use ]
    pub fn size( mut self, size : [ u32; 3 ] ) -> Self
    {
      self.size = size;
      self
    }

    /// Sets the format of the texture
    #[ inline ]
    #[ must_use ]
    pub fn format( mut self, format : GpuTextureFormat ) -> Self
    {
      self.format = format;
      self
    }

    /// Sets the label for the texture
    #[ inline ]
    #[ must_use ]
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Sets the mip map level
    #[ inline ]
    #[ must_use ]
    pub fn mip_level( mut self, mip_level : u32 ) -> Self
    {
      self.mip_level = Some( mip_level );
      self
    }

    /// Sets the sample count
    #[ inline ]
    #[ must_use ]
    pub fn sample_count( mut self, sample_count : u32 ) -> Self
    {
      self.sample_count = Some( sample_count );
      self
    }

    /// Sets the dimension of the texture
    #[ inline ]
    #[ must_use ]
    pub fn dimension( mut self, dimension : GpuTextureDimension ) -> Self
    {
      self.dimension = Some( dimension );
      self
    }

    /// Adds view formats
    #[ inline ]
    #[ must_use ]
    pub fn view_formats( mut self, formats : &[ web_sys::GpuTextureFormat ] ) -> Self
    {
      self.view_formats.extend_from_slice( formats );
      self
    }

    /// Sets the usage flag to COPY_DST
    #[ inline ]
    #[ must_use ]
    pub fn copy_dst( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::COPY_DST;
      self
    }

    /// Sets the usage flag to COPY_SRC
    #[ inline ]
    #[ must_use ]
    pub fn copy_src( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::COPY_SRC;
      self
    }

    /// Sets the usage flag to RENDER_ATTACHMENT
    #[ inline ]
    #[ must_use ]
    pub fn render_attachment( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::RENDER_ATTACHMENT;
      self
    }

    /// Sets the usage flag to STORAGE_BINDING
    #[ inline ]
    #[ must_use ]
    pub fn storage_binding( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::STORAGE_BINDING;
      self
    }

    /// Sets the usage flag to TEXTURE_BINDING
    #[ inline ]
    #[ must_use ]
    pub fn texture_binding( mut self ) -> Self
    {
      self.usage |= web_sys::gpu_texture_usage::TEXTURE_BINDING;
      self
    } 

    /// Creates a synchronous texture on the GPU.
    ///
    /// # Errors
    /// Returns `error::DeviceError::FailedToCreateTexture` if the underlying
    /// `GPUDevice.createTexture` call throws (see [`texture::create`]).
    #[ inline ]
    pub fn create
    ( 
      self,
      device : &web_sys::GpuDevice
    ) -> Result< web_sys::GpuTexture, WebGPUError >
    {

      texture::create( device, &self.into() )
    }
  }

  impl From< TextureDescriptor< '_ > > for web_sys::GpuTextureDescriptor 
  {
    #[ inline ]
    fn from( value: TextureDescriptor< '_ > ) -> Self 
    {
      let size : Vec< js_sys::Number > = value.size.into_iter().map( js_sys::Number::from ).collect();
      let desc = web_sys::GpuTextureDescriptor::new
      (
        value.format,
        &size,
        value.usage
      );

      if let Some( v ) = value.mip_level { desc.set_mip_level_count( v ); }
      if let Some( v ) = value.sample_count { desc.set_sample_count( v ); }
      if let Some( v ) = value.dimension { desc.set_dimension( v ); }
      if let Some( v ) = value.label { desc.set_label( v ); }

      if !value.view_formats.is_empty()
      {
        let view_formats : Vec< js_sys::JsString > = value.view_formats.into_iter()
        .map( | f | wasm_bindgen::JsValue::from( f ).unchecked_into() )
        .collect();
        desc.set_view_formats( &view_formats );
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
  

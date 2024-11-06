/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Default ) ]
  pub struct SamplerDescriptor< 'a >
  {
    ///Defaults to `None`
    label : Option< &'a str >,
    /// Defaults to `ClampToEdge`
    address_mode_u : Option< GpuAddressMode >,
    /// Defaults to `ClampToEdge`
    address_mode_v : Option< GpuAddressMode >,
    /// Defaults to `ClampToEdge`
    address_mode_w : Option< GpuAddressMode >,
    /// If specified, the sampler will be a comparison sampler of the specified type
    compare : Option< GpuCompareFunction >,
    /// Defaults to `Nearest`
    min_filter : Option< GpuFilterMode >,
    /// Defaults to `Nearest`
    mag_filter : Option< GpuFilterMode >,
    /// Defaults to `Nearest`
    mipmap_filter : Option< GpuMipmapFilterMode >,
    /// Defaults to 0
    lod_min : Option< f32 >,
    /// Defaults to 32
    lod_max : Option< f32 >,
    /// Defaults to 1
    anisotropy_max : Option< u16 >
  }

  impl< 'a > SamplerDescriptor< 'a >
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the label for the sampler
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Sets the address mode for u, v and w to ClampToEdge
    pub fn clamp_to_edge( mut self ) -> Self
    {
      self.address_mode_u = Some( web_sys::GpuAddressMode::ClampToEdge );
      self.address_mode_v = Some( web_sys::GpuAddressMode::ClampToEdge );
      self.address_mode_w = Some( web_sys::GpuAddressMode::ClampToEdge );
      self
    }

    /// Sets the address mode for u to ClampToEdge
    pub fn clamp_to_edge_u( mut self ) -> Self
    {
      self.address_mode_u = Some( web_sys::GpuAddressMode::ClampToEdge );
      self
    }

    /// Sets the address mode for v to ClampToEdge
    pub fn clamp_to_edge_v( mut self ) -> Self
    {
      self.address_mode_v = Some( web_sys::GpuAddressMode::ClampToEdge );
      self
    }

    /// Sets the address mode for w to ClampToEdge
    pub fn clamp_to_edge_w( mut self ) -> Self
    {
      self.address_mode_w = Some( web_sys::GpuAddressMode::ClampToEdge );
      self
    }

    /// Sets the address mode for u, v and w to Repeat
    pub fn repeat( mut self ) -> Self
    {
      self.address_mode_u = Some( web_sys::GpuAddressMode::Repeat );
      self.address_mode_v = Some( web_sys::GpuAddressMode::Repeat );
      self.address_mode_w = Some( web_sys::GpuAddressMode::Repeat );
      self
    }

    /// Sets the address mode for u to Repeat
    pub fn repeat_u( mut self ) -> Self
    {
      self.address_mode_u = Some( web_sys::GpuAddressMode::Repeat );
      self
    }

    /// Sets the address mode for v to Repeat
    pub fn repeat_v( mut self ) -> Self
    {
      self.address_mode_v = Some( web_sys::GpuAddressMode::Repeat );
      self
    }

    /// Sets the address mode for w to Repeat
    pub fn repeat_w( mut self ) -> Self
    {
      self.address_mode_w = Some( web_sys::GpuAddressMode::Repeat );
      self
    }

    /// Sets the address mode for u, v and w to MirrorRepeat
    pub fn mirror_repeat( mut self ) -> Self
    {
      self.address_mode_u = Some( web_sys::GpuAddressMode::MirrorRepeat );
      self.address_mode_v = Some( web_sys::GpuAddressMode::MirrorRepeat );
      self.address_mode_w = Some( web_sys::GpuAddressMode::MirrorRepeat );
      self
    }

    /// Sets the address mode for u to MirrorRepeat
    pub fn mirror_repeat_u( mut self ) -> Self
    {
      self.address_mode_u = Some( web_sys::GpuAddressMode::MirrorRepeat );
      self
    }

    /// Sets the address mode for v to MirrorRepeat
    pub fn mirror_repeat_v( mut self ) -> Self
    {
      self.address_mode_v = Some( web_sys::GpuAddressMode::MirrorRepeat );
      self
    }

    /// Sets the address mode for w to MirrorRepeat
    pub fn mirror_repeat_w( mut self ) -> Self
    {
      self.address_mode_w = Some( web_sys::GpuAddressMode::MirrorRepeat );
      self
    }

    /// Sets the filter mode for minification and magnification to Nearest
    pub fn nearest( mut self ) -> Self
    {
      self.min_filter = Some( web_sys::GpuFilterMode::Nearest );
      self.mag_filter = Some( web_sys::GpuFilterMode::Nearest );
      self
    }

    /// Sets the filter mode for minification to Nearest
    pub fn nearest_min( mut self ) -> Self
    {
      self.min_filter = Some( web_sys::GpuFilterMode::Nearest );
      self
    }

    /// Sets the filter mode for magnification to Nearest
    pub fn nearest_mag( mut self ) -> Self
    {
      self.mag_filter = Some( web_sys::GpuFilterMode::Nearest );
      self
    }

    /// Sets the filter mode for minification and magnification to Linear
    pub fn linear( mut self ) -> Self
    {
      self.min_filter = Some( web_sys::GpuFilterMode::Linear );
      self.mag_filter = Some( web_sys::GpuFilterMode::Linear );
      self
    }

    /// Sets the filter mode for minification to Linear
    pub fn linear_min( mut self ) -> Self
    {
      self.min_filter = Some( web_sys::GpuFilterMode::Linear );
      self
    }

    /// Sets the filter mode for magnification to Linear
    pub fn linear_mag( mut self ) -> Self
    {
      self.mag_filter = Some( web_sys::GpuFilterMode::Linear );
      self
    }

    /// Sets the filter mode for mip map to Nearest
    pub fn nearest_mip( mut self ) -> Self
    {
      self.mipmap_filter = Some( web_sys::GpuMipmapFilterMode::Nearest );
      self
    }

    /// Sets the filter mode for mip map to Linear
    pub fn linear_mip( mut self ) -> Self
    {
      self.mipmap_filter = Some( web_sys::GpuMipmapFilterMode::Linear );
      self
    }

    /// Sets the minimun level of detail used internally when sampling a texture
    pub fn lod_min( mut self, lod_min : f32 ) -> Self
    {
      self.lod_min = Some( lod_min );
      self
    }

    /// Sets the maximum level of detail used internally when sampling a texture
    pub fn lod_max( mut self, lod_max : f32 ) -> Self
    {
      self.lod_max = Some( lod_max );
      self
    }

    /// Sets the maximum anisotropy value clamp used by the sampler
    pub fn anisotropy_max( mut self, a_max : u16 ) -> Self
    {
      self.anisotropy_max = Some( a_max );
      self
    }

    /// Sets the compare function used by the sampler
    pub fn compare( mut self, compare_func : GpuCompareFunction ) -> Self
    {
      self.compare = Some( compare_func );
      self
    }
  }

  impl From< SamplerDescriptor< '_ > > for web_sys::GpuSamplerDescriptor 
  {
    fn from( value: SamplerDescriptor< '_ > ) -> Self 
    {
      ( &value ).into()
    }
  }

  impl From< &SamplerDescriptor< '_ > > for web_sys::GpuSamplerDescriptor 
  {
    fn from( value: &SamplerDescriptor< '_ > ) -> Self 
    {
      let descriptor = web_sys::GpuSamplerDescriptor::new();

      if let Some( v ) = value.address_mode_u { descriptor.set_address_mode_u( v ); }
      if let Some( v ) = value.address_mode_v { descriptor.set_address_mode_v( v ); }
      if let Some( v ) = value.address_mode_w { descriptor.set_address_mode_w( v ); }
      if let Some( v ) = value.compare { descriptor.set_compare( v ); }
      if let Some( v ) = value.min_filter { descriptor.set_min_filter( v ); }
      if let Some( v ) = value.mag_filter { descriptor.set_mag_filter( v ); }
      if let Some( v ) = value.mipmap_filter { descriptor.set_mipmap_filter( v ); }
      if let Some( v ) = value.lod_min { descriptor.set_lod_min_clamp( v ); }
      if let Some( v ) = value.lod_max { descriptor.set_lod_max_clamp( v ); }
      if let Some( v ) = value.anisotropy_max { descriptor.set_max_anisotropy( v ); }
      
      descriptor
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    SamplerDescriptor
  };
}

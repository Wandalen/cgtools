/// Internal namespace.
mod private
{
  use web_sys::gpu_color_write;
  use crate::*;

  /// A builder for creating a `web_sys::GpuColorTargetState`.
  #[ derive( Clone ) ]
  pub struct ColorTargetState
  {
    /// The texture format of the color target.
    ///
    /// This must match the format of the texture view used in the render pass.
    ///
    /// Defaults to `GpuTextureFormat::Rgba8unormSrgb`.
    format :GpuTextureFormat,
    /// An optional blend state configuration.
    ///
    /// This defines how the output of the fragment shader is combined with the
    /// existing color in the render target. If `None`, blending is disabled.
    ///
    /// Defaults to `None`.
    blend : Option< BlendState >,
    /// A bitmask that controls which color channels (R, G, B, A) can be written to.
    ///
    /// The bitmask is a combination of `gpu_color_write::RED`, `GREEN`, `BLUE`,
    /// and `ALPHA`.
    ///
    /// Defaults to `gpu_color_write::ALL`.
    write_mask : Option< u32 >
  }

  impl From< ColorTargetState > for web_sys::GpuColorTargetState 
  {
    fn from( value: ColorTargetState ) -> Self 
    {
      let state = web_sys::GpuColorTargetState::new( value.format );
      if let Some( v ) = value.blend { state.set_blend( &v.into() ); } 
      if let Some( v ) = value.write_mask { state.set_write_mask( v ); } 
      state
    }    
  }

  impl ColorTargetState
  {
    /// Creates a new `ColorTargetState` builder with default values.
    pub fn new() -> Self
    {
      let blend = None;
      let write_mask = None; 
      let format = GpuTextureFormat::Rgba8unormSrgb;

      ColorTargetState
      {
        format,
        blend,
        write_mask
      }
    }

    /// Sets the color target's texture format.
    pub fn format( mut self, format : GpuTextureFormat ) -> Self
    {
      self.format = format;
      self
    }

    /// Sets the blend state configuration.
    pub fn blend( mut self, blend : BlendState ) -> Self
    {
      self.blend = Some( blend );
      self
    }

    /// Sets the write mask to allow writing to all color channels.
    pub fn write_all( mut self ) -> Self
    {
      self.write_mask = Some( gpu_color_write::ALL );
      self
    }

    /// Enables writing to the red channel.
    pub fn write_r( mut self ) -> Self
    {
      self.write_mask = add_mask( self.write_mask, gpu_color_write::RED );
      self
    }

    /// Enables writing to the green channel.
    pub fn write_g( mut self ) -> Self
    {
      self.write_mask = add_mask( self.write_mask, gpu_color_write::GREEN );
      self
    }

    /// Enables writing to the blue channel.
    pub fn write_b( mut self ) -> Self
    {
      self.write_mask = add_mask( self.write_mask, gpu_color_write::BLUE );
      self
    }

    /// Enables writing to the alpha channel.
    pub fn write_a( mut self ) -> Self
    {
      self.write_mask = add_mask( self.write_mask, gpu_color_write::ALPHA );
      self
    }
  }

  fn add_mask( mask : Option< u32 >, value : u32 ) -> Option< u32 >
  {
    if mask.is_some() 
    {
      mask.map( | m | m | value )
    }
    else 
    {
      Some( value )   
    }
  }
}

crate::mod_interface!
{

  exposed use
  {
    ColorTargetState
  };

}

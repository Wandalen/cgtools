/// Internal namespace.
mod private
{
  use web_sys::gpu_color_write;

  use crate::*;
  #[ derive( Clone ) ]
  pub struct ColorTargetState
  {
    /// Defaults to `Rgba8unormSrgb`
    format :GpuTextureFormat,
    /// Defaults to `None`
    blend : Option< BlendState >,
    /// Defaults to `ALL`
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

    pub fn blend( mut self, blend : BlendState ) -> Self
    {
      self.blend = Some( blend );
      self
    }

    pub fn write_all( mut self ) -> Self
    {
      self.write_mask = Some( gpu_color_write::ALL );
      self
    }

    pub fn write_r( mut self ) -> Self
    {
      self.write_mask = add_mask( self.write_mask, gpu_color_write::RED );
      self
    }

    pub fn write_g( mut self ) -> Self
    {
      self.write_mask = add_mask( self.write_mask, gpu_color_write::GREEN );
      self
    }

    pub fn write_b( mut self ) -> Self
    {
      self.write_mask = add_mask( self.write_mask, gpu_color_write::BLUE );
      self
    }

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

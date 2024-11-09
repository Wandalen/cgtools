/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Clone ) ]
  pub struct DepthStencilState
  {
    /// Defaults to `Depth24plus`
    format : GpuTextureFormat,
    /// Defaults to `Less`
    depth_compare : GpuCompareFunction,
    /// Defaults to 0
    depth_bias : Option< i32 >,
    /// Defaults to 0
    depth_bias_clamp : Option< f32 >,
    /// Defaults to 0
    depth_bias_slope_scale : Option< f32 >,
    /// Defaults to true
    depth_write_enabled : Option< bool >,
    /// Defaults to None
    stencil_back : Option< StencilFaceState >,
    /// Defaults to None
    stencil_front : Option< StencilFaceState >,
    /// Defaults to 0xFFFFFFFF
    stencil_read_mask : Option< u32 >,
    /// Defaults to 0xFFFFFFFF
    stencil_write_mask : Option< u32 >
  }

  impl DepthStencilState 
  {
    pub fn new() -> Self
    {
      let format = GpuTextureFormat::Depth24plus;
      let depth_compare = GpuCompareFunction::Less;
      let depth_bias = None;
      let depth_bias_clamp = None;
      let depth_bias_slope_scale = None;
      let depth_write_enabled = None;
      let stencil_back = None;
      let stencil_front = None;
      let stencil_read_mask = None;
      let stencil_write_mask = None;

      DepthStencilState
      {
        format,
        depth_compare,
        depth_bias,
        depth_bias_clamp,
        depth_bias_slope_scale,
        depth_write_enabled,
        stencil_back,
        stencil_front,
        stencil_read_mask,
        stencil_write_mask
      }
    } 

    pub fn format( mut self, format : GpuTextureFormat ) -> Self
    {
      self.format = format;
      self
    }  

    pub fn depth_compare( mut self, compare : GpuCompareFunction ) -> Self
    {
      self.depth_compare = compare;
      self
    } 

    pub fn depth_bias( mut self, bias : i32 ) -> Self
    {
      self.depth_bias = Some( bias );
      self
    } 

    pub fn depth_bias_clamp( mut self, clamp : f32 ) -> Self
    {
      self.depth_bias_clamp = Some( clamp );
      self
    }

    pub fn depth_bias_slope_scale( mut self, scale : f32 ) -> Self
    {
      self.depth_bias_slope_scale = Some( scale );
      self
    }

    pub fn disable_depth_write( mut self ) -> Self
    {
      self.depth_write_enabled = Some( false );
      self
    }

    pub fn stencil_back( mut self, stencil : StencilFaceState ) -> Self
    {
      self.stencil_back = Some( stencil );
      self
    }

    pub fn stencil_front( mut self, stencil : StencilFaceState ) -> Self
    {
      self.stencil_front = Some( stencil );
      self
    }

    pub fn stencil_read_mask( mut self, mask : u32 ) -> Self
    {
      self.stencil_read_mask = Some( mask );
      self
    }

    pub fn stencil_write_mask( mut self, mask : u32 ) -> Self
    {
      self.stencil_write_mask = Some( mask );
      self
    }
  }

  impl From< DepthStencilState > for web_sys::GpuDepthStencilState 
  {
    fn from( value: DepthStencilState ) -> Self 
    {
      let state = web_sys::GpuDepthStencilState::new( value.format );

      state.set_depth_compare( value.depth_compare );
      if let Some( v ) = value.depth_bias { state.set_depth_bias( v ); }
      if let Some( v ) = value.depth_bias_clamp { state.set_depth_bias_clamp( v ); }
      if let Some( v ) = value.depth_bias_slope_scale { state.set_depth_bias_slope_scale( v ); }
      if let Some( v ) = value.depth_write_enabled { state.set_depth_write_enabled( v ); }
      if let Some( v ) = value.stencil_back { state.set_stencil_back( &v.into() ); }
      if let Some( v ) = value.stencil_front { state.set_stencil_front( &v.into() ); }
      if let Some( v ) = value.stencil_read_mask { state.set_stencil_read_mask( v ); }
      if let Some( v ) = value.stencil_write_mask { state.set_stencil_write_mask( v ); }

      state
    }   
  }
}

crate::mod_interface!
{

  exposed use
  {
    DepthStencilState
  };

}

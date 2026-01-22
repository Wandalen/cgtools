/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuDepthStencilState`.
  #[ derive( Clone ) ]
  pub struct DepthStencilState
  {
    /// The texture format of the depth-stencil attachment.
    ///
    /// This must match the format of the texture view used in the render pass.
    ///
    /// Defaults to `GpuTextureFormat::Depth24plus`.
    format : GpuTextureFormat,
    /// The comparison function used for depth testing.
    ///
    /// This determines whether a new fragment's depth value should be written
    /// to the depth buffer based on its comparison with the existing value.
    ///
    /// Defaults to `GpuCompareFunction::Less`.
    depth_compare : GpuCompareFunction,
    /// A flag to enable or disable writing to the depth buffer.
    ///
    /// If `true`, the depth value of a fragment that passes the depth test will
    /// be written to the depth buffer.
    ///
    /// Defaults to `true`.
    depth_write_enabled : bool,
    /// A constant value added to the depth value of a fragment.
    ///
    /// This is used for depth biasing to prevent "z-fighting" artifacts.
    ///
    /// Defaults to `0`.
    depth_bias : Option< i32 >,
    /// The maximum depth bias value.
    ///
    /// This clamps the depth bias to a maximum value.
    ///
    /// Defaults to `0`.
    depth_bias_clamp : Option< f32 >,
    /// A scale factor applied to the depth bias.
    ///
    /// This is based on the slope of the fragment's depth value.
    ///
    /// Defaults to `0`.
    depth_bias_slope_scale : Option< f32 >,
    /// Stencil state configuration for fragments that face away from the camera.
    ///
    /// This must be provided if stencil testing is enabled.
    ///
    /// Defaults to `None`.
    stencil_back : Option< StencilFaceState >,
    /// Stencil state configuration for fragments that face towards the camera.
    ///
    /// This must be provided if stencil testing is enabled.
    ///
    /// Defaults to `None`.
    stencil_front : Option< StencilFaceState >,
    /// A bitmask that is ANDed with the stencil reference value and the value
    /// in the stencil buffer during stencil testing.
    ///
    /// Defaults to `0xFFFFFFFF`.
    stencil_read_mask : Option< u32 >,
    /// A bitmask that is ANDed with the stencil value before it is written to
    /// the stencil buffer.
    ///
    /// Defaults to `0xFFFFFFFF`.F
    stencil_write_mask : Option< u32 >
  }

  impl DepthStencilState 
  {
    /// Creates a new `DepthStencilState` with default values.
    pub fn new() -> Self
    {
      let format = GpuTextureFormat::Depth24plus;
      let depth_compare = GpuCompareFunction::Less;
      let depth_write_enabled = true;

      let depth_bias = None;
      let depth_bias_clamp = None;
      let depth_bias_slope_scale = None;
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

    /// Sets the format of the depth-stencil texture.
    pub fn format( mut self, format : GpuTextureFormat ) -> Self
    {
      self.format = format;
      self
    }  

    /// Sets the depth comparison function.
    pub fn depth_compare( mut self, compare : GpuCompareFunction ) -> Self
    {
      self.depth_compare = compare;
      self
    } 

    /// Sets the constant depth bias value.
    pub fn depth_bias( mut self, bias : i32 ) -> Self
    {
      self.depth_bias = Some( bias );
      self
    } 

    /// Sets the depth bias clamp value.
    pub fn depth_bias_clamp( mut self, clamp : f32 ) -> Self
    {
      self.depth_bias_clamp = Some( clamp );
      self
    }

    /// Sets the depth bias slope scale.
    pub fn depth_bias_slope_scale( mut self, scale : f32 ) -> Self
    {
      self.depth_bias_slope_scale = Some( scale );
      self
    }

    /// Disables writing to the depth buffer.
    pub fn disable_depth_write( mut self ) -> Self
    {
      self.depth_write_enabled = false;
      self
    }

    /// Sets the stencil state for back-facing fragments.
    pub fn stencil_back( mut self, stencil : StencilFaceState ) -> Self
    {
      self.stencil_back = Some( stencil );
      self
    }

    /// Sets the stencil state for front-facing fragments.
    pub fn stencil_front( mut self, stencil : StencilFaceState ) -> Self
    {
      self.stencil_front = Some( stencil );
      self
    }

    /// Sets the stencil read mask.
    pub fn stencil_read_mask( mut self, mask : u32 ) -> Self
    {
      self.stencil_read_mask = Some( mask );
      self
    }

    /// Sets the stencil write mask.
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
      state.set_depth_write_enabled( value.depth_write_enabled );

      if let Some( v ) = value.depth_bias { state.set_depth_bias( v ); }
      if let Some( v ) = value.depth_bias_clamp { state.set_depth_bias_clamp( v ); }
      if let Some( v ) = value.depth_bias_slope_scale { state.set_depth_bias_slope_scale( v ); }
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

/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuMultisampleState`.
  #[ derive( Default, Clone ) ]
  pub struct MultiSampleState
  {
    /// A flag to enable or disable alpha-to-coverage.
    ///
    /// When `true`, the alpha value of the fragment shader output is used to
    /// mask the multisample coverage, which is useful for rendering transparent
    /// textures.
    ///
    /// Defaults to `false`.
    alpha_to_coverage_enabled : Option< bool >,
    /// The number of samples per pixel for multisample anti-aliasing.
    ///
    /// Common values are 1, 4, or 8. A value of 1 means no multisampling.
    ///
    /// Defaults to `1`.
    count : Option< u32 >,
    /// A bitmask that is ANDed with the sample coverage mask.
    ///
    /// This can be used to explicitly disable certain samples from being written to.
    ///
    /// Defaults to `0xFFFFFFFF` (all bits set).
    mask : Option< u32 >,
  }

  impl MultiSampleState 
  {
    /// Creates a new `MultiSampleState` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the sample mask.
    pub fn mask( mut self, mask : u32 ) -> Self
    {
      self.mask = Some( mask );
      self
    }

    /// Sets the number of samples per pixel.
    pub fn count( mut self, count : u32 ) -> Self
    {
      self.count = Some( count );
      self
    }

    /// Enables alpha-to-coverage.
    pub fn enable_alpha_to_coverage( mut self ) -> Self
    {
      self.alpha_to_coverage_enabled = Some( true );
      self
    }
  }

  impl From< MultiSampleState > for web_sys::GpuMultisampleState 
  {
    fn from ( value: MultiSampleState ) -> Self 
    {
      let state = web_sys::GpuMultisampleState::new();

      if let Some( v ) = value.alpha_to_coverage_enabled { state.set_alpha_to_coverage_enabled( v ); }
      if let Some( v ) = value.count { state.set_count( v ); }
      if let Some( v ) = value.mask { state.set_mask( v ); }

      state
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    MultiSampleState
  };
}

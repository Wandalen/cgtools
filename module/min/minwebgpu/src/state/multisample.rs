/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Default, Clone ) ]
  pub struct MultiSampleState
  {
    /// Defaults to `false`
    alpha_to_coverage_enabled : Option< bool >,
    /// Defaults to '1'
    count : Option< u32 >,
    /// Defaults to '0xFFFFFFFF'
    mask : Option< u32 >,
  }

  impl MultiSampleState 
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn mask( mut self, mask : u32 ) -> Self
    {
      self.mask = Some( mask );
      self
    }

    pub fn count( mut self, count : u32 ) -> Self
    {
      self.count = Some( count );
      self
    }

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

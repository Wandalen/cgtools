/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Default ) ]
  pub struct StencilFaceState
  {
    /// Defaults to `Always`
    compare : Option< GpuCompareFunction >,
    /// Defaults to 'Keep'
    depth_fail_op : Option< GpuStencilOperation >,
    /// Defaults to 'Keep'
    pass_op : Option< GpuStencilOperation >,
    /// Defaults to 'Keep'
    fail_op : Option< GpuStencilOperation >
  }

  impl StencilFaceState 
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn compare( mut self, compare : GpuCompareFunction ) -> Self
    {
      self.compare = Some( compare );
      self
    }

    pub fn depth_fail_op( mut self, op : GpuStencilOperation ) -> Self
    {
      self.depth_fail_op = Some( op );
      self
    }

    pub fn pass_op( mut self, op : GpuStencilOperation ) -> Self
    {
      self.pass_op = Some( op );
      self
    }

    pub fn fail_op( mut self, op : GpuStencilOperation ) -> Self
    {
      self.fail_op = Some( op );
      self
    }
  }

  impl From< StencilFaceState > for web_sys::GpuStencilFaceState 
  {
    fn from( value: StencilFaceState ) -> Self 
    {
      let state = web_sys::GpuStencilFaceState::new();

      if let Some( v ) = value.compare { state.set_compare( v ); }
      if let Some( v ) = value.depth_fail_op { state.set_depth_fail_op( v ); }
      if let Some( v ) = value.pass_op { state.set_pass_op( v ); }
      if let Some( v ) = value.fail_op { state.set_fail_op( v ); }

      state
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    StencilFaceState
  };
}

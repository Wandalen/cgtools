/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuStencilFaceState`.
  #[ derive( Default, Clone ) ]
  pub struct StencilFaceState
  {
    /// The comparison function used for the stencil test.
    ///
    /// The stencil test compares a reference value against the value in the stencil
    /// buffer. If this test passes, the stencil operations are executed.
    ///
    /// Defaults to `GpuCompareFunction::Always`.
    compare : Option< GpuCompareFunction >,
    /// The stencil operation to perform if the depth test fails.
    ///
    /// This is only relevant if the stencil test passes but the subsequent depth
    /// test fails.
    ///
    /// Defaults to `GpuStencilOperation::Keep`.
    depth_fail_op : Option< GpuStencilOperation >,
    /// The stencil operation to perform if both the stencil and depth tests pass.
    ///
    /// Defaults to `GpuStencilOperation::Keep`.
    pass_op : Option< GpuStencilOperation >,
    /// The stencil operation to perform if the stencil test fails.
    ///
    /// Defaults to `GpuStencilOperation::Keep`.
    fail_op : Option< GpuStencilOperation >
  }

  impl StencilFaceState 
  {
    /// Creates a new `StencilFaceState` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets the stencil comparison function.
    pub fn compare( mut self, compare : GpuCompareFunction ) -> Self
    {
      self.compare = Some( compare );
      self
    }

    /// Sets the operation for when the depth test fails.
    pub fn depth_fail_op( mut self, op : GpuStencilOperation ) -> Self
    {
      self.depth_fail_op = Some( op );
      self
    }

    /// Sets the operation for when both stencil and depth tests pass.
    pub fn pass_op( mut self, op : GpuStencilOperation ) -> Self
    {
      self.pass_op = Some( op );
      self
    }

    /// Sets the operation for when the stencil test fails.
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

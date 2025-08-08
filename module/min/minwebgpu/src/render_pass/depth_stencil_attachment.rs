/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuRenderPassDepthStencilAttachment`.
  #[ derive( Clone ) ]
  pub struct DepthStencilAttachment< 'a >
  {
    /// The texture view that will be used for depth and/or stencil testing.
    /// This is a required field.
    view : &'a web_sys::GpuTextureView,
    /// The value to which the depth buffer will be cleared if `depth_load_op` is `Clear`.
    ///
    /// Defaults to `1.0`.
    depth_clear_value : f32,
    /// The operation to perform on the depth buffer at the beginning of the render pass.
    ///
    /// This can be `Clear` to set the depth to a specific value, or `Load`
    /// to preserve its existing contents.
    ///
    /// Defaults to `GpuLoadOp::Clear`.
    depth_load_op : GpuLoadOp,
    /// The operation to perform on the depth buffer at the end of the render pass.
    ///
    /// This can be `Store` to write the depth results to the texture, or
    /// `Discard` to discard them.
    ///
    /// Defaults to `GpuStoreOp::Store`.
    depth_store_op : GpuStoreOp,
    /// A flag indicating whether the depth buffer is read-only.
    ///
    /// If `true`, the depth buffer cannot be written to during the render pass.
    ///
    /// Defaults to `false`.
    depth_read_only : bool,
    /// The operation to perform on the stencil buffer at the beginning of the render pass.
    ///
    /// This must be set by the user if the `view` has a stencil component.
    stencil_load_op : Option< GpuLoadOp >,
    /// The operation to perform on the stencil buffer at the end of the render pass.
    ///
    /// This must be set by the user if the `view` has a stencil component.
    stencil_store_op : Option< GpuStoreOp >,
    /// The value to which the stencil buffer will be cleared if `stencil_load_op`
    /// is `Clear`.
    ///
    /// Defaults to `0`.
    stencil_clear_value : Option< u32 >,
    /// A flag indicating whether the stencil buffer is read-only.
    ///
    /// If `true`, the stencil buffer cannot be written to during the render pass.
    ///
    /// Defaults to `false`.
    stencil_read_only : bool
  }

  impl< 'a > DepthStencilAttachment< 'a  > 
  {
    /// Creates a new `DepthStencilAttachment` builder with a required texture view.
    pub fn new( view : &'a web_sys::GpuTextureView ) -> Self
    {
      let depth_clear_value = 1.0;
      let depth_load_op = GpuLoadOp::Clear;
      let depth_store_op = GpuStoreOp::Store;
      let depth_read_only = false;

      let stencil_load_op = None;
      let stencil_store_op = None;
      let stencil_clear_value = None;
      let stencil_read_only = false;

      DepthStencilAttachment
      {
        view,
        depth_clear_value,
        depth_load_op,
        depth_store_op,
        depth_read_only,
        stencil_clear_value,
        stencil_load_op,
        stencil_store_op,
        stencil_read_only
      }
    }  

    /// Sets the depth clear value.
    pub fn depth_clear_value( mut self, value : f32 ) -> Self
    {
      self.depth_clear_value = value;
      self
    } 

    /// Sets the depth store operation.
    pub fn depth_store_op( mut self, op : GpuStoreOp ) -> Self
    {
      self.depth_store_op = op;
      self
    } 

    /// Sets the depth load operation.
    pub fn depth_load_op( mut self, op : GpuLoadOp ) -> Self
    {
      self.depth_load_op = op;
      self
    } 

    /// Sets whether the depth buffer is read-only.
    pub fn depth_read_only( mut self, value : bool ) -> Self
    {
      self.depth_read_only = value;
      self
    } 

    /// Sets the stencil clear value.
    pub fn stencil_clear_value( mut self, value : u32 ) -> Self
    {
      self.stencil_clear_value = Some( value );
      self
    } 

    /// Sets the stencil store operation.
    pub fn stencil_store_op( mut self, op : GpuStoreOp ) -> Self
    {
      self.stencil_store_op = Some( op );
      self
    } 

    /// Sets the stencil load operation.
    pub fn stencil_load_op( mut self, op : GpuLoadOp ) -> Self
    {
      self.stencil_load_op = Some( op );
      self
    } 

    /// Sets whether the stencil buffer is read-only.
    pub fn stencil_read_only( mut self, value : bool ) -> Self
    {
      self.stencil_read_only = value;
      self
    }
  }

  impl From< DepthStencilAttachment< '_ > > for web_sys::GpuRenderPassDepthStencilAttachment 
  {
    fn from( value: DepthStencilAttachment< '_ > ) -> Self 
    {
      let a = web_sys::GpuRenderPassDepthStencilAttachment::new( value.view );

      a.set_depth_clear_value( value.depth_clear_value );
      a.set_depth_read_only( value.depth_read_only ); 
      if !value.depth_read_only 
      { 
        a.set_depth_load_op( value.depth_load_op );
        a.set_depth_store_op( value.depth_store_op );
      }

      if let Some( v ) = value.stencil_clear_value { a.set_stencil_clear_value( v ); }
      a.set_stencil_read_only( value.stencil_read_only );
      if !value.stencil_read_only
      {
        if let Some( v ) = value.stencil_load_op { a.set_stencil_load_op( v ); }
        if let Some( v ) = value.stencil_store_op { a.set_stencil_store_op( v ); }
      }

      a
    }   
  }
}

crate::mod_interface!
{
  exposed use
  {
    DepthStencilAttachment
  };
}

/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Clone ) ]
  pub struct DepthStencilAttachment< 'a >
  {
    view : &'a web_sys::GpuTextureView,
    /// Defaults to `1.0`
    depth_clear_value : Option< f32 >,
    /// Defaults to `Clear`
    depth_load_op : Option< GpuLoadOp >,
    /// Defaults to `Store`
    depth_store_op : Option< GpuStoreOp >,
    /// Defaults to `false`
    depth_read_only : Option< bool >,
    /// Defaults to `0`
    stencil_clear_value : Option< u32 >,
    /// Defaults to `clear`
    stencil_load_op : Option< GpuLoadOp >,
    /// Defaults to `store`
    stencil_store_op : Option< GpuStoreOp >,
    /// Defaults to `false`
    stencil_read_only : Option< bool >
  }

  impl< 'a > DepthStencilAttachment< 'a  > 
  {
    pub fn new( view : &'a web_sys::GpuTextureView ) -> Self
    {
      let depth_clear_value = None;
      let depth_load_op = None;
      let depth_store_op = None;
      let depth_read_only = None;
      let stencil_clear_value = None;
      let stencil_load_op = None;
      let stencil_store_op = None;
      let stencil_read_only = None;

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

    pub fn depth_clear_value( mut self, value : f32 ) -> Self
    {
      self.depth_clear_value = Some( value );
      self
    } 

    pub fn depth_store_op( mut self, op : GpuStoreOp ) -> Self
    {
      self.depth_store_op = Some( op );
      self
    } 

    pub fn depth_load_op( mut self, op : GpuLoadOp ) -> Self
    {
      self.depth_load_op = Some( op );
      self
    } 

    pub fn depth_read_only( mut self, value : bool ) -> Self
    {
      self.depth_read_only = Some( value );
      self
    } 

    pub fn stencil_clear_value( mut self, value : u32 ) -> Self
    {
      self.stencil_clear_value = Some( value );
      self
    } 

    pub fn stencil_store_op( mut self, op : GpuStoreOp ) -> Self
    {
      self.stencil_store_op = Some( op );
      self
    } 

    pub fn stencil_load_op( mut self, op : GpuLoadOp ) -> Self
    {
      self.stencil_load_op = Some( op );
      self
    } 

    pub fn stencil_read_only( mut self, value : bool ) -> Self
    {
      self.stencil_read_only = Some( value );
      self
    }
  }

  impl From< DepthStencilAttachment< '_ > > for web_sys::GpuRenderPassDepthStencilAttachment 
  {
    fn from( value: DepthStencilAttachment< '_ > ) -> Self 
    {
      let a = web_sys::GpuRenderPassDepthStencilAttachment::new( value.view );

      if let Some( v ) = value.depth_clear_value { a.set_depth_clear_value( v ); }
      if let Some( v ) = value.depth_load_op { a.set_depth_load_op( v ); }
      if let Some( v ) = value.depth_store_op { a.set_depth_store_op( v ); }
      if let Some( v ) = value.depth_read_only { a.set_depth_read_only( v ); }

      if let Some( v ) = value.stencil_clear_value { a.set_stencil_clear_value( v ); }
      if let Some( v ) = value.stencil_load_op { a.set_stencil_load_op( v ); }
      if let Some( v ) = value.stencil_store_op { a.set_stencil_store_op( v ); }
      if let Some( v ) = value.stencil_read_only { a.set_stencil_read_only( v ); }

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

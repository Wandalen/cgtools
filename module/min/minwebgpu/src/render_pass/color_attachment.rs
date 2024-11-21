/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Clone) ]
  pub struct ColorAttachment< 'a >
  {
    view : &'a web_sys::GpuTextureView, 
    /// Defaults to `Clear`
    load_op : GpuLoadOp,
    /// Defaults to `Store`
    store_op : GpuStoreOp,
    /// Defaults to [ 0.0, 0.0, 0.0, 0.0 ]
    clear_value : Option< [ f32; 4 ] >,
    /// Defaults to `None`
    resolve_target : Option< &'a web_sys::GpuTextureView >,
    /// Defaults to `None`
    depth_slice : Option< u32 >
  }

  impl< 'a > ColorAttachment< 'a > {
    pub fn new( view : &'a web_sys::GpuTextureView ) -> Self
    {
      let load_op = GpuLoadOp::Clear;
      let store_op = GpuStoreOp::Store;
      let clear_value = None;
      let resolve_target = None;
      let depth_slice = None;

      ColorAttachment
      {
        view,
        load_op,
        store_op,
        clear_value,
        resolve_target,
        depth_slice
      }
    }

    pub fn load_op( mut self, op : GpuLoadOp ) -> Self
    {
      self.load_op = op;
      self
    }

    pub fn store_op( mut self, op : GpuStoreOp ) -> Self
    {
      self.store_op = op;
      self
    }

    pub fn clear_value( mut self, color : [ f32; 4 ] ) -> Self
    {
      self.clear_value = Some( color );
      self
    }

    pub fn resolve_target( mut self, view : &'a web_sys::GpuTextureView ) -> Self
    {
      self.resolve_target = Some( view );
      self
    }

    pub fn depth_slice( mut self, id : u32 ) -> Self
    {
      self.depth_slice = Some( id );
      self
    }
  }

  impl From< ColorAttachment< '_ > > for web_sys::GpuRenderPassColorAttachment
  {
    fn from( value: ColorAttachment< '_ > ) -> Self 
    {
      let a =  web_sys::GpuRenderPassColorAttachment::new( value.load_op, value.store_op, value.view);

      if let Some( v ) = value.clear_value { a.set_clear_value( &Vec::from( v ).into() ); }
      if let Some( v ) = value.resolve_target { a.set_resolve_target( &v ); }
      if let Some( v ) = value.depth_slice { a.set_depth_slice( v ); }

      a
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    ColorAttachment
  };
}

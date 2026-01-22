/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuRenderPassColorAttachment`.
  #[ derive( Clone) ]
  pub struct ColorAttachment< 'a >
  {
    /// The texture view that will be rendered into. This is a required field.
    view : &'a web_sys::GpuTextureView, 
    /// The operation to perform on the color attachment at the beginning of the render pass.
    ///
    /// This can be `Clear` to clear the attachment to a specific color, or `Load`
    /// to preserve the existing contents.
    ///
    /// Defaults to `GpuLoadOp::Clear`.
    load_op : GpuLoadOp,
    /// The operation to perform on the color attachment at the end of the render pass.
    ///
    /// This can be `Store` to write the rendered results to the texture, or
    /// `Discard` to discard them.
    ///
    /// Defaults to `GpuStoreOp::Store`.
    store_op : GpuStoreOp,
    /// The color value to use when `load_op` is set to `GpuLoadOp::Clear`.
    ///
    /// This is a 4-component array of floating-point numbers representing RGBA.
    ///
    /// Defaults to `[ 0.0, 0.0, 0.0, 0.0 ]`.
    clear_value : Option< [ f32; 4 ] >,
     /// The multisample resolve target for the attachment.
    ///
    /// If the `view` is a multisampled texture, this specifies a non-multisampled
    /// texture view where the results of the multisample rendering will be
    /// resolved (blended and downsampled).
    ///
    /// Defaults to `None`.
    resolve_target : Option< &'a web_sys::GpuTextureView >,
     /// For 3D textures or texture arrays, this specifies the depth slice or array
    /// layer to render to.
    ///
    /// Defaults to `None`.
    depth_slice : Option< u32 >
  }

  impl< 'a > ColorAttachment< 'a > 
  {
    /// Creates a new `ColorAttachment` builder with a required texture view.
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

    /// Sets the load operation for the attachment.
    pub fn load_op( mut self, op : GpuLoadOp ) -> Self
    {
      self.load_op = op;
      self
    }

    /// Sets the store operation for the attachment.
    pub fn store_op( mut self, op : GpuStoreOp ) -> Self
    {
      self.store_op = op;
      self
    }

    /// Sets the clear color value.
    pub fn clear_value( mut self, color : [ f32; 4 ] ) -> Self
    {
      self.clear_value = Some( color );
      self
    }

    /// Sets the resolve target for the attachment.
    pub fn resolve_target( mut self, view : &'a web_sys::GpuTextureView ) -> Self
    {
      self.resolve_target = Some( view );
      self
    }

    /// Sets the depth slice or array layer to render to.
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

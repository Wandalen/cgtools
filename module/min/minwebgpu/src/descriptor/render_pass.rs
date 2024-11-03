/// Internal namespace.
mod private
{
  use crate::*;

  pub struct RenderPassDescriptor< 'a >
  {
    color_attachments : Vec< web_sys::GpuRenderPassColorAttachment >,
    /// Optional
    depth_stencil_attachment : Option< web_sys::GpuRenderPassDepthStencilAttachment >,
    /// Defaults to `None`
    label : Option< &'a str >,
    /// Defaults to `50000000`
    max_draw_count : Option< f64 >
  }

  impl< 'a > RenderPassDescriptor< 'a > {
    pub fn new() -> Self
    {
      let color_attachments = Vec::new();
      let depth_stencil_attachment = None;
      let label = None;
      let max_draw_count = None;

      RenderPassDescriptor
      {
        color_attachments,
        depth_stencil_attachment,
        label,
        max_draw_count
      }
    }

    pub fn color_attachment
    ( 
      mut self, 
      color_attachment : impl Into< web_sys::GpuRenderPassColorAttachment > 
    ) -> Self
    {
      self.color_attachments.push( color_attachment.into() );
      self
    }

    pub fn depth_stencil_attachment
    ( 
      mut self, 
      depth_stencil_attachment : impl Into< web_sys::GpuRenderPassDepthStencilAttachment > 
    ) -> Self
    {
      self.depth_stencil_attachment = Some( depth_stencil_attachment.into() );
      self
    }

    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    pub fn max_draw_count( mut self, count : f64 ) -> Self
    {
      self.max_draw_count = Some( count );
      self
    }
  }

  impl From< RenderPassDescriptor< '_ > > for web_sys::GpuRenderPassDescriptor {
    fn from( value: RenderPassDescriptor< '_ > ) -> Self 
    {
      let desc = web_sys::GpuRenderPassDescriptor::new( &value.color_attachments.into() );

      if let Some( v ) = value.depth_stencil_attachment { desc.set_depth_stencil_attachment( &v ); }
      if let Some( v ) = value.label { desc.set_label( v ); }
      if let Some( v ) = value.max_draw_count { desc.set_max_draw_count( v ); }

      desc
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    RenderPassDescriptor
  };
}

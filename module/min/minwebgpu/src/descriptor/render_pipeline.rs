/// Internal namespace.
mod private
{
  
  use crate::*;
  pub struct RenderPipelineDescriptor< 'a >
  {
    vertex : web_sys::GpuVertexState,
    /// Defaults to `None`
    label : Option< &'a str >,
    /// Defaults to 'auto'
    layout : Option< &'a web_sys::GpuPipelineLayout >,
    /// Defaults to `None`
    fragment : Option< web_sys::GpuFragmentState >,
    /// Defaults to `None`
    primitive : Option< web_sys::GpuPrimitiveState >,
    /// Defaults to `None`
    depth_stencil : Option< web_sys::GpuDepthStencilState >,
    /// Defaults to `None`
    multisample : Option< web_sys::GpuMultisampleState >

  }

  impl< 'a > RenderPipelineDescriptor< 'a >
  {
    pub fn new< T : Into< web_sys::GpuVertexState > >( vertex : T ) -> Self
    {
      let label = None;
      let layout = None;
      let fragment = None;
      let primitive = None;
      let depth_stencil = None;
      let multisample = None;
      let vertex = vertex.into();

      RenderPipelineDescriptor
      {
        vertex,
        label,
        layout,
        fragment,
        primitive,
        depth_stencil,
        multisample
      }
    }

    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    pub fn layout( mut self, layout : &'a web_sys::GpuPipelineLayout ) -> Self
    {
      self.layout = Some( layout );
      self
    }

    pub fn fragment< T : Into< web_sys::GpuFragmentState > >( mut self, state : T ) -> Self
    {
      self.fragment = Some( state.into() );
      self
    }

    pub fn primitive< T : Into< web_sys::GpuPrimitiveState > >( mut self, state : T ) -> Self
    {
      self.primitive = Some( state.into() );
      self
    }

    pub fn depth_stencil< T : Into< web_sys::GpuDepthStencilState > >( mut self, state : T ) -> Self
    {
      self.depth_stencil = Some( state.into() );
      self
    }

    pub fn multisample< T : Into< web_sys::GpuMultisampleState > >( mut self, state : T ) -> Self
    {
      self.multisample = Some( state.into() );
      self
    }

    pub fn create( self, device : &web_sys::GpuDevice ) -> Result< web_sys::GpuRenderPipeline, WebGPUError >
    {
      let pipeline = device.create_render_pipeline( &self.into() )
      .map_err( | e | DeviceError::FailedToCreateRenderPipeline( format!( "{:?}", e ) ))?;
      
      Ok( pipeline )
    }

    pub async fn create_async( self, device : &web_sys::GpuDevice ) -> Result< web_sys::GpuRenderPipeline, WebGPUError >
    {
      let pipeline = JsFuture::from( device.create_render_pipeline_async( &self.into() ) ).await
      .map_err( | e | DeviceError::FailedToCreateRenderPipeline( format!( "{:?}", e ) ))?;

      let pipeline = web_sys::GpuRenderPipeline::from( pipeline );
      Ok( pipeline )
    }
  }

  impl From< RenderPipelineDescriptor< '_ > > for web_sys::GpuRenderPipelineDescriptor 
  {
    fn from( value: RenderPipelineDescriptor< '_ > ) -> Self 
    {
      let layout = 
      if let Some( l ) = value.layout
      {
        l.into()
      }
      else
      {
        "auto".into()
      };

      let desc = web_sys::GpuRenderPipelineDescriptor::new( &layout, &value.vertex );

      if let Some( v ) = value.label { desc.set_label( v ); }
      if let Some( v ) = value.fragment { desc.set_fragment( &v ); }
      if let Some( v ) = value.primitive { desc.set_primitive( &v ); }
      if let Some( v ) = value.depth_stencil { desc.set_depth_stencil( &v ); }
      if let Some( v ) = value.multisample { desc.set_multisample( &v ); }

      desc
    }    
  }


}

crate::mod_interface!
{
  exposed use
  {
    RenderPipelineDescriptor
  };
}

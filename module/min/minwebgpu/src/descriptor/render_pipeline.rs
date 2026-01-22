/// Internal namespace.
mod private
{
  
  use crate::*;

  /// A builder for creating a `web_sys::GpuRenderPipeline`.
  #[ derive( Clone ) ]
  pub struct RenderPipelineDescriptor< 'a >
  {
    /// The required vertex state for the pipeline. This includes the vertex
    /// shader module and the entry point function.
    vertex : web_sys::GpuVertexState,
    /// An optional debug label for the pipeline.
    ///
    /// Defaults to `None`.
    label : Option< &'a str >,
    /// An optional pipeline layout. This defines the bind groups and push constants
    /// used by the pipeline.
    ///
    /// Defaults to 'auto', which means the GPU will automatically create a layout
    /// based on the shaders.
    layout : Option< &'a web_sys::GpuPipelineLayout >,
    /// The optional fragment state for the pipeline. This includes the fragment
    /// shader module, entry point, and color target formats.
    ///
    /// Defaults to `None`.
    fragment : Option< web_sys::GpuFragmentState >,
    /// An optional primitive state. This defines how vertices are interpreted
    /// to form primitives like triangles or lines.
    ///
    /// Defaults to `None`.
    primitive : Option< web_sys::GpuPrimitiveState >,
    /// An optional depth-stencil state. This is used for depth testing, writing
    /// to the depth buffer, and stencil operations.
    ///
    /// Defaults to `None`.
    depth_stencil : Option< web_sys::GpuDepthStencilState >,
    /// An optional multisample state. This configures multisample anti-aliasing.
    ///
    /// Defaults to `None`.
    multisample : Option< web_sys::GpuMultisampleState >

  }

  impl< 'a > RenderPipelineDescriptor< 'a >
  {
    /// Creates a new `RenderPipelineDescriptor` with the required vertex state.
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

    /// Sets the debug label for the pipeline.
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Sets the pipeline layout.
    pub fn layout( mut self, layout : &'a web_sys::GpuPipelineLayout ) -> Self
    {
      self.layout = Some( layout );
      self
    }

    /// Sets the fragment state for the pipeline.
    pub fn fragment< T : Into< web_sys::GpuFragmentState > >( mut self, state : T ) -> Self
    {
      self.fragment = Some( state.into() );
      self
    }

    /// Sets the primitive state.
    pub fn primitive< T : Into< web_sys::GpuPrimitiveState > >( mut self, state : T ) -> Self
    {
      self.primitive = Some( state.into() );
      self
    }

    /// Sets the depth-stencil state.
    pub fn depth_stencil< T : Into< web_sys::GpuDepthStencilState > >( mut self, state : T ) -> Self
    {
      self.depth_stencil = Some( state.into() );
      self
    }

    /// Sets the multisample state.
    pub fn multisample< T : Into< web_sys::GpuMultisampleState > >( mut self, state : T ) -> Self
    {
      self.multisample = Some( state.into() );
      self
    }

    /// Creates a synchronous render pipeline.
    pub fn create( self, device : &web_sys::GpuDevice ) -> Result< web_sys::GpuRenderPipeline, WebGPUError >
    {
      render_pipeline::create( device, &self.into() )
    }

    /// Creates an asynchronous render pipeline.
    pub async fn create_async( self, device : &web_sys::GpuDevice ) -> Result< web_sys::GpuRenderPipeline, WebGPUError >
    {
      render_pipeline::create_async( device, &self.into() ).await
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

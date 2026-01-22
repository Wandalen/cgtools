/// Internal namespace.
mod private
{
  
  use crate::*;

  /// Describes the configuration for creating a WebGPU compute pipeline.
  #[ derive( Clone ) ]
  pub struct ComputePipelineDescriptor< 'a >
  {
    /// A `GpuProgrammableStage` that defines the compute shader and its entry point.
    /// This is the core part of the compute pipeline descriptor.
    compute : web_sys::GpuProgrammableStage,
    /// An optional label for the compute pipeline, which can be helpful for debugging and
    /// performance tracing.
    /// 
    /// Defaults to `None`.
    label : Option< &'a str >,
    /// The `GpuPipelineLayout` for the compute pipeline. This layout specifies the
    /// bind groups that the pipeline will use. If this is `None`, a default, automatically
    /// generated layout is used.
    /// 
    /// Defaults to 'auto'.
    layout : Option< &'a web_sys::GpuPipelineLayout >,

  }

  impl< 'a > ComputePipelineDescriptor< 'a >
  {
    /// Creates a new `ComputePipelineDescriptor` with a given compute stage.
    pub fn new< T : Into< web_sys::GpuProgrammableStage > >( compute : T ) -> Self
    {
      let label = None;
      let layout = None;
      let compute = compute.into();

      ComputePipelineDescriptor
      {
        compute,
        label,
        layout
      }
    }

    /// Sets an optional label for the compute pipeline.
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Sets the `GpuPipelineLayout` for the compute pipeline.
    pub fn layout( mut self, layout : &'a web_sys::GpuPipelineLayout ) -> Self
    {
      self.layout = Some( layout );
      self
    }

    /// Creates a `web_sys::GpuComputePipeline` synchronously.
    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuComputePipeline
    {
      compute_pipeline::create( device, &self.into() )
    }

    /// Creates a `web_sys::GpuComputePipeline` asynchronously.
    pub async fn create_async( self, device : &web_sys::GpuDevice ) -> Result< web_sys::GpuComputePipeline, WebGPUError >
    {
      compute_pipeline::create_async( device, &self.into() ).await
    }
  }

  impl From< ComputePipelineDescriptor< '_ > > for web_sys::GpuComputePipelineDescriptor 
  {
    fn from( value: ComputePipelineDescriptor< '_ > ) -> Self 
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

      let desc = web_sys::GpuComputePipelineDescriptor::new( &layout, &value.compute );

      if let Some( v ) = value.label { desc.set_label( v ); }

      desc
    }    
  }
}

crate::mod_interface!
{
  exposed use
  {
    ComputePipelineDescriptor
  };
}

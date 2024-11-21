/// Internal namespace.
mod private
{
  
  use crate::*;

  #[ derive( Clone ) ]
  pub struct ComputePipelineDescriptor< 'a >
  {
    compute : web_sys::GpuProgrammableStage,
    /// Defaults to `None`
    label : Option< &'a str >,
    /// Defaults to 'auto'
    layout : Option< &'a web_sys::GpuPipelineLayout >,

  }

  impl< 'a > ComputePipelineDescriptor< 'a >
  {
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


    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuComputePipeline
    {
      compute_pipeline::create( device, &self.into() )
    }

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

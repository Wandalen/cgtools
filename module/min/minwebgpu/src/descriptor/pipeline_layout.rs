/// Internal namespace.
mod private
{
  use crate::*;

  /// Describes the configuration for creating a WebGPU pipeline layout.
  #[ derive( Default, Clone ) ]
  pub struct PipelineLayoutDescriptor< 'a >
  {
    /// An optional label for the pipeline layout. Defaults to `None`.
    label : Option< &'a str >,
    /// A vector of `GpuBindGroupLayout`s that this pipeline layout will contain.
    /// The order of these layouts is important as it corresponds to the `@group(...)`
    /// indices in the WGSL shaders.
    bind_group_layouts : Vec< web_sys::GpuBindGroupLayout >
  }

  impl< 'a > PipelineLayoutDescriptor< 'a >  
  {
    /// Creates a new, empty `PipelineLayoutDescriptor` with default values.
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets an optional label for the pipeline layout.
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Adds a `GpuBindGroupLayout` to the pipeline layout.
    pub fn bind_group
    ( 
      mut self, 
      bind_group : &web_sys::GpuBindGroupLayout
    ) -> Self
    {
      self.bind_group_layouts.push( bind_group.clone() );
      self
    }

    /// Creates a `web_sys::GpuPipelineLayout` from this descriptor.
    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuPipelineLayout
    {
      device.create_pipeline_layout( &self.into() )
    }
  }

  impl From< PipelineLayoutDescriptor< '_ > > for web_sys::GpuPipelineLayoutDescriptor 
  {
    fn from( value: PipelineLayoutDescriptor< '_ > ) -> Self 
    {
      let desc =  web_sys::GpuPipelineLayoutDescriptor::new( &value.bind_group_layouts.into() );

      if let Some( v ) = value.label { desc.set_label( v ); }

      desc
    }
  }
}

crate::mod_interface!
{
  exposed use
  {
    PipelineLayoutDescriptor
  };
}

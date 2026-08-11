/// Internal namespace.
mod private
{
  use crate::{ web_sys, Into, js_sys, IntoIterator };

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
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self::default()
    }

    /// Sets an optional label for the pipeline layout.
    #[ inline ]
    #[ must_use ]
    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    /// Adds a `GpuBindGroupLayout` to the pipeline layout.
    #[ inline ]
    #[ must_use ]
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
    #[ inline ]
    #[ must_use ]
    pub fn create( self, device : &web_sys::GpuDevice ) -> web_sys::GpuPipelineLayout
    {
      device.create_pipeline_layout( &self.into() )
    }
  }

  impl From< PipelineLayoutDescriptor< '_ > > for web_sys::GpuPipelineLayoutDescriptor 
  {
    #[ inline ]
    fn from( value: PipelineLayoutDescriptor< '_ > ) -> Self 
    {
      let bind_group_layouts : Vec< js_sys::JsNullable< web_sys::GpuBindGroupLayout > > =
      value.bind_group_layouts.into_iter().map( js_sys::JsNullable::wrap ).collect();
      let desc = web_sys::GpuPipelineLayoutDescriptor::new( &bind_group_layouts );

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

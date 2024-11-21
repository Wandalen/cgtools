/// Internal namespace.
mod private
{
  use crate::*;

  #[ derive( Default, Clone ) ]
  pub struct PipelineLayoutDescriptor< 'a >
  {
    label : Option< &'a str >,
    bind_group_layouts : Vec< web_sys::GpuBindGroupLayout >
  }

  impl< 'a > PipelineLayoutDescriptor< 'a >  
  {
    pub fn new() -> Self
    {
      Self::default()
    }

    pub fn label( mut self, label : &'a str ) -> Self
    {
      self.label = Some( label );
      self
    }

    pub fn bind_group
    ( 
      mut self, 
      bind_group : &web_sys::GpuBindGroupLayout
    ) -> Self
    {
      self.bind_group_layouts.push( bind_group.clone() );
      self
    }

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

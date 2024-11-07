/// Internal namespace.
mod private
{
  use crate::*;

  pub struct VertexBufferLayout
  {
    array_stride : f64,
    /// Defaults to `Vertex`
    step_mode : GpuVertexStepMode,
    attributes : Vec< web_sys::GpuVertexAttribute >,
    /// Defaults to `false`
    compute_offsets : bool,
  }

  impl VertexBufferLayout
  {
    pub fn new() -> Self
    {
      let array_stride = 0.0;
      let step_mode = GpuVertexStepMode::Vertex;
      let attributes = Vec::new();
      let compute_offsets = false;

      VertexBufferLayout
      {
        array_stride,
        step_mode,
        attributes,
        compute_offsets
      }
    }

    /// Computes the array stride from the given type
    pub fn stride< T : Sized >( mut self ) -> Self
    {
      self.array_stride = std::mem::size_of::< T >() as f64;
      self
    }

    /// Sets the array stride from the given value
    pub fn stride_from_val( mut self, stride : f64 ) -> Self
    {
      self.array_stride = stride;
      self
    }

    /// Sets the step mode to `Vertex`
    pub fn vertex( mut self ) -> Self
    {
      self.step_mode = GpuVertexStepMode::Vertex;
      self
    }

    /// Sets the step mode to `Instance`
    pub fn instance( mut self) -> Self
    {
      self.step_mode = GpuVertexStepMode::Instance;
      self
    }

    /// Adds an attribute to the layout
    pub fn attribute( mut self, attribute : impl Into< web_sys::GpuVertexAttribute > ) -> Self
    {
      self.attributes.push( attribute.into() );
      self
    }

    /// Tells the builder to auto compute offsets for each attribute
    pub fn compute_offsets( mut self ) -> Self
    {
      self.compute_offsets = true;
      self
    }
  }

  impl From< VertexBufferLayout > for web_sys::GpuVertexBufferLayout 
  {
    fn from( mut value: VertexBufferLayout ) -> Self {
      if value.compute_offsets 
      {
        let mut offset : f64 = 0.0;
        for a in value.attributes.iter_mut()
        {
          a.set_offset( offset );
          offset += layout::vertex_attribute::format_to_size( a.get_format() ) as f64;
        }
      }
      
      let layout = web_sys::GpuVertexBufferLayout::new
      ( 
        value.array_stride, 
        &value.attributes.into()
      );

      layout.set_step_mode( value.step_mode );
      layout
    }    
  }

}

crate::mod_interface!
{
  exposed use
  {
    VertexBufferLayout
  };
}

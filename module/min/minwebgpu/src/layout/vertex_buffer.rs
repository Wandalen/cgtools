/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuVertexBufferLayout`.
  #[ derive( Clone ) ]
  pub struct VertexBufferLayout
  { 
    /// Needs to be supplied by the user. If not specified, will be computed automatically
    array_stride : Option< u32 >,
    /// Needs to be supplied by the user
    attributes : Vec< web_sys::GpuVertexAttribute >,
    /// Defaults to `Vertex`
    step_mode : GpuVertexStepMode,
    /// Defaults to `false`
    compute_offsets : bool,
  }

  impl VertexBufferLayout
  {
    /// Creates a new `VertexBufferLayout` with default values.
    pub fn new() -> Self
    {
      let array_stride = None;
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
    pub fn stride< T >( mut self ) -> Self
    {
      self.array_stride = Some( std::mem::size_of::< T >() as u32 );
      self
    }

    /// Sets the array stride from the given value
    pub fn stride_from_value( mut self, stride : u32 ) -> Self
    {
      self.array_stride = Some( stride );
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
      let mut offset : u32 = 0;
      for a in value.attributes.iter_mut()
      {
        // web-sys' `get_offset` returns f64 while `set_offset` takes u32; offsets are byte
        // counts, so narrow to u32.
        let a_offset = a.get_offset() as u32;
        offset = offset.max( a_offset );

        if value.compute_offsets
        {
          a.set_offset( offset );
        }

        let size = layout::vertex_attribute::format_to_size( a.get_format() ) as u32;
        offset += size;
      }

      if value.array_stride.is_none() { value.array_stride = Some( offset ); }


      let layout = web_sys::GpuVertexBufferLayout::new
      (
        value.array_stride.unwrap(),
        &value.attributes
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

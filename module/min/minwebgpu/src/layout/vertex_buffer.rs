/// Internal namespace.
mod private
{
  use crate::{ web_sys, GpuVertexStepMode, Into, layout };

  /// A builder for creating a `web_sys::GpuVertexBufferLayout`.
  #[ derive( Clone ) ]
  pub struct VertexBufferLayout
  { 
    /// Needs to be supplied by the user. If not specified, will be computed automatically
    array_stride : Option< f64 >,
    /// Needs to be supplied by the user
    attributes : Vec< web_sys::GpuVertexAttribute >,
    /// Defaults to `Vertex`
    step_mode : GpuVertexStepMode,
    /// Defaults to `false`
    compute_offsets : bool,
  }

  impl Default for VertexBufferLayout
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl VertexBufferLayout
  {
    /// Creates a new `VertexBufferLayout` with default values.
    #[ inline ]
    #[ must_use ]
    pub fn new() -> Self
    {
      let array_stride = None;
      let step_mode = GpuVertexStepMode::Vertex;
      let attributes = Vec::new();
      let compute_offsets = false;

      VertexBufferLayout
      {
        array_stride,
        attributes,
        step_mode,
        compute_offsets
      }
    }

    /// Computes the array stride from the given type
    #[ inline ]
    #[ must_use ]
    pub fn stride< T >( mut self ) -> Self
    {
      // `size_of::<T>()` reflects a single Rust type's compile-time byte size, which will
      // never approach f64's 2^52 exact-integer limit — the precision loss is unreachable.
      let stride = std::mem::size_of::< T >() as f64;
      self.array_stride = Some( stride );
      self
    }

    /// Sets the array stride from the given value
    #[ inline ]
    #[ must_use ]
    pub fn stride_from_value( mut self, stride : f64 ) -> Self
    {
      self.array_stride = Some( stride );
      self
    }

    /// Sets the step mode to `Vertex`
    #[ inline ]
    #[ must_use ]
    pub fn vertex( mut self ) -> Self
    {
      self.step_mode = GpuVertexStepMode::Vertex;
      self
    }

    /// Sets the step mode to `Instance`
    #[ inline ]
    #[ must_use ]
    pub fn instance( mut self) -> Self
    {
      self.step_mode = GpuVertexStepMode::Instance;
      self
    }

    /// Adds an attribute to the layout
    #[ inline ]
    #[ must_use ]
    pub fn attribute( mut self, attribute : impl Into< web_sys::GpuVertexAttribute > ) -> Self
    {
      self.attributes.push( attribute.into() );
      self
    }

    /// Tells the builder to auto compute offsets for each attribute
    #[ inline ]
    #[ must_use ]
    pub fn compute_offsets( mut self ) -> Self
    {
      self.compute_offsets = true;
      self
    }
  }

  impl From< VertexBufferLayout > for web_sys::GpuVertexBufferLayout 
  {
    #[ inline ]
    fn from( mut value: VertexBufferLayout ) -> Self {
      let mut offset : f64 = 0.0;
      for a in &mut value.attributes
      {
        let a_offset = a.get_offset();
        offset = offset.max( a_offset );

        if value.compute_offsets
        {
          a.set_offset_f64( offset );
        }

        // A single vertex attribute's byte size (a handful of bytes) is nowhere near f64's
        // 2^52 exact-integer limit — the precision loss is unreachable.
        let size = layout::vertex_attribute::format_to_size( a.get_format() ) as f64;
        offset += size;
      }

      if value.array_stride.is_none() { value.array_stride = Some( offset ); }

      
      let layout = web_sys::GpuVertexBufferLayout::new_with_f64
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

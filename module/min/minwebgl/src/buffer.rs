/// Internal namespace.
mod private
{
  use crate::{ web_sys, data_type, GL, WebglError, mem, VectorDataType, IntoVectorDataType };
  pub use web_sys::WebGlBuffer;
  use data_type::Const;

  /// Creates a new WebGL buffer.
  ///
  /// # Arguments
  ///
  /// * `gl` - A reference to the WebGL context.
  ///
  /// # Returns
  ///
  /// * `Result< WebGlBuffer, WebglError >` - A result containing the created WebGL buffer or an error if the buffer creation fails.
  ///
  /// # Errors
  /// Returns `WebglError::FailedToAllocateResource` if the WebGL context fails to allocate the buffer.
  #[ inline ]
  pub fn create( gl : &GL ) -> Result< WebGlBuffer, WebglError >
  {
    gl.create_buffer().ok_or( WebglError::FailedToAllocateResource( "Buffer" ) )
  }

  /// Uploads data to a WebGL ARRAY_BUFFER.
  ///
  /// # Arguments
  ///
  /// * `gl` - A reference to the WebGL context.
  /// * `buffer` - A reference to the WebGL buffer to upload data to.
  /// * `data` - A slice of data to upload.
  /// * `hint` - A usage hint for the buffer (e.g., `GL::STATIC_DRAW`).
  ///
  /// # Example
  ///
  /// ```rust, ignore
  /// minwebgl::buffer::upload( &gl, &buffer, &data, GL::STATIC_DRAW );
  /// ```
  #[ inline ]
  pub fn upload< Data >( gl : &GL, buffer : &WebGlBuffer, data : &Data, data_usage : u32 )
  where
    Data : mem::AsBytes + ?Sized,
  {
    gl.bind_buffer( GL::ARRAY_BUFFER, Some( buffer ) );
    gl.buffer_data_with_u8_array( GL::ARRAY_BUFFER, data.as_bytes(), data_usage );
  }

  /// Describes the attributes of a WebGL buffer.
  #[ derive( Debug, Clone, Copy ) ]
  #[ non_exhaustive ]
  pub struct BufferDescriptor
  {
    /// The vector data type.
    pub vector : VectorDataType,
    /// The offset in the buffer.
    pub offset : i32,
    /// The stride between consecutive elements.
    pub stride : i32,
    /// The divisor for instanced rendering.
    ///
    /// A divisor of 0 indicates that each vertex has its own unique attribute value.
    /// A divisor of 1 means that the entire primitive shares the same attribute value.
    /// A divisor of 2 or more specifies that the attribute value is shared across multiple primitives.
    pub divisor : usize,
    /// Specifies whether integer data values should be normalized when converted to float
    pub normalized : bool
  }

  impl BufferDescriptor
  {
    /// Creates a new `BufferDescriptor` with default values.
    ///
    /// # Returns
    ///
    /// * `BufferDescriptor` - A new buffer descriptor with default settings.
    #[ inline ]
    #[ must_use ]
    pub fn new< I : IntoVectorDataType >() -> Self
    {
      let vector = I::into_vector_data_type();
      Self
      {
        vector,
        offset : 0,
        stride : 0,
        divisor : 0,
        normalized : false
      }
    }

    /// Creates a new `BufferDescriptor` from a raw `VectorDataType`.
    ///
    /// For bridging from a `mingl::VertexAttribute`, where the concrete Rust type
    /// ( `I : IntoVectorDataType` ) that `new` expects isn't known at the call site — only the
    /// already-resolved `VectorDataType` is. Needed because `BufferDescriptor` is `#[non_exhaustive]`,
    /// so it can't be built via struct literal outside this module.
    #[ inline ]
    #[ must_use ]
    pub fn from_vector( vector : VectorDataType ) -> Self
    {
      Self
      {
        vector,
        offset : 0,
        stride : 0,
        divisor : 0,
        normalized : false
      }
    }

    /// Sets whether the buffer attribute should be normalized.
    #[ inline ]
    #[ must_use ]
    pub fn normalized( mut self, normalized : bool ) -> Self
    {
      self.normalized = normalized;
      self
    }

    /// Sets the vector data type.
    #[ inline ]
    #[ must_use ]
    pub fn vector( mut self, src : VectorDataType ) -> Self
    {
      self.vector = src;
      self
    }

    /// Sets the offset.
    #[ inline ]
    #[ must_use ]
    pub fn offset( mut self, src : i32 ) -> Self
    {
      self.offset = src;
      self
    }

    /// Sets the stride.
    #[ inline ]
    #[ must_use ]
    pub fn stride( mut self, src : i32 ) -> Self
    {
      self.stride = src;
      self
    }

    /// Sets the divisor for instanced rendering.
    ///
    /// A divisor of 0 indicates that each vertex has its own unique attribute value.
    /// A divisor of 1 means that the entire primitive shares the same attribute value.
    /// A divisor of 2 or more specifies that the attribute value is shared across multiple primitives.
    #[ inline ]
    #[ must_use ]
    pub fn divisor( mut self, src : usize ) -> Self
    {
      self.divisor = src;
      self
    }

    /// Configures the attribute pointer for a WebGL buffer.
    /// Bear in mind WebGL matrices are column-major, so natural flow in a flat buffer is actually transposed one for WebGL.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGL context.
    /// * `slot` - The attribute slot to configure.
    /// * `gl_buffer` - A reference to the WebGL buffer.
    ///
    /// # Returns
    ///
    /// * `Result<(), WebglError>` - A result indicating success or failure.
    ///
    /// # Errors
    /// Returns `WebglError::DataType` if `self.vector.scalar` has no corresponding WebGL constant.
    ///
    /// # Panics
    /// Panics if the vector's arity ratio, attribute slot index, or `self.divisor` does not fit
    /// into the WebGL API's `u32`/`i32` parameter types — every one of these is a small,
    /// driver-bounded count that fits in practice, so this indicates a corrupt `VectorDataType`.
    #[ inline ]
    pub fn attribute_pointer( &self, gl : &GL, slot : u32, gl_buffer : &WebGlBuffer ) -> Result< u32, WebglError >
    {
      let sz = self.vector.scalar.byte_size();
      gl.bind_buffer( GL::ARRAY_BUFFER, Some( gl_buffer ) );

      if self.vector.nelements() > 1
      {

        let slots : u32 = ( self.vector.natoms() / self.vector.nelements() ).try_into()
        .expect( "vector arity ratio is always non-negative" );
        for i in 0 .. slots
        {
          let i_signed = i32::try_from( i ).expect( "attribute slot index fits in i32" );
          let element_offset = i_signed * sz * self.vector.nelements();
          gl.vertex_attrib_pointer_with_i32
          (
            slot + i,
            self.vector.nelements(),
            *Const::try_from( self.vector.scalar )?, // data type
            self.normalized, // normalization
            self.stride * sz,
            self.offset * sz + element_offset,
          );
          gl.vertex_attrib_divisor( slot + i, u32::try_from( self.divisor ).expect( "divisor fits in u32" ) );
          gl.enable_vertex_attrib_array( slot + i );
        }
        Ok( slots )

      }
      else
      {

        gl.vertex_attrib_pointer_with_i32
        (
          slot,
          self.vector.natoms(),
          *Const::try_from( self.vector.scalar )?, // data type
          self.normalized, // normalization
          self.stride * sz,
          self.offset * sz,
        );
        // if self.divisor != 0
        {
          gl.vertex_attrib_divisor( slot, u32::try_from( self.divisor ).expect( "divisor fits in u32" ) );
        }

        gl.enable_vertex_attrib_array( slot );

        Ok( 1 )
      }

    }
  }

  /// Binds every attribute in a `mingl::VertexBufferLayout` to `gl_buffer`, delegating each
  /// attribute to `BufferDescriptor::attribute_pointer` for the actual GL call ( including its
  /// matrix-splitting behavior for attributes whose `vector.nelements() > 1` ).
  ///
  /// `layout.step_mode` is authoritative: `StepMode::Vertex` always binds with WebGL divisor `0`,
  /// regardless of `layout.divisor`. `StepMode::Instance` binds with `layout.divisor`, defaulting to
  /// `1` ( advance once per instance ) when left at its Rust-default `0`, since a WebGL divisor of
  /// `0` means "per vertex" — the opposite of what `StepMode::Instance` asks for.
  ///
  /// # Errors
  /// Returns `WebglError` if any attribute's `attribute_pointer` call fails.
  #[ inline ]
  pub fn vertex_buffer_layout_bind
  (
    gl : &GL,
    gl_buffer : &WebGlBuffer,
    layout : &mingl::VertexBufferLayout
  ) -> Result< (), WebglError >
  {
    let divisor = match layout.step_mode
    {
      mingl::StepMode::Vertex => 0,
      mingl::StepMode::Instance => if layout.divisor == 0 { 1 } else { layout.divisor },
    };

    for attribute in &layout.attributes
    {
      BufferDescriptor::from_vector( attribute.vector )
      .offset( attribute.offset )
      .stride( layout.stride )
      .divisor( divisor )
      .attribute_pointer( gl, attribute.location, gl_buffer )?;
    }

    Ok( () )
  }

}

crate::mod_interface!
{

  orphan use
  {
    create,
    upload,
    WebGlBuffer,
    BufferDescriptor,
    vertex_buffer_layout_bind,
  };

}

/// Internal namespace.
mod private
{
  use crate::*;

  /// A builder for creating a `web_sys::GpuVertexAttribute`.
  #[ derive( Clone ) ]
  pub struct VertexAttribute
  {
    /// Offset of the attribute. Default: 0.0
    offset : f64,
    /// Location in the shader. Default: 0
    location : u32,
    /// Attribute's format. Default: Float32x3
    format : GpuVertexFormat
  }

  impl VertexAttribute 
  {
    /// Creates a new `VertexAttribute` with default values.
    pub fn new() -> Self
    {
      let offset = 0.0;
      let location = 0;
      let format = GpuVertexFormat::Float32x3;
      VertexAttribute
      {
        offset,
        location,
        format
      }
    }

    /// Sets the location in the shader
    pub fn location( mut self, location : u32 ) -> Self
    {
      self.location = location;
      self
    }

    /// Sets the format of the attribute
    pub fn format( mut self, format : GpuVertexFormat ) -> Self
    {
      self.format = format;
      self
    }

    /// Sets the offset from the size of a type
    pub fn offset< T : Sized >( mut self ) -> Self
    {
      self.offset = std::mem::size_of::< T >() as f64;
      self
    }

    /// Sets the offset from the provided value
    pub fn offset_from_value( mut self, offset : f64 ) -> Self
    {
      self.offset = offset;
      self
    }
  }

  impl From< VertexAttribute > for web_sys::GpuVertexAttribute 
  {
    fn from( value: VertexAttribute ) -> Self {
      let attribute = web_sys::GpuVertexAttribute::new
      (
        value.format, 
        value.offset, 
        value.location
      );

      attribute
    }    
  }

  /// Calculates the size in bytes of a given `GpuVertexFormat`.
  pub fn format_to_size( format : web_sys::GpuVertexFormat ) -> usize
  {
    use web_sys::GpuVertexFormat;
    use std::mem::size_of;
    match format 
    {
      // 8
      GpuVertexFormat::Uint8x2 |
      GpuVertexFormat::Sint8x2 |
      GpuVertexFormat::Unorm8x2 |
      GpuVertexFormat::Snorm8x2 => size_of::< [ u8; 2 ] >(),

      GpuVertexFormat::Uint8x4 |
      GpuVertexFormat::Sint8x4 | 
      GpuVertexFormat::Unorm8x4 |
      GpuVertexFormat::Snorm8x4 => size_of::< [ u8; 4 ] >(),

      // 16
      GpuVertexFormat::Uint16x2 | 
      GpuVertexFormat::Sint16x2 | 
      GpuVertexFormat::Unorm16x2 |
      GpuVertexFormat::Snorm16x2 | 
      GpuVertexFormat::Float16x2 => size_of::< [ u16; 2 ] >(),
      
      GpuVertexFormat::Uint16x4 | 
      GpuVertexFormat::Sint16x4 | 
      GpuVertexFormat::Unorm16x4 |
      GpuVertexFormat::Snorm16x4 | 
      GpuVertexFormat::Float16x4 => size_of::< [ u16; 4 ] >(),

      // 32
      GpuVertexFormat::Float32 | 
      GpuVertexFormat::Uint32  | 
      GpuVertexFormat::Sint32  |
      GpuVertexFormat::Unorm1010102 => size_of::< [ u32; 1 ] >(),

      GpuVertexFormat::Float32x2 | 
      GpuVertexFormat::Uint32x2  | 
      GpuVertexFormat::Sint32x2  => size_of::< [ u32; 2 ] >(),

      GpuVertexFormat::Float32x3 | 
      GpuVertexFormat::Uint32x3  |
      GpuVertexFormat::Sint32x3  => size_of::< [ u32; 3 ] >(),

      GpuVertexFormat::Float32x4 | 
      GpuVertexFormat::Uint32x4  | 
      GpuVertexFormat::Sint32x4  => size_of::< [ u32; 4 ] >(),
      _ => panic!( "Unexpected vertex format")
    }
  }

}

crate::mod_interface!
{
  own use
  {
    format_to_size
  };

  exposed use
  {
    VertexAttribute
  };
}

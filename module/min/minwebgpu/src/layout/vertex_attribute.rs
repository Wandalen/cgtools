/// Internal namespace.
mod private
{
  use crate::{ web_sys, GpuVertexFormat };

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

  impl Default for VertexAttribute
  {
    #[ inline ]
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl VertexAttribute
  {
    /// Creates a new `VertexAttribute` with default values.
    #[ inline ]
    #[ must_use ]
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
    #[ inline ]
    #[ must_use ]
    pub fn location( mut self, location : u32 ) -> Self
    {
      self.location = location;
      self
    }

    /// Sets the format of the attribute
    #[ inline ]
    #[ must_use ]
    pub fn format( mut self, format : GpuVertexFormat ) -> Self
    {
      self.format = format;
      self
    }

    /// Sets the offset from the size of a type
    #[ inline ]
    #[ must_use ]
    pub fn offset< T : Sized >( mut self ) -> Self
    {
      // `size_of::<T>()` reflects a single Rust type's compile-time byte size, which will
      // never approach f64's 2^52 exact-integer limit — the precision loss is unreachable.
      let offset = std::mem::size_of::< T >() as f64;
      self.offset = offset;
      self
    }

    /// Sets the offset from the provided value
    #[ inline ]
    #[ must_use ]
    pub fn offset_from_value( mut self, offset : f64 ) -> Self
    {
      self.offset = offset;
      self
    }
  }

  impl From< VertexAttribute > for web_sys::GpuVertexAttribute 
  {
    #[ inline ]
    fn from( value: VertexAttribute ) -> Self {
      web_sys::GpuVertexAttribute::new_with_f64
      (
        value.format,
        value.offset,
        value.location
      )
    }
  }

  // Fix(BUG-163): `format_to_size` matched only 31 of `GpuVertexFormat`'s 41 variants and
  // panicked on the other 10 via a `_` catch-all, even though every one of them is an ordinary,
  // reachable, spec-defined format a caller can legally pass -- e.g. `.format( GpuVertexFormat
  // ::Uint8 )` panicked despite `Uint8` being a real, named, safe-Rust-constructible variant.
  // Root cause: the match was built by covering the multi-component (`x2`/`x3`/`x4`) formats
  // only, missing every single-component format (`Uint8`, `Sint16`, `Float16`, etc.) and
  // `Unorm8x4Bgra` (added to the spec after this function was first written).
  // Pitfall: `web_sys::GpuVertexFormat` LOOKS like a plain, closed 41-variant enum from its own
  // declaration, but `#[wasm_bindgen]`'s macro expansion marks every JS-string enum it generates
  // `#[non_exhaustive]` regardless -- rustc's E0004 confirmed this still demands a `_` arm even
  // after all 41 named variants are covered. Don't assume a wasm_bindgen enum's exhaustiveness
  // from reading its `pub enum` source; the macro's real expansion can differ.
  /// Calculates the size in bytes of a given `GpuVertexFormat`.
  ///
  /// # Panics
  /// Panics if `format` is not one of the 41 variants named by this `web_sys` version. This is
  /// unreachable through any safe-Rust call in this codebase (`GpuVertexFormat` is never
  /// deserialized from a raw JS string on this path, only ever Rust-constructed via one of its
  /// own named associated consts) -- the wildcard exists only because `#[wasm_bindgen]` marks
  /// every JS-string enum `#[non_exhaustive]` at the macro level. This function is called from
  /// [`web_sys::GpuVertexAttribute`]'s `From<VertexBufferLayout>` conversion (`vertex_buffer.rs`),
  /// which cannot propagate a `Result` -- `From::from` is infallible by trait contract -- so an
  /// internal-invariant panic here, not a silent wrong byte size, is the deliberate choice: a
  /// wrong stride/offset would corrupt buffer layout silently, which is worse than a loud panic
  /// on a case that cannot occur through any real call path.
  #[ inline ]
  #[ must_use ]
  pub fn format_to_size( format : web_sys::GpuVertexFormat ) -> usize
  {
    use web_sys::GpuVertexFormat;
    use std::mem::size_of;
    match format
    {
      // 8
      GpuVertexFormat::Uint8 |
      GpuVertexFormat::Sint8 |
      GpuVertexFormat::Unorm8 |
      GpuVertexFormat::Snorm8 => size_of::< [ u8; 1 ] >(),

      GpuVertexFormat::Uint8x2 |
      GpuVertexFormat::Sint8x2 |
      GpuVertexFormat::Unorm8x2 |
      GpuVertexFormat::Snorm8x2 => size_of::< [ u8; 2 ] >(),

      GpuVertexFormat::Uint8x4 |
      GpuVertexFormat::Sint8x4 |
      GpuVertexFormat::Unorm8x4 |
      GpuVertexFormat::Snorm8x4 |
      GpuVertexFormat::Unorm8x4Bgra => size_of::< [ u8; 4 ] >(),

      // 16
      GpuVertexFormat::Uint16 |
      GpuVertexFormat::Sint16 |
      GpuVertexFormat::Unorm16 |
      GpuVertexFormat::Snorm16 |
      GpuVertexFormat::Float16 => size_of::< [ u16; 1 ] >(),

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

      // Required by rustc: `#[wasm_bindgen]` marks this enum `#[non_exhaustive]` even though
      // all 41 named variants are covered above -- see the `Pitfall` note on this function.
      _ => unreachable!( "GpuVertexFormat variant not recognized by this web_sys version" ),
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

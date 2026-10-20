mod private
{
  use minwebgl as gl;
  use gl::js_sys;
  use gltf::json::Value;
  use wasm_bindgen::prelude::*;
  use crate::webgl::loaders::gltf_json::usize_read;
  use wasm_bindgen_futures::JsFuture;

  // meshoptimizer's own decoder ( MIT, see the crate's `license_third_party` ), vendored
  // unchanged from the `meshoptimizer` npm package; its header says "Built from meshoptimizer
  // 1.2". Its `meshopt_decoder.mjs` is renamed to `.js` so that every server sends it with a
  // JavaScript MIME type; module scripts refuse anything else. It carries its decoder as an inlined, SIMD-enabled wasm
  // module, so the host page needs `wasm-unsafe-eval` in its CSP -- the same permission this
  // crate's own wasm already needs.
  #[ wasm_bindgen( module = "/src/webgl/loaders/meshopt_decoder.js" ) ]
  extern "C"
  {
    type Decoder;

    #[ wasm_bindgen( thread_local_v2, js_name = MeshoptDecoder ) ]
    static DECODER : Decoder;

    #[ wasm_bindgen( method, getter ) ]
    fn ready( this : &Decoder ) -> js_sys::Promise;

    #[ wasm_bindgen( method, catch, js_name = decodeGltfBuffer ) ]
    fn decode_gltf_buffer
    (
      this : &Decoder,
      target : &js_sys::Uint8Array,
      count : u32,
      size : u32,
      source : &js_sys::Uint8Array,
      mode : &str,
      filter : &str
    ) -> Result< (), JsValue >;
  }

  /// The original meshopt extension name.
  pub const EXT_MESHOPT_COMPRESSION : &str = "EXT_meshopt_compression";

  /// The Khronos ratification of [`EXT_MESHOPT_COMPRESSION`] : the same schema, plus the
  /// `COLOR` filter.
  pub const KHR_MESHOPT_COMPRESSION : &str = "KHR_meshopt_compression";

  /// Extension names that mark a buffer view as meshopt-compressed. The one list of
  /// them : `gltf_extensions`' supported-extensions list takes its meshopt entries from
  /// here.
  pub const MESHOPT_EXTENSIONS : [ &str; 2 ] = [ EXT_MESHOPT_COMPRESSION, KHR_MESHOPT_COMPRESSION ];

  /// How a compressed buffer view was encoded.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  #[ non_exhaustive ]
  pub enum MeshoptMode
  {
    /// Vertex attributes, animation data or any other fixed-stride elements.
    Attributes,
    /// A triangle list index buffer.
    Triangles,
    /// Any other index sequence.
    Indices,
  }

  impl MeshoptMode
  {
    /// The name the decoder and the extension use.
    #[ must_use ]
    pub fn as_str( self ) -> &'static str
    {
      match self
      {
        Self::Attributes => "ATTRIBUTES",
        Self::Triangles => "TRIANGLES",
        Self::Indices => "INDICES",
      }
    }

    /// The mode named `name`, as the extension spells it.
    fn from_name( name : &str ) -> Result< Self, String >
    {
      match name
      {
        "ATTRIBUTES" => Ok( Self::Attributes ),
        "TRIANGLES" => Ok( Self::Triangles ),
        "INDICES" => Ok( Self::Indices ),
        other => Err( format!( "unknown mode `{other}`" ) ),
      }
    }

    /// Checks the stride, filter and element count this mode allows.
    fn layout_check( self, stride : usize, filter : MeshoptFilter, count : usize ) -> Result< (), String >
    {
      match self
      {
        Self::Attributes if stride == 0 || stride % 4 != 0 || stride > 256 =>
          Err( format!( "ATTRIBUTES byteStride {stride} must be a multiple of 4 up to 256" ) ),
        Self::Triangles | Self::Indices if stride != 2 && stride != 4 =>
          Err( format!( "{} byteStride {stride} must be 2 or 4", self.as_str() ) ),
        Self::Triangles | Self::Indices if filter != MeshoptFilter::None =>
          Err( format!( "{} data cannot use a filter", self.as_str() ) ),
        Self::Triangles if count % 3 != 0 =>
          Err( format!( "TRIANGLES count {count} is not a multiple of 3" ) ),
        Self::Attributes | Self::Triangles | Self::Indices => Ok( () ),
      }
    }
  }

  /// The post-decode transform applied to attribute data.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
  #[ non_exhaustive ]
  pub enum MeshoptFilter
  {
    /// Data is stored as is.
    None,
    /// Octahedral-encoded unit vectors ( normals, tangents ).
    Octahedral,
    /// Unit quaternions ( rotations ).
    Quaternion,
    /// Floats with a shared exponent.
    Exponential,
    /// YCoCg-encoded colours ( `KHR_meshopt_compression` only ).
    Color,
  }

  impl MeshoptFilter
  {
    /// The name the decoder and the extension use.
    #[ must_use ]
    pub fn as_str( self ) -> &'static str
    {
      match self
      {
        Self::None => "NONE",
        Self::Octahedral => "OCTAHEDRAL",
        Self::Quaternion => "QUATERNION",
        Self::Exponential => "EXPONENTIAL",
        Self::Color => "COLOR",
      }
    }

    /// The filter named `name` ( `None` when absent ), as `extension` allows it: `COLOR`
    /// exists only in `KHR_meshopt_compression`.
    fn from_name( name : Option< &str >, extension : &str ) -> Result< Self, String >
    {
      match name
      {
        None | Some( "NONE" ) => Ok( Self::None ),
        Some( "OCTAHEDRAL" ) => Ok( Self::Octahedral ),
        Some( "QUATERNION" ) => Ok( Self::Quaternion ),
        Some( "EXPONENTIAL" ) => Ok( Self::Exponential ),
        Some( "COLOR" ) if extension == KHR_MESHOPT_COMPRESSION => Ok( Self::Color ),
        Some( other ) => Err( format!( "unknown {extension} filter `{other}`" ) ),
      }
    }

    /// Checks that this filter can decode elements of `stride` bytes.
    fn stride_check( self, stride : usize ) -> Result< (), String >
    {
      let fits = match self
      {
        Self::None | Self::Exponential => true,
        Self::Octahedral | Self::Color => stride == 4 || stride == 8,
        Self::Quaternion => stride == 8,
      };
      if fits { Ok( () ) } else { Err( format!( "filter {} does not support byteStride {stride}", self.as_str() ) ) }
    }
  }

  /// The decode spec parsed from one buffer view's meshopt extension object : where the
  /// view's compressed bytes live and what they decode into. It is not a view itself;
  /// the decoded bytes land in the range of the `gltf::buffer::View` it was read from.
  ///
  /// Its fields are private, so every spec has passed [`Self::parse`]'s checks; the
  /// accessors below only read them.
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub struct MeshoptViewSpec
  {
    /// Buffer holding the compressed bytes.
    buffer : usize,
    /// Offset of the compressed bytes in `buffer`.
    byte_offset : usize,
    /// Length of the compressed bytes.
    byte_length : usize,
    /// Size of one decoded element.
    byte_stride : usize,
    /// Number of decoded elements.
    count : usize,
    /// Encoding.
    mode : MeshoptMode,
    /// Post-decode transform.
    filter : MeshoptFilter,
  }

  impl MeshoptViewSpec
  {
    /// Reads `view`'s meshopt extension. `None` when the view is not compressed.
    #[ must_use ]
    pub fn from_view( view : &gltf::buffer::View< '_ > ) -> Option< Result< Self, String > >
    {
      MESHOPT_EXTENSIONS.iter()
      .find_map( | &name | view.extension_value( name ).map( | value | ( name, value ) ) )
      .map( | ( name, value ) | Self::parse( name, value ) )
    }

    /// Parses and validates an extension object against the rules of `extension`, one of
    /// [`MESHOPT_EXTENSIONS`], for strides, counts and filters. The `COLOR` filter exists
    /// only in `KHR_meshopt_compression`.
    ///
    /// # Errors
    ///
    /// Returns a description of the first missing or invalid field.
    pub fn parse( extension : &str, value : &Value ) -> Result< Self, String >
    {
      let field = | name : &str | -> Result< usize, String >
      {
        let v = value.get( name ).ok_or_else( || format!( "missing `{name}`" ) )?;
        usize_read( v ).ok_or_else( || format!( "`{name}` is not a non-negative integer" ) )
      };

      // A present but non-string `mode` or `filter` is malformed, not absent : reading it as
      // absent would decode with no filter, or report a `mode` that is there as missing.
      let string = | name : &str | -> Result< Option< &str >, String >
      {
        value.get( name ).map( | v | v.as_str().ok_or_else( || format!( "`{name}` is not a string" ) ) ).transpose()
      };
      let mode = MeshoptMode::from_name( string( "mode" )?.ok_or( "missing `mode`" )? )?;
      let filter = MeshoptFilter::from_name( string( "filter" )?, extension )?;

      let spec = Self
      {
        buffer : field( "buffer" )?,
        byte_offset : value.get( "byteOffset" ).map_or( Ok( 0 ), | _ | field( "byteOffset" ) )?,
        byte_length : field( "byteLength" )?,
        byte_stride : field( "byteStride" )?,
        count : field( "count" )?,
        mode,
        filter,
      };
      spec.mode.layout_check( spec.byte_stride, spec.filter, spec.count )?;
      spec.filter.stride_check( spec.byte_stride )?;

      Ok( spec )
    }

    /// Buffer holding the compressed bytes.
    #[ inline ]
    #[ must_use ]
    pub fn buffer( &self ) -> usize
    {
      self.buffer
    }

    /// Offset of the compressed bytes in [`Self::buffer`].
    #[ inline ]
    #[ must_use ]
    pub fn byte_offset( &self ) -> usize
    {
      self.byte_offset
    }

    /// Length of the compressed bytes.
    #[ inline ]
    #[ must_use ]
    pub fn byte_length( &self ) -> usize
    {
      self.byte_length
    }

    /// Size of one decoded element.
    #[ inline ]
    #[ must_use ]
    pub fn byte_stride( &self ) -> usize
    {
      self.byte_stride
    }

    /// Number of decoded elements.
    #[ inline ]
    #[ must_use ]
    pub fn count( &self ) -> usize
    {
      self.count
    }

    /// Encoding.
    #[ inline ]
    #[ must_use ]
    pub fn mode( &self ) -> MeshoptMode
    {
      self.mode
    }

    /// Post-decode transform.
    #[ inline ]
    #[ must_use ]
    pub fn filter( &self ) -> MeshoptFilter
    {
      self.filter
    }

    /// Size of the decoded data in bytes.
    #[ must_use ]
    pub fn decoded_length( &self ) -> Option< usize >
    {
      self.count.checked_mul( self.byte_stride )
    }

    /// Checks that the compressed bytes lie inside their buffer, that the decoded
    /// data fits the view they decode into, and that the two do not overlap: the
    /// decoder writes the view while it reads the compressed bytes, so a shared range
    /// would decode from data it has already overwritten. `buffer_lengths` holds every
    /// loaded buffer's length, by index.
    ///
    /// # Errors
    ///
    /// Returns a description of the first range that does not fit.
    pub fn ranges_check
    (
      &self,
      buffer_lengths : &[ usize ],
      view_buffer : usize,
      view_offset : usize,
      view_length : usize
    )
    -> Result< (), String >
    {
      let source_length = *buffer_lengths.get( self.buffer )
      .ok_or_else( || format!( "compressed data names buffer {}, which does not exist", self.buffer ) )?;
      if self.byte_offset.checked_add( self.byte_length ).map_or( true, | end | end > source_length )
      {
        return Err( format!( "compressed range {}+{} exceeds buffer {} ( {source_length} bytes )", self.byte_offset, self.byte_length, self.buffer ) );
      }

      let target_length = *buffer_lengths.get( view_buffer )
      .ok_or_else( || format!( "view names buffer {view_buffer}, which does not exist" ) )?;
      if view_offset.checked_add( view_length ).map_or( true, | end | end > target_length )
      {
        return Err( format!( "view range {view_offset}+{view_length} exceeds buffer {view_buffer} ( {target_length} bytes )" ) );
      }

      let decoded = match self.decoded_length()
      {
        Some( decoded ) if decoded <= view_length => decoded,
        _ => return Err( format!( "{} x {} decoded bytes do not fit the view's {view_length}", self.count, self.byte_stride ) ),
      };

      // Both ends were bounded by a buffer length above, so neither sum overflows.
      let overlaps = self.buffer == view_buffer
      && self.byte_offset < view_offset + decoded
      && view_offset < self.byte_offset + self.byte_length;
      if overlaps
      {
        return Err( format!
        (
          "compressed range {}+{} overlaps the decoded range {view_offset}+{decoded} in buffer {view_buffer}",
          self.byte_offset, self.byte_length
        ) );
      }
      Ok( () )
    }
  }

  /// Whether `buffer` is a meshopt fallback : it holds no data of its own and only
  /// receives decoded views. The loader allocates it instead of fetching it, even when it
  /// names a `uri` ( that file is for loaders without meshopt support ).
  #[ must_use ]
  pub fn buffer_is_fallback( buffer : &gltf::Buffer< '_ > ) -> bool
  {
    MESHOPT_EXTENSIONS.iter()
    .filter_map( | name | buffer.extension_value( name ) )
    .any( | ext | ext.get( "fallback" ).and_then( Value::as_bool ).unwrap_or( false ) )
  }

  /// Checks that the compressed views of one document cannot corrupt each other. Every
  /// view is decoded in turn into its own range, so no view's decoded range may overlap
  /// another's ( the later decode would overwrite the earlier one's output ), nor another
  /// view's compressed bytes ( a decode would overwrite input another view still reads,
  /// or has read ). Compressed bytes may overlap each other : they are only read.
  /// [`MeshoptViewSpec::ranges_check`] covers one view against itself.
  ///
  /// `decodes` holds, per compressed view : its index, its checked spec, and the buffer
  /// and offset of the range it decodes into.
  ///
  /// # Errors
  ///
  /// Returns a description of the first pair of views that overlap.
  pub fn decodes_overlap_check( decodes : &[ ( usize, &MeshoptViewSpec, usize, usize ) ] ) -> Result< (), String >
  {
    // `ranges_check` bounded every range by its buffer's length, so no end overflows.
    let target = | &( _, spec, buffer, offset ) : &( usize, &MeshoptViewSpec, usize, usize ) |
      ( buffer, offset, offset + spec.count * spec.byte_stride );
    let source = | &( _, spec, _, _ ) : &( usize, &MeshoptViewSpec, usize, usize ) |
      ( spec.buffer, spec.byte_offset, spec.byte_offset + spec.byte_length );
    let overlap = | ( a_buffer, a_start, a_end ) : ( usize, usize, usize ), ( b_buffer, b_start, b_end ) : ( usize, usize, usize ) |
      a_buffer == b_buffer && a_start < b_end && b_start < a_end;

    for ( i, a ) in decodes.iter().enumerate()
    {
      for b in &decodes[ i + 1 .. ]
      {
        if overlap( target( a ), target( b ) )
        {
          return Err( format!( "bufferViews {} and {} decode into overlapping ranges", a.0, b.0 ) );
        }
        if overlap( target( a ), source( b ) ) || overlap( target( b ), source( a ) )
        {
          return Err( format!( "bufferViews {} and {}: one decodes over the other's compressed bytes", a.0, b.0 ) );
        }
      }
    }
    Ok( () )
  }

  /// Decodes every meshopt-compressed buffer view into its own range of `buffers`, so the
  /// rest of the loader reads plain glTF data. Loads nothing and waits for nothing when no
  /// view is compressed.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` when an extension object is malformed, a range does not
  /// fit its buffer, two views overlap ( [`decodes_overlap_check`] ), or the decoder
  /// rejects the data; the details go to the console. Every view is checked before the
  /// first one is decoded.
  pub async fn views_decode
  (
    gltf_file : &gltf::Gltf,
    buffers : &[ js_sys::Uint8Array ]
  )
  -> Result< (), gl::WebglError >
  {
    let buffer_lengths = buffers.iter().map( | b | b.length() as usize ).collect::< Vec< _ > >();
    let decodes = decodes_collect( gltf_file, &buffer_lengths )?;
    if decodes.is_empty()
    {
      return Ok( () );
    }

    JsFuture::from( DECODER.with( Decoder::ready ) ).await
    .map_err( | e |
    {
      gl::browser::error!( "meshopt decoder failed to initialise: {e:?}" );
      gl::WebglError::Other( "glTF meshopt decoder failed to initialise" )
    } )?;

    for ( view, spec ) in &decodes
    {
      view_decode( spec, view, buffers )?;
    }

    Ok( () )
  }

  /// Every compressed view of `gltf_file` with its spec, each checked against
  /// `buffer_lengths` and all of them against each other, in document order.
  fn decodes_collect< 'a >
  (
    gltf_file : &'a gltf::Gltf,
    buffer_lengths : &[ usize ]
  )
  -> Result< Vec< ( gltf::buffer::View< 'a >, MeshoptViewSpec ) >, gl::WebglError >
  {
    let mut decodes = Vec::new();
    for view in gltf_file.views()
    {
      let Some( spec ) = MeshoptViewSpec::from_view( &view ) else { continue };
      let spec = spec
      .and_then( | spec |
      {
        spec.ranges_check( buffer_lengths, view.buffer().index(), view.offset(), view.length() )?;
        Ok( spec )
      } )
      .map_err( | e |
      {
        gl::browser::error!( "glTF bufferView {}: invalid meshopt compression: {e}", view.index() );
        gl::WebglError::Other( "glTF meshopt bufferView is invalid" )
      } )?;
      decodes.push( ( view, spec ) );
    }

    let placed = decodes.iter()
    .map( | ( view, spec ) | ( view.index(), spec, view.buffer().index(), view.offset() ) )
    .collect::< Vec< _ > >();
    decodes_overlap_check( &placed ).map_err( | e |
    {
      gl::browser::error!( "glTF meshopt compression: {e}" );
      gl::WebglError::Other( "glTF meshopt bufferViews overlap" )
    } )?;

    Ok( decodes )
  }

  /// Decodes one checked view's compressed bytes into its own range of `buffers`.
  fn view_decode( spec : &MeshoptViewSpec, view : &gltf::buffer::View< '_ >, buffers : &[ js_sys::Uint8Array ] ) -> Result< (), gl::WebglError >
  {
    // `ranges_check` bounded every value below by a JS array length, so none overflows.
    let as_u32 = | v : usize | u32::try_from( v ).unwrap_or( u32::MAX );
    let source = buffers[ spec.buffer() ].subarray
    (
      as_u32( spec.byte_offset() ),
      as_u32( spec.byte_offset() + spec.byte_length() )
    );
    let target = buffers[ view.buffer().index() ].subarray
    (
      as_u32( view.offset() ),
      as_u32( view.offset() + spec.count() * spec.byte_stride() )
    );

    DECODER.with( | d |
    {
      d.decode_gltf_buffer
      (
        &target,
        as_u32( spec.count() ),
        as_u32( spec.byte_stride() ),
        &source,
        spec.mode().as_str(),
        spec.filter().as_str()
      )
    } )
    .map_err( | e |
    {
      gl::browser::error!( "glTF bufferView {}: meshopt decode failed: {e:?}", view.index() );
      gl::WebglError::Other( "glTF meshopt decode failed" )
    } )
  }
}

crate::mod_interface!
{
  own use views_decode;
}

// The loader's other stages need the extension names and the fallback test; nothing outside
// the crate does.
pub( crate ) use private::{ EXT_MESHOPT_COMPRESSION, KHR_MESHOPT_COMPRESSION, buffer_is_fallback };

/// The parsing and range checks `tests/` reaches, exposed only under `test_internals`.
///
/// Not part of the surface: with the feature off, the default and what every dependent gets,
/// this module does not exist.
#[ cfg( feature = "test_internals" ) ]
#[ doc( hidden ) ]
pub mod internal
{
  pub use super::private::
  {
    EXT_MESHOPT_COMPRESSION,
    KHR_MESHOPT_COMPRESSION,
    MESHOPT_EXTENSIONS,
    MeshoptMode,
    MeshoptFilter,
    MeshoptViewSpec,
    buffer_is_fallback,
    decodes_overlap_check,
  };
}

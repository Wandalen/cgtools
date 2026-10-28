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
  // JavaScript MIME type; module scripts refuse anything else. It inlines two builds of its
  // wasm decoder, one using SIMD and one without, picks the SIMD one when
  // `WebAssembly.validate` accepts it, and compiles it with `WebAssembly.instantiate`, so the
  // host page needs `wasm-unsafe-eval` in its CSP -- the same permission this crate's own wasm
  // already needs.
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

  /// The Khronos ratification of [`EXT_MESHOPT_COMPRESSION`], with the same JSON schema. It
  /// differs in :
  /// - the bitstream : `ATTRIBUTES` data may use version 1 of the vertex codec ( header
  ///   `0xa1`, with its channel and control modes ) besides EXT's version 0. The vendored
  ///   decoder reads both, so nothing here depends on the version;
  /// - the `COLOR` filter, for YCoCg-encoded colours;
  /// - the rules : it drops EXT's "a parent `byteStride` matches the extension's", and forbids
  ///   using both extensions on one buffer view or buffer.
  pub const KHR_MESHOPT_COMPRESSION : &str = "KHR_meshopt_compression";

  /// Why a buffer view or buffer carrying both meshopt extension objects is refused.
  const BOTH_EXTENSIONS : &str = "uses both EXT_meshopt_compression and KHR_meshopt_compression, which KHR_meshopt_compression forbids";

  /// Extension names that mark a buffer view as meshopt-compressed. `gltf_extensions`'
  /// supported-extensions list names the same two constants, written out there by hand;
  /// `gltf_meshopt_test` checks that every name here is in it.
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
    /// Floats, each stored as a 32-bit integer of a signed 8-bit exponent and a signed
    /// 24-bit mantissa. An encoder may give several values the same exponent; the format
    /// does not require it.
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
    /// Whether it was read under `KHR_meshopt_compression` rather than
    /// `EXT_meshopt_compression`, whose rules differ.
    khr : bool,
  }

  impl MeshoptViewSpec
  {
    /// Reads `view`'s meshopt extension. `None` when the view is not compressed, an error when
    /// it carries both extension objects : `KHR_meshopt_compression` must not be used on a
    /// buffer view that also uses `EXT_meshopt_compression`, and picking one would decode the
    /// view under rules its author may not have meant.
    #[ must_use ]
    pub fn from_view( view : &gltf::buffer::View< '_ > ) -> Option< Result< Self, String > >
    {
      let mut found = MESHOPT_EXTENSIONS.iter()
      .filter_map( | &name | view.extension_value( name ).map( | value | ( name, value ) ) );
      let ( name, value ) = found.next()?;
      if found.next().is_some()
      {
        return Some( Err( BOTH_EXTENSIONS.to_owned() ) );
      }
      Some( Self::parse( name, value ) )
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
        khr : extension == KHR_MESHOPT_COMPRESSION,
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

    /// Checks the spec against the view it decodes into and the loaded buffers : the
    /// compressed bytes lie inside their buffer; the view is exactly `count * byteStride`
    /// bytes, as both meshopt specs require of the parent bufferView, and under
    /// `EXT_meshopt_compression` a `byteStride` it declares is the extension's (
    /// `KHR_meshopt_compression` dropped that rule ); and the view does not overlap the
    /// compressed bytes : the decoder writes the view while it reads them, so a shared
    /// range would decode from data it has already overwritten. `buffer_lengths` holds
    /// every loaded buffer's length, by index.
    ///
    /// # Errors
    ///
    /// Returns a description of the first rule the view breaks.
    pub fn ranges_check
    (
      &self,
      buffer_lengths : &[ usize ],
      view_buffer : usize,
      view_offset : usize,
      view_length : usize,
      view_stride : Option< usize >
    )
    -> Result< (), String >
    {
      let source_length = *buffer_lengths.get( self.buffer )
      .ok_or_else( || format!( "compressed data names buffer {}, which does not exist", self.buffer ) )?;
      let source_end = self.byte_offset.checked_add( self.byte_length ).filter( | &end | end <= source_length )
      .ok_or_else( || format!( "compressed range {}+{} exceeds buffer {} ( {source_length} bytes )", self.byte_offset, self.byte_length, self.buffer ) )?;

      let target_length = *buffer_lengths.get( view_buffer )
      .ok_or_else( || format!( "view names buffer {view_buffer}, which does not exist" ) )?;
      let target_end = view_offset.checked_add( view_length ).filter( | &end | end <= target_length )
      .ok_or_else( || format!( "view range {view_offset}+{view_length} exceeds buffer {view_buffer} ( {target_length} bytes )" ) )?;

      if self.decoded_length() != Some( view_length )
      {
        return Err( format!( "{} x {} decoded bytes do not match the view's byteLength {view_length}", self.count, self.byte_stride ) );
      }
      if let Some( stride ) = view_stride.filter( | &s | !self.khr && s != self.byte_stride )
      {
        return Err( format!( "the view's byteStride {stride} is not the extension's {}", self.byte_stride ) );
      }

      // The view is exactly the decoded data, so `target_end` is where the decode stops.
      let overlaps = self.buffer == view_buffer && self.byte_offset < target_end && view_offset < source_end;
      if overlaps
      {
        return Err( format!
        (
          "compressed range {}+{} overlaps the decoded range {view_offset}+{view_length} in buffer {view_buffer}",
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

  /// Checks that decoding the compressed views of one document cannot corrupt any view's
  /// data. Every view is decoded in turn into its own range, so no view's decoded range may
  /// overlap another's ( the later decode would overwrite the earlier one's output ), any
  /// view's compressed bytes ( a decode would overwrite input a view still reads, or has
  /// read ), nor the range of an uncompressed view ( an accessor reading it would get
  /// decoded bytes of another view ). Compressed bytes and uncompressed views may overlap
  /// each other : they are only read.
  ///
  /// `decodes` holds, per compressed view : its index, its checked spec, and the buffer
  /// and offset of the range it decodes into. `plain` holds, per uncompressed view : its
  /// index, buffer, offset and length.
  ///
  /// Sorts every range by buffer and start, then sweeps each buffer once, keeping the
  /// decoded and the read range that reach furthest so far : a range that starts before
  /// one of them ends overlaps it. `O( n log n )` in the number of views, so a file
  /// declaring many small views cannot make the check itself the expensive part.
  ///
  /// # Errors
  ///
  /// Returns a description of the first overlap found, or of a range whose end overflows.
  /// A zero-length range overlaps nothing.
  pub fn decodes_overlap_check
  (
    decodes : &[ ( usize, &MeshoptViewSpec, usize, usize ) ],
    plain : &[ ( usize, usize, usize, usize ) ],
  )
  -> Result< (), String >
  {
    /// What a range holds : a decode writes `Decoded`, the other two are only read.
    #[ derive( Clone, Copy, PartialEq, Eq ) ]
    enum Held { Decoded, Compressed, Plain }
    /// One range : buffer, start, end, what it holds, view.
    type Range = ( usize, usize, usize, Held, usize );

    let mut ranges : Vec< Range > = Vec::with_capacity( decodes.len() * 2 + plain.len() );
    for &( view, spec, buffer, offset ) in decodes
    {
      let decoded_end = spec.decoded_length().and_then( | length | offset.checked_add( length ) )
      .ok_or_else( || format!( "bufferView {view}: the decoded range overflows" ) )?;
      let source_end = spec.byte_offset().checked_add( spec.byte_length() )
      .ok_or_else( || format!( "bufferView {view}: the compressed range overflows" ) )?;
      ranges.push( ( buffer, offset, decoded_end, Held::Decoded, view ) );
      ranges.push( ( spec.buffer(), spec.byte_offset(), source_end, Held::Compressed, view ) );
    }
    for &( view, buffer, offset, length ) in plain
    {
      let end = offset.checked_add( length ).ok_or_else( || format!( "bufferView {view}: its range overflows" ) )?;
      ranges.push( ( buffer, offset, end, Held::Plain, view ) );
    }
    // A zero-length range shares no byte with anything, yet would test as
    // overlapping a range that ends after its start.
    ranges.retain( | &( _, start, end, .. ) | end > start );
    ranges.sort_unstable_by_key( | &( buffer, start, .. ) | ( buffer, start ) );

    let read = | held : Held | if held == Held::Compressed { "the compressed bytes" } else { "the data" };
    let mut reach_decoded : Option< Range > = None;
    let mut reach_read : Option< Range > = None;
    for range in ranges
    {
      let ( buffer, start, end, held, view ) = range;
      let reaching = | furthest : Option< Range > | furthest.filter( | f | f.0 == buffer && f.2 > start );
      if let Some( other ) = reaching( reach_decoded )
      {
        return Err( if held == Held::Decoded
        {
          format!( "bufferViews {} and {view} decode into overlapping ranges", other.4 )
        }
        else
        {
          format!( "bufferView {} decodes over {} of bufferView {view}", other.4, read( held ) )
        } );
      }
      if let Some( other ) = reaching( reach_read ).filter( | _ | held == Held::Decoded )
      {
        return Err( format!( "bufferView {view} decodes over {} of bufferView {}", read( other.3 ), other.4 ) );
      }

      let furthest = if held == Held::Decoded { &mut reach_decoded } else { &mut reach_read };
      if furthest.map_or( true, | f | f.0 != buffer || f.2 < end )
      {
        *furthest = Some( range );
      }
    }
    Ok( () )
  }

  /// Decodes every meshopt-compressed buffer view into its own range of `buffers`, so the
  /// rest of the loader reads plain glTF data. With no compressed view it decodes nothing and
  /// does not wait for the decoder.
  ///
  /// The decoder is not loaded on demand, though : it is a static import of the crate's JS
  /// glue, so every page that links this crate fetches it and starts compiling its wasm at
  /// load, whatever the asset. That import also ties the crate to wasm-bindgen's JS-module
  /// targets ( see the readme's "Compressed glTF" ).
  ///
  /// Every view is checked before the first one is decoded, but the decode itself is not
  /// atomic : when the decoder rejects a view, the views before it are already written into
  /// `buffers`. `gltf::load` drops the buffers on any error; a direct caller should too.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other`, with the details on the console, when :
  /// - a buffer view or buffer breaks a meshopt rule : a malformed extension object, both
  ///   meshopt extensions on one view or buffer, a parent layout that does not match the
  ///   decoded data, a range outside its buffer, compressed bytes in a fallback buffer, or an
  ///   uncompressed view reading a fallback buffer;
  /// - two views overlap ( `decodes_overlap_check` );
  /// - the decoder fails to initialise;
  /// - the decoder rejects the data.
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
      spec_decode( spec, view, buffers )?;
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
    if let Some( buffer ) = gltf_file.buffers().find( | b | MESHOPT_EXTENSIONS.iter().all( | name | b.extension_value( name ).is_some() ) )
    {
      gl::browser::error!( "glTF buffer {}: {BOTH_EXTENSIONS}", buffer.index() );
      return Err( gl::WebglError::Other( "glTF meshopt buffer is invalid" ) );
    }

    // A fallback buffer holds only what compressed views decode into it, so the meshopt specs
    // allow references to it from those views alone, and none from their extension objects.
    let fallback = gltf_file.buffers().map( | b | buffer_is_fallback( &b ) ).collect::< Vec< _ > >();
    let is_fallback = | buffer : usize | fallback.get( buffer ).copied().unwrap_or( false );

    let mut decodes = Vec::new();
    let mut plain = Vec::new();
    for view in gltf_file.views()
    {
      let Some( spec ) = MeshoptViewSpec::from_view( &view ) else
      {
        if is_fallback( view.buffer().index() )
        {
          gl::browser::error!
          (
            "glTF bufferView {} is not compressed but reads fallback buffer {}, which holds no data of its own",
            view.index(), view.buffer().index()
          );
          return Err( gl::WebglError::Other( "glTF meshopt fallback buffer is read by an uncompressed bufferView" ) );
        }
        plain.push( ( view.index(), view.buffer().index(), view.offset(), view.length() ) );
        continue;
      };
      let spec = spec
      .and_then( | spec |
      {
        if is_fallback( spec.buffer() )
        {
          return Err( format!( "compressed data names fallback buffer {}, which holds no data of its own", spec.buffer() ) );
        }
        spec.ranges_check( buffer_lengths, view.buffer().index(), view.offset(), view.length(), view.stride() )?;
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
    decodes_overlap_check( &placed, &plain ).map_err( | e |
    {
      gl::browser::error!( "glTF meshopt compression: {e}" );
      gl::WebglError::Other( "glTF meshopt bufferViews overlap" )
    } )?;

    Ok( decodes )
  }

  /// Decodes one checked view's compressed bytes into its own range of `buffers`.
  fn spec_decode( spec : &MeshoptViewSpec, view : &gltf::buffer::View< '_ >, buffers : &[ js_sys::Uint8Array ] ) -> Result< (), gl::WebglError >
  {
    // `ranges_check` bounded both ranges by a JS array length, so neither fails here; a spec
    // that skipped it is refused rather than decoded out of range.
    let range = | start : usize, length : Option< usize > | -> Option< ( u32, u32 ) >
    {
      let end = start.checked_add( length? )?;
      Some( ( u32::try_from( start ).ok()?, u32::try_from( end ).ok()? ) )
    };
    let ( Some( ( source_start, source_end ) ), Some( ( target_start, target_end ) ) ) =
      ( range( spec.byte_offset(), Some( spec.byte_length() ) ), range( view.offset(), spec.decoded_length() ) )
    else
    {
      gl::browser::error!( "glTF bufferView {}: meshopt ranges do not fit a JS array", view.index() );
      return Err( gl::WebglError::Other( "glTF meshopt bufferView is invalid" ) );
    };
    let source = buffers[ spec.buffer() ].subarray( source_start, source_end );
    let target = buffers[ view.buffer().index() ].subarray( target_start, target_end );
    // Each is at most the decoded length, which fits a `u32`.
    let as_u32 = | v : usize | u32::try_from( v ).unwrap_or( u32::MAX );

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

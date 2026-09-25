mod private
{
  use minwebgl as gl;
  use gl::js_sys;
  use gltf::json::Value;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen_futures::JsFuture;

  // meshoptimizer's own decoder ( MIT, vendored unchanged from the `meshoptimizer` npm
  // package 1.2.0 ). Its `meshopt_decoder.mjs` is renamed to `.js` so that every server
  // sends it with a JavaScript MIME type; module scripts refuse anything else. It carries
  // its decoder as an inlined, SIMD-enabled wasm module, so the host page needs
  // `wasm-unsafe-eval` in its CSP -- the same permission this crate's own wasm already needs.
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

  /// Extension names that mark a buffer view as meshopt-compressed : the original
  /// `EXT_meshopt_compression` and its Khronos ratification, which keeps the same
  /// schema and adds the `COLOR` filter.
  pub const MESHOPT_EXTENSIONS : &[ &str ] = &[ "EXT_meshopt_compression", "KHR_meshopt_compression" ];

  /// How a compressed buffer view was encoded.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
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
  }

  /// The post-decode transform applied to attribute data.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
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
  }

  /// One buffer view's meshopt extension object : where its compressed bytes live and
  /// what they decode into. The decoded bytes land in the view's own `buffer` range.
  #[ derive( Debug, Clone, PartialEq, Eq ) ]
  pub struct MeshoptView
  {
    /// Buffer holding the compressed bytes.
    pub buffer : usize,
    /// Offset of the compressed bytes in `buffer`.
    pub byte_offset : usize,
    /// Length of the compressed bytes.
    pub byte_length : usize,
    /// Size of one decoded element.
    pub byte_stride : usize,
    /// Number of decoded elements.
    pub count : usize,
    /// Encoding.
    pub mode : MeshoptMode,
    /// Post-decode transform.
    pub filter : MeshoptFilter,
  }

  impl MeshoptView
  {
    /// Reads `view`'s meshopt extension. `None` when the view is not compressed.
    #[ must_use ]
    pub fn from_view( view : &gltf::buffer::View< '_ > ) -> Option< Result< Self, String > >
    {
      MESHOPT_EXTENSIONS.iter()
      .find_map( | name | view.extension_value( name ) )
      .map( Self::parse )
    }

    /// Parses and validates an extension object against the extension's rules for
    /// strides, counts and filters.
    ///
    /// # Errors
    ///
    /// Returns a description of the first missing or invalid field.
    pub fn parse( value : &Value ) -> Result< Self, String >
    {
      let field = | name : &str | -> Result< usize, String >
      {
        let v = value.get( name ).ok_or_else( || format!( "missing `{name}`" ) )?;
        v.as_u64()
        .and_then( | v | usize::try_from( v ).ok() )
        .ok_or_else( || format!( "`{name}` is not a non-negative integer" ) )
      };

      let mode = match value.get( "mode" ).and_then( Value::as_str )
      {
        Some( "ATTRIBUTES" ) => MeshoptMode::Attributes,
        Some( "TRIANGLES" ) => MeshoptMode::Triangles,
        Some( "INDICES" ) => MeshoptMode::Indices,
        Some( other ) => return Err( format!( "unknown mode `{other}`" ) ),
        None => return Err( "missing `mode`".to_string() ),
      };
      let filter = match value.get( "filter" ).and_then( Value::as_str )
      {
        None | Some( "NONE" ) => MeshoptFilter::None,
        Some( "OCTAHEDRAL" ) => MeshoptFilter::Octahedral,
        Some( "QUATERNION" ) => MeshoptFilter::Quaternion,
        Some( "EXPONENTIAL" ) => MeshoptFilter::Exponential,
        Some( "COLOR" ) => MeshoptFilter::Color,
        Some( other ) => return Err( format!( "unknown filter `{other}`" ) ),
      };

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

      let stride = spec.byte_stride;
      match spec.mode
      {
        MeshoptMode::Attributes =>
        {
          if stride == 0 || stride % 4 != 0 || stride > 256
          {
            return Err( format!( "ATTRIBUTES byteStride {stride} must be a multiple of 4 up to 256" ) );
          }
        },
        MeshoptMode::Triangles | MeshoptMode::Indices =>
        {
          if stride != 2 && stride != 4
          {
            return Err( format!( "{} byteStride {stride} must be 2 or 4", spec.mode.as_str() ) );
          }
          if spec.filter != MeshoptFilter::None
          {
            return Err( format!( "{} data cannot use a filter", spec.mode.as_str() ) );
          }
        },
      }
      if spec.mode == MeshoptMode::Triangles && spec.count % 3 != 0
      {
        return Err( format!( "TRIANGLES count {} is not a multiple of 3", spec.count ) );
      }
      let filter_fits = match spec.filter
      {
        MeshoptFilter::None | MeshoptFilter::Exponential => true,
        MeshoptFilter::Octahedral | MeshoptFilter::Color => stride == 4 || stride == 8,
        MeshoptFilter::Quaternion => stride == 8,
      };
      if !filter_fits
      {
        return Err( format!( "filter {} does not support byteStride {stride}", spec.filter.as_str() ) );
      }

      Ok( spec )
    }

    /// Size of the decoded data in bytes.
    #[ must_use ]
    pub fn decoded_length( &self ) -> Option< usize >
    {
      self.count.checked_mul( self.byte_stride )
    }

    /// Checks that the compressed bytes lie inside their buffer and that the decoded
    /// data fits the view they decode into. `buffer_lengths` holds every loaded buffer's
    /// length, by index.
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

      match self.decoded_length()
      {
        Some( decoded ) if decoded <= view_length => Ok( () ),
        _ => Err( format!( "{} x {} decoded bytes do not fit the view's {view_length}", self.count, self.byte_stride ) ),
      }
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

  /// Decodes every meshopt-compressed buffer view into its own range of `buffers`, so the
  /// rest of the loader reads plain glTF data. Loads nothing and waits for nothing when no
  /// view is compressed.
  ///
  /// # Errors
  ///
  /// Returns `WebglError::Other` when an extension object is malformed, a range does not
  /// fit its buffer, or the decoder rejects the data; the details go to the console.
  pub async fn views_decode
  (
    gltf_file : &gltf::Gltf,
    buffers : &[ js_sys::Uint8Array ]
  )
  -> Result< (), gl::WebglError >
  {
    let buffer_lengths = buffers.iter().map( | b | b.length() as usize ).collect::< Vec< _ > >();
    let mut decoder_ready = false;

    for view in gltf_file.views()
    {
      let Some( spec ) = MeshoptView::from_view( &view ) else { continue };
      let spec = spec
      .and_then( | spec |
      {
        spec.ranges_check( &buffer_lengths, view.buffer().index(), view.offset(), view.length() )?;
        Ok( spec )
      } )
      .map_err( | e |
      {
        gl::browser::error!( "glTF bufferView {}: invalid meshopt compression: {e}", view.index() );
        gl::WebglError::Other( "Invalid meshopt-compressed buffer view" )
      } )?;

      if !decoder_ready
      {
        JsFuture::from( DECODER.with( Decoder::ready ) ).await
        .map_err( | e |
        {
          gl::browser::error!( "meshopt decoder failed to initialise: {e:?}" );
          gl::WebglError::Other( "meshopt decoder failed to initialise" )
        } )?;
        decoder_ready = true;
      }

      // `ranges_check` bounded every value below by a JS array length, so none overflows.
      let as_u32 = | v : usize | u32::try_from( v ).unwrap_or( u32::MAX );
      let source = buffers[ spec.buffer ].subarray
      (
        as_u32( spec.byte_offset ),
        as_u32( spec.byte_offset + spec.byte_length )
      );
      let target = buffers[ view.buffer().index() ].subarray
      (
        as_u32( view.offset() ),
        as_u32( view.offset() + spec.count * spec.byte_stride )
      );

      DECODER.with( | d |
      {
        d.decode_gltf_buffer
        (
          &target,
          as_u32( spec.count ),
          as_u32( spec.byte_stride ),
          &source,
          spec.mode.as_str(),
          spec.filter.as_str()
        )
      } )
      .map_err( | e |
      {
        gl::browser::error!( "glTF bufferView {}: meshopt decode failed: {e:?}", view.index() );
        gl::WebglError::Other( "meshopt decode failed" )
      } )?;
    }

    Ok( () )
  }
}

crate::mod_interface!
{
  own use
  {
    MESHOPT_EXTENSIONS,
    MeshoptMode,
    MeshoptFilter,
    MeshoptView,
    buffer_is_fallback,
    views_decode
  };
}

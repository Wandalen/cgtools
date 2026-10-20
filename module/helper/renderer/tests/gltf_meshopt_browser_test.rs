//! Browser tests for loading meshopt-compressed glTF.
//!
//! - The decode itself : `meshopt::views_decode` runs meshoptimizer's vendored JS decoder,
//!   which needs a browser, on bytes encoded by a minimal vertex-codec encoder below.
//! - The buffer-index rules of `gltf::load` : a meshopt fallback buffer is allocated rather
//!   than fetched, the GLB binary chunk can only be `buffers[ 0 ]`, a uri-less buffer with no
//!   chunk to take is an error, and oversized fallbacks, alone or together, are refused
//!   instead of allocated.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::loaders::{ gltf, meshopt };

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap()
  }

  fn document() -> gl::web_sys::Document
  {
    gl::web_sys::window().unwrap().document().unwrap()
  }

  /// One triangle's `POSITION` data : three `f32` VEC3 elements, 12 bytes each.
  fn triangle_bytes() -> Vec< u8 >
  {
    [ 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0_f32 ].iter().flat_map( | f | f.to_le_bytes() ).collect()
  }

  /// Encodes `data`, a run of `stride`-byte elements, as `EXT_meshopt_compression`'s
  /// `ATTRIBUTES` mode stores it : meshoptimizer's vertex codec, version 0.
  ///
  /// The codec splits elements into blocks and each block into byte columns. Column `k` holds
  /// byte `k` of every element as zigzag-encoded deltas from the previous element ( the first
  /// element is its own predecessor ), in groups of 16 with 2 header bits per group. This
  /// encoder writes only the two simplest group kinds : header bits 0 for an all-zero group,
  /// which stores nothing, and bits 3 for 16 raw bytes. The stream ends with a tail of at least
  /// 32 bytes whose last `stride` bytes are the first element. Every element must fit one block,
  /// which holds 256 elements for strides up to 32.
  fn meshopt_attributes_encode( data : &[ u8 ], stride : usize ) -> Vec< u8 >
  {
    assert!( stride % 4 == 0 && stride <= 32 && data.len() % stride == 0 );
    let count = data.len() / stride;
    assert!( count <= 256, "one block only" );
    let aligned = count.div_ceil( 16 ) * 16;
    let groups = aligned / 16;

    let mut out = vec![ 0xa0 ];
    for k in 0..stride
    {
      let mut deltas = vec![ 0u8; aligned ];
      let mut previous = data[ k ];
      for ( i, delta ) in deltas.iter_mut().enumerate().take( count )
      {
        let value = data[ i * stride + k ];
        let d = value.wrapping_sub( previous );
        *delta = ( d << 1 ) ^ if d & 0x80 != 0 { 0xff } else { 0 };
        previous = value;
      }

      let mut header = vec![ 0u8; groups.div_ceil( 4 ) ];
      let mut body = Vec::new();
      for ( g, group ) in deltas.chunks( 16 ).enumerate()
      {
        if group.iter().any( | &b | b != 0 )
        {
          header[ g / 4 ] |= 3 << ( ( g % 4 ) * 2 );
          body.extend_from_slice( group );
        }
      }
      out.extend( header );
      out.extend( body );
    }

    out.resize( out.len() + 32_usize.saturating_sub( stride ), 0 );
    out.extend_from_slice( &data[ ..stride ] );
    out
  }

  /// The reason `gltf::load` refuses `uri` : the static message of its `WebglError::Other`,
  /// so a test can tell its intended rejection from any other failure.
  async fn load_refusal( uri : &str ) -> &'static str
  {
    match gltf::load( &document(), uri, &gl_init() ).await
    {
      Ok( _ ) => panic!( "the file must be refused" ),
      Err( gl::WebglError::Other( reason ) ) => reason,
      Err( other ) => panic!( "refused for an unexpected reason: {other:?}" ),
    }
  }

  fn data_uri( mime : &str, bytes : &[ u8 ] ) -> String
  {
    let latin1 : String = bytes.iter().map( | &b | char::from( b ) ).collect();
    format!( "data:{mime};base64,{}", gl::web_sys::window().unwrap().btoa( &latin1 ).unwrap() )
  }

  /// A one-triangle scene whose `POSITION` accessor reads bufferView 0. `buffers` and `view` are
  /// spliced in as JSON; `used` lists extensions for `extensionsUsed` / `extensionsRequired`.
  fn triangle_gltf( used : &[ &str ], buffers : &str, view : &str ) -> String
  {
    let extensions = if used.is_empty()
    {
      String::new()
    }
    else
    {
      let names = used.iter().map( | name | format!( "\"{name}\"" ) ).collect::< Vec< _ > >().join( ", " );
      format!( r#""extensionsUsed" : [ {names} ], "extensionsRequired" : [ {names} ],"# )
    };
    format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        {extensions}
        "buffers" : [ {buffers} ],
        "bufferViews" : [ {view} ],
        "accessors" : [ {{ "bufferView" : 0, "componentType" : 5126, "count" : 3, "type" : "VEC3", "min" : [ 0, 0, 0 ], "max" : [ 1, 1, 0 ] }} ],
        "materials" : [ {{}} ],
        "meshes" : [ {{ "primitives" : [ {{ "attributes" : {{ "POSITION" : 0 }}, "material" : 0 }} ] }} ],
        "nodes" : [ {{ "mesh" : 0 }} ],
        "scenes" : [ {{ "nodes" : [ 0 ] }} ],
        "scene" : 0
      }}"#
    )
  }

  /// The triangle as meshopt output : the compressed bytes in buffer 0 and a fallback buffer 1
  /// that bufferView 0 decodes into, the layout glTF-Transform and gltfpack write.
  fn compressed_triangle_gltf( compressed : &[ u8 ] ) -> String
  {
    let len = compressed.len();
    triangle_gltf
    (
      &[ "EXT_meshopt_compression" ],
      &format!
      (
        r#"{{ "byteLength" : {len}, "uri" : "{}" }},
        {{ "byteLength" : 36, "extensions" : {{ "EXT_meshopt_compression" : {{ "fallback" : true }} }} }}"#,
        data_uri( "application/octet-stream", compressed )
      ),
      &format!
      (
        r#"{{ "buffer" : 1, "byteLength" : 36, "byteStride" : 12, "target" : 34962,
          "extensions" : {{ "EXT_meshopt_compression" :
            {{ "buffer" : 0, "byteLength" : {len}, "byteStride" : 12, "count" : 3, "mode" : "ATTRIBUTES" }} }} }}"#
      )
    )
  }

  /// Packs `json` and `bin` into a GLB container : a 12-byte header, then the JSON chunk padded
  /// with spaces and the BIN chunk padded with zeros, each behind its length and type.
  fn glb( json : &str, bin : &[ u8 ] ) -> Vec< u8 >
  {
    let as_u32 = | n : usize | u32::try_from( n ).unwrap().to_le_bytes();
    let mut json = json.as_bytes().to_vec();
    json.resize( json.len().next_multiple_of( 4 ), b' ' );
    let mut bin = bin.to_vec();
    bin.resize( bin.len().next_multiple_of( 4 ), 0 );

    let mut out = Vec::new();
    out.extend_from_slice( b"glTF" );
    out.extend_from_slice( &2_u32.to_le_bytes() );
    out.extend_from_slice( &as_u32( 12 + 8 + json.len() + 8 + bin.len() ) );
    out.extend_from_slice( &as_u32( json.len() ) );
    out.extend_from_slice( b"JSON" );
    out.extend( json );
    out.extend_from_slice( &as_u32( bin.len() ) );
    out.extend_from_slice( b"BIN\0" );
    out.extend( bin );
    out
  }

  /// Reads back the GPU buffer `load` uploaded for bufferView `view`.
  fn view_bytes_read( gl : &GL, loaded : &gltf::GLTF, view : usize, length : usize ) -> Vec< u8 >
  {
    let mut bytes = vec![ 0u8; length ];
    gl.bind_buffer( gl::COPY_READ_BUFFER, Some( &loaded.gl_buffers[ view ] ) );
    gl.get_buffer_sub_data_with_i32_and_u8_array( gl::COPY_READ_BUFFER, 0, &mut bytes );
    gl.bind_buffer( gl::COPY_READ_BUFFER, None );
    bytes
  }

  /// `views_decode` runs the vendored decoder on each compressed view and writes the decoded
  /// bytes into the view's range of its fallback buffer.
  #[ wasm_bindgen_test( async ) ]
  async fn views_decode_fills_the_fallback_buffer()
  {
    let compressed = meshopt_attributes_encode( &triangle_bytes(), 12 );
    let document = ::gltf::Gltf::from_slice_without_validation( compressed_triangle_gltf( &compressed ).as_bytes() ).unwrap();
    let buffers = [ gl::js_sys::Uint8Array::from( compressed.as_slice() ), gl::js_sys::Uint8Array::new_with_length( 36 ) ];

    meshopt::views_decode( &document, &buffers ).await.expect( "the encoded triangle decodes" );

    assert_eq!( buffers[ 1 ].to_vec(), triangle_bytes() );
  }

  /// Data the decoder rejects comes back as an error, not a JS exception through the wasm frames.
  #[ wasm_bindgen_test( async ) ]
  async fn views_decode_reports_malformed_data()
  {
    let mut compressed = meshopt_attributes_encode( &triangle_bytes(), 12 );
    compressed[ 0 ] = 0xb0;
    let document = ::gltf::Gltf::from_slice_without_validation( compressed_triangle_gltf( &compressed ).as_bytes() ).unwrap();
    let buffers = [ gl::js_sys::Uint8Array::from( compressed.as_slice() ), gl::js_sys::Uint8Array::new_with_length( 36 ) ];

    assert!( meshopt::views_decode( &document, &buffers ).await.is_err(), "an unknown codec header must not decode" );
  }

  /// `load` allocates the fallback buffer instead of fetching it, decodes into it, and uploads
  /// the decoded positions.
  #[ wasm_bindgen_test( async ) ]
  async fn load_uploads_decoded_positions()
  {
    let gl = gl_init();
    let compressed = meshopt_attributes_encode( &triangle_bytes(), 12 );
    let uri = data_uri( "model/gltf+json", compressed_triangle_gltf( &compressed ).as_bytes() );

    let loaded = gltf::load( &document(), &uri, &gl ).await.expect( "the compressed triangle loads" );

    assert_eq!( view_bytes_read( &gl, &loaded, 0, 36 ), triangle_bytes() );
  }

  /// A GLB's binary chunk is `buffers[ 0 ]`.
  #[ wasm_bindgen_test( async ) ]
  async fn load_reads_the_glb_chunk_as_buffer_0()
  {
    let gl = gl_init();
    let json = triangle_gltf( &[], r#"{ "byteLength" : 36 }"#, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf-binary", &glb( &json, &triangle_bytes() ) );

    let loaded = gltf::load( &document(), &uri, &gl ).await.expect( "the GLB triangle loads" );

    assert_eq!( view_bytes_read( &gl, &loaded, 0, 36 ), triangle_bytes() );
  }

  /// Only `buffers[ 0 ]` may omit its uri : a later uri-less buffer is malformed even when the
  /// file carries a binary chunk, and must not receive that chunk.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_a_uri_less_buffer_after_the_first()
  {
    let buffers = format!
    (
      r#"{{ "byteLength" : 36, "uri" : "{}" }}, {{ "byteLength" : 36 }}"#,
      data_uri( "application/octet-stream", &triangle_bytes() )
    );
    let json = triangle_gltf( &[], &buffers, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf-binary", &glb( &json, &triangle_bytes() ) );

    assert_eq!( load_refusal( &uri ).await, "glTF buffer other than buffer 0 has no uri" );
  }

  /// A uri-less buffer in a file with no binary chunk has no data to load.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_a_uri_less_buffer_without_a_glb_chunk()
  {
    let json = triangle_gltf( &[], r#"{ "byteLength" : 36 }"#, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert_eq!( load_refusal( &uri ).await, "glTF buffer 0 has no uri and no GLB binary chunk" );
  }

  /// A fallback buffer's size comes only from its declared `byteLength`, so a huge declaration
  /// is refused with an error rather than allocated or thrown as a `RangeError`.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_an_oversized_fallback_buffer()
  {
    let buffers = format!
    (
      r#"{{ "byteLength" : 36, "uri" : "{}" }},
      {{ "byteLength" : 3000000000, "extensions" : {{ "EXT_meshopt_compression" : {{ "fallback" : true }} }} }}"#,
      data_uri( "application/octet-stream", &triangle_bytes() )
    );
    let json = triangle_gltf( &[ "EXT_meshopt_compression" ], &buffers, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert_eq!( load_refusal( &uri ).await, "glTF meshopt fallback buffers are too large" );
  }

  /// The fallback limit holds for the document, not per buffer : two fallbacks that each fit
  /// alone but not together are refused, so a file cannot reach gigabytes by declaring many.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_fallback_buffers_too_large_together()
  {
    let fallback = r#"{ "byteLength" : 600000000, "extensions" : { "EXT_meshopt_compression" : { "fallback" : true } } }"#;
    let buffers = format!
    (
      r#"{{ "byteLength" : 36, "uri" : "{}" }}, {fallback}, {fallback}"#,
      data_uri( "application/octet-stream", &triangle_bytes() )
    );
    let json = triangle_gltf( &[ "EXT_meshopt_compression" ], &buffers, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert_eq!( load_refusal( &uri ).await, "glTF meshopt fallback buffers are too large" );
  }

  /// `KHR_meshopt_compression` must not be used on a buffer that also uses
  /// `EXT_meshopt_compression`, so a fallback buffer carrying both objects is refused.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_a_buffer_carrying_both_extensions()
  {
    let compressed = meshopt_attributes_encode( &triangle_bytes(), 12 );
    let json = compressed_triangle_gltf( &compressed ).replace
    (
      r#"{ "EXT_meshopt_compression" : { "fallback" : true } }"#,
      r#"{ "EXT_meshopt_compression" : { "fallback" : true }, "KHR_meshopt_compression" : { "fallback" : true } }"#
    );
    assert!( json.contains( r#""KHR_meshopt_compression" : { "fallback" : true }"# ), "the fixture carries both objects" );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert_eq!( load_refusal( &uri ).await, "glTF meshopt buffer is invalid" );
  }

  /// A fallback buffer holds only what compressed views decode into it. An uncompressed view
  /// reading it would read zeros, so the meshopt specs allow references to it only from
  /// compressed views, and `load` refuses any other.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_an_uncompressed_view_on_a_fallback_buffer()
  {
    let buffers = format!
    (
      r#"{{ "byteLength" : 36, "uri" : "{}" }},
      {{ "byteLength" : 36, "extensions" : {{ "EXT_meshopt_compression" : {{ "fallback" : true }} }} }}"#,
      data_uri( "application/octet-stream", &triangle_bytes() )
    );
    let json = triangle_gltf( &[ "EXT_meshopt_compression" ], &buffers, r#"{ "buffer" : 1, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert_eq!( load_refusal( &uri ).await, "glTF meshopt fallback buffer is read by an uncompressed bufferView" );
  }

  /// Nor may a compressed view's extension object name a fallback buffer as the place its
  /// compressed bytes live : there are none there. The fallback here is long enough to hold
  /// both ranges apart, so only this rule can refuse the file.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_compressed_data_in_a_fallback_buffer()
  {
    let buffers = r#"{ "byteLength" : 72, "extensions" : { "EXT_meshopt_compression" : { "fallback" : true } } }"#;
    let view = r#"{ "buffer" : 0, "byteLength" : 36, "byteStride" : 12, "target" : 34962,
      "extensions" : { "EXT_meshopt_compression" :
        { "buffer" : 0, "byteOffset" : 36, "byteLength" : 36, "byteStride" : 12, "count" : 3, "mode" : "ATTRIBUTES" } } }"#;
    let json = triangle_gltf( &[ "EXT_meshopt_compression" ], buffers, view );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert_eq!( load_refusal( &uri ).await, "glTF meshopt bufferView is invalid" );
  }

  /// The fallback limit is checked from the JSON before any buffer is fetched : buffer 0 here
  /// names a file that does not exist, so a loader that fetched first would fail on it instead.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_oversized_fallbacks_before_fetching_any_buffer()
  {
    let buffers = r#"{ "byteLength" : 36, "uri" : "no_such_buffer_is_ever_fetched.bin" },
      { "byteLength" : 3000000000, "extensions" : { "EXT_meshopt_compression" : { "fallback" : true } } }"#;
    let json = triangle_gltf( &[ "EXT_meshopt_compression" ], buffers, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert_eq!( load_refusal( &uri ).await, "glTF meshopt fallback buffers are too large" );
  }
}

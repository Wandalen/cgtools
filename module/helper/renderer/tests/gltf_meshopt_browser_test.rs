//! Browser tests for loading meshopt-compressed glTF.
//!
//! - The decode itself : `meshopt::views_decode` runs meshoptimizer's vendored JS decoder,
//!   which needs a browser, on bytes encoded by a minimal vertex-codec encoder below.
//! - The meshopt rules of `gltf::load` : a fallback buffer is allocated rather than fetched,
//!   oversized fallbacks, alone or together, are refused before anything is fetched or
//!   allocated, fallback buffers are referenced only as the meshopt specs allow, and no
//!   view decodes over another view's data.
//!
//! The buffer rules every asset follows, meshopt or not, are in `gltf_buffers_browser_test.rs`.

mod gltf_browser_helpers;

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use renderer::webgl::loaders::{ gltf, meshopt };
  use renderer::webgl::Object3D;
  use crate::gltf_browser_helpers::*;

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

    match meshopt::views_decode( &document, &buffers ).await
    {
      Err( gl::WebglError::Other( reason ) ) => assert_eq!( reason, "glTF meshopt decode failed", "an unknown codec header must not decode" ),
      other => panic!( "the decoder must reject the data, got {other:?}" ),
    }
  }

  /// A document whose bufferView 0 decodes `count` elements of 4 bytes from `compressed`
  /// ( buffer 0 ) into fallback buffer 1, with `filter` spliced into the extension object.
  fn filtered_document( compressed : &[ u8 ], count : usize, filter : &str ) -> ::gltf::Gltf
  {
    filtered_document_with( "EXT_meshopt_compression", compressed, count, 4, filter )
  }

  /// [`filtered_document`] under `extension`, for elements of `stride` bytes.
  fn filtered_document_with( extension : &str, compressed : &[ u8 ], count : usize, stride : usize, filter : &str ) -> ::gltf::Gltf
  {
    let len = compressed.len();
    let decoded = count * stride;
    let json = format!
    (
      r#"{{ "asset" : {{ "version" : "2.0" }},
        "extensionsUsed" : [ "{extension}" ], "extensionsRequired" : [ "{extension}" ],
        "buffers" : [ {{ "byteLength" : {len}, "uri" : "compressed.bin" }},
          {{ "byteLength" : {decoded}, "extensions" : {{ "{extension}" : {{ "fallback" : true }} }} }} ],
        "bufferViews" : [ {{ "buffer" : 1, "byteLength" : {decoded}, "byteStride" : {stride},
          "extensions" : {{ "{extension}" :
            {{ "buffer" : 0, "byteLength" : {len}, "byteStride" : {stride}, "count" : {count}, "mode" : "ATTRIBUTES", "filter" : "{filter}" }} }} }} ] }}"#
    );
    ::gltf::Gltf::from_slice_without_validation( json.as_bytes() ).unwrap()
  }

  /// Encodes `raw` ( elements of `stride` bytes ) as a view filtered by `filter` under
  /// `extension`, decodes it with `views_decode`, and returns the decoded bytes.
  async fn filtered_decode( extension : &str, raw : &[ u8 ], stride : usize, filter : &str ) -> Vec< u8 >
  {
    let compressed = meshopt_attributes_encode( raw, stride );
    let document = filtered_document_with( extension, &compressed, raw.len() / stride, stride, filter );
    let buffers = [ gl::js_sys::Uint8Array::from( compressed.as_slice() ), gl::js_sys::Uint8Array::new_with_length( raw.len() as u32 ) ];
    meshopt::views_decode( &document, &buffers ).await.expect( "the filtered data decodes" );
    buffers[ 1 ].to_vec()
  }

  /// `OCTAHEDRAL` stores a unit vector as signed `x`, `y` on the octahedron and `z` as the
  /// encoded 1.0, and rebuilds `z` and normalises. The four inputs decode to +Z, +X, -Y and,
  /// from the folded corner, -Z, each at the 8-bit unit length 127; `w` passes through.
  /// Without the filter the raw bytes would come out unchanged.
  #[ wasm_bindgen_test( async ) ]
  async fn views_decode_applies_the_octahedral_filter()
  {
    let raw : Vec< u8 > = [ [ 0, 0, 127, 9 ], [ 127, 0, 127, 9 ], [ 0, -127, 127, 9 ], [ 127, 127, 127, 9 ] ]
    .iter().flatten().map( | &v : &i8 | v.to_le_bytes()[ 0 ] ).collect();
    let decoded = filtered_decode( "EXT_meshopt_compression", &raw, 4, "OCTAHEDRAL" ).await;
    let vectors = decoded.as_chunks::< 4 >().0.iter().map( | v | v.map( | b | i8::from_le_bytes( [ b ] ) ) ).collect::< Vec< _ > >();
    assert_eq!( vectors, [ [ 0, 0, 127, 9 ], [ 127, 0, 0, 9 ], [ 0, -127, 0, 9 ], [ 0, 0, -127, 9 ] ] );
  }

  /// `QUATERNION` stores three 16-bit components and, in the fourth, a scale and which
  /// component was dropped ( its low 2 bits ); the decoder rebuilds the dropped one and puts
  /// it back in place. Both inputs are the identity rotation with the dropped component at
  /// index 3 and at index 0, so each decodes to the 1.0 ( 32767 ) at that index.
  #[ wasm_bindgen_test( async ) ]
  async fn views_decode_applies_the_quaternion_filter()
  {
    let raw : Vec< u8 > = [ [ 0_i16, 0, 0, 32767 ], [ 0, 0, 0, 32764 ] ].iter().flatten().flat_map( | v | v.to_le_bytes() ).collect();
    let decoded = filtered_decode( "EXT_meshopt_compression", &raw, 8, "QUATERNION" ).await;
    let quaternions = decoded.as_chunks::< 8 >().0.iter()
    .map( | q | [ 0, 2, 4, 6 ].map( | i | i16::from_le_bytes( [ q[ i ], q[ i + 1 ] ] ) ) )
    .collect::< Vec< _ > >();
    assert_eq!( quaternions, [ [ 0, 0, 0, 32767 ], [ 32767, 0, 0, 0 ] ] );
  }

  /// `COLOR` ( `KHR_meshopt_compression` only ) stores a colour as YCoCg : `R = Y + Co - Cg`,
  /// `G = Y + Cg`, `B = Y - Co - Cg`. A grey stays grey, and `Co` / `Cg` of 10 move the
  /// channels by 10 the way the spec's equations say; full alpha stays full.
  #[ wasm_bindgen_test( async ) ]
  async fn views_decode_applies_the_color_filter()
  {
    let raw : Vec< u8 > = [ [ 100_u8, 0, 0, 255 ], [ 100, 10, 0, 255 ], [ 100, 0, 10, 255 ] ].concat();
    let decoded = filtered_decode( "KHR_meshopt_compression", &raw, 4, "COLOR" ).await;
    assert_eq!
    (
      decoded.as_chunks::< 4 >().0,
      [ [ 100, 100, 100, 255 ], [ 110, 100, 90, 255 ], [ 90, 110, 90, 255 ] ]
    );
  }

  /// The decoder applies a view's filter after decoding. `EXPONENTIAL` stores each float as
  /// a signed 8-bit exponent over a signed 24-bit mantissa : 3 * 2^0, 1 * 2^-1 and -5 * 2^2
  /// must come out as the floats 3, 0.5 and -20. Without the filter the raw integers would.
  #[ wasm_bindgen_test( async ) ]
  async fn views_decode_applies_the_exponential_filter()
  {
    let words = [ 0x0000_0003_u32, 0xff00_0001, 0x02ff_fffb ];
    let raw : Vec< u8 > = words.iter().flat_map( | w | w.to_le_bytes() ).collect();
    let compressed = meshopt_attributes_encode( &raw, 4 );
    let document = filtered_document( &compressed, 3, "EXPONENTIAL" );
    let buffers = [ gl::js_sys::Uint8Array::from( compressed.as_slice() ), gl::js_sys::Uint8Array::new_with_length( 12 ) ];

    meshopt::views_decode( &document, &buffers ).await.expect( "the filtered data decodes" );

    let floats = buffers[ 1 ].to_vec().as_chunks::< 4 >().0.iter().map( | b | f32::from_le_bytes( *b ) ).collect::< Vec< _ > >();
    assert_eq!( floats, [ 3.0, 0.5, -20.0 ] );
  }

  /// A filter name the extension does not define is refused before anything is decoded,
  /// rather than passed to the decoder or read as no filter.
  #[ wasm_bindgen_test( async ) ]
  async fn views_decode_refuses_an_unknown_filter()
  {
    let raw = [ 0_u8; 12 ];
    let compressed = meshopt_attributes_encode( &raw, 4 );
    let document = filtered_document( &compressed, 3, "WAVELET" );
    let buffers = [ gl::js_sys::Uint8Array::from( compressed.as_slice() ), gl::js_sys::Uint8Array::new_with_length( 12 ) ];

    match meshopt::views_decode( &document, &buffers ).await
    {
      Err( gl::WebglError::Other( reason ) ) => assert_eq!( reason, "glTF meshopt bufferView is invalid" ),
      other => panic!( "an unknown filter must be refused, got {other:?}" ),
    }
    assert_eq!( buffers[ 1 ].to_vec(), [ 0_u8; 12 ], "nothing is decoded" );
  }

  /// A fallback buffer may name a uri, for loaders without meshopt support. `load` never
  /// fetches it : the file named here does not exist, so fetching it would fail the load.
  #[ wasm_bindgen_test( async ) ]
  async fn load_never_fetches_a_fallback_buffers_uri()
  {
    let gl = gl_init();
    let compressed = meshopt_attributes_encode( &triangle_bytes(), 12 );
    let json = compressed_triangle_gltf( &compressed ).replace
    (
      r#"{ "byteLength" : 36, "extensions""#,
      r#"{ "byteLength" : 36, "uri" : "no_such_fallback_file_is_ever_fetched.bin", "extensions""#
    );
    assert!( json.contains( "no_such_fallback_file_is_ever_fetched.bin" ), "the fallback names a uri" );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    let loaded = gltf::load( &document(), &uri, &gl ).await.expect( "the fallback's uri is not fetched" );

    assert_eq!( view_bytes_read( &gl, &loaded, 0, 36 ), triangle_bytes() );
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

  /// Both extensions on one view : normalized `i16` positions ( `KHR_mesh_quantization` ),
  /// padded to 8 bytes and compressed with `EXT_meshopt_compression`. `load` must upload the
  /// decoded integers unchanged, for the GPU to normalize, and give the mesh bounds divided by
  /// 32767, the quantized accessor's raw `max` of `( 32767, 16384, 0 )` read as -1..1.
  #[ wasm_bindgen_test( async ) ]
  async fn load_decodes_and_dequantizes_a_compressed_quantized_mesh()
  {
    let mut raw = Vec::new();
    for v in [ [ 0_i16, 0, 0 ], [ 32767, 0, 0 ], [ 0, 16384, 0 ] ]
    {
      for c in v { raw.extend( c.to_le_bytes() ); }
      raw.extend( [ 0, 0 ] );
    }
    let compressed = meshopt_attributes_encode( &raw, 8 );
    let len = compressed.len();
    let json = format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        "extensionsUsed" : [ "EXT_meshopt_compression", "KHR_mesh_quantization" ],
        "extensionsRequired" : [ "EXT_meshopt_compression", "KHR_mesh_quantization" ],
        "buffers" : [ {{ "byteLength" : {len}, "uri" : "{}" }},
          {{ "byteLength" : 24, "extensions" : {{ "EXT_meshopt_compression" : {{ "fallback" : true }} }} }} ],
        "bufferViews" : [ {{ "buffer" : 1, "byteLength" : 24, "byteStride" : 8, "target" : 34962,
          "extensions" : {{ "EXT_meshopt_compression" :
            {{ "buffer" : 0, "byteLength" : {len}, "byteStride" : 8, "count" : 3, "mode" : "ATTRIBUTES" }} }} }} ],
        "accessors" : [ {{ "bufferView" : 0, "componentType" : 5122, "normalized" : true, "count" : 3, "type" : "VEC3",
          "min" : [ 0, 0, 0 ], "max" : [ 32767, 16384, 0 ] }} ],
        "materials" : [ {{}} ],
        "meshes" : [ {{ "primitives" : [ {{ "attributes" : {{ "POSITION" : 0 }}, "material" : 0 }} ] }} ],
        "nodes" : [ {{ "mesh" : 0 }} ],
        "scenes" : [ {{ "nodes" : [ 0 ] }} ],
        "scene" : 0
      }}"#,
      data_uri( "application/octet-stream", &compressed )
    );
    let gl = gl_init();
    let loaded = gltf::load( &document(), &data_uri( "model/gltf+json", json.as_bytes() ), &gl ).await
    .expect( "the compressed quantized triangle loads" );

    assert_eq!( view_bytes_read( &gl, &loaded, 0, 24 ), raw, "the decoded integers, uploaded as they are" );
    let bounds = loaded.meshes[ 0 ].borrow().bounding_box();
    let near = | a : f32, b : f32 | ( a - b ).abs() < 1e-6;
    let ( min, max ) = ( bounds.min.0, bounds.max.0 );
    assert!( min.iter().all( | &c | near( c, 0.0 ) ), "min {min:?}" );
    assert!( near( max[ 0 ], 1.0 ) && near( max[ 1 ], 16384.0 / 32767.0 ) && near( max[ 2 ], 0.0 ), "max {max:?}" );
  }

  /// A compressed morph target : its displacements are read on the CPU, from the copy `load`
  /// takes of the buffers, so that copy must be taken after the decode. The target's view
  /// decodes into a fallback buffer; read before the decode it would hold zeros.
  #[ wasm_bindgen_test( async ) ]
  async fn load_reads_a_compressed_morph_target()
  {
    let displacements = [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0_f32 ].iter().flat_map( | f | f.to_le_bytes() ).collect::< Vec< u8 > >();
    let compressed = meshopt_attributes_encode( &displacements, 12 );
    let len = compressed.len();
    let json = format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        "extensionsUsed" : [ "EXT_meshopt_compression" ], "extensionsRequired" : [ "EXT_meshopt_compression" ],
        "buffers" : [ {{ "byteLength" : {}, "uri" : "{}" }},
          {{ "byteLength" : 36, "extensions" : {{ "EXT_meshopt_compression" : {{ "fallback" : true }} }} }} ],
        "bufferViews" :
        [
          {{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }},
          {{ "buffer" : 1, "byteLength" : 36, "byteStride" : 12,
            "extensions" : {{ "EXT_meshopt_compression" :
              {{ "buffer" : 0, "byteOffset" : 36, "byteLength" : {len}, "byteStride" : 12, "count" : 3, "mode" : "ATTRIBUTES" }} }} }}
        ],
        "accessors" :
        [
          {{ "bufferView" : 0, "componentType" : 5126, "count" : 3, "type" : "VEC3", "min" : [ 0, 0, 0 ], "max" : [ 1, 1, 0 ] }},
          {{ "bufferView" : 1, "componentType" : 5126, "count" : 3, "type" : "VEC3", "min" : [ 0, 0, 0 ], "max" : [ 1, 1, 1 ] }}
        ],
        "materials" : [ {{}} ],
        "meshes" : [ {{ "primitives" : [ {{ "attributes" : {{ "POSITION" : 0 }}, "targets" : [ {{ "POSITION" : 1 }} ], "material" : 0 }} ], "weights" : [ 0.5 ] }} ],
        "nodes" : [ {{ "mesh" : 0 }} ],
        "scenes" : [ {{ "nodes" : [ 0 ] }} ],
        "scene" : 0
      }}"#,
      36 + len,
      data_uri( "application/octet-stream", &[ triangle_bytes(), compressed ].concat() )
    );
    let loaded = gltf::load( &document(), &data_uri( "model/gltf+json", json.as_bytes() ), &gl_init() ).await
    .expect( "the triangle with a compressed target loads" );

    let skeleton = loaded.nodes.iter().find_map( | node | match &node.borrow().object
    {
      Object3D::Mesh( mesh ) => mesh.borrow().skeleton.clone(),
      _ => None,
    } )
    .expect( "the mesh has a skeleton for its morph target" );
    let mut skeleton = skeleton.borrow_mut();
    let data = skeleton.displacements_as_mut().as_mut().expect( "the skeleton holds the morph target" );
    // Per vertex : [ position.xyz, 1 ], [ normal.xyz, 1 ], [ tangent.xyz, 1 ].
    let expected = vec!
    [
      1.0, 0.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0,
      0.0, 1.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0,
      0.0, 0.0, 1.0, 1.0,  0.0, 0.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0_f32,
    ];
    assert_eq!( data.displacements_data_pack(), expected, "the decoded displacements" );
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

  /// A compressed view may decode into a buffer that holds data of its own, but not over an
  /// uncompressed view of it : that view's accessors would read the decoded bytes. Here view 0
  /// decodes into bytes 0..36 of buffer 1, and view 1 reads 12 bytes of buffer 1 at `offset`.
  /// Placed at 36 it loads, so only the overlap can refuse it at 12.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_a_decode_over_an_uncompressed_view()
  {
    let compressed = meshopt_attributes_encode( &triangle_bytes(), 12 );
    let len = compressed.len();
    let json = | offset : usize | triangle_gltf
    (
      &[ "EXT_meshopt_compression" ],
      &format!
      (
        r#"{{ "byteLength" : {len}, "uri" : "{}" }}, {{ "byteLength" : 48, "uri" : "{}" }}"#,
        data_uri( "application/octet-stream", &compressed ),
        data_uri( "application/octet-stream", &[ triangle_bytes(), vec![ 0; 12 ] ].concat() )
      ),
      &format!
      (
        r#"{{ "buffer" : 1, "byteLength" : 36, "byteStride" : 12, "target" : 34962,
          "extensions" : {{ "EXT_meshopt_compression" :
            {{ "buffer" : 0, "byteLength" : {len}, "byteStride" : 12, "count" : 3, "mode" : "ATTRIBUTES" }} }} }},
        {{ "buffer" : 1, "byteOffset" : {offset}, "byteLength" : 12 }}"#
      )
    );

    let gl = gl_init();
    let apart = data_uri( "model/gltf+json", json( 36 ).as_bytes() );
    let loaded = gltf::load( &document(), &apart, &gl ).await.expect( "views side by side load" );
    assert_eq!( view_bytes_read( &gl, &loaded, 0, 36 ), triangle_bytes() );

    let overlapping = data_uri( "model/gltf+json", json( 12 ).as_bytes() );
    assert_eq!( load_refusal( &overlapping ).await, "glTF meshopt bufferViews overlap" );
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

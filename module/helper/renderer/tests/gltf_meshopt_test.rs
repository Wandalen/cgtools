//! Off-GPU checks for loading meshopt-compressed, quantized glTF
//! ( `EXT_meshopt_compression` / `KHR_mesh_quantization` ) : parsing and validating
//! each buffer view's extension object, the range checks run before decoding,
//! fallback-buffer detection, validation that lets these extensions through, and the
//! dequantized position bounds and accessor reads. The decode itself runs meshoptimizer's JS decoder, so
//! its tests, and those of `load`'s buffer-index rules, are in `gltf_meshopt_browser_test.rs`.

use renderer::webgl::loaders::gltf_quantization::position_bounds_dequantize;
use renderer::webgl::loaders::gltf_extensions::{ document_validate, required_extensions_check };
use renderer::webgl::loaders::meshopt::{ buffer_is_fallback, MeshoptFilter, MeshoptMode, MeshoptView };
use serde_json::json;

/// The layout glTF-Transform writes : compressed bytes in buffer 0, and a data-less
/// fallback buffer 1 that the views decode into. Indices, octahedral normals and
/// quantized positions, like `rara_585_solune_8.5.glb`.
const MESHOPT_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "EXT_meshopt_compression", "KHR_mesh_quantization" ],
  "extensionsRequired": [ "EXT_meshopt_compression", "KHR_mesh_quantization" ],
  "accessors":
  [
    { "bufferView": 0, "componentType": 5125, "count": 6, "type": "SCALAR" },
    { "bufferView": 1, "componentType": 5120, "normalized": true, "count": 4, "type": "VEC3" },
    { "bufferView": 2, "componentType": 5122, "normalized": true, "count": 4, "type": "VEC3",
      "min": [ -32767, -16384, 0 ], "max": [ 32767, 16384, 32767 ] }
  ],
  "bufferViews":
  [
    { "buffer": 1, "byteOffset": 0, "byteLength": 24, "target": 34963,
      "extensions": { "EXT_meshopt_compression":
        { "buffer": 0, "byteOffset": 0, "byteLength": 10, "byteStride": 4, "count": 6, "mode": "TRIANGLES" } } },
    { "buffer": 1, "byteOffset": 24, "byteLength": 16, "byteStride": 4, "target": 34962,
      "extensions": { "EXT_meshopt_compression":
        { "buffer": 0, "byteOffset": 12, "byteLength": 12, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES", "filter": "OCTAHEDRAL" } } },
    { "buffer": 1, "byteOffset": 40, "byteLength": 32, "byteStride": 8, "target": 34962,
      "extensions": { "EXT_meshopt_compression":
        { "buffer": 0, "byteOffset": 24, "byteLength": 20, "byteStride": 8, "count": 4, "mode": "ATTRIBUTES" } } }
  ],
  "buffers":
  [
    { "byteLength": 44, "uri": "placeholder.bin" },
    { "byteLength": 72, "extensions": { "EXT_meshopt_compression": { "fallback": true } } }
  ]
}
"#;

/// The extension name the fixtures parse under, unless a test says otherwise.
const EXT : &str = "EXT_meshopt_compression";

fn fixture() -> gltf::Gltf
{
  gltf::Gltf::from_slice_without_validation( MESHOPT_FIXTURE.as_bytes() ).expect( "fixture is well-formed JSON" )
}

#[ test ]
fn reads_each_views_extension()
{
  let gltf = fixture();
  let specs = gltf.views()
  .map( | v | MeshoptView::from_view( &v ).expect( "every fixture view is compressed" ).unwrap() )
  .collect::< Vec< _ > >();

  assert_eq!
  (
    specs[ 0 ],
    MeshoptView
    {
      buffer : 0, byte_offset : 0, byte_length : 10, byte_stride : 4, count : 6,
      mode : MeshoptMode::Triangles, filter : MeshoptFilter::None
    }
  );
  assert_eq!( specs[ 1 ].filter, MeshoptFilter::Octahedral );
  assert_eq!( specs[ 2 ].byte_stride, 8 );
  assert_eq!( specs[ 2 ].decoded_length(), Some( 32 ) );
}

#[ test ]
fn a_plain_view_is_not_compressed()
{
  let gltf = gltf::Gltf::from_slice_without_validation
  (
    br#"{ "asset": { "version": "2.0" }, "bufferViews": [ { "buffer": 0, "byteLength": 4 } ], "buffers": [ { "byteLength": 4, "uri": "a.bin" } ] }"#
  ).unwrap();
  assert!( MeshoptView::from_view( &gltf.views().next().unwrap() ).is_none() );
  assert!( !buffer_is_fallback( &gltf.buffers().next().unwrap() ) );
}

#[ test ]
fn the_khr_name_is_read_too()
{
  let gltf = gltf::Gltf::from_slice_without_validation
  (
    br#"{ "asset": { "version": "2.0" },
      "bufferViews": [ { "buffer": 0, "byteLength": 16, "extensions": { "KHR_meshopt_compression":
        { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES", "filter": "COLOR" } } } ],
      "buffers": [ { "byteLength": 16, "uri": "a.bin", "extensions": { "KHR_meshopt_compression": { "fallback": true } } } ] }"#
  ).unwrap();
  let spec = MeshoptView::from_view( &gltf.views().next().unwrap() ).unwrap().unwrap();
  assert_eq!( spec.filter, MeshoptFilter::Color );
  assert_eq!( spec.byte_offset, 0, "byteOffset defaults to 0" );
  assert!( buffer_is_fallback( &gltf.buffers().next().unwrap() ), "a fallback with a uri is still not fetched" );
}

#[ test ]
fn finds_the_fallback_buffer()
{
  let gltf = fixture();
  let fallback = gltf.buffers().map( | b | buffer_is_fallback( &b ) ).collect::< Vec< _ > >();
  assert_eq!( fallback, [ false, true ] );
}

#[ test ]
fn rejects_malformed_extension_objects()
{
  let base = json!( { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 3, "mode": "ATTRIBUTES" } );
  assert!( MeshoptView::parse( EXT, &base ).is_ok() );

  let with = | key : &str, value : serde_json::Value |
  {
    let mut v = base.clone();
    v[ key ] = value;
    MeshoptView::parse( EXT, &v )
  };
  let without = | key : &str |
  {
    let mut v = base.clone();
    v.as_object_mut().unwrap().remove( key );
    MeshoptView::parse( EXT, &v )
  };

  assert!( without( "buffer" ).is_err() );
  assert!( without( "byteLength" ).is_err() );
  assert!( without( "byteStride" ).is_err() );
  assert!( without( "count" ).is_err() );
  assert!( without( "mode" ).is_err() );
  assert!( with( "count", json!( -1 ) ).is_err(), "negative count" );
  assert!( with( "count", json!( 1.5 ) ).is_err(), "fractional count" );
  assert!( with( "mode", json!( "STRIPS" ) ).is_err() );
  assert!( with( "filter", json!( "WAVELET" ) ).is_err() );
  assert!( with( "byteStride", json!( 6 ) ).is_err(), "ATTRIBUTES stride must be a multiple of 4" );
  assert!( with( "byteStride", json!( 260 ) ).is_err(), "ATTRIBUTES stride is at most 256" );
  assert!( with( "filter", json!( "QUATERNION" ) ).is_err(), "QUATERNION needs stride 8" );
  assert!( with( "filter", json!( "OCTAHEDRAL" ) ).is_ok() );

  let indices = json!( { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 6, "mode": "TRIANGLES" } );
  assert!( MeshoptView::parse( EXT, &indices ).is_ok() );
  let mut odd = indices.clone();
  odd[ "count" ] = json!( 7 );
  assert!( MeshoptView::parse( EXT, &odd ).is_err(), "TRIANGLES count must be a multiple of 3" );
  let mut filtered = indices.clone();
  filtered[ "filter" ] = json!( "OCTAHEDRAL" );
  assert!( MeshoptView::parse( EXT, &filtered ).is_err(), "index data takes no filter" );
  let mut wide = indices;
  wide[ "byteStride" ] = json!( 8 );
  assert!( MeshoptView::parse( EXT, &wide ).is_err(), "index stride is 2 or 4" );
}

#[ test ]
fn the_color_filter_is_khr_only()
{
  let color = json!( { "buffer": 0, "byteLength": 8, "byteStride": 4, "count": 4, "mode": "ATTRIBUTES", "filter": "COLOR" } );
  assert_eq!( MeshoptView::parse( "KHR_meshopt_compression", &color ).unwrap().filter, MeshoptFilter::Color );
  assert!( MeshoptView::parse( EXT, &color ).is_err(), "EXT_meshopt_compression defines no COLOR filter" );
}

#[ test ]
fn checks_ranges_against_the_loaded_buffers()
{
  let spec = MeshoptView
  {
    buffer : 0, byte_offset : 12, byte_length : 12, byte_stride : 4, count : 4,
    mode : MeshoptMode::Attributes, filter : MeshoptFilter::None
  };
  let lengths = [ 44, 72 ];

  assert!( spec.ranges_check( &lengths, 1, 24, 16 ).is_ok() );
  assert!( spec.ranges_check( &[ 20, 72 ], 1, 24, 16 ).is_err(), "compressed bytes run past buffer 0" );
  assert!( spec.ranges_check( &lengths, 1, 64, 16 ).is_err(), "view runs past buffer 1" );
  assert!( spec.ranges_check( &lengths, 1, 24, 12 ).is_err(), "decoded 16 bytes do not fit a 12-byte view" );
  assert!( spec.ranges_check( &lengths, 2, 0, 16 ).is_err(), "view names a missing buffer" );
  assert!( MeshoptView { buffer : 5, ..spec.clone() }.ranges_check( &lengths, 1, 24, 16 ).is_err(), "source names a missing buffer" );
  assert!( MeshoptView { byte_offset : usize::MAX, ..spec.clone() }.ranges_check( &lengths, 1, 24, 16 ).is_err(), "overflow" );
  assert!( MeshoptView { count : usize::MAX, ..spec }.ranges_check( &lengths, 1, 24, 16 ).is_err(), "overflow" );
}

/// The decoder writes the view while reading the compressed bytes, so the two may share a
/// buffer only if their ranges are disjoint; touching ends are fine.
#[ test ]
fn rejects_a_decode_target_overlapping_its_source()
{
  // 12 compressed bytes at 0..12 of buffer 0, decoding 16 bytes.
  let spec = MeshoptView
  {
    buffer : 0,
    byte_offset : 0,
    byte_length : 12,
    byte_stride : 4,
    count : 4,
    mode : MeshoptMode::Attributes,
    filter : MeshoptFilter::None,
  };
  let lengths = [ 64 ];

  assert!( spec.ranges_check( &lengths, 0, 12, 16 ).is_ok(), "the view starts where the compressed bytes end" );
  assert!( spec.ranges_check( &lengths, 0, 32, 16 ).is_ok(), "disjoint ranges in one buffer" );
  assert!( spec.ranges_check( &lengths, 0, 8, 16 ).is_err(), "the view starts inside the compressed bytes" );
  assert!( spec.ranges_check( &lengths, 0, 0, 16 ).is_err(), "the view starts where the compressed bytes start" );
  let after = MeshoptView { byte_offset : 20, ..spec.clone() };
  assert!( after.ranges_check( &lengths, 0, 8, 16 ).is_err(), "the decoded bytes run into compressed bytes after them" );
  assert!( after.ranges_check( &lengths, 0, 4, 16 ).is_ok(), "the decoded bytes end where the compressed bytes start" );
}

#[ test ]
fn meshopt_and_quantization_pass_validation()
{
  let gltf = fixture();
  assert!( gltf::Gltf::from_slice( MESHOPT_FIXTURE.as_bytes() ).is_err(), "the gltf crate alone refuses these extensions" );
  assert!( document_validate( &gltf ).is_ok() );
  assert!( required_extensions_check( &gltf ).is_ok() );
}

#[ test ]
fn other_validation_errors_still_refuse_the_file()
{
  let broken = MESHOPT_FIXTURE.replace( r#"{ "bufferView": 0, "componentType""#, r#"{ "bufferView": 9, "componentType""# );
  let gltf = gltf::Gltf::from_slice_without_validation( broken.as_bytes() ).unwrap();
  assert!( document_validate( &gltf ).is_err(), "an accessor pointing at a missing bufferView" );
}

#[ test ]
fn draco_is_still_refused()
{
  let gltf = gltf::Gltf::from_slice_without_validation
  (
    br#"{ "asset": { "version": "2.0" }, "extensionsRequired": [ "KHR_draco_mesh_compression" ] }"#
  ).unwrap();
  assert!( required_extensions_check( &gltf ).is_err() );
  assert!( document_validate( &gltf ).is_err(), "document_validate applies the extensionsRequired rule as well" );
}

#[ test ]
fn normalized_position_bounds_are_dequantized()
{
  use gltf::accessor::DataType;

  let gltf = fixture();
  let acc = gltf.accessors().nth( 2 ).unwrap();
  let raw = gltf::mesh::BoundingBox { min : [ -32767.0, -16384.0, 0.0 ], max : [ 32767.0, 16384.0, 32767.0 ] };

  let b = position_bounds_dequantize( raw, acc.data_type(), acc.normalized() );
  assert_eq!( b.min, [ -1.0, -16384.0 / 32767.0, 0.0 ] );
  assert_eq!( b.max, [ 1.0, 16384.0 / 32767.0, 1.0 ] );

  let one = | v : f32, t, n | position_bounds_dequantize( gltf::mesh::BoundingBox { min : [ v; 3 ], max : [ v; 3 ] }, t, n ).min[ 0 ];
  assert_eq!( one( -128.0, DataType::I8, true ), -1.0, "the signed minimum clamps to -1" );
  assert_eq!( one( 127.0, DataType::I8, true ), 1.0 );
  assert_eq!( one( 255.0, DataType::U8, true ), 1.0 );
  assert_eq!( one( 65535.0, DataType::U16, true ), 1.0 );
  assert_eq!( one( 300.0, DataType::I16, false ), 300.0, "non-normalized integers are already in shader units" );
  assert_eq!( one( 2.5, DataType::F32, false ), 2.5 );
}

/// One `VEC3` accessor per component type `accessor_vec3_read` matches on, plus a
/// `SCALAR` one. The `u8` and `u16` views pad each element with 0x7f to a 4- and an
/// 8-byte stride, so a reader that ignores `byteStride` picks the padding up.
const COMPONENT_TYPES_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "KHR_mesh_quantization" ],
  "accessors":
  [
    { "bufferView": 0, "componentType": 5121, "normalized": true, "count": 2, "type": "VEC3" },
    { "bufferView": 0, "componentType": 5121, "count": 2, "type": "VEC3" },
    { "bufferView": 1, "componentType": 5123, "normalized": true, "count": 2, "type": "VEC3" },
    { "bufferView": 1, "componentType": 5123, "count": 2, "type": "VEC3" },
    { "bufferView": 2, "componentType": 5125, "count": 1, "type": "VEC3" },
    { "bufferView": 3, "componentType": 5126, "count": 1, "type": "VEC3" },
    { "bufferView": 4, "componentType": 5120, "normalized": true, "count": 1, "type": "VEC3" },
    { "bufferView": 5, "componentType": 5122, "normalized": true, "count": 1, "type": "VEC3" },
    { "bufferView": 3, "componentType": 5126, "count": 3, "type": "SCALAR" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 8, "byteStride": 4 },
    { "buffer": 0, "byteOffset": 8, "byteLength": 16, "byteStride": 8 },
    { "buffer": 0, "byteOffset": 24, "byteLength": 12 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 12 },
    { "buffer": 0, "byteOffset": 48, "byteLength": 3 },
    { "buffer": 0, "byteOffset": 52, "byteLength": 6 }
  ],
  "buffers": [ { "byteLength": 58, "uri": "placeholder.bin" } ]
}
"#;

/// `COMPONENT_TYPES_FIXTURE`'s bytes, laid out view by view.
fn component_types_buffer() -> Vec< u8 >
{
  let mut bytes = Vec::new();
  for v in [ [ 255u8, 0, 51 ], [ 0, 255, 102 ] ]
  {
    bytes.extend( v );
    bytes.push( 0x7f );
  }
  for v in [ [ 65535u16, 0, 13107 ], [ 0, 65535, 26214 ] ]
  {
    for c in v { bytes.extend( c.to_le_bytes() ); }
    bytes.extend( [ 0x7f, 0x7f ] );
  }
  for c in [ 1u32, 2, 70000 ] { bytes.extend( c.to_le_bytes() ); }
  for c in [ 0.5f32, -1.5, 2.0 ] { bytes.extend( c.to_le_bytes() ); }
  bytes.extend( [ -128i8, 127, 0 ].map( | c | c.to_le_bytes()[ 0 ] ) );
  bytes.push( 0 );
  for c in [ -32768i16, 32767, 16384 ] { bytes.extend( c.to_le_bytes() ); }
  assert_eq!( bytes.len(), 58 );
  bytes
}

/// Every integer arm of `accessor_vec3_read` reads its own type through a real accessor,
/// with the view's stride, and dequantizes only a normalized one. Morph targets may only
/// be `i8` / `i16` ( `gltf_skeleton_displacements_test.rs` reads those through
/// `skeleton_displacements_data_load` ), so `u8` / `u16` / `u32` are reached here directly.
#[ test ]
fn accessor_vec3_read_reads_every_component_type()
{
  use renderer::webgl::loaders::gltf_quantization::accessor_vec3_read;

  let gltf = gltf::Gltf::from_slice_without_validation( COMPONENT_TYPES_FIXTURE.as_bytes() ).expect( "fixture is well-formed JSON" );
  let buffers = vec![ component_types_buffer() ];
  let read = | i : usize | accessor_vec3_read( gltf.accessors().nth( i ).expect( "fixture accessor" ), &buffers );
  let near = | actual : Vec< [ f32; 3 ] >, expected : &[ [ f32; 3 ] ], what : &str |
  {
    assert_eq!( actual.len(), expected.len(), "{what}: {actual:?}" );
    for ( a, e ) in actual.iter().zip( expected )
    {
      assert!( a.iter().zip( e ).all( | ( a, e ) | ( a - e ).abs() < 1e-6 ), "{what}: {actual:?} vs {expected:?}" );
    }
  };

  near( read( 0 ).expect( "u8" ), &[ [ 1.0, 0.0, 0.2 ], [ 0.0, 1.0, 0.4 ] ], "normalized u8" );
  near( read( 1 ).expect( "u8" ), &[ [ 255.0, 0.0, 51.0 ], [ 0.0, 255.0, 102.0 ] ], "plain u8" );
  near( read( 2 ).expect( "u16" ), &[ [ 1.0, 0.0, 0.2 ], [ 0.0, 1.0, 0.4 ] ], "normalized u16" );
  near( read( 3 ).expect( "u16" ), &[ [ 65535.0, 0.0, 13107.0 ], [ 0.0, 65535.0, 26214.0 ] ], "plain u16" );
  near( read( 4 ).expect( "u32" ), &[ [ 1.0, 2.0, 70000.0 ] ], "u32" );
  near( read( 5 ).expect( "f32" ), &[ [ 0.5, -1.5, 2.0 ] ], "f32" );
  near( read( 6 ).expect( "i8" ), &[ [ -1.0, 1.0, 0.0 ] ], "normalized i8, the minimum clamped to -1" );
  near( read( 7 ).expect( "i16" ), &[ [ -1.0, 1.0, 16384.0 / 32767.0 ] ], "normalized i16, the minimum clamped to -1" );
  assert!( read( 8 ).is_none(), "a SCALAR accessor is not read as VEC3" );
  assert!( accessor_vec3_read( gltf.accessors().next().unwrap(), &[] ).is_none(), "missing buffer data reads as None" );
}

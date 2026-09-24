//! Off-GPU checks for loading meshopt-compressed, quantized glTF
//! ( `EXT_meshopt_compression` / `KHR_mesh_quantization` ) : parsing and validating
//! each buffer view's extension object, the range checks run before decoding,
//! fallback-buffer detection, validation that lets these extensions through, and the
//! dequantized position bounds. The decode itself runs meshoptimizer's JS decoder and
//! is covered in the browser.

use renderer::webgl::loaders::gltf::{ document_validate, position_bounds_dequantize, required_extensions_check };
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
  assert!( MeshoptView::parse( &base ).is_ok() );

  let with = | key : &str, value : serde_json::Value |
  {
    let mut v = base.clone();
    v[ key ] = value;
    MeshoptView::parse( &v )
  };
  let without = | key : &str |
  {
    let mut v = base.clone();
    v.as_object_mut().unwrap().remove( key );
    MeshoptView::parse( &v )
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
  assert!( MeshoptView::parse( &indices ).is_ok() );
  let mut odd = indices.clone();
  odd[ "count" ] = json!( 7 );
  assert!( MeshoptView::parse( &odd ).is_err(), "TRIANGLES count must be a multiple of 3" );
  let mut filtered = indices.clone();
  filtered[ "filter" ] = json!( "OCTAHEDRAL" );
  assert!( MeshoptView::parse( &filtered ).is_err(), "index data takes no filter" );
  let mut wide = indices;
  wide[ "byteStride" ] = json!( 8 );
  assert!( MeshoptView::parse( &wide ).is_err(), "index stride is 2 or 4" );
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
  assert!( document_validate( &gltf ).is_ok() );
  assert!( required_extensions_check( &gltf ).is_err() );
}

#[ test ]
fn normalized_position_bounds_are_dequantized()
{
  let gltf = fixture();
  let acc = gltf.accessors().nth( 2 ).unwrap();
  let raw = gltf::mesh::BoundingBox { min : [ -32767.0, -16384.0, 0.0 ], max : [ 32767.0, 16384.0, 32767.0 ] };

  let b = position_bounds_dequantize( raw, acc.data_type(), acc.normalized() );
  assert_eq!( b.min, [ -1.0, -16384.0 / 32767.0, 0.0 ] );
  assert_eq!( b.max, [ 1.0, 16384.0 / 32767.0, 1.0 ] );

  use gltf::accessor::DataType;
  let one = | v : f32, t, n | position_bounds_dequantize( gltf::mesh::BoundingBox { min : [ v; 3 ], max : [ v; 3 ] }, t, n ).min[ 0 ];
  assert_eq!( one( -128.0, DataType::I8, true ), -1.0, "the signed minimum clamps to -1" );
  assert_eq!( one( 127.0, DataType::I8, true ), 1.0 );
  assert_eq!( one( 255.0, DataType::U8, true ), 1.0 );
  assert_eq!( one( 65535.0, DataType::U16, true ), 1.0 );
  assert_eq!( one( 300.0, DataType::I16, false ), 300.0, "non-normalized integers are already in shader units" );
  assert_eq!( one( 2.5, DataType::F32, false ), 2.5 );
}

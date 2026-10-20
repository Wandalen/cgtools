//! Off-GPU checks for `KHR_mesh_quantization` support : normalized `POSITION` bounds turned
//! into the shader's units, and `VEC3` accessors of every component type read back as floats.
//! The end-to-end load of a quantized mesh is in `gltf_quantization_browser_test.rs`.

use renderer::webgl::loaders::gltf_quantization::position_bounds_dequantize;
use renderer::webgl::loaders::gltf_quantization::internal::accessor_vec3_read;

#[ test ]
#[ allow( clippy::float_cmp, reason = "each expected value is the same single f32 division the dequantization does ( or an exact 0, ±1 or pass-through literal ), so the results are bit-identical" ) ]
fn normalized_position_bounds_are_dequantized()
{
  use gltf::accessor::DataType;

  // A normalized `i16` POSITION accessor's raw bounds.
  let raw = gltf::mesh::BoundingBox { min : [ -32767.0, -16384.0, 0.0 ], max : [ 32767.0, 16384.0, 32767.0 ] };

  let b = position_bounds_dequantize( raw, DataType::I16, true );
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

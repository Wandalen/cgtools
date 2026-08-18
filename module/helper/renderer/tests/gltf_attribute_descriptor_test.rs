//! Unit-level validation tests for
//! `renderer::webgl::loaders::gltf::attribute_descriptor_make` -- computes a vertex attribute's
//! `gl::BufferDescriptor` ( data type, offset, stride, normalized, dimensionality ) from its glTF
//! accessor. Pure data transform over the accessor's own metadata, no `gl` / `GL` / `WebGl` calls
//! anywhere in its body. Split out of `attribute_info_make` and made `pub` alongside this test per
//! task 441.
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use renderer::webgl::loaders::gltf::attribute_descriptor_make;
use minwebgl as gl;

/// Two accessors : a tightly-packed `FLOAT` `VEC2` at a nonzero accessor-level `byteOffset`
/// ( exercises offset scaling with no explicit `byteStride` ), and a `normalized` `UNSIGNED_BYTE`
/// `VEC4` at zero offset ( exercises the `normalized` flag and a distinct scalar type /
/// dimensionality ).
const ACCESSOR_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "accessors":
  [
    { "bufferView": 0, "byteOffset": 8, "componentType": 5126, "count": 2, "type": "VEC2" },
    { "bufferView": 1, "byteOffset": 0, "componentType": 5121, "normalized": true, "count": 2, "type": "VEC4" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 24 },
    { "buffer": 0, "byteOffset": 24, "byteLength": 8 }
  ],
  "buffers": [ { "byteLength": 32, "uri": "placeholder.bin" } ]
}
"#;

#[ test ]
fn computes_descriptor_for_tightly_packed_float_vec2_with_nonzero_offset()
{
  let gltf = gltf::Gltf::from_slice( ACCESSOR_FIXTURE.as_bytes() ).unwrap();
  let acc = gltf.accessors().next().expect( "fixture declares a first accessor" );

  let descriptor = attribute_descriptor_make( &acc );

  assert_eq!( descriptor.vector.scalar, gl::DataType::F32 );
  assert_eq!( descriptor.vector.natoms, 2, "VEC2 has 2 components" );
  assert_eq!( descriptor.vector.nelements, 1 );
  assert_eq!( descriptor.offset, 2, "byteOffset 8 / 4-byte F32 = 2 elements" );
  assert_eq!( descriptor.stride, 0, "no explicit byteStride falls back to 0" );
  assert!( !descriptor.normalized, "normalized defaults to false when absent from the fixture" );
}

#[ test ]
fn computes_descriptor_for_normalized_u8_vec4()
{
  let gltf = gltf::Gltf::from_slice( ACCESSOR_FIXTURE.as_bytes() ).unwrap();
  let acc = gltf.accessors().nth( 1 ).expect( "fixture declares a second accessor" );

  let descriptor = attribute_descriptor_make( &acc );

  assert_eq!( descriptor.vector.scalar, gl::DataType::U8 );
  assert_eq!( descriptor.vector.natoms, 4, "VEC4 has 4 components" );
  assert_eq!( descriptor.vector.nelements, 1 );
  assert_eq!( descriptor.offset, 0 );
  assert_eq!( descriptor.stride, 0 );
  assert!( descriptor.normalized, "fixture sets normalized: true" );
}

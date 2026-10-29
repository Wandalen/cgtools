//! Unit-level validation tests for
//! `renderer::webgl::loaders::gltf::skeleton_displacements_data_load` -- packs a glTF
//! primitive's morph-target position / normal / tangent displacements ( plus mesh-level morph
//! weights ) into a `skeleton::DisplacementsData`. Pure data transform over the parsed document
//! and raw buffer bytes, no `gl` / `GL` / `WebGl` calls anywhere in its body. Originally
//! private, made `pub` alongside this test per task 299.
//!
//! Needs the `native` feature ( its `[[test]]` entry in `Cargo.toml` ), so a plain
//! `cargo test -p renderer` skips it, the morph-target dequantization test included; run it
//! with `--features native` or `--all-features`, as CI does.
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use renderer::webgl::loaders::gltf::skeleton_displacements_data_load;
use gltf::mesh::iter::MorphTargets;

/// One mesh, one primitive, one morph target declaring `POSITION` only ( no `NORMAL`/`TANGENT` )
/// over 3 vertices. The base `POSITION` accessor ( index 0 ) is required for a valid primitive but
/// its bytes are never read by `skeleton_displacements_data_load` -- only the morph target's own
/// accessor ( index 1 ) is.
const MORPH_TARGET_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "meshes":
  [
    {
      "primitives":
      [
        {
          "attributes": { "POSITION": 0 },
          "targets": [ { "POSITION": 1 } ]
        }
      ]
    }
  ],
  "accessors":
  [
    { "bufferView": 0, "byteOffset": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [ 0.0, 0.0, 0.0 ], "max": [ 0.0, 0.0, 0.0 ] },
    { "bufferView": 1, "byteOffset": 0, "componentType": 5126, "count": 3, "type": "VEC3" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 36 }
  ],
  "buffers": [ { "byteLength": 72, "uri": "placeholder.bin" } ]
}
"#;

/// Base `POSITION` bytes are irrelevant filler ; the target `POSITION` displacement bytes are a
/// distinct, verifiable per-vertex pattern.
fn morph_target_buffers() -> Vec< Vec< u8 > >
{
  let mut bytes = vec![ 0u8; 36 ];
  for v in [ [ 1.0f32, 0.0, 0.0 ], [ 0.0, 1.0, 0.0 ], [ 0.0, 0.0, 1.0 ] ]
  {
    for c in v { bytes.extend_from_slice( &c.to_le_bytes() ); }
  }
  vec![ bytes ]
}

#[ test ]
fn returns_none_when_no_primitive_carries_morph_targets()
{
  let result = skeleton_displacements_data_load( None, &[ 3 ], None, &[] );
  assert!( result.is_none(), "no morph targets means no displacement data to pack" );
}

#[ test ]
fn packs_one_position_morph_target_and_stores_weights()
{
  let gltf = gltf::Gltf::from_slice( MORPH_TARGET_FIXTURE.as_bytes() ).unwrap();
  let mesh = gltf.meshes().next().expect( "fixture declares one mesh" );
  let primitive = mesh.primitives().next().expect( "fixture declares one primitive" );
  let morph_targets : Vec< MorphTargets< '_ > > = vec![ primitive.morph_targets() ];
  let buffers = morph_target_buffers();

  let mut result = skeleton_displacements_data_load( Some( &morph_targets ), &[ 3 ], Some( vec![ 0.5 ] ), &buffers )
  .expect( "a primitive with one POSITION morph target must produce DisplacementsData" );

  assert_eq!( result.attributes_count(), 3, "the loader zero-fills normals/tangents as placeholders for any morph target that omits them, so all 3 channels are always packed, not only the declared POSITION one" );

  let packed = result.displacements_data_pack();
  // Per vertex : [ position.xyz, 1.0 ], [ normal.xyz, 1.0 ], [ tangent.xyz, 1.0 ] -- normal/tangent
  // are the zero-filled placeholders since the fixture's morph target declares POSITION only.
  let expected : Vec< f32 > = vec!
  [
    1.0, 0.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0,
    0.0, 1.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0,
    0.0, 0.0, 1.0, 1.0,  0.0, 0.0, 0.0, 1.0,  0.0, 0.0, 0.0, 1.0
  ];
  assert_eq!( packed.len(), expected.len() );
  for ( actual, expect ) in packed.iter().zip( expected.iter() )
  {
    assert!( ( actual - expect ).abs() < f32::EPSILON, "packed displacement mismatch : {packed:?} vs {expected:?}" );
  }

  let weights = result.morph_weights_get();
  let weights = weights.borrow();
  assert_eq!( weights.len(), 1 );
  assert!( ( weights[ 0 ] - 0.5 ).abs() < f32::EPSILON );
}

/// `KHR_mesh_quantization` morph targets over 3 vertices, in the types its table allows :
/// `TANGENT` as normalized `i16` padded to a `byteStride` of 8, `NORMAL` as normalized `i8`
/// and `POSITION` as plain `i8` ( the only target that may be unnormalized ), each `i8` padded
/// to a stride of 4.
const QUANTIZED_MORPH_TARGET_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "KHR_mesh_quantization" ],
  "meshes":
  [
    {
      "primitives":
      [
        {
          "attributes": { "POSITION": 0 },
          "targets": [ { "POSITION": 3, "NORMAL": 2, "TANGENT": 1 } ]
        }
      ]
    }
  ],
  "accessors":
  [
    { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [ 0.0, 0.0, 0.0 ], "max": [ 0.0, 0.0, 0.0 ] },
    { "bufferView": 1, "componentType": 5122, "normalized": true, "count": 3, "type": "VEC3" },
    { "bufferView": 2, "componentType": 5120, "normalized": true, "count": 3, "type": "VEC3" },
    { "bufferView": 2, "byteOffset": 12, "componentType": 5120, "count": 3, "type": "VEC3" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 24, "byteStride": 8 },
    { "buffer": 0, "byteOffset": 60, "byteLength": 24, "byteStride": 4 }
  ],
  "buffers": [ { "byteLength": 84, "uri": "placeholder.bin" } ]
}
"#;

/// The quantized fixture's bytes : zeroed base positions, then each target's raw integers with
/// distinct padding bytes, so a reader that ignores the stride picks up the padding.
fn quantized_morph_target_buffers() -> Vec< Vec< u8 > >
{
  let mut bytes = vec![ 0u8; 36 ];
  for v in [ [ 32767i16, 0, -32768 ], [ 16384, -16384, 0 ], [ 0, 0, 32767 ] ]
  {
    for c in v { bytes.extend_from_slice( &c.to_le_bytes() ); }
    bytes.extend_from_slice( &[ 0x7f, 0x7f ] );
  }
  for v in [ [ 127i8, 0, 0 ], [ 0, -127, 0 ], [ 0, 0, 64 ], [ 3, -2, 1 ], [ 0, 5, 0 ], [ -7, 0, 0 ] ]
  {
    bytes.extend( v.map( | c | c.to_le_bytes()[ 0 ] ) );
    bytes.push( 0x7f );
  }
  vec![ bytes ]
}

/// Quantized morph targets are dequantized like the GPU would read them, not reinterpreted as
/// `f32` or dropped to zero.
///
/// ## Root Cause
/// Morph displacements were read through `gltf`'s `ReadPositionDisplacements`, which is an
/// `f32` iterator that never checks the accessor's component type.
///
/// ## Why Not Caught
/// Every morph fixture stored `f32` displacements.
///
/// ## Fix Applied
/// `accessor_vec3_read` reads each accessor by its declared type and applies the glTF
/// normalization equations when `normalized` is set.
///
/// ## Prevention
/// Loads `i16` / `i8` targets, normalized and not, with padding between elements.
///
/// ## Pitfall
/// Quantized `VEC3` elements are padded to 4-byte alignment, so the view's `byteStride` is
/// larger than the element : the padding must be skipped, not read.
#[ test ]
fn dequantizes_integer_morph_targets()
{
  let gltf = gltf::Gltf::from_slice_without_validation( QUANTIZED_MORPH_TARGET_FIXTURE.as_bytes() ).unwrap();
  let mesh = gltf.meshes().next().expect( "fixture declares one mesh" );
  let primitive = mesh.primitives().next().expect( "fixture declares one primitive" );
  let morph_targets : Vec< MorphTargets< '_ > > = vec![ primitive.morph_targets() ];
  let buffers = quantized_morph_target_buffers();

  let mut result = skeleton_displacements_data_load( Some( &morph_targets ), &[ 3 ], None, &buffers )
  .expect( "a primitive with quantized morph targets must produce DisplacementsData" );
  let packed = result.displacements_data_pack();

  let half = 16384.0 / 32767.0;
  // Per vertex : the position, normal and tangent displacement.
  let expected : Vec< f32 > = vec!
  [
    3.0, -2.0, 1.0, 1.0,  1.0, 0.0, 0.0, 1.0,  1.0, 0.0, -1.0, 1.0,
    0.0, 5.0, 0.0, 1.0,  0.0, -1.0, 0.0, 1.0,  half, -half, 0.0, 1.0,
    -7.0, 0.0, 0.0, 1.0,  0.0, 0.0, 64.0 / 127.0, 1.0,  0.0, 0.0, 1.0, 1.0
  ];
  assert_eq!( packed.len(), expected.len() );
  for ( actual, expect ) in packed.iter().zip( expected.iter() )
  {
    assert!( ( actual - expect ).abs() < 1e-6, "packed displacement mismatch : {packed:?} vs {expected:?}" );
  }
}

/// One primitive over 3 vertices with two `POSITION` morph targets : target 0 holds 3
/// displacements, target 1 only 2, each `7.0`.
const SHORT_MORPH_TARGET_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "meshes":
  [
    {
      "primitives":
      [
        {
          "attributes": { "POSITION": 0 },
          "targets": [ { "POSITION": 1 }, { "POSITION": 2 } ]
        }
      ]
    }
  ],
  "accessors":
  [
    { "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [ 0.0, 0.0, 0.0 ], "max": [ 0.0, 0.0, 0.0 ] },
    { "bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3" },
    { "bufferView": 2, "componentType": 5126, "count": 2, "type": "VEC3" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteOffset": 0, "byteLength": 36 },
    { "buffer": 0, "byteOffset": 36, "byteLength": 36 },
    { "buffer": 0, "byteOffset": 72, "byteLength": 24 }
  ],
  "buffers": [ { "byteLength": 96, "uri": "placeholder.bin" } ]
}
"#;

/// A morph target channel must hold one displacement per vertex : packing interleaves the
/// targets vertex by vertex, and indexed a shorter target past its end. The short channel is
/// zero-filled like an unreadable one.
#[ test ]
fn zero_fills_a_morph_channel_that_does_not_hold_one_value_per_vertex()
{
  let gltf = gltf::Gltf::from_slice( SHORT_MORPH_TARGET_FIXTURE.as_bytes() ).unwrap();
  let primitive = gltf.meshes().next().expect( "fixture mesh" ).primitives().next().expect( "fixture primitive" );
  let morph_targets : Vec< MorphTargets< '_ > > = vec![ primitive.morph_targets() ];
  let mut bytes = morph_target_buffers().remove( 0 );
  for c in [ 7.0_f32; 6 ] { bytes.extend_from_slice( &c.to_le_bytes() ); }

  let mut result = skeleton_displacements_data_load( Some( &morph_targets ), &[ 3 ], None, &[ bytes ] )
  .expect( "a primitive with morph targets produces DisplacementsData" );
  let packed = result.displacements_data_pack();

  // 3 vertices, 2 targets, 3 channels, 4 floats each.
  assert_eq!( packed.len(), 3 * 2 * 3 * 4 );
  // Every one of the 18 packed vec4s ends in 1.0; target 0 adds its three unit components.
  assert_eq!( packed.iter().filter( | &&v | ( v - 1.0 ).abs() < f32::EPSILON ).count(), 18 + 3, "target 0 is packed" );
  assert!( packed.iter().all( | &v | ( v - 7.0 ).abs() > f32::EPSILON ), "the short target 1 is not : {packed:?}" );
}

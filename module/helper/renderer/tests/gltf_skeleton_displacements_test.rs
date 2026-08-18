//! Unit-level validation tests for
//! `renderer::webgl::loaders::gltf::skeleton_displacements_data_load` -- packs a glTF
//! primitive's morph-target position / normal / tangent displacements ( plus mesh-level morph
//! weights ) into a `skeleton::DisplacementsData`. Pure data transform over the parsed document
//! and raw buffer bytes, no `gl` / `GL` / `WebGl` calls anywhere in its body. Originally
//! private, made `pub` alongside this test per task 441.
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

//! `renderer::webgl::loaders::gltf::skeleton_transforms_data_load` refusing a skin whose
//! inverse bind matrices the `gltf` crate's reader cannot read safely. That reader takes the
//! accessor as `MAT4` `f32` without checking, and locates its data with unchecked arithmetic,
//! so the loader checks the type, one matrix per joint, and the ranges first.
//!
//! Needs the `native` feature ( its `[[test]]` entry in `Cargo.toml` ); run it with
//! `--features native` or `--all-features`, as CI does.
#![ cfg( all( feature = "native", not( target_arch = "wasm32" ) ) ) ]

use std::{ cell::RefCell, rc::Rc };
use renderer::webgl::{ Node, loaders::gltf::skeleton_transforms_data_load };

/// Seven skins over two nodes. Skin 0 is well formed : one joint, one `MAT4` `f32` matrix.
/// Each later skin breaks one thing about its matrices, named in the test.
const SKINS_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "nodes": [ {}, {} ],
  "skins":
  [
    { "joints": [ 0 ], "inverseBindMatrices": 0 },
    { "joints": [ 0 ], "inverseBindMatrices": 1 },
    { "joints": [ 0 ], "inverseBindMatrices": 2 },
    { "joints": [ 0 ], "inverseBindMatrices": 3 },
    { "joints": [ 0 ], "inverseBindMatrices": 4 },
    { "joints": [ 0 ], "inverseBindMatrices": 5 },
    { "joints": [ 0, 1 ], "inverseBindMatrices": 0 }
  ],
  "accessors":
  [
    { "bufferView": 0, "componentType": 5126, "count": 1, "type": "MAT4" },
    { "bufferView": 0, "byteOffset": 18446744073709551615, "componentType": 5126, "count": 1, "type": "MAT4" },
    { "bufferView": 0, "componentType": 5126, "count": 0, "type": "MAT4" },
    { "bufferView": 0, "componentType": 5126, "count": 1, "type": "VEC3" },
    { "bufferView": 0, "componentType": 5123, "count": 1, "type": "MAT4" },
    { "bufferView": 1, "componentType": 5126, "count": 1, "type": "MAT4" }
  ],
  "bufferViews":
  [
    { "buffer": 0, "byteLength": 64 },
    { "buffer": 0, "byteLength": 64, "byteStride": 4 }
  ],
  "buffers": [ { "byteLength": 64, "uri": "placeholder.bin" } ]
}
"#;

#[ test ]
fn refuses_inverse_bind_matrices_it_cannot_read()
{
  let gltf = gltf::Gltf::from_slice_without_validation( SKINS_FIXTURE.as_bytes() ).expect( "fixture is well-formed JSON" );
  let nodes : Vec< Rc< RefCell< Node > > > = ( 0..2 ).map( | _ | Rc::new( RefCell::new( Node::new() ) ) ).collect();
  let identity = [ 1.0_f32, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0 ];
  let buffers = vec![ identity.iter().flat_map( | c | c.to_le_bytes() ).collect::< Vec< u8 > >() ];
  let load = | i : usize | skeleton_transforms_data_load( &gltf.skins().nth( i ).expect( "fixture skin" ), &nodes, &buffers );

  let transforms = load( 0 ).expect( "a well-formed skin loads" );
  assert_eq!( transforms.joints_get().len(), 1 );
  assert!( Rc::ptr_eq( &transforms.joints_get()[ 0 ], &nodes[ 0 ] ) );

  for ( i, what ) in
  [
    ( 1, "a huge accessor offset" ),
    ( 2, "a zero count" ),
    ( 3, "a VEC3 accessor" ),
    ( 4, "u16 components" ),
    ( 5, "a byteStride shorter than a matrix" ),
    ( 6, "one matrix for two joints" ),
  ]
  {
    assert!( load( i ).is_none(), "a skin with {what} is refused" );
  }
}

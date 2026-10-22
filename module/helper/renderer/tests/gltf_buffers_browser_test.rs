//! Browser tests for the buffer rules `gltf::load` applies to every asset, compressed or not :
//! the GLB binary chunk can only be `buffers[ 0 ]`, a uri-less buffer with no chunk to take is
//! an error, and a morph target is read from the CPU copy `load` takes for it. The meshopt
//! fallback rules are in `gltf_meshopt_browser_test.rs`.

mod gltf_browser_helpers;

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use renderer::webgl::loaders::gltf;
  use renderer::webgl::Object3D;
  use crate::gltf_browser_helpers::*;

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

  /// A morph target is read on the CPU, so `load` must hand the skeleton stage its copy of the
  /// buffers ( `cpu_buffers_needed` ), which it skips for a static mesh. The triangle's one
  /// `POSITION` target moves each vertex along a different axis; the mesh's skeleton must pack
  /// those displacements, with zero normals and tangents, and keep the mesh's weight. Read
  /// from no buffers, the target would have no data.
  #[ wasm_bindgen_test( async ) ]
  async fn load_reads_morph_targets_from_the_cpu_copy()
  {
    let displacements = [ 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0_f32 ].iter().flat_map( | f | f.to_le_bytes() ).collect::< Vec< u8 > >();
    let json = format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        "buffers" : [ {{ "byteLength" : 72, "uri" : "{}" }} ],
        "bufferViews" : [ {{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }}, {{ "buffer" : 0, "byteOffset" : 36, "byteLength" : 36 }} ],
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
      data_uri( "application/octet-stream", &[ triangle_bytes(), displacements ].concat() )
    );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    let loaded = gltf::load( &document(), &uri, &gl_init() ).await.expect( "the morphed triangle loads" );

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
    assert_eq!( data.displacements_data_pack(), expected, "the target's displacements, read from the buffer" );
    assert_eq!( *data.morph_weights_get().borrow(), vec![ 0.5_f32 ], "the mesh's weight" );
  }
}

//! Browser test for loading a `KHR_mesh_quantization` mesh end to end : `gltf::load` must
//! hand the scene the bounds the shader sees, not the raw integers the accessor stores.

mod gltf_browser_helpers;

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use renderer::webgl::loaders::gltf;
  use crate::gltf_browser_helpers::*;

  /// A triangle whose `POSITION` is normalized `i16`, each element padded to 8 bytes, with the
  /// raw integer bounds the accessor declares : `max` is `( 32767, 16384, 0 )`.
  fn quantized_triangle() -> String
  {
    let mut bytes = Vec::new();
    for v in [ [ 0_i16, 0, 0 ], [ 32767, 0, 0 ], [ 0, 16384, 0 ] ]
    {
      for c in v { bytes.extend( c.to_le_bytes() ); }
      bytes.extend( [ 0, 0 ] );
    }
    format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        "extensionsUsed" : [ "KHR_mesh_quantization" ], "extensionsRequired" : [ "KHR_mesh_quantization" ],
        "buffers" : [ {{ "byteLength" : 24, "uri" : "{}" }} ],
        "bufferViews" : [ {{ "buffer" : 0, "byteLength" : 24, "byteStride" : 8, "target" : 34962 }} ],
        "accessors" : [ {{ "bufferView" : 0, "componentType" : 5122, "normalized" : true, "count" : 3, "type" : "VEC3",
          "min" : [ 0, 0, 0 ], "max" : [ 32767, 16384, 0 ] }} ],
        "materials" : [ {{}} ],
        "meshes" : [ {{ "primitives" : [ {{ "attributes" : {{ "POSITION" : 0 }}, "material" : 0 }} ] }} ],
        "nodes" : [ {{ "mesh" : 0 }} ],
        "scenes" : [ {{ "nodes" : [ 0 ] }} ],
        "scene" : 0
      }}"#,
      data_uri( "application/octet-stream", &bytes )
    )
  }

  /// WebGL reads a normalized `i16` as -1..1, so the mesh's bounds are the accessor's raw
  /// bounds divided by 32767 : `max` is `( 1, 16384 / 32767, 0 )`. Taking the raw bounds would
  /// make the box 32767 times too large, and with it culling and framing.
  #[ wasm_bindgen_test( async ) ]
  async fn load_dequantizes_normalized_position_bounds()
  {
    let uri = data_uri( "model/gltf+json", quantized_triangle().as_bytes() );

    let loaded = gltf::load( &document(), &uri, &gl_init() ).await.expect( "the quantized triangle loads" );

    let bounds = loaded.meshes[ 0 ].borrow().bounding_box();
    let near = | a : f32, b : f32 | ( a - b ).abs() < 1e-6;
    let ( min, max ) = ( bounds.min.0, bounds.max.0 );
    assert!( min.iter().all( | &c | near( c, 0.0 ) ), "min {min:?}" );
    assert!( near( max[ 0 ], 1.0 ) && near( max[ 1 ], 16384.0 / 32767.0 ) && near( max[ 2 ], 0.0 ), "max {max:?}" );
  }
}

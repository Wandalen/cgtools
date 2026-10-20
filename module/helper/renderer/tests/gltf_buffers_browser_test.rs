//! Browser tests for the buffer rules `gltf::load` applies to every asset, compressed or not :
//! the GLB binary chunk can only be `buffers[ 0 ]`, and a uri-less buffer with no chunk to take
//! is an error. The meshopt fallback rules are in `gltf_meshopt_browser_test.rs`.

mod gltf_browser_helpers;

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use renderer::webgl::loaders::gltf;
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
}

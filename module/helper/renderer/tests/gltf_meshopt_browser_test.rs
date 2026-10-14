//! Browser tests for the buffer-index rules of `gltf::load` : the GLB binary chunk can only be
//! `buffers[ 0 ]`, a uri-less buffer with no chunk to take is an error, and an oversized meshopt
//! fallback buffer is refused instead of allocated.

#[ cfg( target_arch = "wasm32" ) ]
#[ cfg( test ) ]
mod tests
{
  use wasm_bindgen_test::wasm_bindgen_test;
  wasm_bindgen_test::wasm_bindgen_test_configure!( run_in_browser );
  use minwebgl as gl;
  use gl::GL;
  use renderer::webgl::loaders::gltf;

  fn gl_init() -> GL
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::make().unwrap();
    gl::context::from_canvas_with( &canvas, gl::context::ContextOptions::default() ).unwrap()
  }

  fn document() -> gl::web_sys::Document
  {
    gl::web_sys::window().unwrap().document().unwrap()
  }

  /// One triangle's `POSITION` data : three `f32` VEC3 elements, 12 bytes each.
  fn triangle_bytes() -> Vec< u8 >
  {
    [ 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0_f32 ].iter().flat_map( | f | f.to_le_bytes() ).collect()
  }

  fn data_uri( mime : &str, bytes : &[ u8 ] ) -> String
  {
    let latin1 : String = bytes.iter().map( | &b | char::from( b ) ).collect();
    format!( "data:{mime};base64,{}", gl::web_sys::window().unwrap().btoa( &latin1 ).unwrap() )
  }

  /// A one-triangle scene whose `POSITION` accessor reads bufferView 0. `buffers` and `view` are
  /// spliced in as JSON; `used` lists extensions for `extensionsUsed` / `extensionsRequired`.
  fn triangle_gltf( used : &[ &str ], buffers : &str, view : &str ) -> String
  {
    let extensions = if used.is_empty()
    {
      String::new()
    }
    else
    {
      let names = used.iter().map( | name | format!( "\"{name}\"" ) ).collect::< Vec< _ > >().join( ", " );
      format!( r#""extensionsUsed" : [ {names} ], "extensionsRequired" : [ {names} ],"# )
    };
    format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        {extensions}
        "buffers" : [ {buffers} ],
        "bufferViews" : [ {view} ],
        "accessors" : [ {{ "bufferView" : 0, "componentType" : 5126, "count" : 3, "type" : "VEC3", "min" : [ 0, 0, 0 ], "max" : [ 1, 1, 0 ] }} ],
        "materials" : [ {{}} ],
        "meshes" : [ {{ "primitives" : [ {{ "attributes" : {{ "POSITION" : 0 }}, "material" : 0 }} ] }} ],
        "nodes" : [ {{ "mesh" : 0 }} ],
        "scenes" : [ {{ "nodes" : [ 0 ] }} ],
        "scene" : 0
      }}"#
    )
  }

  /// Packs `json` and `bin` into a GLB container : a 12-byte header, then the JSON chunk padded
  /// with spaces and the BIN chunk padded with zeros, each behind its length and type.
  fn glb( json : &str, bin : &[ u8 ] ) -> Vec< u8 >
  {
    let as_u32 = | n : usize | u32::try_from( n ).unwrap().to_le_bytes();
    let mut json = json.as_bytes().to_vec();
    json.resize( json.len().next_multiple_of( 4 ), b' ' );
    let mut bin = bin.to_vec();
    bin.resize( bin.len().next_multiple_of( 4 ), 0 );

    let mut out = Vec::new();
    out.extend_from_slice( b"glTF" );
    out.extend_from_slice( &2_u32.to_le_bytes() );
    out.extend_from_slice( &as_u32( 12 + 8 + json.len() + 8 + bin.len() ) );
    out.extend_from_slice( &as_u32( json.len() ) );
    out.extend_from_slice( b"JSON" );
    out.extend( json );
    out.extend_from_slice( &as_u32( bin.len() ) );
    out.extend_from_slice( b"BIN\0" );
    out.extend( bin );
    out
  }

  /// Reads back the GPU buffer `load` uploaded for bufferView `view`.
  fn view_bytes_read( gl : &GL, loaded : &gltf::GLTF, view : usize, length : usize ) -> Vec< u8 >
  {
    let mut bytes = vec![ 0u8; length ];
    gl.bind_buffer( gl::COPY_READ_BUFFER, Some( &loaded.gl_buffers[ view ] ) );
    gl.get_buffer_sub_data_with_i32_and_u8_array( gl::COPY_READ_BUFFER, 0, &mut bytes );
    gl.bind_buffer( gl::COPY_READ_BUFFER, None );
    bytes
  }

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
    let gl = gl_init();
    let buffers = format!
    (
      r#"{{ "byteLength" : 36, "uri" : "{}" }}, {{ "byteLength" : 36 }}"#,
      data_uri( "application/octet-stream", &triangle_bytes() )
    );
    let json = triangle_gltf( &[], &buffers, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf-binary", &glb( &json, &triangle_bytes() ) );

    assert!( gltf::load( &document(), &uri, &gl ).await.is_err() );
  }

  /// A uri-less buffer in a file with no binary chunk has no data to load.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_a_uri_less_buffer_without_a_glb_chunk()
  {
    let gl = gl_init();
    let json = triangle_gltf( &[], r#"{ "byteLength" : 36 }"#, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert!( gltf::load( &document(), &uri, &gl ).await.is_err() );
  }

  /// A fallback buffer's size comes only from its declared `byteLength`, so a huge declaration
  /// is refused with an error rather than allocated or thrown as a `RangeError`.
  #[ wasm_bindgen_test( async ) ]
  async fn load_refuses_an_oversized_fallback_buffer()
  {
    let gl = gl_init();
    let buffers = format!
    (
      r#"{{ "byteLength" : 36, "uri" : "{}" }},
      {{ "byteLength" : 3000000000, "extensions" : {{ "EXT_meshopt_compression" : {{ "fallback" : true }} }} }}"#,
      data_uri( "application/octet-stream", &triangle_bytes() )
    );
    let json = triangle_gltf( &[ "EXT_meshopt_compression" ], &buffers, r#"{ "buffer" : 0, "byteLength" : 36, "target" : 34962 }"# );
    let uri = data_uri( "model/gltf+json", json.as_bytes() );

    assert!( gltf::load( &document(), &uri, &gl ).await.is_err() );
  }
}

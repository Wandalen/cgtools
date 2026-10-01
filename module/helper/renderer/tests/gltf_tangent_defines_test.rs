//! Live `gltf::load` test: a primitive's TANGENT attribute must reach both shader stages.
//!
//! The loader records vertex attributes as defines on a scratch material and copies them onto
//! each primitive's material. `main.frag` builds its tangent frame from `vTangent` only under
//! `USE_TANGENTS`, so a define copied to the vertex stage alone leaves every asset that ships
//! tangents on the derivative frame instead of its authored tangents.

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

  /// A one-material glTF with two triangles as `data:` URIs: primitive 0 has POSITION, NORMAL
  /// and TANGENT, primitive 1 only POSITION and NORMAL.
  fn two_primitive_gltf_uri() -> String
  {
    let positions = [ 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0_f32 ];
    let normals = [ 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0_f32 ];
    let tangents = [ 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0_f32 ];
    let bytes : Vec< u8 > = positions.iter().chain( &normals ).chain( &tangents ).flat_map( | f | f.to_le_bytes() ).collect();
    let window = gl::web_sys::window().unwrap();
    let latin1 : String = bytes.iter().map( | &b | char::from( b ) ).collect();
    let buffer = window.btoa( &latin1 ).unwrap();

    let json = format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        "buffers" : [ {{ "byteLength" : {len}, "uri" : "data:application/octet-stream;base64,{buffer}" }} ],
        "bufferViews" : [
          {{ "buffer" : 0, "byteOffset" : 0, "byteLength" : 36 }},
          {{ "buffer" : 0, "byteOffset" : 36, "byteLength" : 36 }},
          {{ "buffer" : 0, "byteOffset" : 72, "byteLength" : 48 }}
        ],
        "accessors" : [
          {{ "bufferView" : 0, "componentType" : 5126, "count" : 3, "type" : "VEC3", "min" : [ 0, 0, 0 ], "max" : [ 1, 1, 0 ] }},
          {{ "bufferView" : 1, "componentType" : 5126, "count" : 3, "type" : "VEC3" }},
          {{ "bufferView" : 2, "componentType" : 5126, "count" : 3, "type" : "VEC4" }}
        ],
        "materials" : [ {{}} ],
        "meshes" : [ {{ "primitives" : [
          {{ "attributes" : {{ "POSITION" : 0, "NORMAL" : 1, "TANGENT" : 2 }}, "material" : 0 }},
          {{ "attributes" : {{ "POSITION" : 0, "NORMAL" : 1 }}, "material" : 0 }}
        ] }} ],
        "nodes" : [ {{ "mesh" : 0 }} ],
        "scenes" : [ {{ "nodes" : [ 0 ] }} ],
        "scene" : 0
      }}"#,
      len = bytes.len()
    );
    format!( "data:model/gltf+json;base64,{}", window.btoa( &json ).unwrap() )
  }

  /// A TANGENT attribute turns on `USE_TANGENTS` in the fragment stage, where `main.frag` reads
  /// `vTangent`, and a primitive without one keeps the derivative frame.
  #[ wasm_bindgen_test( async ) ]
  async fn tangent_attribute_reaches_the_fragment_defines()
  {
    let gl = gl_init();
    let document = gl::web_sys::window().unwrap().document().unwrap();
    let gltf = gltf::load( &document, &two_primitive_gltf_uri(), &gl ).await.expect( "fixture loads" );

    let mesh = gltf.meshes[ 0 ].borrow();
    let fragment_defines = | i : usize | mesh.primitives[ i ].borrow().material.borrow().fragment_defines_str().to_owned();
    let with_tangents = fragment_defines( 0 );
    let without_tangents = fragment_defines( 1 );

    assert!( with_tangents.contains( "#define USE_TANGENTS" ), "{with_tangents}" );
    assert!( !without_tangents.contains( "USE_TANGENTS" ), "{without_tangents}" );
  }
}

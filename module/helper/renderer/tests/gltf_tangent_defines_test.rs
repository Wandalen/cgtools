//! Live `gltf::load` tests on a small `data:` URI asset.
//!
//! - A primitive's TANGENT attribute must reach both shader stages. The loader records vertex
//!   attributes as defines on a scratch material and copies them onto each primitive's material.
//!   `main.frag` builds its tangent frame from `vTangent` only under `USE_TANGENTS`, so a define
//!   copied to the vertex stage alone leaves every asset that ships tangents on the derivative
//!   frame instead of its authored tangents.
//! - `extensionsRequired` must be checked by `load` itself. `load` parses without `gltf`'s own
//!   validation and relies on one `document_validate` call for every check, so a test of
//!   `document_validate` alone wouldn't notice that call being removed or moved.

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
  /// and TANGENT, primitive 1 only POSITION and NORMAL. `required` names extensions listed in
  /// both `extensionsUsed` and `extensionsRequired`; nothing in the asset uses them.
  fn two_primitive_gltf_uri( required : &[ &str ] ) -> String
  {
    let positions = [ 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0_f32 ];
    let normals = [ 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0_f32 ];
    let tangents = [ 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0_f32 ];
    let bytes : Vec< u8 > = positions.iter().chain( &normals ).chain( &tangents ).flat_map( | f | f.to_le_bytes() ).collect();
    let window = gl::web_sys::window().unwrap();
    let latin1 : String = bytes.iter().map( | &b | char::from( b ) ).collect();
    let buffer = window.btoa( &latin1 ).unwrap();
    let extensions = if required.is_empty()
    {
      String::new()
    }
    else
    {
      let names = required.iter().map( | name | format!( "\"{name}\"" ) ).collect::< Vec< _ > >().join( ", " );
      format!( r#""extensionsUsed" : [ {names} ], "extensionsRequired" : [ {names} ],"# )
    };

    let json = format!
    (
      r#"{{
        "asset" : {{ "version" : "2.0" }},
        {extensions}
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
  ///
  /// ## Root Cause
  /// The loader records attribute defines on a scratch material for both stages but copied only
  /// its vertex defines onto each primitive's material.
  ///
  /// ## Why Not Caught
  /// No test loaded an asset with TANGENT data, and the derivative frame still shades plausibly.
  ///
  /// ## Fix Applied
  /// The fragment defines are copied as well, and `main.vert` moves the tangent into world space.
  ///
  /// ## Prevention
  /// Loads a two-primitive asset through `gltf::load` and checks that only the primitive with
  /// TANGENT gets `USE_TANGENTS` in its fragment defines.
  ///
  /// ## Pitfall
  /// The two stages compile from separate define sets; a define recorded for both must reach both.
  // test_kind: bug_reproducer(BUG-535)
  #[ wasm_bindgen_test( async ) ]
  async fn tangent_attribute_reaches_the_fragment_defines()
  {
    let gl = gl_init();
    let document = gl::web_sys::window().unwrap().document().unwrap();
    let gltf = gltf::load( &document, &two_primitive_gltf_uri( &[] ), &gl ).await.expect( "fixture loads" );

    let mesh = gltf.meshes[ 0 ].borrow();
    let fragment_defines = | i : usize | mesh.primitives[ i ].borrow().material.borrow().fragment_defines_str().to_owned();
    let with_tangents = fragment_defines( 0 );
    let without_tangents = fragment_defines( 1 );

    assert!( with_tangents.contains( "#define USE_TANGENTS" ), "{with_tangents}" );
    assert!( !without_tangents.contains( "USE_TANGENTS" ), "{without_tangents}" );
  }

  /// An asset that requires an extension this loader doesn't support must fail to load, as glTF's
  /// "Specifying Extensions" requires of a conformant client.
  #[ wasm_bindgen_test( async ) ]
  async fn load_rejects_an_unsupported_required_extension()
  {
    let gl = gl_init();
    let document = gl::web_sys::window().unwrap().document().unwrap();
    let uri = two_primitive_gltf_uri( &[ "KHR_draco_mesh_compression" ] );

    let result = gltf::load( &document, &uri, &gl ).await;

    assert!( result.is_err(), "an asset requiring KHR_draco_mesh_compression must not load" );
  }

  /// The required extensions this loader supports must still load through `load`, although
  /// `gltf`'s own validation, which `load` skips, rejects them.
  #[ wasm_bindgen_test( async ) ]
  async fn load_accepts_the_supported_required_extensions()
  {
    let gl = gl_init();
    let document = gl::web_sys::window().unwrap().document().unwrap();
    let uri = two_primitive_gltf_uri( &[ "KHR_materials_clearcoat", "KHR_materials_anisotropy" ] );

    let result = gltf::load( &document, &uri, &gl ).await;

    assert!( result.is_ok(), "an asset requiring the clearcoat and anisotropy extensions must load" );
  }
}

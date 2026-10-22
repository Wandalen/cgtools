//! Verifies the glTF loader's required-extension gate
//! ( `renderer::webgl::loaders::gltf_extensions::required_extensions_check` ) -- the pure,
//! off-GPU check that enforces glTF 2.0's "Specifying Extensions" rule: a
//! conformant client MUST refuse to load an asset whose `extensionsRequired`
//! names an extension it doesn't support, rather than silently proceeding and
//! producing incomplete/incorrect output. `load()` runs this check immediately
//! after parsing, before any buffer/image/GL work, so it is a pure function
//! over an already-parsed `gltf::Gltf` -- zero `WebGl2RenderingContext`/`gl::`
//! calls anywhere in its body -- same off-GPU pattern as
//! `gltf_light_parsing_test.rs` / `gltf_node_scene_test.rs`.
//!
//! Fixtures use `gltf::Gltf::from_slice_without_validation` rather than
//! `from_slice`: the `gltf` crate's own `Document::validate()` ( gltf-json
//! 1.4.1's `root_validate_hook` ) independently rejects any `extensionsRequired`
//! entry outside its own compile-time `ENABLED_EXTENSIONS` list, which -- for
//! this crate's enabled Cargo features -- resolves to only `"KHR_lights_punctual"`
//! ( `KHR_materials_specular` is notably absent from that upstream list despite
//! being a real, enabled Cargo feature with real typed support in the `gltf`
//! crate -- an apparent upstream gap, not something this loader controls ).
//! Using `from_slice_without_validation` isolates this loader's own
//! `required_extensions_check` from that separate, unrelated upstream gate, so
//! these tests exercise exactly this loader's own checks in `gltf_extensions.rs`.

use renderer::webgl::loaders::gltf_extensions::{ document_validate, required_extensions_check, texture_transform_check };

const UNSUPPORTED_REQUIRED_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsRequired": [ "KHR_texture_basisu" ],
  "extensionsUsed": [ "KHR_texture_basisu" ]
}
"#;

#[ test ]
fn rejects_unsupported_required_extension()
{
  let gltf = gltf::Gltf::from_slice_without_validation( UNSUPPORTED_REQUIRED_FIXTURE.as_bytes() )
  .expect( "structurally well-formed JSON must parse without validation" );

  let result = required_extensions_check( &gltf );

  assert!
  (
    result.is_err(),
    "extensionsRequired names 'KHR_texture_basisu', which this loader does not implement -- \
    must be rejected, not silently proceeded past"
  );
}

const SUPPORTED_REQUIRED_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsRequired": [ "KHR_lights_punctual" ],
  "extensions": { "KHR_lights_punctual": { "lights": [] } }
}
"#;

#[ test ]
fn accepts_supported_required_extension()
{
  let gltf = gltf::Gltf::from_slice_without_validation( SUPPORTED_REQUIRED_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!
  (
    required_extensions_check( &gltf ).is_ok(),
    "KHR_lights_punctual is genuinely implemented by this loader ( light_list_get / light_get ) -- \
    must not be rejected"
  );
}

const TWO_REQUIRED_ONE_UNSUPPORTED_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsRequired": [ "KHR_lights_punctual", "KHR_draco_mesh_compression" ],
  "extensions": { "KHR_lights_punctual": { "lights": [] } }
}
"#;

#[ test ]
fn rejects_when_any_required_extension_is_unsupported()
{
  // One supported entry must not mask a second, unsupported one -- every entry
  // in `extensionsRequired` has to individually clear the whitelist.
  let gltf = gltf::Gltf::from_slice_without_validation( TWO_REQUIRED_ONE_UNSUPPORTED_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!
  (
    required_extensions_check( &gltf ).is_err(),
    "KHR_draco_mesh_compression is unsupported even though KHR_lights_punctual ( also required ) is supported"
  );
}

const NO_REQUIRED_EXTENSIONS_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" }
}
"#;

#[ test ]
fn accepts_asset_with_no_required_extensions()
{
  let gltf = gltf::Gltf::from_slice_without_validation( NO_REQUIRED_EXTENSIONS_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!
  (
    required_extensions_check( &gltf ).is_ok(),
    "the typical case -- no extensionsRequired at all -- must not be rejected"
  );
}

const MATERIAL_LAYER_EXTENSIONS_REQUIRED_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsUsed": [ "KHR_materials_clearcoat", "KHR_materials_anisotropy" ],
  "extensionsRequired": [ "KHR_materials_clearcoat", "KHR_materials_anisotropy" ],
  "materials": [
    {
      "extensions": {
        "KHR_materials_clearcoat": { "clearcoatFactor": 1.0 },
        "KHR_materials_anisotropy": { "anisotropyStrength": 0.5 }
      }
    }
  ]
}
"#;

/// `material_layer_extensions_apply` reads both extensions' JSON by hand, so an asset that
/// requires them must pass this loader's own gate.
#[ test ]
fn accepts_required_clearcoat_and_anisotropy()
{
  let gltf = gltf::Gltf::from_slice_without_validation( MATERIAL_LAYER_EXTENSIONS_REQUIRED_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!( required_extensions_check( &gltf ).is_ok(), "clearcoat / anisotropy are implemented -- must not be rejected" );
}

/// `gltf::Gltf::from_slice` rejects this valid asset: gltf-json's own extensionsRequired rule
/// only knows the extensions it has typed support for. `load` therefore parses without that
/// validation and runs `document_validate`, which must accept the asset.
#[ test ]
fn document_validate_ignores_upstream_required_extension_list()
{
  assert!
  (
    gltf::Gltf::from_slice( MATERIAL_LAYER_EXTENSIONS_REQUIRED_FIXTURE.as_bytes() ).is_err(),
    "precondition: upstream validation rejects hand-read required extensions"
  );
  let gltf = gltf::Gltf::from_slice_without_validation( MATERIAL_LAYER_EXTENSIONS_REQUIRED_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!( document_validate( &gltf ).is_ok(), "only the upstream extensionsRequired rule may be skipped" );
}

const DRACO_REQUIRED_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensionsRequired": [ "KHR_draco_mesh_compression" ],
  "extensionsUsed": [ "KHR_draco_mesh_compression" ]
}
"#;

/// `document_validate` skips gltf-json's own extensionsRequired rule, so it must apply this
/// loader's rule itself: a caller that validates with it alone must not accept an asset
/// requiring an extension the loader can't decode.
#[ test ]
fn document_validate_alone_rejects_unsupported_required_extension()
{
  let gltf = gltf::Gltf::from_slice_without_validation( DRACO_REQUIRED_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!( document_validate( &gltf ).is_err(), "KHR_draco_mesh_compression is not supported" );
}

const DANGLING_NODE_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "scenes": [ { "nodes": [ 3 ] } ]
}
"#;

/// Skipping the upstream extension rule must not skip the rest of gltf-json's validation:
/// a scene naming a node that doesn't exist is still an error.
#[ test ]
fn document_validate_still_reports_structural_errors()
{
  let gltf = gltf::Gltf::from_slice_without_validation( DANGLING_NODE_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!( document_validate( &gltf ).is_err(), "an out-of-range node index must fail validation" );
}

/// A one-primitive mesh whose `TEXCOORD_0` has `uv_type` ( a glTF componentType ), with
/// `used` in `extensionsUsed`. Structurally valid, so `document_validate` gets as far as
/// `texture_transform_check`.
fn texcoord_document( uv_type : u32, used : &[ &str ] ) -> gltf::Gltf
{
  let used = used.iter().map( | name | format!( "\"{name}\"" ) ).collect::< Vec< _ > >().join( ", " );
  let normalized = if uv_type == 5126 { "" } else { r#", "normalized" : true"# };
  let json = format!
  (
    r#"{{
      "asset" : {{ "version" : "2.0" }},
      "extensionsUsed" : [ {used} ],
      "buffers" : [ {{ "byteLength" : 64, "uri" : "a.bin" }} ],
      "bufferViews" : [ {{ "buffer" : 0, "byteLength" : 36 }}, {{ "buffer" : 0, "byteOffset" : 36, "byteLength" : 24 }} ],
      "accessors" :
      [
        {{ "bufferView" : 0, "componentType" : 5126, "count" : 3, "type" : "VEC3", "min" : [ 0, 0, 0 ], "max" : [ 1, 1, 0 ] }},
        {{ "bufferView" : 1, "componentType" : {uv_type}{normalized}, "count" : 3, "type" : "VEC2" }}
      ],
      "meshes" : [ {{ "primitives" : [ {{ "attributes" : {{ "POSITION" : 0, "TEXCOORD_0" : 1 }} }} ] }} ]
    }}"#
  );
  gltf::Gltf::from_slice_without_validation( json.as_bytes() ).expect( "well-formed JSON" )
}

/// `KHR_mesh_quantization` relies on `KHR_texture_transform` to dequantize texture
/// coordinates, which this loader does not apply. An asset with integer texture coordinates
/// that uses it would draw its textures misplaced, so it is refused with that reason.
#[ test ]
fn quantized_texture_coordinates_with_a_texture_transform_are_refused()
{
  let gltf = texcoord_document( 5123, &[ "KHR_mesh_quantization", "KHR_texture_transform" ] );
  for result in [ texture_transform_check( &gltf ), document_validate( &gltf ) ]
  {
    match result
    {
      Err( minwebgl::WebglError::Other( reason ) ) =>
        assert_eq!( reason, "glTF asset needs KHR_texture_transform for its quantized texture coordinates" ),
      other => panic!( "must be refused, got {other:?}" ),
    }
  }
}

/// Quantized texture coordinates with no texture transform, as glTF-Transform writes them
/// ( normalized to 0..1 ), load : the GPU dequantizes them.
#[ test ]
fn quantized_texture_coordinates_without_a_texture_transform_load()
{
  let gltf = texcoord_document( 5123, &[ "KHR_mesh_quantization" ] );
  assert!( texture_transform_check( &gltf ).is_ok() );
  assert!( document_validate( &gltf ).is_ok() );
}

/// With float texture coordinates a texture transform only moves a texture, so an asset
/// using one still loads ( with a warning that it is ignored ).
#[ test ]
fn float_texture_coordinates_with_a_texture_transform_load()
{
  let gltf = texcoord_document( 5126, &[ "KHR_texture_transform" ] );
  assert!( texture_transform_check( &gltf ).is_ok() );
  assert!( document_validate( &gltf ).is_ok() );
}

/// An asset that requires `KHR_texture_transform` is refused before any of this, as any
/// unsupported required extension is.
#[ test ]
fn a_required_texture_transform_is_refused()
{
  let gltf = gltf::Gltf::from_slice_without_validation
  (
    br#"{ "asset" : { "version" : "2.0" }, "extensionsUsed" : [ "KHR_texture_transform" ], "extensionsRequired" : [ "KHR_texture_transform" ] }"#
  ).unwrap();
  assert!( required_extensions_check( &gltf ).is_err() );
}

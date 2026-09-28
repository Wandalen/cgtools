//! Verifies the glTF loader's required-extension gate
//! ( `renderer::webgl::loaders::gltf::required_extensions_check` ) -- the pure,
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
//! these tests exercise exactly the logic added to `gltf.rs`.

use renderer::webgl::loaders::gltf::{ document_validate, required_extensions_check };

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

#[ test ]
fn accepts_required_clearcoat_and_anisotropy()
{
  // `materials_create` reads both extensions' JSON by hand, so an asset that requires them
  // must pass this loader's own gate.
  let gltf = gltf::Gltf::from_slice_without_validation( MATERIAL_LAYER_EXTENSIONS_REQUIRED_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!( required_extensions_check( &gltf ).is_ok(), "clearcoat / anisotropy are implemented -- must not be rejected" );
}

#[ test ]
fn document_validate_ignores_upstream_required_extension_list()
{
  // `gltf::Gltf::from_slice` rejects this valid asset: gltf-json's own extensionsRequired rule
  // only knows the extensions it has typed support for. `load` therefore parses without that
  // validation and runs `document_validate`, which must accept the asset.
  assert!
  (
    gltf::Gltf::from_slice( MATERIAL_LAYER_EXTENSIONS_REQUIRED_FIXTURE.as_bytes() ).is_err(),
    "precondition: upstream validation rejects hand-read required extensions"
  );
  let gltf = gltf::Gltf::from_slice_without_validation( MATERIAL_LAYER_EXTENSIONS_REQUIRED_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!( document_validate( &gltf ).is_ok(), "only the upstream extensionsRequired rule may be skipped" );
}

const DANGLING_NODE_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "scenes": [ { "nodes": [ 3 ] } ]
}
"#;

#[ test ]
fn document_validate_still_reports_structural_errors()
{
  // Skipping the upstream extension rule must not skip the rest of gltf-json's validation:
  // a scene naming a node that doesn't exist is still an error.
  let gltf = gltf::Gltf::from_slice_without_validation( DANGLING_NODE_FIXTURE.as_bytes() )
  .expect( "fixture is well-formed JSON" );

  assert!( document_validate( &gltf ).is_err(), "an out-of-range node index must fail validation" );
}

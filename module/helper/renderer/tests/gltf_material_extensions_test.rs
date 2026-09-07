//! Verifies the glTF loader's OpenPBR Surface ingestion
//! ( `renderer::webgl::loaders::gltf::material_openpbr_params_read` ) — the
//! pure, off-GPU extraction of the scalar/color factor carriers that the
//! ratified `KHR_materials_*` extensions use to transport ASWF OpenPBR Surface
//! parameters ( see the ASWF spec at
//! https://academysoftwarefoundation.github.io/OpenPBR/ ). OpenPBR has no glTF
//! extension of its own; a real-time asset ships its lobes through the Khronos
//! extension set instead, which is what this loader reads. Zero
//! `WebGl2RenderingContext`/`gl::` calls anywhere in the function's body —
//! same off-GPU pattern as `gltf_extensions_required_test.rs`.
//!
//! Presence semantics exercised here: every parameter is `Some` only while its
//! extension is present, already defaulted to the extension's own schema
//! default when the key is omitted; `KHR_materials_volume.attenuationDistance`
//! is the sole exception ( omitted → `None`, the schema's `+inf` meaning a
//! non-absorbing medium ).
//!
//! Fixtures use `gltf::Gltf::from_slice_without_validation` for the same
//! reason as `gltf_extensions_required_test.rs` does: the `gltf` crate's own
//! `Document::validate()` rejects `extensionsRequired` entries outside its
//! compile-time `ENABLED_EXTENSIONS` list, which is unrelated to this loader's
//! read path ( it parses extension JSON by name, no Cargo feature needed ).

use renderer::webgl::loaders::gltf::material_openpbr_params_read;
use renderer::webgl::material::OpenPbrParams;

/// Parses a single-material glTF JSON fixture and runs the reader over it.
fn params_from_material_json( json : &str ) -> OpenPbrParams
{
  let gltf = gltf::Gltf::from_slice_without_validation( json.as_bytes() )
  .expect( "fixture must be structurally well-formed JSON" );
  let material = gltf.materials().next().expect( "fixture must define one material" );
  material_openpbr_params_read( &material )
}

#[ test ]
fn no_extensions_yields_all_none()
{
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "pbrMetallicRoughness": {} } ]
  }
  "#;

  let params = params_from_material_json( fixture );

  assert_eq!( params, OpenPbrParams::default(), "a material with no extensions must read as all-`None`" );
}

#[ test ]
fn unrelated_extensions_do_not_leak_into_openpbr_params()
{
  // The extensions this reader does NOT own — clearcoat/anisotropy are parsed
  // into PbrMaterial's own fields elsewhere — must leave OpenPbrParams empty,
  // not accidentally populate it.
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": {
      "KHR_materials_clearcoat": { "clearcoatFactor": 0.8 },
      "KHR_materials_anisotropy": { "anisotropyStrength": 0.5 }
    } } ]
  }
  "#;

  let params = params_from_material_json( fixture );

  assert_eq!( params, OpenPbrParams::default(), "reader must ignore extensions outside its mapping" );
}

#[ test ]
fn ior_reads_value_and_defaults_to_1_5()
{
  let explicit = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_ior": { "ior": 2.4 } } } ]
  }
  "#;
  assert_eq!( params_from_material_json( explicit ).ior, Some( 2.4 ) );

  let omitted = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_ior": {} } } ]
  }
  "#;
  assert_eq!( params_from_material_json( omitted ).ior, Some( 1.5 ) );
}

#[ test ]
fn sheen_reads_color_and_roughness()
{
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_sheen": {
      "sheenColorFactor": [ 0.6, 0.3, 0.1 ],
      "sheenRoughnessFactor": 0.4
    } } } ]
  }
  "#;

  let params = params_from_material_json( fixture );

  let color = params.sheen_color_factor.expect( "sheenColorFactor must be present" );
  assert!( ( color[ 0 ] - 0.6 ).abs() < 1e-6, "red channel" );
  assert!( ( color[ 1 ] - 0.3 ).abs() < 1e-6, "green channel" );
  assert!( ( color[ 2 ] - 0.1 ).abs() < 1e-6, "blue channel" );
  assert_eq!( params.sheen_roughness_factor, Some( 0.4 ) );
}

#[ test ]
fn sheen_present_but_omitted_keys_take_schema_defaults()
{
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_sheen": {} } } ]
  }
  "#;

  let params = params_from_material_json( fixture );

  assert_eq!( params.sheen_color_factor, Some( [ 0.0, 0.0, 0.0 ] ), "sheen disabled by default" );
  assert_eq!( params.sheen_roughness_factor, Some( 0.0 ) );
}

#[ test ]
fn transmission_reads_factor()
{
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_transmission": { "transmissionFactor": 0.75 } } } ]
  }
  "#;

  assert_eq!( params_from_material_json( fixture ).transmission_factor, Some( 0.75 ) );
}

#[ test ]
fn volume_reads_thickness_and_attenuation()
{
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_volume": {
      "thicknessFactor": 2.0,
      "attenuationDistance": 3.5,
      "attenuationColor": [ 0.9, 0.6, 0.3 ]
    } } } ]
  }
  "#;

  let params = params_from_material_json( fixture );

  assert_eq!( params.volume_thickness_factor, Some( 2.0 ) );
  assert_eq!( params.volume_attenuation_distance, Some( 3.5 ) );
  let color = params.volume_attenuation_color.expect( "attenuationColor must be present" );
  assert!( ( color[ 0 ] - 0.9 ).abs() < 1e-6 );
  assert!( ( color[ 1 ] - 0.6 ).abs() < 1e-6 );
  assert!( ( color[ 2 ] - 0.3 ).abs() < 1e-6 );
}

#[ test ]
fn volume_omitted_attenuation_distance_means_non_absorbing()
{
  // Omitted attenuationDistance is the schema's +inf ( transparent medium ),
  // represented as None; attenuationColor still defaults to white.
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_volume": {} } } ]
  }
  "#;

  let params = params_from_material_json( fixture );

  assert_eq!( params.volume_thickness_factor, Some( 0.0 ) );
  assert_eq!( params.volume_attenuation_distance, None, "+inf default has no finite carrier" );
  assert_eq!( params.volume_attenuation_color, Some( [ 1.0, 1.0, 1.0 ] ) );
}

#[ test ]
fn iridescence_reads_full_set_with_defaults()
{
  let explicit = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_iridescence": {
      "iridescenceFactor": 0.6,
      "iridescenceIor": 1.7,
      "iridescenceThicknessMinimum": 250.0,
      "iridescenceThicknessMaximum": 900.0
    } } } ]
  }
  "#;
  let params = params_from_material_json( explicit );
  assert_eq!( params.iridescence_factor, Some( 0.6 ) );
  assert_eq!( params.iridescence_ior, Some( 1.7 ) );
  assert_eq!( params.iridescence_thickness_minimum, Some( 250.0 ) );
  assert_eq!( params.iridescence_thickness_maximum, Some( 900.0 ) );

  let omitted = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_iridescence": {} } } ]
  }
  "#;
  let params = params_from_material_json( omitted );
  assert_eq!( params.iridescence_factor, Some( 0.0 ) );
  assert_eq!( params.iridescence_ior, Some( 1.3 ) );
  assert_eq!( params.iridescence_thickness_minimum, Some( 100.0 ) );
  assert_eq!( params.iridescence_thickness_maximum, Some( 400.0 ) );
}

#[ test ]
fn emissive_strength_reads_and_defaults_to_1()
{
  let explicit = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_emissive_strength": { "emissiveStrength": 12.0 } } } ]
  }
  "#;
  assert_eq!( params_from_material_json( explicit ).emissive_strength, Some( 12.0 ) );

  let omitted = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_emissive_strength": {} } } ]
  }
  "#;
  assert_eq!( params_from_material_json( omitted ).emissive_strength, Some( 1.0 ) );
}

#[ test ]
fn dispersion_reads_strength()
{
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_dispersion": { "dispersion": 0.5 } } } ]
  }
  "#;
  assert_eq!( params_from_material_json( fixture ).dispersion, Some( 0.5 ) );
}

#[ test ]
fn diffuse_transmission_reads_factor_and_color()
{
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_diffuse_transmission": {
      "diffuseTransmissionFactor": 0.25,
      "diffuseTransmissionColorFactor": [ 1.0, 0.8, 0.6 ]
    } } } ]
  }
  "#;

  let params = params_from_material_json( fixture );

  assert_eq!( params.diffuse_transmission_factor, Some( 0.25 ) );
  let color = params.diffuse_transmission_color_factor.expect( "diffuseTransmissionColorFactor must be present" );
  assert!( ( color[ 0 ] - 1.0 ).abs() < 1e-6 );
  assert!( ( color[ 1 ] - 0.8 ).abs() < 1e-6 );
  assert!( ( color[ 2 ] - 0.6 ).abs() < 1e-6 );

  // Present-but-empty reads schema defaults.
  let omitted = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": { "KHR_materials_diffuse_transmission": {} } } ]
  }
  "#;
  let params = params_from_material_json( omitted );
  assert_eq!( params.diffuse_transmission_factor, Some( 0.0 ) );
  assert_eq!( params.diffuse_transmission_color_factor, Some( [ 1.0, 1.0, 1.0 ] ) );
}

#[ test ]
fn combined_fixture_reads_every_extension_independently()
{
  let fixture = r#"
  {
    "asset": { "version": "2.0" },
    "materials": [ { "extensions": {
      "KHR_materials_ior": { "ior": 1.8 },
      "KHR_materials_sheen": { "sheenColorFactor": [ 0.5, 0.5, 0.5 ], "sheenRoughnessFactor": 0.3 },
      "KHR_materials_transmission": { "transmissionFactor": 1.0 },
      "KHR_materials_volume": { "thicknessFactor": 1.0, "attenuationColor": [ 0.8, 0.8, 0.8 ] },
      "KHR_materials_iridescence": { "iridescenceFactor": 1.0 },
      "KHR_materials_emissive_strength": { "emissiveStrength": 2.0 },
      "KHR_materials_dispersion": { "dispersion": 0.25 },
      "KHR_materials_diffuse_transmission": { "diffuseTransmissionFactor": 0.1 }
    } } ]
  }
  "#;

  let params = params_from_material_json( fixture );

  assert_eq!( params.ior, Some( 1.8 ) );
  assert_eq!( params.sheen_color_factor, Some( [ 0.5, 0.5, 0.5 ] ) );
  assert_eq!( params.sheen_roughness_factor, Some( 0.3 ) );
  assert_eq!( params.transmission_factor, Some( 1.0 ) );
  assert_eq!( params.volume_thickness_factor, Some( 1.0 ) );
  assert_eq!( params.volume_attenuation_distance, None, "omitted among otherwise-present extensions" );
  assert_eq!( params.volume_attenuation_color, Some( [ 0.8, 0.8, 0.8 ] ) );
  assert_eq!( params.iridescence_factor, Some( 1.0 ) );
  assert_eq!( params.iridescence_ior, Some( 1.3 ), "per-extension default survives sibling extensions" );
  assert_eq!( params.emissive_strength, Some( 2.0 ) );
  assert_eq!( params.dispersion, Some( 0.25 ) );
  assert_eq!( params.diffuse_transmission_factor, Some( 0.1 ) );
  assert_eq!( params.diffuse_transmission_color_factor, Some( [ 1.0, 1.0, 1.0 ] ) );
}

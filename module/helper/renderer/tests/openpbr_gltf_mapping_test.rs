//! Verifies the glTF-carrier → `OpenPbrSurface` bridge
//! ( `renderer::webgl::material::openpbr_from_gltf` ) — adoption-plan N1
//! remainder. A glTF asset expresses only a subset of OpenPBR parameters; this
//! mapping starts from the spec defaults and overrides exactly what the core
//! `pbrMetallicRoughness` + `KHR_materials_*` scalar factors can carry, so the
//! result is deterministic and natively testable without a GPU.

use renderer::webgl::material::{ OpenPbrFromGltf, OpenPbrSurface, openpbr_from_gltf };

/// A dielectric: red diffuse base, zero metalness, mid roughness.
#[ test ]
fn dielectric_base_maps_to_openpbr_defaults_plus_factors()
{
  let mut input = OpenPbrFromGltf::default();
  input.base_color = [ 1.0, 0.0, 0.0, 1.0 ];
  input.base_metalness = 0.0;
  input.specular_roughness = 0.4;

  let surface = openpbr_from_gltf( &input );

  let mut expected = OpenPbrSurface::spec_default();
  expected.base_color = [ 1.0, 0.0, 0.0 ];
  expected.base_metalness = 0.0;
  expected.specular_roughness = 0.4;
  expected.specular_ior = 1.5;
  expected.geometry_opacity = 1.0;

  assert_eq!( surface, expected );
}

/// Metal: the base color becomes the metal tint, metalness drives the mix.
#[ test ]
fn metal_base_maps_color_to_metal_tint()
{
  let mut input = OpenPbrFromGltf::default();
  input.base_color = [ 0.929, 0.788, 0.374, 1.0 ];
  input.base_metalness = 1.0;
  input.specular_roughness = 0.02;
  input.specular_color = Some( [ 0.987, 1.013, 0.997 ] );
  input.specular_weight = Some( 0.8 );

  let surface = openpbr_from_gltf( &input );

  let mut expected = OpenPbrSurface::spec_default();
  expected.base_color = [ 0.929, 0.788, 0.374 ];
  expected.base_metalness = 1.0;
  expected.specular_roughness = 0.02;
  expected.specular_color = [ 0.987, 1.013, 0.997 ];
  expected.specular_weight = 0.8;

  assert_eq!( surface, expected );
}

/// Base color alpha feeds `geometry_opacity`.
#[ test ]
fn base_alpha_maps_to_geometry_opacity()
{
  let mut input = OpenPbrFromGltf::default();
  input.base_color = [ 0.5, 0.5, 0.5, 0.4 ];
  input.base_metalness = 0.0;

  let surface = openpbr_from_gltf( &input );

  assert_eq!( surface.geometry_opacity, 0.4 );
  assert_eq!( surface.base_color, [ 0.5, 0.5, 0.5 ] );
}

/// Clearcoat presence maps to the coat lobes with the glTF fixed IOR 1.5.
#[ test ]
fn clearcoat_maps_to_coat_with_gltf_ior()
{
  let mut input = OpenPbrFromGltf::default();
  input.base_metalness = 0.0;
  input.coat_weight = Some( 0.8 );
  input.coat_roughness = Some( 0.1 );

  let surface = openpbr_from_gltf( &input );

  assert_eq!( surface.base_metalness, 0.0 );
  assert_eq!( surface.coat_weight, 0.8 );
  assert_eq!( surface.coat_roughness, 0.1 );
  assert_eq!( surface.coat_ior, 1.5, "glTF clearcoat is fixed-IOR 1.5, not the spec's 1.6" );
}

/// KHR sheen presence maps to the fuzz lobes; sheen has no separate weight so
/// presence ⇒ weight 1.0 ( black default colour still disables the layer ).
#[ test ]
fn sheen_maps_to_fuzz_with_implicit_weight()
{
  let mut input = OpenPbrFromGltf::default();
  input.base_metalness = 0.0;
  input.fuzz_present = true;
  input.fuzz_color = Some( [ 0.315, 0.237, 0.465 ] );
  input.fuzz_roughness = Some( 0.5 );

  let surface = openpbr_from_gltf( &input );

  assert_eq!( surface.fuzz_weight, 1.0 );
  assert_eq!( surface.fuzz_color, [ 0.315, 0.237, 0.465 ] );
  assert_eq!( surface.fuzz_roughness, 0.5 );
}

/// Anisotropy and transmission factors pass straight through ( clamped ).
#[ test ]
fn anisotropy_and_transmission_map_through()
{
  let mut input = OpenPbrFromGltf::default();
  input.base_metalness = 0.0;
  input.specular_roughness_anisotropy = Some( 0.9 );
  input.specular_ior = 1.52;
  input.transmission_weight = Some( 0.6 );

  let surface = openpbr_from_gltf( &input );

  assert_eq!( surface.specular_roughness_anisotropy, 0.9 );
  assert_eq!( surface.specular_ior, 1.52 );
  assert_eq!( surface.transmission_weight, 0.6 );
}

/// Lobes glTF cannot express keep their spec defaults — nothing is guessed.
#[ test ]
fn unrepresentable_lobes_keep_spec_defaults()
{
  let mut input = OpenPbrFromGltf::default();
  input.base_metalness = 0.0;

  let surface = openpbr_from_gltf( &input );
  let spec_default = OpenPbrSurface::spec_default();

  // Subsurface / volume / dispersion / iridescence / emission luminance are
  // outside the glTF scalar carrier set — they stay exactly at the spec
  // default rather than being invented from factors glTF does not have.
  assert_eq!( surface.base_metalness, 0.0 );
  assert_eq!( surface.subsurface_weight, spec_default.subsurface_weight );
  assert_eq!( surface.transmission_scatter, spec_default.transmission_scatter );
  assert_eq!( surface.thin_film_weight, spec_default.thin_film_weight );
  assert_eq!( surface.emission_luminance, spec_default.emission_luminance );
}

/// Out-of-range factors are clamped to OpenPBR ranges.
#[ test ]
fn out_of_range_factors_are_clamped()
{
  let mut input = OpenPbrFromGltf::default();
  input.base_color = [ 0.1, 0.2, 0.3, 2.0 ];
  input.base_metalness = 1.5;
  input.specular_roughness = -0.2;
  input.specular_ior = 0.5;

  let surface = openpbr_from_gltf( &input );

  assert_eq!( surface.base_metalness, 1.0 );
  assert_eq!( surface.specular_roughness, 0.0 );
  assert_eq!( surface.specular_ior, 1.0 );
  assert_eq!( surface.geometry_opacity, 1.0 );
}

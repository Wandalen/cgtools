//! Verifies the runtime bridge `OpenPbrSurface → PbrMaterial` factors
//! ( `renderer::webgl::material::openpbr_to_runtime` ) — the mapping that lets a
//! surface loaded from `.mtlx`/`.usda` render through the existing `USE_OPENPBR`
//! material path. Pure data reduction, natively testable; the consuming
//! `PbrMaterial::openpbr_surface_apply` method applies the same factors.

use renderer::webgl::material::{ OpenPbrParams, OpenPbrRuntime, OpenPbrSurface, openpbr_params_from_surface, openpbr_to_runtime };

/// A default surface reduces to the runtime defaults ( no overrides ).
#[ test ]
fn spec_default_reduces_to_runtime_defaults()
{
  let runtime = openpbr_to_runtime( &OpenPbrSurface::spec_default() );

  assert_eq!
  (
    runtime,
    OpenPbrRuntime
    {
      base_color_rgba : [ 0.8, 0.8, 0.8, 1.0 ],
      base_metalness : 0.0,
      specular_roughness : 0.3,
      specular_ior : None,       // IOR 1.5 == legacy glTF F0, no override
      specular_weight : None,    // 1.0 == default
      specular_color : None,     // white == default
      coat_weight : None,
      coat_roughness : None,
      fuzz_color : None,         // fuzz_weight 0 → disabled
      fuzz_roughness : None,
    }
  );
}

/// Metal: base color/opacity drive RGBA; metalness/roughness pass through.
#[ test ]
fn metal_surface_reduces_to_runtime_factors()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.base_color = [ 0.929, 0.788, 0.374 ];
  surface.base_metalness = 1.0;
  surface.specular_roughness = 0.02;
  surface.specular_ior = 1.8;    // custom IOR → override
  surface.specular_weight = 0.8; // → override
  surface.specular_color = [ 0.9, 1.1, 1.0 ];
  surface.geometry_opacity = 0.5;

  let runtime = openpbr_to_runtime( &surface );

  assert_eq!( runtime.base_color_rgba, [ 0.929, 0.788, 0.374, 0.5 ] );
  assert_eq!( runtime.base_metalness, 1.0 );
  assert_eq!( runtime.specular_roughness, 0.02 );
  assert_eq!( runtime.specular_ior, Some( 1.8 ) );
  assert_eq!( runtime.specular_weight, Some( 0.8 ) );
  assert_eq!( runtime.specular_color, Some( [ 0.9, 1.1, 1.0 ] ) );
}

/// Coat and fuzz only appear in the runtime factors when enabled.
#[ test ]
fn coat_and_fuzz_are_conditional()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.coat_weight = 0.9;
  surface.coat_roughness = 0.1;
  surface.fuzz_weight = 0.5;
  surface.fuzz_color = [ 0.315, 0.237, 0.465 ];
  surface.fuzz_roughness = 0.5;

  let runtime = openpbr_to_runtime( &surface );

  assert_eq!( runtime.coat_weight, Some( 0.9 ) );
  assert_eq!( runtime.coat_roughness, Some( 0.1 ) );
  // `fuzz_weight` 0.5 rides in the colour: the KHR sheen carrier has no weight
  // of its own, so carrying it separately would make the weight a bare on/off.
  assert_eq!( runtime.fuzz_color, Some( [ 0.315 * 0.5, 0.237 * 0.5, 0.465 * 0.5 ] ) );
  assert_eq!( runtime.fuzz_roughness, Some( 0.5 ) );
}

/// A surface with a disabled coat/fuzz keeps those carriers `None`.
#[ test ]
fn disabled_layers_yield_no_carriers()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.coat_weight = 0.0;
  surface.coat_roughness = 0.5; // roughness without weight is irrelevant
  surface.fuzz_weight = 0.0;
  surface.fuzz_color = [ 0.1, 0.2, 0.3 ];

  let runtime = openpbr_to_runtime( &surface );

  assert_eq!( runtime.coat_weight, None );
  assert_eq!( runtime.coat_roughness, None );
  assert_eq!( runtime.fuzz_color, None );
  assert_eq!( runtime.fuzz_roughness, None );
}

/// Lobes without a runtime sink are dropped from the reduction entirely.
#[ test ]
fn unsupported_lobes_are_not_reduced()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.subsurface_weight = 0.8;
  surface.transmission_weight = 1.0;
  surface.thin_film_weight = 1.0;

  let runtime = openpbr_to_runtime( &surface );

  // Only the expressible fields exist on the runtime type — nothing to assert
  // beyond the fact that it reduced to the ( base-only ) factors.
  assert_eq!( runtime.base_color_rgba[ 3 ], 1.0 );
  assert_eq!( runtime.fuzz_color, None );
}

/// `openpbr_params_from_surface` carries thin-film iridescence + fuzz + IOR
/// into the glTF-shaped shader carriers.
#[ test ]
fn params_from_surface_maps_thin_film_fuzz_and_ior()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.specular_ior = 1.52;
  surface.fuzz_weight = 0.5;
  surface.fuzz_color = [ 0.3, 0.2, 0.4 ];
  surface.fuzz_roughness = 0.6;
  surface.thin_film_weight = 0.8;
  surface.thin_film_thickness = 0.45; // µm
  surface.thin_film_ior = 1.4;

  let params = openpbr_params_from_surface( &surface );

  assert_eq!( params.ior, Some( 1.52 ) );
  assert_eq!( params.sheen_color_factor, Some( [ 0.3 * 0.5, 0.2 * 0.5, 0.4 * 0.5 ] ) );
  assert_eq!( params.sheen_roughness_factor, Some( 0.6 ) );
  assert_eq!( params.iridescence_factor, Some( 0.8 ) );
  assert_eq!( params.iridescence_ior, Some( 1.4 ) );
  assert_eq!( params.iridescence_thickness_minimum, Some( 450.0 ) );
  assert_eq!( params.iridescence_thickness_maximum, Some( 450.0 ) );
}

/// Defaults ( no thin film / fuzz / non-default IOR ) leave the carriers empty,
/// so the material keeps the legacy path.
#[ test ]
fn params_from_surface_default_surface_yields_empty_params()
{
  let params = openpbr_params_from_surface( &OpenPbrSurface::spec_default() );

  assert_eq!( params, OpenPbrParams::default() );
}

/// Transmission ( §3.3 / §3.4 ): the weight rides `transmission_factor`, the
/// absorption length scale `transmission_depth` rides the glTF
/// `attenuationDistance` carrier, and a non-white `transmission_color` becomes
/// the attenuation-color carrier. Zero weight leaves all of them unset.
/// `openpbr_transmission_test.rs` covers the depth/thickness distinction and
/// the Beer's-law math those two carriers drive.
#[ test ]
fn params_from_surface_maps_transmission_carriers()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.transmission_weight = 1.0;
  surface.transmission_depth = 0.4;
  surface.transmission_color = [ 0.9, 0.95, 1.0 ];

  let params = openpbr_params_from_surface( &surface );
  assert_eq!( params.transmission_factor, Some( 1.0 ) );
  assert_eq!( params.volume_attenuation_distance, Some( 0.4 ) );
  assert_eq!( params.volume_attenuation_color, Some( [ 0.9, 0.95, 1.0 ] ) );

  // white tint is the default -> no carrier needed
  surface.transmission_color = [ 1.0, 1.0, 1.0 ];
  let params = openpbr_params_from_surface( &surface );
  assert_eq!( params.volume_attenuation_color, None );

  // zero weight -> nothing mapped, stays fully diffuse
  let mut opaque = OpenPbrSurface::spec_default();
  opaque.transmission_weight = 0.0;
  opaque.transmission_depth = 2.0;
  let params = openpbr_params_from_surface( &opaque );
  assert_eq!( params.transmission_factor, None );
  assert_eq!( params.volume_attenuation_distance, None );
}

/// `fuzz_weight` has to *scale* the layer, not merely gate it. OpenPBR composes
/// the fuzz as `layer( base, fuzz, fuzz_weight )` while `KHR_materials_sheen`
/// has no weight of its own, so the weight rides in `sheenColorFactor` — without
/// that the slider is a bare on/off and every value above zero looks identical.
#[ test ]
fn fuzz_weight_scales_the_layer_rather_than_gating_it()
{
  let colored = | weight : f32 |
  {
    let mut surface = OpenPbrSurface::spec_default();
    surface.fuzz_weight = weight;
    surface.fuzz_color = [ 0.8, 0.6, 0.4 ];
    openpbr_params_from_surface( &surface ).sheen_color_factor
  };

  assert_eq!( colored( 1.0 ), Some( [ 0.8, 0.6, 0.4 ] ), "full weight passes the colour through" );
  assert_eq!( colored( 0.5 ), Some( [ 0.4, 0.3, 0.2 ] ), "half weight halves the layer" );
  assert_eq!( colored( 0.25 ), Some( [ 0.2, 0.15, 0.1 ] ), "a quarter weight quarters it" );
  assert_eq!( colored( 0.0 ), None, "no weight disables the layer entirely" );
}

/// A weight outside `[ 0, 1 ]` must not amplify the layer past its own colour —
/// `sheenColorFactor` is a reflectivity and the albedo scaling that pairs with it
/// assumes the same bound.
#[ test ]
fn fuzz_weight_is_clamped()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.fuzz_weight = 4.0;
  surface.fuzz_color = [ 0.8, 0.6, 0.4 ];
  assert_eq!( openpbr_params_from_surface( &surface ).sheen_color_factor, Some( [ 0.8, 0.6, 0.4 ] ) );
}

/// The glTF direction sets `fuzz_weight = 1` on the presence of
/// `KHR_materials_sheen`, so folding the weight into the colour keeps the round
/// trip exact rather than darkening a material each time it crosses the bridge.
#[ test ]
fn gltf_sheen_round_trips_through_the_weight()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.fuzz_weight = 1.0;
  surface.fuzz_color = [ 0.315, 0.237, 0.465 ];

  assert_eq!
  (
    openpbr_params_from_surface( &surface ).sheen_color_factor,
    Some( [ 0.315, 0.237, 0.465 ] )
  );
}

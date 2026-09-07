//! Verifies the MaterialX (`.mtlx`) lane of OpenPBR Surface ingestion
//! ( `renderer::webgl::loaders::openpbr_mtlx::openpbr_surfaces_from_mtlx` ) —
//! the pure, off-GPU reader that maps `open_pbr_surface` nodes onto the
//! canonical [`OpenPbrSurface`] parameter model. Fixtures below are verbatim
//! copies of the ASWF OpenPBR example library
//! ( `AcademySoftwareFoundation/OpenPBR`, `examples/open_pbr_*.mtlx` ).
//!
//! Semantics exercised: parameters absent from the document keep their spec
//! defaults ( parsing starts from `OpenPbrSurface::spec_default` ); multiple
//! surfaces per file are returned in document order; malformed XML and
//! unparsable scalar inputs are hard errors, while non-scalar ( texture /
//! geometry-map ) inputs are ignored by design.

use renderer::webgl::loaders::openpbr_mtlx::{ MtlxError, openpbr_surfaces_from_mtlx };
use renderer::webgl::material::OpenPbrSurface;

/// Verbatim `examples/open_pbr_default.mtlx`.
const DEFAULT_MTLX : &str = r#"<?xml version="1.0"?>
<materialx version="1.39" colorspace="acescg">
  <surfacematerial name="Default" type="material">
    <input name="surfaceshader" type="surfaceshader" nodename="open_pbr_surface_surfaceshader" />
    <input name="displacementshader" type="displacementshader" value="" />
  </surfacematerial>
  <open_pbr_surface name="open_pbr_surface_surfaceshader" type="surfaceshader">
    <input name="base_weight" type="float" value="1.0" />
    <input name="base_color" type="color3" value="0.8, 0.8, 0.8" />
    <input name="base_diffuse_roughness" type="float" value="0.0" />
    <input name="base_metalness" type="float" value="0.0" />
    <input name="specular_weight" type="float" value="1.0" />
    <input name="specular_color" type="color3" value="1, 1, 1" />
    <input name="specular_roughness" type="float" value="0.3" />
    <input name="specular_ior" type="float" value="1.5" />
    <input name="specular_roughness_anisotropy" type="float" value="0.0" />
    <input name="transmission_weight" type="float" value="0.0" />
    <input name="transmission_color" type="color3" value="1, 1, 1" />
    <input name="transmission_depth" type="float" value="0.0" />
    <input name="transmission_scatter" type="color3" value="0, 0, 0" />
    <input name="transmission_scatter_anisotropy" type="float" value="0.0" />
    <input name="transmission_dispersion_scale" type="float" value="0.0" />
    <input name="transmission_dispersion_abbe_number" type="float" value="20.0" />
    <input name="subsurface_weight" type="float" value="0" />
    <input name="subsurface_color" type="color3" value="0.8, 0.8, 0.8" />
    <input name="subsurface_radius" type="float" value="1.0" />
    <input name="subsurface_radius_scale" type="color3" value="1.0, 0.5, 0.25" />
    <input name="subsurface_scatter_anisotropy" type="float" value="0.0" />
    <input name="fuzz_weight" type="float" value="0.0" />
    <input name="fuzz_color" type="color3" value="1, 1, 1" />
    <input name="fuzz_roughness" type="float" value="0.5" />
    <input name="coat_weight" type="float" value="0.0" />
    <input name="coat_color" type="color3" value="1, 1, 1" />
    <input name="coat_roughness" type="float" value="0.0" />
    <input name="coat_roughness_anisotropy" type="float" value="0.0" />
    <input name="coat_ior" type="float" value="1.6" />
    <input name="coat_darkening" type="float" value="1.0" />
    <input name="thin_film_weight" type="float" value="0" />
    <input name="thin_film_thickness" type="float" value="0.5" />
    <input name="thin_film_ior" type="float" value="1.4" />
    <input name="emission_luminance" type="float" value="0.0" />
    <input name="emission_color" type="color3" value="1, 1, 1" />
    <input name="geometry_opacity" type="float" value="1" />
    <input name="geometry_thin_walled" type="boolean" value="false" />
  </open_pbr_surface>
</materialx>"#;

/// Verbatim `examples/open_pbr_gold.mtlx`.
const GOLD_MTLX : &str = r#"<?xml version="1.0"?>
<materialx version="1.39" colorspace="acescg">
  <surfacematerial name="Gold" type="material">
    <input name="surfaceshader" type="surfaceshader" nodename="open_pbr_surface_surfaceshader" />
  </surfacematerial>
  <open_pbr_surface name="open_pbr_surface_surfaceshader" type="surfaceshader">
    <input name="base_color" type="color3" value="0.929, 0.788, 0.374" />
    <input name="base_metalness" type="float" value="1.0" />
    <input name="specular_color" type="color3" value="0.987, 1.013, 0.997" />
    <input name="specular_roughness" type="float" value="0.02" />
  </open_pbr_surface>
</materialx>"#;

/// Verbatim `examples/open_pbr_glass.mtlx`.
const GLASS_MTLX : &str = r#"<?xml version="1.0"?>
<materialx version="1.39" colorspace="acescg">
  <surfacematerial name="Glass" type="material">
    <input name="surfaceshader" type="surfaceshader" nodename="open_pbr_surface_surfaceshader" />
  </surfacematerial>
  <open_pbr_surface name="open_pbr_surface_surfaceshader" type="surfaceshader">
    <input name="specular_roughness" type="float" value="0.0" />
    <input name="specular_ior" type="float" value="1.52" />
    <input name="transmission_weight" type="float" value="1.0" />
    <input name="transmission_dispersion_scale" type="float" value="1.0" />
    <input name="transmission_dispersion_abbe_number" type="float" value="64" />
  </open_pbr_surface>
</materialx>"#;

/// Verbatim `examples/open_pbr_velvet.mtlx`.
const VELVET_MTLX : &str = r#"<?xml version="1.0"?>
<materialx version="1.39" colorspace="acescg">
  <surfacematerial name="Velvet" type="material">
    <input name="surfaceshader" type="surfaceshader" nodename="open_pbr_surface_surfaceshader" />
  </surfacematerial>
  <open_pbr_surface name="open_pbr_surface_surfaceshader" type="surfaceshader">
    <input name="base_color" type="color3" value="0.062, 0.01, 0.269" />
    <input name="base_diffuse_roughness" type="float" value="1.0" />
    <input name="specular_roughness" type="float" value="1.0" />
    <input name="fuzz_weight" type="float" value="0.5" />
    <input name="fuzz_color" type="color3" value="0.315, 0.237, 0.465" />
    <input name="fuzz_roughness" type="float" value="0.5" />
  </open_pbr_surface>
</materialx>"#;

#[ test ]
fn aswf_default_surface_equals_spec_default()
{
  let surfaces = openpbr_surfaces_from_mtlx( DEFAULT_MTLX ).expect( "default example must parse" );

  assert_eq!( surfaces.len(), 1, "one surface per file" );
  assert_eq!
  (
    surfaces[ 0 ],
    OpenPbrSurface::spec_default(),
    "the ASWF default .mtlx explicitly lists every spec default"
  );
}

#[ test ]
fn gold_surface_sets_metal_params_keeps_defaults()
{
  let surfaces = openpbr_surfaces_from_mtlx( GOLD_MTLX ).expect( "gold example must parse" );

  assert_eq!( surfaces.len(), 1 );
  let surface = &surfaces[ 0 ];

  let mut expected = OpenPbrSurface::spec_default();
  expected.base_color = [ 0.929, 0.788, 0.374 ];
  expected.base_metalness = 1.0;
  expected.specular_color = [ 0.987, 1.013, 0.997 ];
  expected.specular_roughness = 0.02;

  assert_eq!( surface, &expected, "absent inputs keep spec defaults" );
}

#[ test ]
fn glass_surface_sets_transmission_params()
{
  let surfaces = openpbr_surfaces_from_mtlx( GLASS_MTLX ).expect( "glass example must parse" );

  let mut expected = OpenPbrSurface::spec_default();
  expected.specular_roughness = 0.0;
  expected.specular_ior = 1.52;
  expected.transmission_weight = 1.0;
  expected.transmission_dispersion_scale = 1.0;
  expected.transmission_dispersion_abbe_number = 64.0;

  assert_eq!( surfaces.len(), 1 );
  assert_eq!( &surfaces[ 0 ], &expected );
}

#[ test ]
fn velvet_surface_sets_fuzz_params()
{
  let surfaces = openpbr_surfaces_from_mtlx( VELVET_MTLX ).expect( "velvet example must parse" );

  let mut expected = OpenPbrSurface::spec_default();
  expected.base_color = [ 0.062, 0.01, 0.269 ];
  expected.base_diffuse_roughness = 1.0;
  expected.specular_roughness = 1.0;
  expected.fuzz_weight = 0.5;
  expected.fuzz_color = [ 0.315, 0.237, 0.465 ];
  expected.fuzz_roughness = 0.5;

  assert_eq!( surfaces.len(), 1 );
  assert_eq!( &surfaces[ 0 ], &expected );
}

#[ test ]
fn multiple_surfaces_return_in_document_order()
{
  let fixture = r#"<materialx version="1.39" colorspace="acescg">
  <open_pbr_surface name="a" type="surfaceshader">
    <input name="base_metalness" type="float" value="0.5" />
  </open_pbr_surface>
  <open_pbr_surface name="b" type="surfaceshader">
    <input name="base_metalness" type="float" value="1.0" />
  </open_pbr_surface>
</materialx>"#;

  let surfaces = openpbr_surfaces_from_mtlx( fixture ).expect( "fixture must parse" );

  assert_eq!( surfaces.len(), 2 );
  assert_eq!( surfaces[ 0 ].base_metalness, 0.5 );
  assert_eq!( surfaces[ 1 ].base_metalness, 1.0 );
}

#[ test ]
fn non_surface_and_non_scalar_elements_are_ignored()
{
  // `surfacematerial` siblings and texture/geometry inputs ( `filename`,
  // `vector3` ) must not error nor alter the surface.
  let fixture = r#"<materialx version="1.39">
  <surfacematerial name="M" type="material">
    <input name="surfaceshader" type="surfaceshader" nodename="s" />
  </surfacematerial>
  <open_pbr_surface name="s" type="surfaceshader">
    <input name="geometry_normal" type="filename" value="n.png" />
    <input name="geometry_coat_normal" type="vector3" value="0, 0, 1" />
    <input name="base_color" type="color3" value="0.5, 0.25, 0.125" />
  </open_pbr_surface>
</materialx>"#;

  let surfaces = openpbr_surfaces_from_mtlx( fixture ).expect( "texture inputs are deferred, not errors" );

  let mut expected = OpenPbrSurface::spec_default();
  expected.base_color = [ 0.5, 0.25, 0.125 ];

  assert_eq!( surfaces.len(), 1 );
  assert_eq!( &surfaces[ 0 ], &expected );
}

#[ test ]
fn malformed_xml_is_a_hard_error()
{
  let result = openpbr_surfaces_from_mtlx( "<materialx><open_pbr_surface></materialx>" );

  assert!
  (
    matches!( result, Err( MtlxError::Xml( _ ) ) ),
    "mismatched tags must surface as an XML error, not a partial surface"
  );
}

#[ test ]
fn unparsable_scalar_input_is_a_hard_error()
{
  let fixture = r#"<materialx version="1.39">
  <open_pbr_surface name="s" type="surfaceshader">
    <input name="base_metalness" type="float" value="not-a-number" />
  </open_pbr_surface>
</materialx>"#;

  let result = openpbr_surfaces_from_mtlx( fixture );

  match result
  {
    Err( MtlxError::Input { name, type_name, value } ) =>
    {
      assert_eq!( name, "base_metalness" );
      assert_eq!( type_name, "float" );
      assert_eq!( value, "not-a-number" );
    }
    other => panic!( "expected MtlxError::Input, got {other:?}" ),
  }
}

#[ test ]
fn unknown_scalar_parameter_is_a_hard_error_not_silent()
{
  // A scalar-typed input this model does not carry ( wrong name ) must not be
  // silently dropped — that would mis-apply a real OpenPBR file.
  let fixture = r#"<materialx version="1.39">
  <open_pbr_surface name="s" type="surfaceshader">
    <input name="not_a_param" type="float" value="1.0" />
  </open_pbr_surface>
</materialx>"#;

  assert!( openpbr_surfaces_from_mtlx( fixture ).is_err() );
}

#[ test ]
fn connected_or_unset_inputs_are_skipped_literal_scalars_applied()
{
  // Mirrors real content ( e.g. OpenPBRShaderPlayground `iceCube.mtlx` ):
  // scalar inputs wired to upstream texture nodes carry `nodename` and no
  // `value`, and must not error — only literal scalar values are extracted,
  // everything else stays at the spec default until the texture step.
  let fixture = r#"<materialx version="1.39" colorspace="lin_rec709">
  <image name="Roughness" type="float">
    <input name="file" type="filename" colorspace="Raw" value="../textures/iceCube_rougness.tif" />
  </image>
  <open_pbr_surface name="s" type="surfaceshader">
    <input name="specular_roughness" type="float" nodename="Roughness" />
    <input name="specular_roughness_anisotropy" type="float" value="0.5" />
    <input name="geometry_normal" type="vector3" nodename="normalmap1" />
    <input name="base_weight" type="float" value="" />
  </open_pbr_surface>
</materialx>"#;

  let surfaces = openpbr_surfaces_from_mtlx( fixture ).expect( "connected inputs parse without error" );

  assert_eq!( surfaces.len(), 1 );
  // Connected roughness stays at the spec default; the literal scalar applied.
  assert_eq!( surfaces[ 0 ].specular_roughness, 0.3 );
  assert_eq!( surfaces[ 0 ].specular_roughness_anisotropy, 0.5 );
  assert_eq!( surfaces[ 0 ].base_weight, 1.0 );
}

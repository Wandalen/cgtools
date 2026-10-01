//! Reading `KHR_materials_clearcoat` / `KHR_materials_anisotropy` extension JSON
//! ( `renderer::webgl::loaders::gltf_extensions::internal::{ clearcoat_parse, anisotropy_parse }`,
//! reachable under the `test_internals` feature ).
//!
//! The `gltf` crate has no typed accessor for either extension, so the loader reads their JSON
//! by hand; these pure, off-GPU checks pin the defaults, `texCoord`, the coat normal `scale` and
//! the rotation, which no other test would notice going wrong.

#![ cfg( feature = "test_internals" ) ]
#![ expect( clippy::float_cmp, reason = "parsed factors pass through unchanged from the JSON literals; exact comparison is the point" ) ]

use gltf::json::Value;
use renderer::webgl::loaders::gltf_extensions::internal::
{
  anisotropy_parse,
  clearcoat_parse,
  AnisotropyParams,
  ClearcoatParams,
  ExtensionTextureRef,
};

fn json( s : &str ) -> Value
{
  gltf::json::deserialize::from_str( s ).expect( "fixture is valid JSON" )
}

#[ test ]
fn clearcoat_empty_object_uses_extension_defaults()
{
  let cc = clearcoat_parse( &json( "{}" ) );
  assert_eq!
  (
    cc,
    ClearcoatParams
    {
      factor : 0.0,
      roughness_factor : 0.0,
      texture : None,
      roughness_texture : None,
      normal_texture : None,
      normal_scale : 1.0,
    }
  );
}

#[ test ]
fn clearcoat_reads_factors_textures_tex_coord_and_normal_scale()
{
  let cc = clearcoat_parse( &json( r#"
  {
    "clearcoatFactor" : 0.75,
    "clearcoatRoughnessFactor" : 0.25,
    "clearcoatTexture" : { "index" : 2 },
    "clearcoatRoughnessTexture" : { "index" : 3, "texCoord" : 1 },
    "clearcoatNormalTexture" : { "index" : 4, "texCoord" : 2, "scale" : 0.5 }
  }"# ) );

  assert_eq!( cc.factor, 0.75 );
  assert_eq!( cc.roughness_factor, 0.25 );
  assert_eq!( cc.texture, Some( ExtensionTextureRef { index : 2, tex_coord : 0 } ), "texCoord defaults to 0" );
  assert_eq!( cc.roughness_texture, Some( ExtensionTextureRef { index : 3, tex_coord : 1 } ) );
  assert_eq!( cc.normal_texture, Some( ExtensionTextureRef { index : 4, tex_coord : 2 } ) );
  assert_eq!( cc.normal_scale, 0.5 );
}

#[ test ]
fn clearcoat_normal_texture_without_scale_keeps_unit_scale()
{
  let cc = clearcoat_parse( &json( r#"{ "clearcoatNormalTexture" : { "index" : 0 } }"# ) );
  assert_eq!( cc.normal_scale, 1.0 );
}

#[ test ]
fn texture_without_integer_index_is_ignored()
{
  // A textureInfo without a usable index can't name a texture; it must not become index 0.
  let cc = clearcoat_parse( &json( r#"{ "clearcoatTexture" : { "texCoord" : 1 }, "clearcoatRoughnessTexture" : { "index" : -1 } }"# ) );
  assert_eq!( cc.texture, None );
  assert_eq!( cc.roughness_texture, None );
}

#[ test ]
fn tex_coord_keeps_any_set_and_falls_back_to_zero_when_malformed()
{
  // A UV set the shader doesn't declare is bounded where its define is written, for core and
  // extension textures alike; parsing keeps it. A negative or fractional texCoord isn't a UV
  // set at all and reads as the default, 0.
  let cc = clearcoat_parse( &json( r#"
  {
    "clearcoatTexture" : { "index" : 0, "texCoord" : 5 },
    "clearcoatRoughnessTexture" : { "index" : 1, "texCoord" : -1 },
    "clearcoatNormalTexture" : { "index" : 2, "texCoord" : 1.5 }
  }"# ) );

  assert_eq!( cc.texture, Some( ExtensionTextureRef { index : 0, tex_coord : 5 } ) );
  assert_eq!( cc.roughness_texture, Some( ExtensionTextureRef { index : 1, tex_coord : 0 } ) );
  assert_eq!( cc.normal_texture, Some( ExtensionTextureRef { index : 2, tex_coord : 0 } ) );
}

#[ test ]
fn anisotropy_empty_object_uses_extension_defaults()
{
  assert_eq!( anisotropy_parse( &json( "{}" ) ), AnisotropyParams { strength : 0.0, rotation : 0.0, texture : None } );
}

#[ test ]
fn anisotropy_reads_strength_rotation_and_texture()
{
  let an = anisotropy_parse( &json( r#"
  {
    "anisotropyStrength" : 0.6,
    "anisotropyRotation" : 1.5,
    "anisotropyTexture" : { "index" : 7, "texCoord" : 1 }
  }"# ) );

  assert_eq!( an, AnisotropyParams { strength : 0.6, rotation : 1.5, texture : Some( ExtensionTextureRef { index : 7, tex_coord : 1 } ) } );
}

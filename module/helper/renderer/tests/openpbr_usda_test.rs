//! Verifies the `.usda` → `.mtlx` asset-reference resolver
//! ( `renderer::webgl::loaders::openpbr_usda::usda_mtlx_references` ) — the
//! N3-lite USD lane. OpenPBR USD content ( e.g. the Digital Production Example
//! Library OpenPBRShaderPlayground corpus ) keeps its surface in external
//! `.mtlx` files referenced from `Material` prims; this resolver finds those
//! references so the N2 `.mtlx` reader can take over. Pure text parsing, no
//! GPU / no USD composition engine.

use renderer::webgl::loaders::openpbr_usda::{ UsdaMtlxReference, usda_mtlx_references };

/// Trimmed, faithful subset of `ShdrPlygrnd/ShdrPlygrnd_OpenPBR.usda`.
const PLAYGROUND_USDA : &str = r#"#usda 1.0
(
    defaultPrim = "World"
)

over "World"
{
    over "Looks"
    {
        def Material "iceCube" (
            prepend inherits = </__class_mtl__/iceCube>
            prepend references = @./iceCube.mtlx@</MaterialX/Materials/iceCube>
        )
        {
        }

        def Material "iceCubesInner" (
            prepend inherits = </__class_mtl__/iceCubesInner>
            prepend references = @./iceCubesInner.mtlx@</MaterialX/Materials/iceCubesInner>
        )
        {
        }
    }
}
"#;

#[ test ]
fn playground_usda_yields_material_to_mtlx_references()
{
  let refs = usda_mtlx_references( PLAYGROUND_USDA );

  assert_eq!
  (
    refs,
    vec!
    [
      UsdaMtlxReference
      {
        material : "iceCube".to_string(),
        mtlx_asset : "./iceCube.mtlx".to_string(),
        mtlx_target : Some( "/MaterialX/Materials/iceCube".to_string() ),
      },
      UsdaMtlxReference
      {
        material : "iceCubesInner".to_string(),
        mtlx_asset : "./iceCubesInner.mtlx".to_string(),
        mtlx_target : Some( "/MaterialX/Materials/iceCubesInner".to_string() ),
      },
    ]
  );
}

#[ test ]
fn reference_without_target_keeps_mtlx_target_none()
{
  let usda = r#"#usda 1.0
def Material "M" (
    prepend references = @./material.mtlx@
)
{
}
"#;

  let refs = usda_mtlx_references( usda );

  assert_eq!( refs.len(), 1 );
  assert_eq!( refs[ 0 ].material, "M" );
  assert_eq!( refs[ 0 ].mtlx_asset, "./material.mtlx" );
  assert_eq!( refs[ 0 ].mtlx_target, None );
}

#[ test ]
fn multiple_references_in_one_material_are_all_collected()
{
  let usda = r#"#usda 1.0
def Material "Double" (
    prepend references = @./base.mtlx@</MaterialX/Materials/base>,
    prepend references = @./detail.mtlx@</MaterialX/Materials/detail>
)
{
}
"#;

  let refs = usda_mtlx_references( usda );

  assert_eq!( refs.len(), 2 );
  assert_eq!( refs[ 0 ].mtlx_asset, "./base.mtlx" );
  assert_eq!( refs[ 1 ].mtlx_asset, "./detail.mtlx" );
  assert!( refs.iter().all( | r | r.material == "Double" ) );
}

#[ test ]
fn over_material_prims_are_recognised()
{
  let usda = r#"#usda 1.0
over "Looks"
{
    over Material "overridden" (
        prepend references = @./override.mtlx@</MaterialX/Materials/overridden>
    )
    {
    }
}
"#;

  let refs = usda_mtlx_references( usda );

  assert_eq!( refs.len(), 1 );
  assert_eq!( refs[ 0 ].material, "overridden" );
}

#[ test ]
fn non_material_defs_and_other_layers_are_ignored()
{
  let usda = r#"#usda 1.0
def Mesh "Chair" (
    prepend references = @./chairGeo.usd@</Chair>
)
{
}
def Material "Inline" (
    prepend apiSchemas = ["MaterialBindingAPI"]
)
{
    token outputs:surface.connect = </Inline/PBRShader.outputs:surface>
    def Shader "PBRShader" {
        float inputs:metallic = 1
    }
}
"#;

  // The Mesh's usd reference is not a Material; the inline material has no
  // external .mtlx asset, so nothing is reported.
  assert!( usda_mtlx_references( usda ).is_empty() );
}

#[ test ]
fn empty_document_yields_no_references()
{
  assert!( usda_mtlx_references( "#usda 1.0\n( defaultPrim = \"World\" )\n" ).is_empty() );
}

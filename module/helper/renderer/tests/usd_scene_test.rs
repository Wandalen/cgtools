//! Native tests for the USD scene ingestion lane ( OpenPBR adoption plan
//! §2.3 N3 first slice ) : the in-memory `ar::Resolver` + stage open, the
//! pure `usd_mesh_extract` conversion ( fan triangulation, primvar corner
//! resolution ), `usd_local_to_world` composition, and the pure scene pass
//! `usd_scene_analyze` ( hierarchy, shading-subtree skipping, material binding
//! + preview-surface mapping, visibility/purpose filtering ). Runs entirely
//! off-GPU and off-filesystem - fixtures are inline `.usda` text served
//! through [`UsdInMemoryResolver`], which is exactly how the browser lane will
//! feed HTTP-fetched bytes. Gated behind the crate's `native-formats` feature
//! ( see `required-features` in `Cargo.toml` ). The GL half
//! ( `usd_geometry_create` / `usd_scene_load` ) needs a browser and is not
//! covered here.
#![ allow( clippy::float_cmp, reason = "usda fixtures round-trip exactly-representable binary literals ( deterministic f64-parse -> f32-cast ); exact comparison is the point - eps tolerance would hide real regressions" ) ]

use renderer::webgl::loaders::usd::{ UsdAssetProvider, UsdError, UsdInMemoryResolver, UsdPrimData, usd_local_to_world, usd_mesh_extract, usd_scene_analyze, usd_stage_open };
use openusd_schemas::geom;

/// A unit quad in the XY plane at z=0 : CCW from +Z, two triangles after fan
/// triangulation, faceVarying UVs authored in corner order, vertex normals.
const QUAD_USDA : &str = r#"#usda 1.0
(
    defaultPrim = "World"
    upAxis = "Y"
)

def Xform "World"
{
    double3 xformOp:translate = ( 2, 0, 0 )
    uniform token[] xformOpOrder = [ "xformOp:translate" ]

    def Mesh "Quad"
    {
        uniform int[] faceVertexCounts = [ 4 ]
        uniform int[] faceVertexIndices = [ 0, 1, 2, 3 ]
        uniform normal3f[] normals = [ ( 0, 0, 1 ), ( 0, 0, 1 ), ( 0, 0, 1 ), ( 0, 0, 1 ) ] (
            interpolation = "vertex"
        )
        uniform point3f[] points = [ ( -1, -1, 0 ), ( 1, -1, 0 ), ( 1, 1, 0 ), ( -1, 1, 0 ) ]
        uniform float2[] primvars:st = [ ( 0, 0 ), ( 1, 0 ), ( 1, 1 ), ( 0, 1 ) ] (
            interpolation = "faceVarying"
        )
        uniform token subdivisionScheme = "none"
    }
}
"#;

fn stage_with( files : &[ ( &str, &str ) ] ) -> openusd::usd::Stage
{
  let mut resolver = UsdInMemoryResolver::new();
  for ( path, text ) in files
  {
    resolver.insert( *path, text.as_bytes().to_vec() );
  }
  usd_stage_open( "scene.usda", resolver ).expect( "stage opens from memory" )
}

/// Finds the single mesh prim on the stage.
fn only_mesh( stage : &openusd::usd::Stage ) -> geom::Mesh
{
  let mut paths = Vec::new();
  stage.traverse( openusd::usd::PrimPredicate::DEFAULT_PROXIES, | p | paths.push( p.clone() ) ).unwrap();
  paths.into_iter()
    .find_map( | p | geom::Mesh::get( stage, p ).ok().flatten() )
    .expect( "fixture defines one Mesh prim" )
}

#[ test ]
fn stage_opens_from_in_memory_usda()
{
  let stage = stage_with( &[ ( "scene.usda", QUAD_USDA ) ] );
  let mesh = only_mesh( &stage );
  assert_eq!( mesh.path().as_str(), "/World/Quad" );
}

#[ test ]
fn quad_triangulates_to_two_triangles()
{
  let stage = stage_with( &[ ( "scene.usda", QUAD_USDA ) ] );
  let mesh = only_mesh( &stage );
  let data = usd_mesh_extract( &mesh ).expect( "quad extracts" );

  // Fan triangulation of one 4-gon : 6 indices.
  assert_eq!( data.indices.len(), 6, "quad -> 2 triangles" );
  // Vertex + normal corners dedup to the 4 authored points; the faceVarying
  // UVs are authored one-per-corner in the same order, so no splits happen.
  assert_eq!( data.positions.len(), 4, "vertex-interpolated attributes dedup to 4 corners" );
  assert_eq!( data.positions[ 0 ], [ -1.0, -1.0, 0.0 ] );
  assert_eq!( data.positions[ 2 ], [ 1.0, 1.0, 0.0 ] );
  let normals = data.normals.expect( "normals authored" );
  assert_eq!( normals.len(), 4 );
  assert_eq!( normals[ 0 ], [ 0.0, 0.0, 1.0 ] );
}

#[ test ]
fn face_varying_uvs_survive_extraction()
{
  let stage = stage_with( &[ ( "scene.usda", QUAD_USDA ) ] );
  let mesh = only_mesh( &stage );
  let data = usd_mesh_extract( &mesh ).expect( "quad extracts" );
  let uvs = data.uvs.expect( "primvars:st extracted" );
  assert_eq!( uvs.len(), 4 );
  assert_eq!( uvs[ 0 ], [ 0.0, 0.0 ] );
  assert_eq!( uvs[ 2 ], [ 1.0, 1.0 ] );
}

#[ test ]
fn triangle_soup_passes_through_unchanged()
{
  // A 3-gon needs no triangulation; counts/indices round-trip.
  let usda = r#"#usda 1.0
def Mesh "Tri"
{
    uniform int[] faceVertexCounts = [ 3 ]
    uniform int[] faceVertexIndices = [ 2, 1, 0 ]
    uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 0, 1, 0 ) ]
}
"#;
  let stage = stage_with( &[ ( "scene.usda", usda ) ] );
  let mesh = only_mesh( &stage );
  let data = usd_mesh_extract( &mesh ).expect( "tri extracts" );
  // Corners become the output vertex order : index 0 is the source face's
  // first corner ( point 2 ), etc.
  assert_eq!( data.indices, vec![ 0, 1, 2 ] );
  assert_eq!( data.positions[ 0 ], [ 0.0, 1.0, 0.0 ] );
  assert_eq!( data.positions[ 2 ], [ 0.0, 0.0, 0.0 ] );
  assert!( data.normals.is_none() );
  assert!( data.uvs.is_none() );
}

#[ test ]
fn indexed_face_varying_uvs_resolve_through_indices()
{
  // Classic subdiv-export pattern : st is `vertex` on a shared pool, and an
  // `st:indices` array maps corners into it. One uv per mesh vertex here,
  // referenced in corner order, so corners dedup and uv follows the vertex.
  let usda = r#"#usda 1.0
def Mesh "UvQuad"
{
    uniform int[] faceVertexCounts = [ 4 ]
    uniform int[] faceVertexIndices = [ 0, 1, 2, 3 ]
    uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 1, 1, 0 ), ( 0, 1, 0 ) ]
    uniform float2[] primvars:st = [ ( 0, 0 ), ( 1, 0 ), ( 1, 1 ), ( 0, 1 ) ] (
        interpolation = "faceVarying"
    )
    uniform int[] primvars:st:indices = [ 3, 2, 1, 0 ]
}
"#;
  let stage = stage_with( &[ ( "scene.usda", usda ) ] );
  let mesh = only_mesh( &stage );
  let data = usd_mesh_extract( &mesh ).expect( "indexed uv quad extracts" );
  let uvs = data.uvs.expect( "indexed uvs resolved" );
  // Corner 0 -> st index 3 -> (0,1) ; corner 2 -> st index 1 -> (1,0).
  assert_eq!( uvs[ 0 ], [ 0.0, 1.0 ] );
  assert_eq!( uvs[ 2 ], [ 1.0, 0.0 ] );
}

#[ test ]
fn counts_exceeding_indices_is_malformed_not_panic()
{
  // faceVertexCounts promises 4 corners, faceVertexIndices holds 3.
  let usda = r#"#usda 1.0
def Mesh "Broken"
{
    uniform int[] faceVertexCounts = [ 4 ]
    uniform int[] faceVertexIndices = [ 0, 1, 2 ]
    uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 0, 1, 0 ) ]
}
"#;
  let stage = stage_with( &[ ( "scene.usda", usda ) ] );
  let mesh = only_mesh( &stage );
  let err = usd_mesh_extract( &mesh ).unwrap_err();
  assert!( matches!( err, UsdError::Malformed( _ ) ), "expected Malformed, got {err:?}" );
}

#[ test ]
fn local_to_world_multiplies_ancestor_xforms()
{
  let stage = stage_with( &[ ( "scene.usda", QUAD_USDA ) ] );
  let mesh = only_mesh( &stage );
  let m = usd_local_to_world( &stage, mesh.path() ).expect( "xform composes" );
  // World carries a translate of (2,0,0) ; the mesh itself has no ops.
  // gf::Matrix4d is row-vector convention ( translation in the last row,
  // indices 12..14 ; see `gf::Matrix4d::translation` ).
  assert_eq!( m.0[ 12 ], 2.0, "translation.x from ancestor Xform" );
  assert_eq!( m.0[ 13 ], 0.0 );
  assert_eq!( m.0[ 14 ], 0.0 );
}

#[ test ]
fn mesh_without_points_is_a_malformed_error()
{
  let usda = r#"#usda 1.0
def Mesh "Empty"
{
    uniform int[] faceVertexCounts = [ 3 ]
    uniform int[] faceVertexIndices = [ 0, 1, 2 ]
}
"#;
  let stage = stage_with( &[ ( "scene.usda", usda ) ] );
  let mesh = only_mesh( &stage );
  let err = usd_mesh_extract( &mesh ).unwrap_err();
  assert!( matches!( err, UsdError::Malformed( _ ) ), "expected Malformed, got {err:?}" );
}

#[ test ]
fn out_of_range_point_index_is_malformed_not_panic()
{
  let usda = r#"#usda 1.0
def Mesh "BadIndex"
{
    uniform int[] faceVertexCounts = [ 3 ]
    uniform int[] faceVertexIndices = [ 0, 1, 99 ]
    uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 0, 1, 0 ) ]
}
"#;
  let stage = stage_with( &[ ( "scene.usda", usda ) ] );
  let mesh = only_mesh( &stage );
  let err = usd_mesh_extract( &mesh ).unwrap_err();
  assert!( matches!( err, UsdError::Malformed( _ ) ), "expected Malformed, got {err:?}" );
}

// ---------------------------------------------------------------------------
// usd_scene_analyze
// ---------------------------------------------------------------------------

/// Finds one analyzed prim by path.
fn prim<'a>( prims : &'a [ UsdPrimData ], path : &str ) -> Option< &'a UsdPrimData >
{
  prims.iter().find( | p | p.path == path )
}

/// One triangle mesh bound to a `UsdPreviewSurface` material under a `Looks`
/// scope, plus an invisible mesh and a proxy-purpose mesh that analyze must
/// drop. Also exercises ancestor-inherited material binding ( `/World/Tri`
/// has no own binding - its parent Xform does ).
const SCENE_USDA : &str = r#"#usda 1.0
(
    defaultPrim = "World"
)

def Xform "World" (
    apiSchemas = [ "MaterialBindingAPI" ]
)
{
    rel material:binding = </World/Looks/BrickMat>

    def "Looks"
    {
        def Material "BrickMat"
        {
            token outputs:surface.connect = </World/Looks/BrickMat/Surface.outputs:surface>

            def Shader "Surface"
            {
                uniform token info:id = "UsdPreviewSurface"
                color3f inputs:diffuseColor = ( 0.8, 0.3, 0.1 )
                float inputs:metallic = 1.0
                float inputs:roughness = 0.25
                token outputs:surface
            }
        }
    }

    def Mesh "Tri"
    {
        uniform int[] faceVertexCounts = [ 3 ]
        uniform int[] faceVertexIndices = [ 0, 1, 2 ]
        uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 0, 1, 0 ) ]
    }

    def Mesh "Bound" (
        apiSchemas = [ "MaterialBindingAPI" ]
    )
    {
        rel material:binding = </World/Looks/BrickMat>
        uniform int[] faceVertexCounts = [ 3 ]
        uniform int[] faceVertexIndices = [ 0, 1, 2 ]
        uniform point3f[] points = [ ( 0, 0, 0 ), ( 2, 0, 0 ), ( 0, 2, 0 ) ]
    }

    def Mesh "Hidden"
    {
        uniform token visibility = "invisible"
        uniform int[] faceVertexCounts = [ 3 ]
        uniform int[] faceVertexIndices = [ 0, 1, 2 ]
        uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 0, 1, 0 ) ]
    }

    def Mesh "Proxy"
    {
        uniform token purpose = "proxy"
        uniform int[] faceVertexCounts = [ 3 ]
        uniform int[] faceVertexIndices = [ 0, 1, 2 ]
        uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 0, 1, 0 ) ]
    }
}
"#;

#[ test ]
fn analyze_keeps_geometry_groups_and_meshes()
{
  let stage = stage_with( &[ ( "scene.usda", SCENE_USDA ) ] );
  let prims = usd_scene_analyze( &stage, None ).expect( "scene analyzes" );

  let world = prim( &prims, "/World" ).expect( "/World group kept" );
  assert!( world.mesh.is_none(), "Xform groups carry no geometry" );
  assert_eq!( world.parent, None, "/World is a scene root ( parent is abs root )" );

  let tri = prim( &prims, "/World/Tri" ).expect( "/World/Tri mesh kept" );
  let data = tri.mesh.as_ref().expect( "mesh data" );
  assert_eq!( data.positions.len(), 3 );
  assert_eq!( data.indices, vec![ 0, 1, 2 ] );
  assert_eq!( tri.parent.as_deref(), Some( "/World" ), "parent link by path" );
}

#[ test ]
fn analyze_skips_the_shading_subtree()
{
  let stage = stage_with( &[ ( "scene.usda", SCENE_USDA ) ] );
  let prims = usd_scene_analyze( &stage, None ).expect( "scene analyzes" );

  assert!( prim( &prims, "/World/Looks/BrickMat" ).is_none(), "Material prim must not render" );
  assert!( prims.iter().all( | p | !p.path.contains( "Surface" ) ), "shader children must not render" );
  assert!( prims.iter().any( | p | p.path == "/World/Looks" ), "the Looks container itself is a harmless group" );
}

#[ test ]
fn analyze_resolves_bound_material_via_api_and_inheritance()
{
  let stage = stage_with( &[ ( "scene.usda", SCENE_USDA ) ] );
  let prims = usd_scene_analyze( &stage, None ).expect( "scene analyzes" );

  // Direct binding on /World/Bound.
  let bound = prim( &prims, "/World/Bound" ).expect( "bound mesh kept" );
  assert_eq!( bound.material_path.as_deref(), Some( "/World/Looks/BrickMat" ) );
  let m = bound.material.as_ref().expect( "preview surface resolved" );
  assert_eq!( m.base_color_rgba, [ 0.8, 0.3, 0.1, 1.0 ] );
  assert_eq!( m.metallic, Some( 1.0 ) );
  assert_eq!( m.roughness, Some( 0.25 ) );
  assert_eq!( m.emissive, None );

  // Inherited binding : /World/Tri has no own MaterialBindingAPI - the walk
  // up to /World must find the binding authored there.
  let tri = prim( &prims, "/World/Tri" ).expect( "tri mesh kept" );
  assert_eq!( tri.material_path.as_deref(), Some( "/World/Looks/BrickMat" ), "binding inherited from ancestor Xform" );
}

#[ test ]
fn analyze_drops_invisible_and_proxy_prims()
{
  let stage = stage_with( &[ ( "scene.usda", SCENE_USDA ) ] );
  let prims = usd_scene_analyze( &stage, None ).expect( "scene analyzes" );

  assert!( prim( &prims, "/World/Hidden" ).is_none(), "invisible mesh dropped" );
  assert!( prim( &prims, "/World/Proxy" ).is_none(), "proxy-purpose mesh dropped" );
}

#[ test ]
fn analyze_converts_transform_to_column_major_f32()
{
  let stage = stage_with( &[ ( "scene.usda", QUAD_USDA ) ] );
  let prims = usd_scene_analyze( &stage, None ).expect( "analyzes" );
  let world = prim( &prims, "/World" ).expect( "World group" );
  // gf row-vector translate (2,0,0) : elements 12..14 ; fed unchanged to
  // `F32x4x4::from_column_major` this is the column-vector translation column.
  assert_eq!( world.local_to_parent, [ 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 2.0, 0.0, 0.0, 1.0 ] );
  // The mesh itself has no ops -> identity.
  let quad = prim( &prims, "/World/Quad" ).expect( "Quad mesh" );
  assert_eq!( quad.local_to_parent[ 12 ], 0.0 );
  assert_eq!( quad.local_to_parent[ 0 ], 1.0 );
}

#[ test ]
fn vertex_interpolated_normals_index_by_face_vertex_indices()
{
  // The discriminating case the earlier fixtures could not see : two triangles
  // sharing vertices ( 6 corners ) with FOUR vertex-interpolated normals. If the
  // `interpolation = "vertex"` metadata is honored, corners dedup to 4 vertices
  // and every normal is an authored value. If it is silently lost ( fallback to
  // faceVarying ), 6 vertices are emitted and the 5th/6th normals fall off the
  // end of the array into [0,0,0] - which is exactly the "dark sphere with
  // purple triangles" symptom from the viewer's USD mode.
  let usda = r#"#usda 1.0
def Mesh "TwoTris"
{
    uniform int[] faceVertexCounts = [ 3, 3 ]
    uniform int[] faceVertexIndices = [ 0, 1, 2, 0, 2, 3 ]
    uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 0, 1, 0 ), ( 1, 1, 0 ) ]
    uniform normal3f[] normals = [ ( 1, 0, 0 ), ( 0, 1, 0 ), ( 0, 0, 1 ), ( 0, -1, 0 ) ]
    (
        interpolation = "vertex"
    )
}
"#;
  let stage = stage_with( &[ ( "scene.usda", usda ) ] );
  let mesh = only_mesh( &stage );
  let data = usd_mesh_extract( &mesh ).expect( "vertex-normal mesh extracts" );

  assert_eq!( data.positions.len(), 4, "shared corners dedup under vertex interpolation" );
  let normals = data.normals.expect( "normals present" );
  assert_eq!( normals.len(), 4 );
  assert_eq!( normals, vec![ [ 1.0, 0.0, 0.0 ], [ 0.0, 1.0, 0.0 ], [ 0.0, 0.0, 1.0 ], [ 0.0, -1.0, 0.0 ] ] );
  assert!( !normals.iter().any( | n | n == &[ 0.0, 0.0, 0.0 ] ), "no zero-filled fallback normals" );
}

// ---------------------------------------------------------------------------
// .mtlx-bound materials ( the native OpenPBR content lane )
// ---------------------------------------------------------------------------

/// Same surface as the gltf_viewer's embedded `open_pbr_gold.mtlx`.
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
</materialx>
"#;

/// A Material whose only content is an external `.mtlx` reference ( the
/// OpenPBRShaderPlayground pattern ). The stage store deliberately lacks
/// `gold.mtlx` ( unresolved reference = diagnostic, the prim keeps its
/// `references` metadata ); the *asset provider* passed to analyze holds it.
const MTLX_SCENE_USDA : &str = r#"#usda 1.0
def Xform "World"
{
    def Material "Gold" (
        prepend references = @./gold.mtlx@</MaterialX/Materials/Gold>
    )
    {
    }

    def Mesh "Ball" (
        apiSchemas = [ "MaterialBindingAPI" ]
    )
    {
        rel material:binding = </World/Gold>
        uniform int[] faceVertexCounts = [ 3 ]
        uniform int[] faceVertexIndices = [ 0, 1, 2 ]
        uniform point3f[] points = [ ( 0, 0, 0 ), ( 1, 0, 0 ), ( 0, 1, 0 ) ]
    }
}
"#;

#[ test ]
fn material_from_mtlx_carries_surface_and_reduction()
{
  let data = renderer::webgl::loaders::usd::usd_material_from_mtlx( GOLD_MTLX ).expect( "gold.mtlx parses" );
  let surface = data.surface.expect( "full OpenPbrSurface captured" );
  assert_eq!( surface.base_color, [ 0.929, 0.788, 0.374 ] );
  assert_eq!( surface.base_metalness, 1.0 );
  assert_eq!( surface.specular_roughness, 0.02 );
  // The glTF-shaped reduction mirrors the surface.
  assert_eq!( data.base_color_rgba[ .. 3 ], [ 0.929, 0.788, 0.374 ] );
  assert_eq!( data.metallic, Some( 1.0 ) );
  assert_eq!( data.roughness, Some( 0.02 ) );
}

#[ test ]
fn analyze_resolves_mtlx_referenced_material_via_provider()
{
  let mut provider = UsdInMemoryResolver::new();
  provider.insert( "gold.mtlx", GOLD_MTLX.as_bytes().to_vec() );

  let stage = stage_with( &[ ( "scene.usda", MTLX_SCENE_USDA ) ] );
  let prims = usd_scene_analyze( &stage, Some( &provider ) ).expect( "analyzes with mtlx provider" );

  let ball = prim( &prims, "/World/Ball" ).expect( "mesh kept" );
  assert_eq!( ball.material_path.as_deref(), Some( "/World/Gold" ), "binding resolved to the Material prim" );
  let m = ball.material.as_ref().expect( "material data present" );
  let surface = m.surface.as_ref().expect( "mtlx lane: full surface resolved through the provider" );
  assert_eq!( surface.base_color, [ 0.929, 0.788, 0.374 ] );
  assert_eq!( surface.base_metalness, 1.0 );
}

#[ test ]
fn analyze_without_provider_falls_back_but_keeps_binding()
{
  // Same scene, no asset provider : the reference cannot be read, but the
  // binding path is still reported and the material is the neutral default.
  let stage = stage_with( &[ ( "scene.usda", MTLX_SCENE_USDA ) ] );
  let prims = usd_scene_analyze( &stage, None ).expect( "analyzes without provider" );
  let ball = prim( &prims, "/World/Ball" ).expect( "mesh kept" );
  assert_eq!( ball.material_path.as_deref(), Some( "/World/Gold" ) );
  let m = ball.material.as_ref().expect( "default material" );
  assert!( m.surface.is_none(), "no provider -> no surface lane" );
}

#[ test ]
fn resolver_serves_asset_text_for_the_provider_trait()
{
  let mut resolver = UsdInMemoryResolver::new();
  resolver.insert( "./gold.mtlx", GOLD_MTLX.as_bytes().to_vec() );
  // normalized lookups agree across `./` spellings
  let text = UsdAssetProvider::asset_text( &resolver, "gold.mtlx" ).expect( "provider reads normalized asset" );
  assert!( text.contains( "open_pbr_surface" ) );
  assert!( UsdAssetProvider::asset_text( &resolver, "missing.mtlx" ).is_none() );
}

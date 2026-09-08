//! Native tests for the USD scene ingestion lane ( OpenPBR adoption plan
//! §2.3 N3 first slice ) : the in-memory `ar::Resolver` + stage open, and the
//! pure `usd_mesh_extract` conversion ( fan triangulation, primvar corner
//! resolution ) plus `usd_local_to_world` composition. Runs entirely off-GPU
//! and off-filesystem - fixtures are inline `.usda` text served through
//! [`UsdInMemoryResolver`], which is exactly how the browser lane will feed
//! HTTP-fetched bytes. Gated behind the crate's `native-formats` feature ( see
//! `required-features` in `Cargo.toml` ).

use renderer::webgl::loaders::usd::{ UsdError, UsdInMemoryResolver, usd_local_to_world, usd_mesh_extract, usd_stage_open };
use openusd::sdf;
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
    .filter_map( | p | geom::Mesh::get( stage, p ).ok().flatten() )
    .next()
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

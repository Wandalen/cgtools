//! OpenPBR test scenes built procedurally — no external assets.
//!
//! Step 1: a correct, well-lit sphere ( geometry verified ).
//! Step 2: apply a real `.mtlx` OpenPBR surface to that sphere via the
//! `OpenPbrSurface → PbrMaterial` runtime bridge.

use std::{ cell::RefCell, rc::Rc };

use minwebgl as gl;
use primitive_generation::{ AttributesData, PrimitiveData, Transform, primitives_data_to_gltf };
use renderer::webgl::material::{ OpenPbrSurface, PbrMaterial };
use renderer::webgl::loaders::gltf::GLTF;

/// A unit icosphere subdivided `4` times ( 2562 vertices / 5120 faces ),
/// scaled to `radius`, with radial normals ( a sphere's normal is its own
/// position ).
fn icosphere_attributes( radius : f32 ) -> AttributesData
{
  let ( raw_positions, indices ) = primitive_generation::solid::icosphere_subdivided( 4 );

  let positions : Vec< [ f32; 3 ] > = raw_positions.iter()
  .map( | p | [ p[ 0 ] * radius, p[ 1 ] * radius, p[ 2 ] * radius ] )
  .collect();

  let normals : Vec< [ f32; 3 ] > = raw_positions.iter()
  .map( | p |
  {
    let len = ( p[ 0 ] * p[ 0 ] + p[ 1 ] * p[ 1 ] + p[ 2 ] * p[ 2 ] ).sqrt();
    if len > 1e-6 { [ p[ 0 ] / len, p[ 1 ] / len, p[ 2 ] / len ] } else { [ 0.0, 1.0, 0.0 ] }
  } )
  .collect();

  AttributesData { positions, indices, normals }
}

/// Builds a renderer `GLTF` scene containing a single icosphere whose material
/// is driven by a parsed OpenPBR surface ( from the `.mtlx` lane ) through
/// `PbrMaterial::openpbr_surface_apply` — the `USE_OPENPBR` shading path.
#[ must_use ]
pub fn sphere_with_surface( gl : &gl::WebGl2RenderingContext, surface : &OpenPbrSurface ) -> GLTF
{
  let attributes = Rc::new( RefCell::new( icosphere_attributes( 0.5 ) ) );

  let primitive = PrimitiveData
  {
    name : Some( Box::from( "openpbr_sphere" ) ),
    parent : None,
    attributes : Some( attributes ),
    color : gl::F32x4::from( [ 1.0, 1.0, 1.0, 1.0 ] ),
    transform : Transform::default(),
  };

  let gltf = primitives_data_to_gltf( gl, &[ primitive ] );

  // Swap the generated fallback material for one configured from the surface.
  let material = gltf.materials.first().cloned().expect( "sphere material exists" );
  let mut configured = PbrMaterial::new( gl );
  configured.openpbr_surface_apply( surface );
  // The icosphere is a closed, outward-wound surface: cull its back faces so
  // back-facing fragments don't z-fight / sparkle at the silhouette contour.
  configured.cull_mode = Some( renderer::webgl::material::CullMode::Back );
  configured.double_sided = false;
  *material.borrow_mut() = Box::new( configured );

  gltf
}

/// Writes a `.usda` scene text ( the same icosphere ) whose `Material` prim
/// references an external `mat.mtlx` — the native OpenPBR content pattern
/// ( OpenPBRShaderPlayground layout ) exercised through `loaders::usd` :
/// composition, mesh extraction, binding resolution, mtlx lane and GL
/// assembly in one pass.
#[ must_use ]
pub fn usd_sphere_scene( radius : f32 ) -> String
{
  use std::fmt::Write as _;

  let attributes = icosphere_attributes( radius );

  let mut s = String::with_capacity( 96 * ( attributes.positions.len() + attributes.indices.len() ) + 512 );
  s.push_str( "#usda 1.0\n(\n    defaultPrim = \"World\"\n    upAxis = \"Y\"\n    metersPerUnit = 1.0\n)\n\ndef Xform \"World\"\n{\n" );
  s.push_str( "    def Material \"Mtl\"\n    (\n        prepend references = @./mat.mtlx@\n    )\n    {\n    }\n\n" );
  s.push_str( "    def Mesh \"Sphere\"\n    (\n        apiSchemas = [ \"MaterialBindingAPI\" ]\n    )\n    {\n" );
  s.push_str( "        rel material:binding = </World/Mtl>\n" );

  // faceVertexCounts : every face is a triangle.
  let mut row = String::new();
  s.push_str( "        uniform int[] faceVertexCounts = [ " );
  for i in 0..attributes.indices.len() / 3
  {
    let _ = write!( row, "3{0}", if i + 1 == attributes.indices.len() / 3 { "" } else { ", " } );
    if row.len() > 72 { s.push_str( &row ); row.clear(); }
  }
  s.push_str( &row );
  s.push_str( " ]\n" );

  // faceVertexIndices.
  row.clear();
  s.push_str( "        uniform int[] faceVertexIndices = [ " );
  for ( i, idx ) in attributes.indices.iter().enumerate()
  {
    let _ = write!( row, "{idx}{0}", if i + 1 == attributes.indices.len() { "" } else { ", " } );
    if row.len() > 72 { s.push_str( &row ); row.clear(); }
  }
  s.push_str( &row );
  s.push_str( " ]\n" );

  // points + radial normals ( vertex interpolation ).
  row.clear();
  s.push_str( "        uniform point3f[] points = [ " );
  for ( i, p ) in attributes.positions.iter().enumerate()
  {
    let _ = write!( row, "( {0}, {1}, {2} ){3}", p[ 0 ], p[ 1 ], p[ 2 ], if i + 1 == attributes.positions.len() { "" } else { ", " } );
    if row.len() > 72 { s.push_str( &row ); row.clear(); }
  }
  s.push_str( &row );
  s.push_str( " ]\n" );

  row.clear();
  s.push_str( "        uniform normal3f[] normals = [ " );
  for ( i, n ) in attributes.normals.iter().enumerate()
  {
    let _ = write!( row, "( {0}, {1}, {2} ){3}", n[ 0 ], n[ 1 ], n[ 2 ], if i + 1 == attributes.normals.len() { "" } else { ", " } );
    if row.len() > 72 { s.push_str( &row ); row.clear(); }
  }
  s.push_str( &row );
  s.push_str( " ]\n        (\n            interpolation = \"vertex\"\n        )\n" );

  s.push_str( "        uniform token subdivisionScheme = \"none\"\n    }\n}\n" );
  s
}

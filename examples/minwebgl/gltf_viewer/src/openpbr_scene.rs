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

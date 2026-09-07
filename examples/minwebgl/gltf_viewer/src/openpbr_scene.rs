//! OpenPBR test scenes built procedurally — no external assets.
//!
//! Step 1 goal: a correct, well-lit solid-colour sphere so geometry can be
//! verified before any OpenPBR shading is applied.

use std::{ cell::RefCell, rc::Rc };

use minwebgl as gl;
use primitive_generation::{ AttributesData, PrimitiveData, Transform, primitives_data_to_gltf };
use renderer::webgl::{ cast_unchecked_material_to_ref_mut, loaders::gltf::GLTF, material::PbrMaterial };

/// A unit-radius icosphere at the origin, scaled to `radius`, with per-vertex
/// normals derived from the vertex direction ( normals of a sphere point along
/// the position ).
fn icosphere_attributes( radius : f32 ) -> AttributesData
{
  let ( raw_positions, indices ) = primitive_generation::solid::icosphere();

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

/// Builds a renderer `GLTF` scene containing a single solid-colour icosphere
/// ( no textures, plain `PbrMaterial` base colour ) — the geometry-verification
/// step of the OpenPBR test mode.
#[ must_use ]
pub fn solid_color_icosphere( gl : &gl::WebGl2RenderingContext, color : [ f32; 4 ] ) -> GLTF
{
  let attributes = Rc::new( RefCell::new( icosphere_attributes( 0.5 ) ) );

  let primitive = PrimitiveData
  {
    name : Some( Box::from( "openpbr_sphere" ) ),
    parent : None,
    attributes : Some( attributes ),
    color : gl::F32x4::from( color ),
    transform : Transform::default(),
  };

  let gltf = primitives_data_to_gltf( gl, &[ primitive ] );

  // Solid colour: override the generated fallback material.
  if let Some( material ) = gltf.materials.first()
  {
    let mut material = cast_unchecked_material_to_ref_mut::< PbrMaterial >( material.borrow_mut() );
    material.base_color_factor = gl::F32x4::from( color );
    material.metallic_factor = 0.0;
    material.roughness_factor = 0.4;
  }

  gltf
}

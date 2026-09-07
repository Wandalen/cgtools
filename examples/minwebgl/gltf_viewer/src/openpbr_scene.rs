//! OpenPBR test scenes built procedurally — no external assets.
//!
//! Step 1 goal: a correct, well-lit solid-colour sphere so geometry can be
//! verified before any OpenPBR shading is applied.

use std::{ cell::RefCell, rc::Rc };

use minwebgl as gl;
use primitive_generation::{ AttributesData, PrimitiveData, Transform, primitives_data_to_gltf };
use renderer::webgl::{ cast_unchecked_material_to_ref_mut, loaders::gltf::GLTF, material::PbrMaterial };

use std::f32::consts::{ PI, TAU };

/// Pushes a triangle, fixing its winding so the face points outward from a
/// sphere centered at the origin ( every vertex is on the sphere, so the
/// outward direction is the centroid's own direction ).
fn triangle_push_outward
(
  positions : &[ [ f32; 3 ] ],
  indices : &mut Vec< u32 >,
  a : u32,
  b : u32,
  c : u32
)
{
  let pa = positions[ a as usize ];
  let pb = positions[ b as usize ];
  let pc = positions[ c as usize ];

  let cx = ( pa[ 0 ] + pb[ 0 ] + pc[ 0 ] ) / 3.0;
  let cy = ( pa[ 1 ] + pb[ 1 ] + pc[ 1 ] ) / 3.0;
  let cz = ( pa[ 2 ] + pb[ 2 ] + pc[ 2 ] ) / 3.0;
  let len = ( cx * cx + cy * cy + cz * cz ).sqrt();
  let ( nx, ny, nz ) = if len > 1e-9 { ( cx / len, cy / len, cz / len ) } else { ( 1.0, 0.0, 0.0 ) };

  let e1 = [ pb[ 0 ] - pa[ 0 ], pb[ 1 ] - pa[ 1 ], pb[ 2 ] - pa[ 2 ] ];
  let e2 = [ pc[ 0 ] - pa[ 0 ], pc[ 1 ] - pa[ 1 ], pc[ 2 ] - pa[ 2 ] ];
  let cross =
  [
    e1[ 1 ] * e2[ 2 ] - e1[ 2 ] * e2[ 1 ],
    e1[ 2 ] * e2[ 0 ] - e1[ 0 ] * e2[ 2 ],
    e1[ 0 ] * e2[ 1 ] - e1[ 1 ] * e2[ 0 ],
  ];
  let dot = cross[ 0 ] * nx + cross[ 1 ] * ny + cross[ 2 ] * nz;

  if dot >= 0.0
  {
    indices.extend_from_slice( &[ a, b, c ] );
  }
  else
  {
    indices.extend_from_slice( &[ a, c, b ] );
  }
}

/// A latitude/longitude ( UV ) sphere with smooth normals, radius `radius`,
/// `segments` longitude divisions and `bands` latitude bands between the two
/// poles. Each pole is a single shared vertex ( no degenerate fan triangles ).
fn uv_sphere_attributes( radius : f32, segments : u32, bands : u32 ) -> AttributesData
{
  let mut positions : Vec< [ f32; 3 ] > = Vec::new();
  let mut normals : Vec< [ f32; 3 ] > = Vec::new();

  // Row `r` covers theta = PI * r / bands, `0` ( north pole ) ..= `bands`
  // ( south pole ); rows `0` and `bands` are single pole vertices.
  let mut row_start = vec![ 0usize; ( bands + 1 ) as usize ];

  positions.push( [ 0.0, radius, 0.0 ] );
  normals.push( [ 0.0, 1.0, 0.0 ] );
  row_start[ 0 ] = 0;

  for r in 1 .. bands
  {
    let theta = PI * r as f32 / bands as f32;
    let ( st, ct ) = theta.sin_cos();
    row_start[ r as usize ] = positions.len();
    for s in 0 .. segments
    {
      let phi = TAU * s as f32 / segments as f32;
      let ( sp, cp ) = phi.sin_cos();
      positions.push( [ radius * st * cp, radius * ct, radius * st * sp ] );
      normals.push( [ st * cp, ct, st * sp ] );
    }
  }

  row_start[ bands as usize ] = positions.len();
  positions.push( [ 0.0, -radius, 0.0 ] );
  normals.push( [ 0.0, -1.0, 0.0 ] );

  let at = | row : u32, s : u32 | -> u32
  {
    if row == 0
    {
      0
    }
    else if row == bands
    {
      row_start[ bands as usize ] as u32
    }
    else
    {
      ( row_start[ row as usize ] as u32 ) + s
    }
  };

  let mut indices = Vec::new();
  for r in 0 .. bands
  {
    for s in 0 .. segments
    {
      let s2 = ( s + 1 ) % segments;
      let a = at( r, s );
      let b = at( r, s2 );
      let c = at( r + 1, s );
      let d = at( r + 1, s2 );
      triangle_push_outward( &positions, &mut indices, a, b, c );
      triangle_push_outward( &positions, &mut indices, a, c, d );
    }
  }

  AttributesData { positions, indices, normals }
}

/// Builds a renderer `GLTF` scene containing a single solid-colour UV sphere
/// ( no textures, plain `PbrMaterial` base colour ) — the geometry-verification
/// step of the OpenPBR test mode.
#[ must_use ]
pub fn solid_color_sphere( gl : &gl::WebGl2RenderingContext, color : [ f32; 4 ] ) -> GLTF
{
  let attributes = Rc::new( RefCell::new( uv_sphere_attributes( 0.5, 64, 40 ) ) );

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

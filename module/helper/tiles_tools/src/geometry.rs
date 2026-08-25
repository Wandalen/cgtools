//! Flat 2D geometry generators for tile shapes, focusing on hexagons: a triangulated
//! fill, an outline, the corner points, and a batching helper that replicates one
//! shape across grid coordinates into a single buffer.
//!
//! Every generator returns an independent-primitive soup, ready for the `TRIANGLES`
//! or `LINES` draw mode — no `TRIANGLE_FAN` or `LINE_LOOP` mode is ever required.
//! This is a hard constraint, not a preference: [ `from_iter` ] concatenates many
//! cells into one buffer drawn with a single call, and mode-level fans/loops cannot
//! express disjoint shapes within one draw call.

use crate::coordinates;
use coordinates::pixel::Pixel;
use ndarray_cg::{ F32x2, F32x3x3, IntoVector as _, Vector };

/// Generates flat 2d geometry for a whole grid from an iterator of coordinates and a
/// `geometry_producer`, concatenating one copy of the produced shape per coordinate
/// into a single buffer.
///
/// Converts each coordinate to its 2d position and places the shape there,
/// additionally applying `transform` to every shape point first. `geometry_producer`
/// is expected to produce a flat list of `[ x, y ]` positions; its primitive kind
/// ( independent triangles or independent line segments ) is preserved verbatim, so
/// the result is drawable with the same mode as a single shape.
pub fn from_iter< I, C, F >( iter : I, geometry_producer : F, transform : F32x3x3 ) -> Vec< f32 >
where
  I : Iterator< Item = C >,
  C : Into< Pixel >,
  F : Fn() -> Vec< f32 >,
{
  let shape = geometry_producer();
  let mut points = vec![];
  for coord in iter
  {
    let Pixel { data : [ x, y ] } = coord.into();
    let y = -y;

    for point in shape.chunks( 2 )
    {
      let pos = transform * Vector( [ point[ 0 ], point[ 1 ], 1.0 ] );
      points.push( x + pos.x() );
      points.push( y + pos.y() );
    }
  }
  points
}

/// Assembles the 4-triangle independent-triangle soup filling the hexagon whose 6
/// corner points are given, anchoring every triangle at `points[ 0 ]`.
///
/// The topology is fan-patterned ( every triangle shares vertex 0 — the standard
/// triangulation of a convex polygon ), but the encoding is 4 standalone triangles:
/// the module-level no-`TRIANGLE_FAN`/no-`LINE_LOOP` constraint governs the draw
/// mode the output requires, and this output needs only `TRIANGLES`.
fn triangles_from_vertices( points : &[ F32x2; 6 ] ) -> Vec< f32 >
{
  let first = points[ 0 ];
  let mut positions = vec![];

  for w in points[ 1.. ].windows( 2 )
  {
    let a = w[ 0 ];
    let b = w[ 1 ];

    positions.push( first[ 0 ] );
    positions.push( first[ 1 ] );
    positions.push( a[ 0 ] );
    positions.push( a[ 1 ] );
    positions.push( b[ 0 ] );
    positions.push( b[ 1 ] );
  }

  positions
}

/// Generates 2d positions of the 4 independent triangles that fill a hexagon of
/// unit radius. Center is at ( 0, 0 ); distance from center to each vertex is 1.0.
///
/// Intended draw mode: `TRIANGLES` — every consecutive 3 positions ( 6 floats ) form
/// one standalone triangle; 4 triangles, 24 floats total. `TRIANGLE_FAN` mode is
/// never required ( see the module docs for why that constraint exists ).
///
/*
      ______________________
     / ____                 \
    /       ____             \
   /             ____         \
  /                   ____     \
 /                         ____ \
/________________________________\
\                          ____  /
 \                    ____      /
  \              ____          /
   \        ____              /
    \  ____                  /
     \______________________/
*/
#[ must_use ]
pub fn hexagon_triangles() -> Vec< f32 >
{
  triangles_from_vertices( &hexagon_vertices() )
}

/// Generates the same 4-independent-triangle fill as [ `hexagon_triangles` ], with
/// every corner point transformed by `transform` before triangle assembly.
///
/// Intended draw mode: `TRIANGLES` — 4 standalone triangles, 24 floats total.
#[ must_use ]
pub fn hexagon_triangles_with_transform( transform : F32x3x3 ) -> Vec< f32 >
{
  let mut points = hexagon_vertices();
  for point in &mut points
  {
    let p = transform * Vector( [ point[ 0 ], point[ 1 ], 1.0 ] );
    point.0 = [ p.x(), p.y() ];
  }

  triangles_from_vertices( &points )
}

/// Generates 2d positions of the 6 independent line segments that outline a hexagon
/// of unit radius. Center is at ( 0, 0 ); distance from center to each vertex is 1.0.
///
/// Intended draw mode: `LINES` — every consecutive 2 positions ( 4 floats ) form one
/// standalone segment; 6 segments ( including the explicit closing segment from the
/// last vertex back to the first ), 24 floats total. `LINE_LOOP` mode is never
/// required ( see the module docs for why that constraint exists ).
///
/*
      ______________________
     /                      \
    /                        \
   /                          \
  /                            \
 /                              \
/                                \
\                                /
 \                              /
  \                            /
   \                          /
    \                        /
     \______________________/
*/
#[ must_use ]
pub fn hexagon_lines() -> Vec< f32 >
{
  let points = hexagon_vertices();
  let mut positions = vec![];
  for w in points.windows( 2 )
  {
    let a = w[ 0 ];
    let b = w[ 1 ];

    positions.push( a[ 0 ] );
    positions.push( a[ 1 ] );
    positions.push( b[ 0 ] );
    positions.push( b[ 1 ] );
  }

  positions.push( points[ 5 ][ 0 ] );
  positions.push( points[ 5 ][ 1 ] );
  positions.push( points[ 0 ][ 0 ] );
  positions.push( points[ 0 ][ 1 ] );

  positions
}

/// Generates the six corner points of a hexagon of unit radius, in counterclockwise
/// order starting from ( 1, 0 ). Center is at ( 0, 0 ); distance from center to each
/// vertex is 1.0 ( vertex `i` sits at angle `60° × i` ).
///
///
/*
      *                    *


*            (0; 0)              *(1; 0)




     *                      *
*/
#[ must_use ]
pub fn hexagon_vertices() -> [ F32x2; 6 ]
{
  let mut points : [ F32x2; 6 ] = Default::default();
  for ( i, point ) in points.iter_mut().enumerate()
  {
    let angle = ( ( 60 * i ) as f32 ).to_radians();
    *point = ( angle.cos(), angle.sin() ).into_vector();
  }
  points
}

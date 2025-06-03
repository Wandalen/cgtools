use crate::coordinates;
use coordinates::pixel::Pixel;
use ndarray_cg::{ F32x2, F32x3x3, IntoVector as _, Vector };

// aaa : use geometry instead of mesh. rename also file
// aaa : if geometry is generated description must have information about what kind of primitive it's based on
// aaa : no fans or loops
// aaa : description should be much more descriptive and preferably visual

/// Generates a 2d mesh from an iterator of coordinates and a `mesh_producer`.
/// Converts each coordinate to 2d position and places the mesh at that position, additionaly applying `transform`.
/// `mesh_producer` is expected to produce a list of 2d positions.
pub fn from_iter< I, C, F >( iter : I, mesh_producer : F, transform : F32x3x3 ) -> Vec< f32 >
where
  I : Iterator< Item = C >,
  C : Into< Pixel >,
  F : Fn() -> Vec< f32 >
{
  let mesh = mesh_producer(); // aaa : I thought grid_mesh generate mesh? confusing names
  let mut points = vec![];
  for coord in iter
  {
    let Pixel { data : [ x, y ] } = coord.into();
    let y = -y;

    for point in mesh.chunks( 2 )
    {
      let pos = transform * Vector( [ point[ 0 ], point[ 1 ], 1.0 ] );
      points.push( x + pos.x() );
      points.push( y + pos.y() );
    }
  }
  points
}

/// Generates a list of 2d positions of triangles that form a hexagon of a unit radius.
/// Center is at (0, 0). Distance from center to each vertex is 1.0.
/// The hexagon is divided into 4 triangles.
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
pub fn hexagon_triangles() -> Vec< f32 >
{
  let points = hexagon_vertices();
  let mut positions = vec![];

  let first = points.first().unwrap();

  for w in points[ 1.. ].windows( 2 )
  {
    let point1 = w[ 0 ];
    let point2 = w[ 1 ];

    positions.push( first[ 0 ] );
    positions.push( first[ 1 ] );
    positions.push( point1[ 0 ] );
    positions.push( point1[ 1 ] );
    positions.push( point2[ 0 ] );
    positions.push( point2[ 1 ] );
  }

  positions
}

/// Generates a list of 2d positions of lines that form a hexagon of a unit radius.
/// Center is at (0, 0). Distance from center to each vertex is 1.0.
/// The hexagon is formed of into 6 lines.
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
pub fn hexagon_lines() -> Vec< f32 >
{
  let points = hexagon_vertices();
  let mut positions = vec![];
  for w in points.windows( 2 )
  {
    let point1 = w[ 0 ];
    let point2 = w[ 1 ];

    positions.push( point1[ 0 ] );
    positions.push( point1[ 1 ] );
    positions.push( point2[ 0 ] );
    positions.push( point2[ 1 ] );
  }

  positions.push( points.last().unwrap()[ 0 ] );
  positions.push( points.last().unwrap()[ 1 ] );
  positions.push( points.first().unwrap()[ 0 ] );
  positions.push( points.first().unwrap()[ 1 ] );

  positions
}

/// Generates the six corner points of a hexagon.
/// Outputs a list of 2d point of a hexagon of a unit radius.
/// Center is at (0, 0). Distance from center to each vertex is 1.0.
///
///
/*
      *                    *


*            (0; 0)              *(1; 0)




     *                      *
*/
pub fn hexagon_vertices() -> [ F32x2; 6 ]
{
  let mut points : [ F32x2; 6 ] = Default::default();
  for i in 0..6
  {
    let angle = ( ( 60 * i ) as f32 ).to_radians();
    points[ i ] = ( angle.cos(), angle.sin() ).into_vector();
  }
  points
}

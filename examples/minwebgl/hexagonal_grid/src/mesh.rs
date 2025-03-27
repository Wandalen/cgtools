use crate::*;
use coordinates::Axial;
use layout::HexLayout;
use minwebgl::{ math::Vector, F32x4x4 };

/// Generates line mesh in the manner of LINE LOOP for a hexagon.
///
/// # Returns
/// A `Vec<f32>` containing the x and y coordinates of the hexagon's outline.
pub fn hex_line_loop_mesh( layout : &HexLayout ) -> Vec< f32 >
{
  let points = hex_vertices( layout);
  let mut positions = vec![];
  for point in points
  {
    positions.push( point.0 );
    positions.push( point.1 );
  }

  positions
}

/// Generates triangle mesh in the manner of TRIANGLE FAN for a hexagon.
///
/// # Returns
/// A `Vec<f32>` containing the x and y coordinates of the triangles.
pub fn hex_triangle_fan_mesh( layout : &HexLayout ) -> Vec< f32 >
{
  let points = hex_vertices( layout );
  let mut positions = vec![];

  let hex_center = ( 0.0, 0.0 );
  positions.push( hex_center.0 );
  positions.push( hex_center.1 );

  for point in points
  {
    positions.push( point.0 );
    positions.push( point.1 );
  }

  let point = points.first().unwrap();
  positions.push( point.0 );
  positions.push( point.1 );

  positions
}

/// Generates a triangle mesh for a grid of hexagons.
///
/// # Parameters
/// - `coords`: An iterator of `Axial` coordinates.
/// - `layout`: The layout of the hexagons.
/// - `transform`: A 4x4 matrix to transform the hexagons.
///
/// # Returns
/// A `Vec<f32>` containing the x and y coordinates of the triangles.
pub fn grid_triangle_mesh< C >( coords : C, layout : &HexLayout, transform : F32x4x4 ) -> Vec< f32 >
where
  C : Iterator< Item = Axial >,
{
  let mut points = vec![];
  for coord in coords
  {
    let Pixel { x, y } = layout.hex_pixel_coord( coord );
    let y = -y;
    let mesh = hex_triangle_mesh( layout );
    for point in mesh.chunks( 2 )
    {
      let pos = Vector( [ point[ 0 ], point[ 1 ], 0.0, 1.0 ] );
      let pos = transform * pos;
      points.push( x + pos.x() );
      points.push( y + pos.y() );
    }
  }
  points
}

/// Generates a triangular mesh of a hexagon.
///
/// # Returns
/// A `Vec<f32>` containing the x and y coordinates of the triangles.
pub fn hex_triangle_mesh( layout : &HexLayout ) -> Vec< f32 >
{
  let points = hex_vertices( layout );
  let mut positions = vec![];

  let hex_center = ( 0.0, 0.0 );

  for w in points.windows( 2 )
  {
    let point1 = w[ 0 ];
    let point2 = w[ 1 ];

    positions.push( hex_center.0 );
    positions.push( hex_center.1 );
    positions.push( point1.0 );
    positions.push( point1.1 );
    positions.push( point2.0 );
    positions.push( point2.1 );
  }

  let point1 = points.last().unwrap();
  let point2 = points.first().unwrap();

  positions.push( hex_center.0 );
  positions.push( hex_center.1 );
  positions.push( point1.0 );
  positions.push( point1.1 );
  positions.push( point2.0 );
  positions.push( point2.1 );

  positions
}

/// Generates the six corner points of a hexagon.
///
/// # Returns
/// An array of six `(f32, f32)` tuples representing the x and y coordinates of the hexagon's corners.
pub fn hex_vertices( layout : &HexLayout ) -> [ ( f32, f32 ); 6 ]
{
  let mut points : [ ( f32, f32 ); 6 ] = Default::default();
  for i in 0..6
  {
    let angle = 60 * i;
    let angle = ( angle as f32 ).to_radians() + layout.orientation.orientation_angle();
    points[ i ] = ( angle.cos() * layout.size, angle.sin() * layout.size );
  }
  points
}

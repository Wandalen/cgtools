use crate::coordinates;
use coordinates::pixel::Pixel;
use coordinates::hexagonal::PixelConversion;
use ndarray_cg::{ F32x2, F32x4x4, IntoVector as _, Vector };

// // qqq : use geometry instead of mesh. rename also file
// // qqq : if geometry is generated description must have information about what kind of primitive it's based on
// // qqq : no fans or loops
// // qqq : description should be much more descriptive and preferably visual

// /// Generates a line mesh for a grid of hexagons.
// ///
// /// # Parameters
// /// - `coords`: An iterator of `Axial` coordinates.
// /// - `layout`: The layout of the hexagons.
// /// - `transform`: A 4x4 matrix to transform the hexagons.
// ///
// /// # Returns
// /// A `Vec<f32>` containing the x and y coordinates of the triangles.
// // aaa : use it in example instead drawing each heaxgon individually
// pub fn grid_line< I, C >
// (
//   coords : I,
//   layout : &HexLayout,
//   transform : Option< F32x4x4 >
// )
// -> Vec< f32 >
// where
//   I : Iterator< Item = C >,
//   C : CoordinateConversion
// {
//   grid( coords, layout, transform, hex_line )
// }

// /// Generates a triangle mesh for a grid of hexagons.
// ///
// /// # Parameters
// /// - `coords`: An iterator of `Axial` coordinates.
// /// - `layout`: The layout of the hexagons.
// /// - `transform`: A 4x4 matrix to transform the hexagons.
// ///
// /// # Returns
// /// A `Vec<f32>` containing the x and y coordinates of the triangles.
// pub fn grid_triangle< I, C >
// (
//   coords : I,
//   layout : &HexLayout,
//   transform : Option< F32x4x4 >
// )
// -> Vec< f32 >
// where
//   I : Iterator< Item = C >,
//   C : CoordinateConversion
// {
//   grid( coords, layout, transform, hex_triangle )
// }

// // qqq : not clear what does it do
// /// Generates a mesh for a grid of hexagons.
// ///
// /// # Parameters
// /// - `coords`: An iterator of `Axial` coordinates.
// /// - `layout`: The layout of the hexagons.
// /// - `transform`: A 4x4 matrix to transform the hexagons.
// /// - `mesh`: A function that return mesh of a hexagon
// ///
// /// # Returns
// /// A `Vec<f32>` containing the x and y coordinates of the triangles.
// pub fn grid< I, C, F >
// (
//   coords : I,
//   layout : &HexLayout,
//   transform : Option< F32x4x4 >, // qqq : here and in all such places, make presence/absence of transform a parameter to make compile time this diffirentation instead of doing it multiple times in a loop.
//   mesh_producer : F, // qqq : use verb for name nad use such verb to help understand what is it. mesh confuses
// )
// -> Vec< f32 >
// where
//   I : Iterator< Item = C >,
//   C : CoordinateConversion,
//   F : Fn( &HexLayout ) -> Vec< f32 >
// {
//   let mut points = vec![];
//   for coord in coords
//   {
//     let Pixel { data : [ x, y ] } = layout.pixel_coord( coord );
//     let y = -y;
//     let mesh = mesh_producer( layout ); // qqq : I thought grid_mesh generate mesh? confusing names
//     for point in mesh.chunks( 2 )
//     {
//       let mut pos = Vector( [ point[ 0 ], point[ 1 ], 0.0, 1.0 ] );
//       if let Some( transform ) = transform
//       {
//         pos = transform * pos;
//       }
//       points.push( x + pos.x() );
//       points.push( y + pos.y() );
//     }
//   }
//   points
// }


// /// Generates line mesh in the manner of LINE LOOP for a hexagon.
// ///
// /// # Returns
// /// A `Vec<f32>` containing the x and y coordinates of the hexagon's outline.
// // qqq : no loop geometries. use strip geometry intead
// pub fn hex_line_loop( layout : &HexLayout ) -> Vec< f32 >
// {
//   let points = hex_vertices( layout );
//   let mut positions = vec![];
//   for point in points
//   {
//     positions.push( point.0 );
//     positions.push( point.1 );
//   }

//   positions
// }

// /// Generates line mesh for a hexagon.
// ///
// /// # Returns
// /// A `Vec<f32>` containing the x and y coordinates of the hexagon's outline.
// pub fn hex_line( layout : &HexLayout ) -> Vec< f32 >
// {
//   let points = hex_vertices( layout );
//   let mut positions = vec![];
//   for window in points.windows( 2 )
//   {
//     if let [ point1, point2 ] = window
//     {
//       positions.push( point1.0 );
//       positions.push( point1.1 );
//       positions.push( point2.0 );
//       positions.push( point2.1 );
//     }
//   }

//   let last = points.last().unwrap();
//   let first = points.first().unwrap();
//   positions.push( last.0 );
//   positions.push( last.1 );
//   positions.push( first.0 );
//   positions.push( first.1 );

//   positions
// }

// // qqq : how does it look. not enough description to understand
// /// Generates a triangular mesh of a hexagon.
// ///
// /// # Returns
// /// A `Vec<f32>` containing the x and y coordinates of the triangles.
// pub fn hex_triangle( layout : &HexLayout ) -> Vec< f32 >
// {
//   let points = hex_vertices( layout );
//   let mut positions = vec![];

//   let hex_center = ( 0.0, 0.0 );

//   for w in points.windows( 2 )
//   {
//     let point1 = w[ 0 ];
//     let point2 = w[ 1 ];

//     positions.push( hex_center.0 );
//     positions.push( hex_center.1 );
//     positions.push( point1.0 );
//     positions.push( point1.1 );
//     positions.push( point2.0 );
//     positions.push( point2.1 );
//   }

//   let point1 = points.last().unwrap();
//   let point2 = points.first().unwrap();

//   positions.push( hex_center.0 );
//   positions.push( hex_center.1 );
//   positions.push( point1.0 );
//   positions.push( point1.1 );
//   positions.push( point2.0 );
//   positions.push( point2.1 );

//   positions
// }

// /// Generates triangle mesh in the manner of TRIANGLE FAN for a hexagon.
// ///
// /// # Returns
// /// A `Vec<f32>` containing the x and y coordinates of the triangles.
// // qqq : no fans, use triangle strip
// pub fn hex_triangle_fan_mesh( layout : &HexLayout ) -> Vec< f32 >
// {
//   let points = hex_vertices( layout );
//   let mut positions = vec![];

//   let hex_center = ( 0.0, 0.0 );
//   positions.push( hex_center.0 );
//   positions.push( hex_center.1 );

//   for point in points
//   {
//     positions.push( point.0 );
//     positions.push( point.1 );
//   }

//   let point = points.first().unwrap();
//   positions.push( point.0 );
//   positions.push( point.1 );

//   positions
// }

pub fn from_iter< I, C, F >( iter : I, hex_size : f32, mesh_producer : F, transform : F32x4x4 ) -> Vec< f32 >
where
  I : Iterator< Item = C >,
  C : PixelConversion,
  F : Fn() -> Vec< f32 >
{
  let mesh = mesh_producer(); // aaa : I thought grid_mesh generate mesh? confusing names
  let mut points = vec![];
  for coord in iter
  {
    let Pixel { data : [ x, y ] } = coord.to_pixel( hex_size );
    let y = -y;

    for point in mesh.chunks( 2 )
    {
      let pos = transform * Vector( [ hex_size * point[ 0 ], hex_size * point[ 1 ], 0.0, 1.0 ] );
      points.push( x + pos.x() );
      points.push( y + pos.y() );
    }
  }
  points
}

///
///
///
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

/// Generates the six corner points of a hexagon.
pub fn hexagon_vertices() -> [ F32x2; 6 ]
{
  let mut points : [  F32x2; 6 ] = Default::default();
  for i in 0..6
  {
    let angle = ( ( 60 * i ) as f32 ).to_radians();
    points[ i ] = ( angle.cos(), angle.sin() ).into_vector();
  }
  points
}

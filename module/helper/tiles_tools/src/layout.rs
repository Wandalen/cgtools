use crate::coordinates::hexagonal;
use hexagonal::{ Coordinate, Flat, Offset, Pointy, Axial, PixelConversion };
use ndarray_cg::{ F32x2, I32x2 };
use std::marker::PhantomData;

/// RECTANGULAR LAYOUT
pub struct RectangularGrid< Parity, Orientation >
{
  /// TOP LEFT CORNER, BOTTOM RIGHT CORNER
  pub bounds : [ I32x2; 2 ],
  pub hex_size : f32,
  parity : PhantomData< Parity >,
  orientation : PhantomData< Orientation >,
}

impl< Parity, Orientation > RectangularGrid< Parity, Orientation >
{
  /// Creates a new `RectLayout` instance with the specified bounds.
  ///
  /// # Parameters
  /// - `bounds`: The bounds of the layout.
  ///
  /// # Returns
  /// A new `RectLayout` instance.
  pub const fn new( hex_size : f32, bounds : [ I32x2; 2 ] ) -> Self
  {
    assert!( bounds[ 0 ].0[ 0 ] <= bounds[ 1 ].0[ 0 ], "Incorrect bounds" );
    assert!( bounds[ 0 ].0[ 1 ] <= bounds[ 1 ].0[ 1 ], "Incorrect bounds" );
    assert!( hex_size > 0.0, "Hex size must be positive" );

    Self
    {
      hex_size,
      bounds,
      parity : PhantomData,
      orientation : PhantomData,
    }
  }
}

// const SQRT_THREE : f32 = 1.73205080757;

impl< Parity > RectangularGrid< Parity, Pointy >
where
  Coordinate< Offset< Parity >, Pointy > : Into< Coordinate< Axial, Pointy > >,
{
  pub fn center( &self ) -> F32x2
  {
    let [ min, max ] = self.bounds;

    let min1 = Coordinate::< Offset< Parity >, Pointy >::new( min[ 0 ], min[ 1 ] );
    let min1 = Into::< Coordinate< Axial, Pointy > >::into( min1 ).to_pixel( self.hex_size );
    let min_x = if min[ 1 ] + 1 <= max[ 1 ]
    {
      let min2 = Coordinate::< Offset< Parity >, Pointy >::new( min[ 0 ], min[ 1 ] + 1 );
      let min2 = Into::< Coordinate< Axial, Pointy > >::into( min2 ).to_pixel( self.hex_size );
      min1[ 0 ].min( min2[ 0 ] )
    }
    else
    {
      min1[ 0 ]
    };
    let min_y = min1[ 1 ];

    let max1 = Coordinate::< Offset< Parity >, Pointy >::new( max[ 0 ], max[ 1 ] );
    let max1 = Into::< Coordinate< Axial, Pointy > >::into( max1 ).to_pixel( self.hex_size );
    let max_x = if max[ 1 ] - 1 >= min[ 1 ]
    {
      let max2 = Coordinate::< Offset< Parity >, Pointy >::new( max[ 0 ], max[ 1 ] - 1 );
      let max2 = Into::< Coordinate< Axial, Pointy > >::into( max2 ).to_pixel( self.hex_size );
      max1[ 0 ].max( max2[ 0 ] )
    }
    else
    {
      max1[ 0 ]
    };
    let max_y = max1[ 1 ];

    F32x2::new( ( min_x + max_x ) / 2.0, ( min_y + max_y ) / 2.0 )

    // let width_count = self.bounds[ 1 ][ 0 ] - self.bounds[ 0 ][ 0 ] + 1;
    // let width = SQRT_THREE * self.hex_size;
    // let width = width_count as f32 * width + width / 2.0 * ( width_count > 1 ) as i32 as f32;
    // let height_count = self.bounds[ 1 ][ 1 ] - self.bounds[ 0 ][ 1 ] + 1;
    // let height = self.hex_size * 2.0;
    // let height = height + 0.75 * height * ( height_count - 1 ) as f32;
    // F32x2::new( width / 2.0, height / 2.0 )
  }
}

impl< Parity > RectangularGrid< Parity, Flat >
where
  Coordinate< Offset< Parity >, Flat > : Into< Coordinate< Axial, Flat > >,
{
  pub fn center( &self ) -> F32x2
  {
    let [ min, max ] = self.bounds;

    let min1 = Coordinate::< Offset< Parity >, Flat >::new( min[ 0 ], min[ 1 ] );
    let min1 = Into::< Coordinate< Axial, Flat > >::into( min1 ).to_pixel( self.hex_size );
    let min_y = if min[ 0 ] + 1 <= max[ 0 ]
    {
      let min2 = Coordinate::< Offset< Parity >, Flat >::new( min[ 0 ] + 1, min[ 1 ] );
      let min2 = Into::< Coordinate< Axial, Flat > >::into( min2 ).to_pixel( self.hex_size );
      min1[ 1 ].min( min2[ 1 ] )
    }
    else
    {
      min1[ 1 ]
    };
    let min_x = min1[ 0 ];

    let max1 = Coordinate::< Offset< Parity >, Flat >::new( max[ 0 ], max[ 1 ] );
    let max1 = Into::< Coordinate< Axial, Flat > >::into( max1 ).to_pixel( self.hex_size );
    let max_y = if max[ 0 ] - 1 >= min[ 0 ]
    {
      let max2 = Coordinate::< Offset< Parity >, Flat >::new( max[ 0 ] - 1, max[ 1 ] );
      let max2 = Into::< Coordinate< Axial, Flat > >::into( max2 ).to_pixel( self.hex_size );
      max1[ 1 ].max( max2[ 1 ] )
    }
    else
    {
      max1[ 1 ]
    };
    let max_x = max1[ 0 ];

    F32x2::new( ( min_x + max_x ) / 2.0, ( min_y + max_y ) / 2.0 )

    // let width_count = self.bounds[ 1 ].0[ 0 ] - self.bounds[ 0 ].0[ 0 ] + 1;
    // let width = self.hex_size * 2.0;
    // let width = width + 0.75 * width * ( width_count - 1 ) as f32;
    // let height_count = self.bounds[ 1 ].0[ 1 ] - self.bounds[ 0 ].0[ 1 ] + 1;
    // let height = SQRT_THREE * self.hex_size;
    // let height = height_count as f32 * height + height / 2.0 * ( height_count > 1 ) as i32 as f32;
    // F32x2::new( width / 2.0, height / 2.0 )
  }
}

impl< Parity, Orientation > RectangularGrid< Parity, Orientation >
{
  pub fn coordinates( &self ) -> impl Iterator< Item = Coordinate< Offset< Parity >, Orientation > >
  {
    let min = self.bounds[ 0 ];
    let max = self.bounds[ 1 ];
    let current = min;

    CoordinateIterator::< Parity, Orientation >
    {
      current,
      max,
      min,
      parity : PhantomData,
      orientation : PhantomData,
    }
  }
}
struct CoordinateIterator< Parity, Orientation >
{
  current : I32x2,
  max : I32x2,
  min : I32x2,
  parity : PhantomData< Parity >,
  orientation : PhantomData< Orientation >,
}

impl< Parity, Orientation > Iterator for CoordinateIterator< Parity, Orientation >
{
  type Item = Coordinate< Offset< Parity >, Orientation >;

  fn next( &mut self ) -> Option< Self::Item >
  {
    if self.current[ 1 ] <= self.max[ 1 ]
    {
      let ret = Coordinate::< Offset< _ >, _ >::new( self.current[ 0 ], self.current[ 1 ] );
      self.current[ 0 ] += 1;
      if self.current[ 0 ] > self.max[ 0 ]
      {
        self.current[ 0 ] = self.min[ 0 ];
        self.current[ 1 ] += 1;
      }
      return Some( ret );
    }
    else
    {
      None
    }
  }
}

// /// An enum that represents the orientation of the hexagons (e.g., "pointy-topped" or "flat-topped").
// #[ derive( Debug, Copy, Clone ) ]
// pub enum Orientation
// {
//   Pointy,
//   Flat,
// }

// impl Orientation
// {
//   /// Orientation angle of the hexagons in radians.
//   pub fn orientation_angle( &self ) -> f32
//   {
//     match self
//     {
//       Self::Pointy => 30.0f32.to_radians(),
//       Self::Flat => 0.0f32.to_radians(),
//     }
//   }
// }

// /// A struct that defines geometric properties of the hexagonal grid layout.
// #[ derive( Debug, Copy, Clone ) ]
// pub struct HexLayout
// {
//   /// The orientation of the hexagons in the grid.
//   pub orientation : Orientation,
//   /// Size of a hexagon, the distance from the center to a corner.
//   pub size : f32, // qqq : naming is not descriptive enough
// }

// impl HexLayout
// {
//   /// Calculates coordinates of a hexagon that contains the given pixel position.
//   ///
//   /// # Parameters
//   /// - `pixel`: The pixel coordinates.
//   ///
//   /// # Returns
//   /// A coordinate representing the hexagon.
//   pub fn hex_coord< C >( &self, pixel : Pixel ) -> C
//   where
//     C : CoordinateConversion
//   {
//     C::from_pixel( pixel, self.size )
//   }

//   /// Calculates the 2d pixel position of a hexagon center based on its coordinates.
//   ///
//   /// # Parameters
//   /// - `coord`: The coordinates of the hexagon.
//   ///
//   /// # Returns
//   /// A `Pixel` containing the x and y coordinates of the hexagon center.
//   pub fn pixel_coord< C >( &self, coord : C ) -> Pixel
//   where
//     C : CoordinateConversion
//   {
//     coord.to_pixel( self.size )
//   }

//   /// Calculates the horizontal distance between neighbor hexagons in the grid.
//   pub fn horizontal_spacing( &self ) -> f32
//   {
//     match self.orientation
//     {
//       Orientation::Pointy => pointy_layout_spacings( self.size ).0,
//       Orientation::Flat => flat_layout_spacings( self.size ).0,
//     }
//   }

//   /// Calculates the vertical distance between neighbor hexagons in the grid.
//   pub fn vertical_spacing( &self ) -> f32
//   {
//     match self.orientation
//     {
//       Orientation::Pointy => pointy_layout_spacings( self.size ).1,
//       Orientation::Flat => flat_layout_spacings( self.size ).1,
//     }
//   }

//   /// Calculates a point that lies right in the center of a grid of hexagons.
//   ///
//   /// # Parameters
//   /// - `coords`: An iterator over the coordinates of the hexagons.
//   /// - `layout`: The layout of the hexagons.
//   ///
//   /// # Returns
//   /// A tuple containing the x and y coordinates of the center of the grid.
//   pub fn grid_center< I, C >( &self, coords : I ) -> [ f32; 2 ]
//   where
//     I : Iterator< Item = C >,
//     C : CoordinateConversion
//   {
//     // TODO: split this function into bounds_calculation and center_calculation based on bounds
//     let mut min_x = f32::INFINITY;
//     let mut max_x = f32::NEG_INFINITY;
//     let mut min_y = f32::INFINITY;
//     let mut max_y = f32::NEG_INFINITY;

//     for coord in coords
//     {
//       let Pixel { data : [ x, y ] } = self.pixel_coord::<  >( coord );
//       min_x = min_x.min( x );
//       max_x = max_x.max( x );
//       min_y = min_y.min( y );
//       max_y = max_y.max( y );
//     }

//     [ min_x + ( max_x - min_x ) / 2.0, min_y + ( max_y - min_y ) / 2.0]
//   }
// }

// /// Calculates the horizontal and vertical spacings between neighbor hexagons in a pointy layout.
// fn pointy_layout_spacings( size : f32 ) -> ( f32, f32 )
// {
//   ( 3.0f32.sqrt() * size , 1.5 * size )
// }

// /// Calculates the horizontal and vertical spacings between neighbor hexagons in a flat layout.
// fn flat_layout_spacings( size : f32 ) -> ( f32, f32 )
// {
//   ( 1.5 * size, 3.0f32.sqrt() * size )
// }

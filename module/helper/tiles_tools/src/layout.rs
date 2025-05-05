use crate::coordinates::{ hexagonal, pixel };
use hexagonal::{ Coordinate, Flat, Offset, Pointy, Axial };
use pixel::Pixel;
use ndarray_cg::{ F32x2, I32x2 };
use std::marker::PhantomData;

/// RECTANGULAR LAYOUT
#[ derive( Debug ) ]
pub struct RectangularGrid< Parity, Orientation >
{
  /// Inclusive maximum and minimum coordinates of the grid.
  pub bounds : [ Coordinate< Offset< Parity >, Orientation >; 2 ],
}

impl< Parity, Orientation > Clone for RectangularGrid< Parity, Orientation >
{
  fn clone( &self ) -> Self
  {
    Self
    {
      bounds : self.bounds,
    }
  }
}

impl< Parity, Orientation > Copy for RectangularGrid< Parity, Orientation > {}

impl< Parity, Orientation > RectangularGrid< Parity, Orientation >
{
  /// Creates a new `RectLayout` instance with the specified bounds.
  ///
  /// # Parameters
  /// - `bounds`: The bounds of the layout. Minimal and maximal inclusive offset coordinates.
  ///
  /// # Returns
  /// A new `RectLayout` instance.
  pub const fn new( bounds : [ Coordinate< Offset< Parity >, Orientation >; 2 ] ) -> Self
  {
    assert!( bounds[ 0 ].q <= bounds[ 1 ].q, "Incorrect bounds" );
    assert!( bounds[ 0 ].r <= bounds[ 1 ].r, "Incorrect bounds" );

    Self
    {
      bounds,
    }
  }

  pub fn coordinates( &self ) -> impl Iterator< Item = Coordinate< Offset< Parity >, Orientation > >
  {
    let min = self.bounds[ 0 ];
    let max = self.bounds[ 1 ];
    let current = min;

    RectangularGridIterator::< Parity, Orientation >
    {
      current : current.into(),
      max : max.into(),
      min : min.into(),
      _marker : PhantomData,
    }
  }
}

impl< Parity > RectangularGrid< Parity, Pointy >
where
  Coordinate< Offset< Parity >, Pointy > : Into< Coordinate< Axial, Pointy > >,
{
  /// Position of a point right in the center of the whole grid.
  pub fn center( &self ) -> F32x2
  {
    let [ min, max ] = self.bounds;

    let min1 : Pixel = Into::< Coordinate< Axial, Pointy > >::into( min ).into();
    let min_x = if min.r + 1 <= max.r
    {
      let min2 = Coordinate::< Offset< Parity >, Pointy >::new( min.q, min.r + 1 );
      let min2 : Pixel = Into::< Coordinate< Axial, Pointy > >::into( min2 ).into();
      min1[ 0 ].min( min2[ 0 ] )
    }
    else
    {
      min1[ 0 ]
    };
    let min_y = min1[ 1 ];

    let max1 : Pixel = Into::< Coordinate< Axial, Pointy > >::into( max ).into();
    let max_x = if max.r - 1 >= min.r
    {
      let max2 = Coordinate::< Offset< Parity >, Pointy >::new( max.q, max.r - 1 );
      let max2 : Pixel = Into::< Coordinate< Axial, Pointy > >::into( max2 ).into();
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
  /// Position of a point right in the center of the whole grid.
  pub fn center( &self ) -> F32x2
  {
    let [ min, max ] = self.bounds;

    let min1 : Pixel = Into::< Coordinate< Axial, Flat > >::into( min ).into();
    let min_y = if min.r + 1 <= max.r
    {
      let min2 = Coordinate::< Offset< Parity >, Flat >::new( min.q + 1, min.r );
      let min2 : Pixel = Into::< Coordinate< Axial, Flat > >::into( min2 ).into();
      min1[ 1 ].min( min2[ 1 ] )
    }
    else
    {
      min1[ 1 ]
    };
    let min_x = min1[ 0 ];

    let max1 : Pixel = Into::< Coordinate< Axial, Flat > >::into( max ).into();
    let max_y = if max.r - 1 >= min.r
    {
      let max2 = Coordinate::< Offset< Parity >, Flat >::new( max.q - 1, max.r );
      let max2 : Pixel = Into::< Coordinate< Axial, Flat > >::into( max2 ).into();
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

struct RectangularGridIterator< Parity, Orientation >
{
  current : I32x2,
  max : I32x2,
  min : I32x2,
  _marker : PhantomData< Coordinate< Offset< Parity >, Orientation > >,
}

impl< Parity, Orientation > Iterator for RectangularGridIterator< Parity, Orientation >
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

use minwebgl as gl;

/// Encapsulated the 2d position and the index in an array
#[ derive( Debug, Clone, Default, PartialEq ) ]
pub struct Point2D( pub gl::F32x2, pub usize );


impl spart::kdtree::KdPoint for Point2D
{
  fn dims( &self ) -> usize 
  {
    2
  }

  fn coord( &self, axis: usize ) -> Result< f64, spart::errors::SpartError >
  {
    match axis
    {
      0 => Ok( f64::from( self.0.x() ) ),
      1 => Ok( f64::from( self.0.y() ) ),
      _ => Err
      ( 
        spart::errors::SpartError::InvalidDimension 
        {
          requested: axis,
          available: 2,
        }
      ),
    }
  }
}

impl spart::geometry::DistanceMetric< Point2D > for spart::geometry::EuclideanDistance
{
  #[ inline ]
  fn distance_sq( p1: &Point2D, p2: &Point2D ) -> f64
  {
    f64::from( p1.0.distance_squared( &p2.0 ) )
  }
}
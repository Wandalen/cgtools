use minwebgl as gl;

/// Encapsulated the 2d position and the index in an array
#[ derive( Debug, Clone, Default, PartialEq ) ]
pub struct Point2D( pub gl::F32x2, pub usize );


impl spart::kd_tree::KdPoint for Point2D
{
  fn dims( &self ) -> usize 
  {
    2
  }

  fn coord( &self, axis: usize ) -> Result< f64, spart::exceptions::SpartError > 
  {
    match axis 
    {
      0 => Ok( self.0.x() as f64 ),
      1 => Ok( self.0.y() as f64 ),
      _ => Err
      ( 
        spart::exceptions::SpartError::InvalidDimension 
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
  fn distance_sq( p1: &Point2D, p2: &Point2D ) -> f64 
  {
    p1.0.distance_squared( &p2.0 ) as f64   
  }
}
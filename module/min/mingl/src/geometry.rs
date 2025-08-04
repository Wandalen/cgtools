mod private
{
  use crate::*;

  #[ derive( Debug, Clone, Copy ) ]
  pub struct BoundingBox
  {
    pub min : F32x3,
    pub max : F32x3
  }

  impl Default for BoundingBox
  {
    fn default() -> Self
    {
      BoundingBox
      {
        min : F32x3::MAX,
        max : F32x3::MIN
      }
    }
  }

  impl BoundingBox
  {
    // Create a bounding box from the min and max
    pub fn new< T : Into< F32x3 > >( min : T, max : T ) -> Self
    {
      Self
      {
        min : min.into(),
        max : max.into()
      }
    }

    pub fn center( &self ) -> F32x3
    {
      ( self.max + self.min ) / 2.0
    }

    /// Computes the bounding box of the model from the provided positions array
    /// Positions should be in the form [ x, y, z, x, y, z, ...]
    pub fn compute( positions : &[ f32 ] ) -> Self
    {
      let mut bounding_box = BoundingBox::default();

      for i in 0..positions.len() / 3
      {
        let x = positions[ i * 3 + 0 ];
        let y = positions[ i * 3 + 1 ];
        let z = positions[ i * 3 + 2 ];

        let p = F32x3::new( x, y, z );

        bounding_box.min = p.min( bounding_box.min );
        bounding_box.max = p.max( bounding_box.max );
      }

      bounding_box
    }

    /// Computes the bounding box of the model from the provided 2D positions array
    /// Positions should be in the form [ x, y, x, y, ... ]
    pub fn compute2d( positions : &[ f32 ] ) -> Self
    {
      let mut bounding_box = BoundingBox::default();

      for i in 0..positions.len() / 2
      {
        let x = positions[ i * 2 + 0 ];
        let y = positions[ i * 2 + 1 ];

        let p = F32x3::new( x, y, 0.0 );

        bounding_box.min = p.min( bounding_box.min );
        bounding_box.max = p.max( bounding_box.max );
      }

      bounding_box
    }

    pub fn combine( mut self, other : &BoundingBox ) -> Self
    {
      self.combine_mut( other );
      self
    }

    pub fn combine_mut( &mut self, other : &BoundingBox )
    {
      self.min = self.min.min( other.min );
      self.max = self.max.max( other.max );
    }

    pub fn apply_transform( mut self, transform : F32x4x4 ) -> Self
    {
      self.apply_transform_mut( transform );
      self
    }

    pub fn apply_transform_mut( &mut self, transform : F32x4x4 )
    {
      let mut points : [ F32x4; 8 ] = Default::default();
      points[ 0 ] = transform * self.min.to_homogenous();
      points[ 1 ] = transform * F32x3::new( self.min.x(), self.max.y(), self.min.z() ).to_homogenous();
      points[ 2 ] = transform * F32x3::new( self.max.x(), self.max.y(), self.min.z() ).to_homogenous();
      points[ 3 ] = transform * F32x3::new( self.max.x(), self.min.y(), self.min.z() ).to_homogenous();

      points[ 4 ] = transform * self.max.to_homogenous();
      points[ 5 ] = transform * F32x3::new( self.max.x(), self.min.y(), self.max.z() ).to_homogenous();
      points[ 6 ] = transform * F32x3::new( self.min.x(), self.min.y(), self.max.z() ).to_homogenous();
      points[ 7 ] = transform * F32x3::new( self.min.x(), self.max.y(), self.max.z() ).to_homogenous();

      let mut min = F32x4::MAX;
      let mut max = F32x4::MIN;

      for p in points.iter()
      {
        min = min.min( *p );
        max = max.max( *p );
      }

      self.min = min.truncate();
      self.max = max.truncate();
    }

    pub fn left( &self ) -> f32
    {
      self.min.x()
    }

    pub fn right( &self ) -> f32
    {
      self.max.x()
    }

    pub fn down( &self ) -> f32
    {
      self.min.y()
    }

    pub fn up( &self ) -> f32
    {
      self.max.y()
    }

    pub fn width( &self ) -> f32
    {
      ( self.left() - self.right() ).abs()
    }

    pub fn height( &self ) -> f32
    {
      ( self.up() - self.down() ).abs()
    }
  }

  #[ derive( Debug ) ]
  pub struct BoundingSphere
  {
    pub center : F32x3,
    pub radius : f32
  }

  impl Default for BoundingSphere
  {
    fn default() -> Self
    {
      BoundingSphere
      {
        center : F32x3::ZERO,
        radius : 0.0
      }
    }
  }

  impl BoundingSphere
  {
    pub fn new< T : Into< F32x3 > >( center : T, radius : f32 ) -> Self
    {
      Self
      {
        center : center.into(),
        radius
      }
    }

    /// Computes the bounding sphere of the model form the provided positions array.
    /// Positions should be in the form [ x, y, z, x, y, z, ...].
    /// Requires BoundingBox to be computed first.
    pub fn compute( positions : &[ f32 ], bounding_box : &BoundingBox ) -> Self
    {
      let mut bs = BoundingSphere::default();
      bs.center =  bounding_box.center();

      for i in 0..positions.len() / 3
      {
        let x = positions[ i * 3 + 0 ];
        let y = positions[ i * 3 + 1 ];
        let z = positions[ i * 3 + 2 ];
        let p = ndarray_cg::F32x3::new( x, y, z );

        bs.radius = bs.center.distance_squared( &p ).max( bs.radius );
      }

      bs.radius = bs.radius.sqrt();

      bs
    }
  }
}

crate::mod_interface!
{
  own use
  {
    BoundingBox,
    BoundingSphere
  };
}

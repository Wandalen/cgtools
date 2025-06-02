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

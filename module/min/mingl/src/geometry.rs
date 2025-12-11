//! This module provides structures and methods for handling 3D bounding volumes,
//! specifically axis-aligned bounding boxes (AABB) and bounding spheres.
//! These are commonly used for optimizations like frustum culling and collision detection.

/// Internal namespace for implementation details.
mod private
{
  use crate::*;

  /// Represents a 3D axis-aligned bounding box (AABB).
  ///
  /// An AABB is defined by its minimum and maximum corner points.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct BoundingBox
  {
    /// The corner of the box with the smallest x, y, and z coordinates.
    pub min : F32x3,
    /// The corner of the box with the largest x, y, and z coordinates.
    pub max : F32x3
  }

  impl Default for BoundingBox
  {
    /// Creates a default, inverted bounding box.
    ///
    /// The `min` is set to positive infinity and `max` to negative infinity,
    /// which is useful as a starting point for computing a new bounding box.
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
    /// Creates a new bounding box from two corner points.
    pub fn new< T : Into< F32x3 > >( min : T, max : T ) -> Self
    {
      Self
      {
        min : min.into(),
        max : max.into()
      }
    }

    /// Calculates the geometric center of the bounding box.
    pub fn center( &self ) -> F32x3
    {
      ( self.max + self.min ) / 2.0
    }

    /// Computes the bounding box for a set of 3D vertices.
    ///
    /// # Arguments
    /// * `positions` - A slice of `f32` where vertices are laid out sequentially as `[x, y, z, x, y, z, ...]`.
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

    /// Mutably computes the bounding box for a set of 3D vertices.
    ///
    /// # Arguments
    /// * `positions` - A slice of `f32` where vertices are laid out sequentially as `[x, y, z, x, y, z, ...]`.
    pub fn compute_mut( &mut self, positions : &[ f32 ] )
    {
      *self = Self::compute( positions );
    }

    /// Computes the bounding box for a set of 2D vertices, with z-component as 0.
    ///
    /// # Arguments
    /// * `positions` - A slice of `f32` where vertices are laid out sequentially as `[x, y, x, y, ...]`.
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

    /// Creates a new bounding box that encompasses both this one and another.
    pub fn combine( mut self, other : &BoundingBox ) -> Self
    {
      self.combine_mut( other );
      self
    }

    /// Expands this bounding box to also encompass another one.
    pub fn combine_mut( &mut self, other : &BoundingBox )
    {
      self.min = self.min.min( other.min );
      self.max = self.max.max( other.max );
    }

    /// Returns a new bounding box that is the result of applying a transformation to this one.
    pub fn apply_transform( mut self, transform : F32x4x4 ) -> Self
    {
      self.apply_transform_mut( transform );
      self
    }

    /// Applies a transformation to this bounding box, recalculating its min and max points.
    ///
    /// This is done by transforming all 8 corners of the box and finding the new min/max.
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

    /// Returns the minimum x-coordinate of the box.
    pub fn left( &self ) -> f32
    {
      self.min.x()
    }

    /// Returns the maximum x-coordinate of the box.
    pub fn right( &self ) -> f32
    {
      self.max.x()
    }

    /// Returns the minimum y-coordinate of the box.
    pub fn down( &self ) -> f32
    {
      self.min.y()
    }

    /// Returns the maximum y-coordinate of the box.
    pub fn up( &self ) -> f32
    {
      self.max.y()
    }

    /// Calculates the width of the bounding box (difference in x-coordinates).
    pub fn width( &self ) -> f32
    {
      ( self.left() - self.right() ).abs()
    }

    /// Calculates the height of the bounding box (difference in y-coordinates).
    pub fn height( &self ) -> f32
    {
      ( self.up() - self.down() ).abs()
    }
  }

  /// Represents a bounding sphere in 3D space, defined by a center and radius.
  #[ derive( Debug ) ]
  pub struct BoundingSphere
  {
    /// The center point of the sphere.
    pub center : F32x3,
    /// The radius of the sphere.
    pub radius : f32
  }

  impl Default for BoundingSphere
  {
    /// Creates a default bounding sphere at the origin with a radius of zero.
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
    /// Creates a new bounding sphere from a center point and a radius.
    pub fn new< T : Into< F32x3 > >( center : T, radius : f32 ) -> Self
    {
      Self
      {
        center : center.into(),
        radius
      }
    }

    /// Computes a bounding sphere for a set of 3D vertices.
    ///
    /// This method uses the center of the provided `bounding_box` and finds the
    /// maximum squared distance to any vertex to determine the radius.
    ///
    /// # Arguments
    /// * `positions` - A slice of `f32` where vertices are laid out as `[x, y, z, x, y, z, ...]`.
    /// * `bounding_box` - A pre-computed `BoundingBox` for the same set of vertices.
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

    /// Mutably computes a bounding sphere for a set of 3D vertices.
    ///
    /// This method uses the center of the provided `bounding_box` and finds the
    /// maximum squared distance to any vertex to determine the radius.
    ///
    /// # Arguments
    /// * `positions` - A slice of `f32` where vertices are laid out as `[x, y, z, x, y, z, ...]`.
    /// * `bounding_box` - A pre-computed `BoundingBox` for the same set of vertices.
    pub fn compute_mut( &mut self, positions : &[ f32 ], bounding_box : &BoundingBox )
    {
      *self = Self::compute( positions, bounding_box );
    }
  }
}

// This macro exposes the public interface of the module.
crate::mod_interface!
{
  /// Exposes the `BoundingBox` and `BoundingSphere` structs for public use.
  own use
  {
    BoundingBox,
    BoundingSphere
  };
}

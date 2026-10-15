mod private
{
  use crate::{Vector, MatEl, VectorIter};
  //use vector::arithmetics::inner_product::*;

  impl< E > Vector< E, 4 >
  where
    E : MatEl,
  {

    /// Create a new vector
    #[ inline( always ) ]
    pub const fn new( x : E, y : E, z : E, w : E ) -> Self
    {
      Self( [ x, y, z, w ] )
    }

    /// The `x` component of vector
    #[ inline ]
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    /// The `y` component of vector
    #[ inline ]
    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

    /// The `z` component of vector
    #[ inline ]
    pub fn z( &self ) -> E
    {
      self.0[ 2 ]
    }

    /// The `w` component of vector
    // Fix(BUG-043): was self.0[ 2 ] — copy-pasted from z() with the index never bumped to 3.
    // Root cause: w() authored directly below z() in the same commit, by copying its body.
    // Pitfall: a copy-adjacent accessor is exactly the pattern most likely to carry a silent
    // stale-index slip — always verify a copied index against the field it now names.
    #[ inline ]
    pub fn w( &self ) -> E
    {
      self.0[ 3 ]
    }

    /// Truncates `w` component of a vector creating vector of 3 elements
    #[ inline ]
    pub fn truncate( &self ) -> Vector< E, 3 >
    {
      Vector::< E, 3 >::new( self.x(), self.y(), self.z() )
    }
  }

  impl< E, Vec2 > From< ( Vec2, Vec2 ) > for Vector< E, 4 >
  where
  Vec2 : VectorIter< E, 2 >,
  E : MatEl
  {
    #[ inline ]
    fn from( value: ( Vec2, Vec2 ) ) -> Self
    {
      let mut iter1 = value.0.vector_iter();
      let mut iter2 = value.1.vector_iter();
      let x = *iter1.next().unwrap();
      let y = *iter1.next().unwrap();
      let z = *iter2.next().unwrap();
      let w = *iter2.next().unwrap();

      Self( [ x, y, z, w ] )
    }
  }
}

crate::mod_interface!
{
}

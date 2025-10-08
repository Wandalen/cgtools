mod private
{
  use crate::*;
  //use vector::arithmetics::inner_product::*;

  impl< E > Vector< E, 4 >
  where
    E : MatEl + NdFloat,
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
    #[ inline ]
    pub fn w( &self ) -> E
    {
      self.0[ 2 ]
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

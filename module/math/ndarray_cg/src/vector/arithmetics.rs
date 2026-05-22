/// Internal namespace.
mod private
{
  use crate::*;
  // use vector::arithmetics::inner_product::*;
  // use vector::arithmetics::{ normalized, mag };
  use vector::{ normalized, mag, mag2, min, max, dot };

  impl< E : MatNum, const LEN : usize > Vector< E, LEN >
  {

    /// Compute the squared length of the vector. Available for any numeric
    /// scalar (integers included) since it does not require `sqrt`.
    pub fn mag2( &self ) -> E
    {
      mag2( self )
    }

    /// Compute squared length of the vector between two points in space.
    /// Available for any numeric scalar.
    pub fn distance_squared( &self, rhs : &Self ) -> E
    {
      mag2( &( *self - *rhs ) )
    }

    /// Compute the dot product of two vectors.
    pub fn dot( &self, rhs : &Self ) -> E
    {
      dot( self, rhs )
    }
  }

  impl< E : MatEl + NdFloat, const LEN : usize > Vector< E, LEN >
  {

    /// Normalize the vector. Requires float scalar (uses `sqrt`).
    pub fn normalize( self ) -> Self
    {
      normalized( &self )
    }

    /// Compute the length of the vector. Requires float scalar (uses `sqrt`).
    pub fn mag( &self ) -> E
    {
      mag( self )
    }

    /// Compute a vector whose elements are the minimum of both vectors:
    /// `r[ i ] = a[ i ].min( b[ i ] )`. Currently float-only.
    pub fn min( self, rhs : Self ) -> Self
    {
      min( &self, &rhs )
    }

    /// Compute a vector whose elements are the maximum of both vectors:
    /// `r[ i ] = a[ i ].max( b[ i ] )`. Currently float-only.
    pub fn max( self, rhs : Self ) -> Self
    {
      max( &self, &rhs )
    }

    /// Compute length of the vector between two points in space. Requires
    /// float scalar (uses `sqrt`).
    pub fn distance( &self, rhs : &Self ) -> E
    {
      ( rhs - self ).mag()
    }
  }

}

crate::mod_interface!
{
  // xxx : reuse
  reuse ::mdmath_core::vector::arithmetics;
}

/// Internal namespace.
mod private
{
  use crate::*;
  use vector::arithmetics::inner_product::*;

  impl< E : MatEl + NdFloat, const LEN : usize > Vector< E, LEN >
  {
    /// Normalizes the vector
    pub fn normalize( self ) -> Self
    {
      normalized( &self )
    }

    /// Compute the length of the vector
    pub fn mag( &self ) -> E
    {
      mag( self )
    }

    /// Computer the squared length of the vector
    pub fn mag2( &self ) -> E
    {
      mag2( self )
    }

    /// Compute a vector, whose elements are minimum of both vectors: `r[ i ] = a[ i ].min( b [ i ] )`
    pub fn min( self, rhs : Self ) -> Self
    {
      min( &self, &rhs )
    }

    /// Compute a vector, whose elements are maximum of both vectors: `r[ i ] = a[ i ].max( b [ i ] )`
    pub fn max( self, rhs : Self ) -> Self
    {
      max( &self, &rhs )
    }

    /// Computes length of the vector between two points in space
    pub fn distance( &self, rhs: &Self ) -> E
    {
      ( rhs - self ).mag()
    }

    /// Computes squared length of the vector between two points in space
    pub fn distance_squared( &self, rhs: &Self ) -> E
    {
      ( rhs - self ).mag2()
    }
  }

}

crate::mod_interface!
{
  own use ::mdmath_core::vector::inner_product;

  /// Mul trait implementations
  layer mul;
  /// Sub trait implementations
  layer sub;
  /// Add trait implementations
  layer add;
  /// Div trait implementations
  layer div;
}
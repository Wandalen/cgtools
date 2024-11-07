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
  }

}

crate::mod_interface!
{
  own use ::mdmath_core::vector::inner_product;

  layer mul;
  layer sub;
  layer add;
  layer div;
}

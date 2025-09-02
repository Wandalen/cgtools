mod private
{
  use crate::*;
  //use vector::arithmetics::inner_product::*;

  impl< E > Vector< E, 2 >
  where
    E : MatEl + NdFloat,
  {

    /// Create a new vector
    #[ inline( always ) ]
    pub const fn new( x : E, y : E ) -> Self
    {
      Self( [ x, y ] )
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

    /// Calculates vector norm
    pub fn norm( &self ) -> E
    {
      let n = self.x().powi( 2 ) +
      self.y().powi( 2 );

      n.sqrt()
    }
  }

}

crate::mod_interface!
{

}

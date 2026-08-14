mod private
{
  use crate::{Vector, MatEl};

  impl< E > Vector< E, 1 >
  where
    E : MatEl,
  {

    /// Create a new vector
    #[ inline( always ) ]
    pub const fn new( x : E ) -> Self
    {
      Self( [ x ] )
    }

    /// The `x` component of vector
    #[ inline ]
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }
  }

}

crate::mod_interface!
{
}

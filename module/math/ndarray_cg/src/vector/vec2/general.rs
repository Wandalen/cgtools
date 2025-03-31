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

    #[ inline ]
    pub fn x( &self ) -> E
    {
      self.0[ 0 ]
    }

    #[ inline ]
    pub fn y( &self ) -> E
    {
      self.0[ 1 ]
    }

  }

  // qqq : xxx : enable please
  // impl< E > IntoVector< E, 2 > for T
  // where
  //   E : MatEl,
  //   T : VectorWithLength< 2 >
  // {
  //   fn into_vector( self ) -> Vector< E, 2 >
  //   {
  //     Vector::< E, 2 >( [ self.0, self.1 ] )
  //   }
  // }

  // tuple

  impl< E > IntoVector< E, 2 > for ( E, E )
  where
    E : MatEl
  {
    fn into_vector( self ) -> Vector< E, 2 >
    {
      Vector::< E, 2 >( [ self.0, self.1 ] )
    }
  }

  impl< E > AsVector< E, 2 > for ( E, E )
  where
    E : MatEl
  {
    fn as_vector( &self ) -> Vector< E, 2 >
    {
      Vector::< E, 2 >( [ self.0, self.1 ] )
    }
  }

  impl< E > FromVector< ( E, E ), E, 2 > for Vector< E, 2 >
  where
    E : MatEl
  {
    fn from_vector( self ) -> ( E, E )
    {
      ( self.0[ 0 ], self.0[ 1 ] )
    }
  }

  // array

  impl< E > IntoVector< E, 2 > for [ E ; 2 ]
  where
    E : MatEl
  {
    fn into_vector( self ) -> Vector< E, 2 >
    {
      Vector::< E, 2 >( self )
    }
  }

  impl< E > AsVector< E, 2 > for [ E ; 2 ]
  where
    E : MatEl
  {
    fn as_vector( &self ) -> Vector< E, 2 >
    {
      Vector::< E, 2 >( [ self[ 0 ], self[ 1 ] ] )
    }
  }

  impl< E > FromVector< [ E ; 2 ], E, 2 > for Vector< E, 2 >
  where
    E : MatEl
  {
    fn from_vector( self ) -> [ E ; 2 ]
    {
      self.0
    }
  }

}

crate::mod_interface!
{

}

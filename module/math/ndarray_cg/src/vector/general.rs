mod private
{
  use crate::*;

  // qqq : xxx : document please

  impl< E, const N : usize > Vector< E, N >
  where
    E : MatEl,
  {

    /// Creates a vector from a single value : [ v ; N ]
    #[ inline( always ) ]
    pub const fn splat( v : E ) -> Self
    {
        Vector::< E, N >( [ v; N ] )
    }

    pub fn to_array( &self ) -> [ E; N ]
    {
      self.0
    }

  }

  impl< E, const N : usize > Default for Vector< E, N >
  where
    E : MatEl + Default,
  {
    #[ inline( always ) ]
    fn default() -> Self
    {
      Vector( [ E::default() ; N ] )
    }
  }

  impl< E, const N : usize > Collection for Vector< E, N >
  where
    E : MatEl,
  {
    type Scalar = E;
  }

  impl< E, const N : usize > ConstLength for Vector< E, N >
  where
    E : MatEl,
  {
    const LEN : usize = N;
  }

  impl< E, const N : usize > ArrayRef< E, N > for Vector< E, N >
  where
    E : MatEl,
  {
    #[ inline( always ) ]
    fn array_ref( &self ) -> &[ E ; N ]
    {
      &self.0
    }
  }

  impl< E, const N : usize > ArrayMut< E, N > for Vector< E, N >
  where
    E : MatEl,
  {
    #[ inline( always ) ]
    fn vector_mut( &mut self ) -> &mut [ E ; N ]
    {
      &mut self.0
    }
  }

  impl< E, const N : usize > VectorIter< E, N > for Vector< E, N >
  where
    E : MatEl,
  {
    fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
    where
      E : 'a,
    {
      <[ E ]>::iter( &self.0 )
    }
  }

  impl< E, const N : usize > VectorIterMut< E, N > for Vector< E, N >
  where
    E : MatEl,
  {
    fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut E >
    where
      E : 'a,
    {
      <[ E ]>::iter_mut( &mut self.0 )
    }
  }

  impl< E, const N : usize > TryFrom< &[ E ] > for Vector< E, N >
  where
    E : MatEl,
  {
    type Error = &'static str;
    // qqq : xxx : use typed error
    fn try_from( value : &[ E ] ) -> Result< Self, Self::Error >
    {
      if value.len() != N
      {
        return Err( "Slice length does not equal vector's length" );
      }
      Ok( Self( value.try_into().unwrap() ) )
    }
  }

  impl< E : MatEl, const N : usize > From< [ E; N ] > for Vector< E, N >
  {
    fn from( value: [ E; N ] ) -> Self
    {
      Vector( value )
    }
  }

  impl< E : MatEl, const N : usize > From< Vector< E, N > > for [ E; N ]
  {
    fn from( value: Vector< E, N > ) -> Self
    {
      value.0
    }
  }

  impl< E, const N : usize > From< E > for Vector< E, N >
  where
    E : MatEl
  {
    fn from ( value: E ) -> Self
    {
      Self::from( [ value; N ] )
    }
  }

  // xxx : enable and test cover
  pub trait IntoVector< E, const N : usize >
  where
    E : MatEl,
  {
    fn into_vector( self ) -> Vector< E, N >;
    fn as_vector( &self ) -> Vector< E, N >
    where
      Self : Clone,
    {
      self.clone().into_vector()
    }
  }

  impl< T, E, const N : usize > IntoVector< E, N > for T
  where
    E : MatEl,
    T : IntoArray< E, N >,
  {
    fn into_vector( self ) -> Vector< E, N >
    {
      Vector::< E, N >( self.into_array() )
    }
  }

  // // xxx : enable and test cover
  // pub trait AsVector< E, const N : usize >
  // where
  //   E : MatEl,
  // {
  //   fn as_vector( &self ) -> Vector< E, N >;
  // }

  // xxx : enable and test cover
  pub trait FromVector< Dst, E, const N : usize >
  where
    E : MatEl,
    // Self : Vector< E, N >,
  {
    fn from_vector( self ) -> Dst;
  }

  // xxx : enable?
  // impl< E, Src, const N : usize > From< Src > for Vector< E, N >
  // where
  //   E : MatEl,
  //   Src : VectorIter< E, N >
  // {
  //   fn from( value: Src ) -> Self
  //   {
  //     Self::default()
  //     // Self( *value.array_ref() )
  //   }
  // }

  pub trait VectorSpace< const SIZE : usize >
  where
    Self : Collection + Indexable + VectorIter< < Self as Collection >::Scalar, SIZE >,
  {
  }

  impl< T, const SIZE : usize > VectorSpace< SIZE > for T
  where
    Self : Collection + Indexable + VectorIter< < Self as Collection >::Scalar, SIZE >,
  {
  }

  pub trait VectorSpaceMut< const SIZE : usize >
  where
    Self : VectorSpace< SIZE > + VectorIterMut< < Self as Collection >::Scalar, SIZE >,
  {
  }

  impl< T, const SIZE : usize > VectorSpaceMut< SIZE > for T
  where
    Self : VectorSpace< SIZE > + VectorIterMut< < Self as Collection >::Scalar, SIZE >,
  {
  }

}

crate::mod_interface!
{

  exposed use
  {

    IntoVector,
    // AsVector,
    FromVector,

    VectorSpace,
    VectorSpaceMut,

  };

}

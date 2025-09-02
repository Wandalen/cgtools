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
      Vector::< E, N >( [ v ; N ] )
    }

    /// Return underlying array data
    pub fn to_array( &self ) -> [ E ; N ]
    {
      self.0
    }

    /// Creates vector from given raw array
    pub fn from_array( src : [ E ; N ] ) -> Self
    {
      Self( src )
    }

    /// Creates vector from given raw slice. Assumes slice's length is equal to vector size
    ///
    /// # Panics
    ///
    /// Panics if `src` lenght does not match vector size
    pub fn from_slice( src : &[ E ] ) -> Self
    {
      assert_eq!( src.len(), N );
      Self
      (
        < [ E; N ] as core::convert::TryFrom< &[ E ] > >::try_from( src )
        .expect( &format!( "Slice length does not match array length : {} <> {}", src.len(), N ) )
      )
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

  // impl< E, const N : usize > From< E > for Vector< E, N >
  // where
  //   E : MatEl
  // {
  //   fn from ( value: E ) -> Self
  //   {
  //     Self::from( [ value; N ] )
  //   }
  // }

  // xxx : enable and test cover
  /// A trait for types that can be converted into a `Vector`.
  ///
  /// This provides a common interface for various data structures
  /// to be transformed into a `Vector` type.
  pub trait IntoVector< E, const N : usize >
  where
    E : MatEl,
  {
    /// Consumes the object and converts it into a `Vector<E, N>`.
    fn into_vector( self ) -> Vector< E, N >;

    /// Creates a `Vector<E, N>` by cloning the object first.
    ///
    /// This is a convenience method that allows conversion without consuming the original object,
    /// for types that implement `Clone`.
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

  // // xxx : enable and test cover, maybe
  // pub trait FromVector< Dst, E, const N : usize >
  // where
  //   E : MatEl,
  //   // Self : Vector< E, N >,
  // {
  //   fn from_vector( self ) -> Dst;
  // }

  /// A marker trait that groups together the essential immutable behaviors of a fixed-size vector.
  ///
  /// This trait combines `Collection`, `Indexable`, and `VectorIter`, providing a convenient
  /// bound for generic functions that operate on vector-like types. Any type that satisfies
  /// these bounds will automatically implement `VectorSpace`.
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

  /// A marker trait that extends `VectorSpace` with mutable iteration capabilities.
  ///
  /// This provides a convenient bound for generic functions that require mutable access
  /// to the elements of a vector-like type. Any type that implements `VectorSpace` and
  /// `VectorIterMut` will automatically implement `VectorSpaceMut`.
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

  impl< E, const SIZE : usize > AbsDiffEq for Vector< E, SIZE >
  where
    E : AbsDiffEq + MatEl,
    E::Epsilon : Copy,
  {
    type Epsilon = < [ E ] as AbsDiffEq< [ E ] > >::Epsilon;

    fn default_epsilon() -> Self::Epsilon
    {
      E::default_epsilon()
    }

    fn abs_diff_eq( &self, other: &Self, epsilon: Self::Epsilon ) -> bool
    {
      < [ E ] as AbsDiffEq< [ E ] > >::abs_diff_eq( &self.0, &other.0, epsilon )
    }
  }

  impl< E, const SIZE : usize > RelativeEq for Vector< E, SIZE >
  where
    E : RelativeEq + MatEl,
    E::Epsilon : Copy,
  {
    fn default_max_relative() -> Self::Epsilon
    {
      E::default_max_relative()
    }

    fn relative_eq( &self, other: &Self, epsilon: Self::Epsilon, max_relative: Self::Epsilon ) -> bool
    {
      < [ E ] as RelativeEq< [ E ] > >::relative_eq( &self.0, &other.0, epsilon, max_relative )
    }
  }

  impl< E, const SIZE : usize > UlpsEq for Vector< E, SIZE >
  where
    E : UlpsEq + MatEl,
    E::Epsilon : Copy,
  {
    fn default_max_ulps() -> u32
    {
      E::default_max_ulps()
    }

    fn ulps_eq( &self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32 ) -> bool
    {
      < [ E ] as UlpsEq< [ E ] > >::ulps_eq( &self.0, &other.0, epsilon, max_ulps )
    }
  }

}

crate::mod_interface!
{

  exposed use
  {

    IntoVector,
    // AsVector,
    // FromVector,

    VectorSpace,
    VectorSpaceMut,

  };

}

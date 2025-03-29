//! Provides traits and implementations for working with vectors and collections of scalars.
//! This module is useful for operations that require fixed-size arrays and compile-time length information.

/// Internal namespace.
mod private
{

  // =

  /// A trait for collections of scalars.
  ///
  /// This trait defines an associated type `Scalar`, representing the type of elements
  /// in the collection.
  pub trait Collection
  {
    /// The scalar type contained in the collection.
    type Scalar;
  }

  /// Implementation of `Collection` for references to collections.
  impl< T > Collection for &T
  where
    T : Collection,
  {
    type Scalar = T::Scalar;
  }

  /// Implementation of `Collection` for mutable references to collections.
  impl< T > Collection for &mut T
  where
    T : Collection,
  {
    type Scalar = T::Scalar;
  }

  // =

  /// A trait implemented for entities with known length at compile-time.
  ///
  /// This trait defines a constant `LEN`, representing the length of the entity.
  pub trait ConstLength
  {
    /// The length of the entity.
    const LEN : usize;
  }

  /// Implementation of `ConstLength` for references to entities.
  impl< T > ConstLength for &T
  where
    T : ConstLength,
  {
    const LEN : usize = T::LEN;
  }

  /// Implementation of `ConstLength` for mutable references to entities.
  impl< T > ConstLength for &mut T
  where
    T : ConstLength,
  {
    const LEN : usize = T::LEN;
  }

  // =

  // xxx : implement it for scalar interpreted as vector
  // xxx : implement it for all vectors

  /// A trait indicate that entity in case of referencing it can be interpreted as such having specified length `LEN`.
  ///
  /// This trait defines a constant `LEN`, representing the length of the entity.
  pub trait VectorWithLength< const LEN : usize >
  {
  }

  /// Implementation of `VectorWithLength` for references to entities.
  impl< T, const LEN : usize > VectorWithLength< LEN > for &T
  where
    T : VectorWithLength< LEN >,
  {
  }

  /// Implementation of `VectorWithLength` for mutable references to entities.
  impl< T, const LEN : usize > VectorWithLength< LEN > for &mut T
  where
    T : VectorWithLength< LEN >,
  {
  }

  // =

  // xxx : implement it for all vectors

  /// A trait indicate that entity in case of mutable referencing it can be interpreted as such having specified length `LEN`.
  ///
  /// This trait defines a constant `LEN`, representing the length of the entity.
  pub trait VectorWithLengthMut< const LEN : usize >
  where
    Self : VectorWithLength< LEN >,
  {
  }

  /// Implementation of `VectorWithLengthMut` for references to entities.
  impl< T, const LEN : usize > VectorWithLengthMut< LEN > for &T
  where
    Self : VectorWithLength< LEN > + VectorWithLengthMut< LEN > +,
  {
  }

  /// Implementation of `VectorWithLengthMut` for mutable references to entities.
  impl< T, const LEN : usize > VectorWithLengthMut< LEN > for &mut T
  where
    Self : VectorWithLength< LEN > + VectorWithLengthMut< LEN >,
  {
  }

  // =

  /// A trait for accessing a reference to a fixed-size array from a collection.
  ///
  /// This trait requires the collection to implement `Collection`.
  pub trait VectorRef< E, const N : usize >
  {
    /// Returns a reference to a fixed-size array from the collection.
    ///
    /// # Returns
    /// - `&[ E ; N ]`: A reference to the fixed-size array.
    fn vector_ref( &self ) -> &[ E ; N ];
  }

  /// Implementation of `VectorRef` for references to collections.
  impl< T, E, const N : usize > VectorRef< E, N > for &T
  where
    T : VectorRef< E, N >,
  {
    fn vector_ref( &self ) -> &[ E; N ]
    {
      < T as VectorRef< E, N > >::vector_ref( self )
    }
  }

  /// A trait for accessing a mutable reference to a fixed-size array from a collection.
  ///
  /// This trait requires the collection to implement `Collection`.
  pub trait VectorMut< E, const N : usize >
  {
    /// Returns a mutable reference to a fixed-size array from the collection.
    ///
    /// # Returns
    /// - `&mut [ E; N ]`: A mutable reference to the fixed-size array.
    fn vector_mut( &mut self ) -> &mut [ E ; N ];
  }

  /// Implementation of `VectorMut` for mutable references to collections.
  impl< T, E, const N : usize > VectorMut< E, N > for &mut T
  where
    T : VectorMut< E, N >,
  {
    fn vector_mut( &mut self ) -> &mut [ E; N ]
    {
      <T as VectorMut< E, N > >::vector_mut( self )
    }
  }

  /// Trait that encapsulates an vector elements iterator with specific characteristics and implemetning `CloneDyn`.
  pub trait VectorIterator< 'a, E >
  where
    E : 'a,
    Self : Iterator< Item = E > + ExactSizeIterator< Item = E > + DoubleEndedIterator,
    // Self : clone_dyn_types::CloneDyn,
  {
  }

  impl< 'a, E, T > VectorIterator< 'a, E > for T
  where
    E : 'a,
    Self : Iterator< Item = E > + ExactSizeIterator< Item = E > + DoubleEndedIterator,
    // Self : clone_dyn_types::CloneDyn,
  {
  }

  /// Trait that encapsulates an vector elements iterator with specific characteristics and implemetning `CloneDyn`.
  pub trait VectorIteratorRef< 'a, E >
  where
    E : 'a,
    Self : VectorIterator< 'a, E > + clone_dyn_types::CloneDyn,
  {
  }

  impl< 'a, E, T > VectorIteratorRef< 'a, E > for T
  where
    E : 'a,
    Self : VectorIterator< 'a, E > + clone_dyn_types::CloneDyn,
  {
  }

  /// Trait to get iterator over elements of a vector. Should be implemented even for scalars.
  pub trait VectorIter< E, const N : usize >
  {
    fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
    where
      E : 'a;
  }

  /// Implementation of `VectorRef` for references to collections.
  impl< T, E, const N : usize > VectorIter< E, N > for &T
  where
    T : VectorIter< E, N >,
  {
    fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
    where E : 'a
    {
      < T as VectorIter< E, N > >::vector_iter( self )
    }
  }

  /// Implementation of `VectorRef` for references to collections.
  impl< T, E, const N : usize > VectorIter< E, N > for &mut T
  where
    T : VectorIter< E, N >,
  {
    fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
    where E : 'a
    {
      < T as VectorIter< E, N > >::vector_iter( self )
    }
  }

  /// Trait to get iterator over elements of a vector.
  pub trait VectorIterMut< E, const N : usize >
  where
    Self : VectorIter< E, N >,
  {
    fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut E >
    where
      E : 'a;
  }

  /// Implementation of `VectorRef` for references to collections.
  impl< T, E, const N : usize > VectorIterMut< E, N > for &mut T
  where
    T : VectorIterMut< E, N > + VectorIter< E, N >,
  {
    fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut E >
    where E : 'a
    {
      < T as VectorIterMut< E, N > >::vector_iter_mut( self )
    }
  }

}

mod array;
#[ cfg( feature = "index" ) ]
mod index;
mod slice;

mod tuple0;
mod tuple1;
mod tuple2;
mod tuple3;
mod tuple4;

crate::mod_interface!
{

  /// Inner product and everithing caused by that.
  #[ cfg( feature = "inner_product" ) ]
  layer inner_product;

  /// Float functions for a vector.
  #[ cfg( feature = "float" ) ]
  layer float;

  exposed use
  {
    Collection,
    ConstLength,
    VectorWithLength,
    VectorWithLengthMut,
    VectorRef,
    VectorMut,
    VectorIterator,
    VectorIteratorRef,
    VectorIter,
    VectorIterMut,
  };

}

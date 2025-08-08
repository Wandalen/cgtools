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

  // = IntoArray

  /// The `IntoArray` trait is used to convert a collection into a fixed-size array.
  ///
  /// This trait provides two methods:
  ///  - `into_array`, which consumes the collection and returns a fixed-size array `[E; N]`.
  ///  - `as_array`, which borrows the collection (without consuming it) and returns a new fixed-size array
  ///    `[E; N]` by cloning the collection.
  ///
  /// This can be useful when a collection needs to be represented as an array with a known, fixed size.
  ///
  /// # Type Parameters
  ///
  /// - `E`: The type of the elements in the resulting array.
  /// - `N`: The fixed number of elements in the array.
  ///
  /// # Examples
  ///
  /// Basic usage with consumption:
  ///
  /// ```ignore
  /// use mdmath_core::IntoArray;
  /// struct MyCollection;
  ///
  /// impl IntoArray< i32, 3 > for MyCollection
  /// {
  ///   fn into_array( self ) -> [ i32; 3 ]
  ///   {
  ///     [ 1, 2, 3 ]
  ///   }
  /// }
  ///
  /// let coll = MyCollection;
  /// let array = coll.into_array();
  /// assert_eq!( array, [ 1, 2, 3 ] );
  /// ```
  ///
  /// Basic usage without consumption:
  ///
  /// ```ignore
  /// use mdmath_core::IntoArray;
  /// struct MyCollection;
  ///
  /// impl IntoArray< i32, 3 > for MyCollection
  /// {
  ///   fn into_array( self ) -> [ i32; 3 ]
  ///   {
  ///     [ 1, 2, 3 ]
  ///   }
  /// }
  ///
  /// let coll = MyCollection;
  /// let array = coll.as_array();
  /// assert_eq!( array, [ 1, 2, 3 ] );
  /// ```
  pub trait IntoArray< E, const N : usize >
  {
    /// Consumes the collection and returns a fixed-size array.
    ///
    /// # Returns
    ///
    /// - `[E; N]`: The fixed-size array produced from the collection.
    fn into_array( self ) -> [ E; N ];

    /// Returns a fixed-size array without consuming the collection.
    ///
    /// This method clones the collection and then converts it into an array using the `into_array` method.
    ///
    /// # Constraints
    ///
    /// The collection must implement [`Clone`].
    ///
    /// # Returns
    ///
    /// - `[E; N]`: The fixed-size array produced from a clone of the collection.
    #[ inline ]
    fn as_array( &self ) -> [ E; N ]
    where
      Self : Clone,
    {
      self.clone().into_array()
    }
  }

  impl< T, E, const N : usize > IntoArray< E, N > for &T
  where
    T : IntoArray< E, N > + Clone,
  {
    #[ inline ]
    fn into_array( self ) -> [ E; N ]
    {
      < T as IntoArray< E, N > >::into_array( ( *self ).clone() )
    }
  }

  impl< T, E, const N : usize > IntoArray< E, N > for &mut T
  where
    T : IntoArray< E, N > + Clone,
  {
    #[ inline ]
    fn into_array( self ) -> [ E; N ]
    {
      < T as IntoArray< E, N > >::into_array( ( *self ).clone() )
    }
  }

  // = ArrayRef / ArrayMut

  /// A trait for accessing a reference to a fixed-size array from a collection.
  ///
  /// This trait requires the collection to implement `Collection`.
  pub trait ArrayRef< E, const N : usize >
  {
    /// Returns a reference to a fixed-size array from the collection.
    ///
    /// # Returns
    /// - `&[ E ; N ]`: A reference to the fixed-size array.
    fn array_ref( &self ) -> &[ E ; N ];
  }

  /// Implementation of `ArrayRef` for references to collections.
  impl< T, E, const N : usize > ArrayRef< E, N > for &T
  where
    T : ArrayRef< E, N >,
  {
    #[ inline ]
    fn array_ref( &self ) -> &[ E; N ]
    {
      < T as ArrayRef< E, N > >::array_ref( self )
    }
  }

  /// A trait for accessing a mutable reference to a fixed-size array from a collection.
  ///
  /// This trait requires the collection to implement `Collection`.
  pub trait ArrayMut< E, const N : usize >
  {
    /// Returns a mutable reference to a fixed-size array from the collection.
    ///
    /// # Returns
    /// - `&mut [ E; N ]`: A mutable reference to the fixed-size array.
    fn vector_mut( &mut self ) -> &mut [ E ; N ];
  }

  /// Implementation of `ArrayMut` for mutable references to collections.
  impl< T, E, const N : usize > ArrayMut< E, N > for &mut T
  where
    T : ArrayMut< E, N >,
  {
    #[ inline ]
    fn vector_mut( &mut self ) -> &mut [ E; N ]
    {
      <T as ArrayMut< E, N > >::vector_mut( self )
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
    /// Returns an iterator over references to the elements of the vector.
    fn vector_iter< 'a >( &'a self ) -> impl VectorIteratorRef< 'a, &'a E >
    where
      E : 'a;
  }

  /// Implementation of `ArrayRef` for references to collections.
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

  /// Implementation of `ArrayRef` for references to collections.
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
    /// Returns an iterator over mutable references to the elements of the vector.
    fn vector_iter_mut< 'a >( &'a mut self ) -> impl VectorIterator< 'a, &'a mut E >
    where
      E : 'a;
  }

  /// Implementation of `ArrayRef` for references to collections.
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
  #[ cfg( feature = "arithmetics" ) ]
  layer arithmetics;

  /// Float functions for a vector.
  #[ cfg( feature = "float" ) ]
  layer float;

  exposed use
  {
    Collection,
    ConstLength,
    VectorWithLength,
    VectorWithLengthMut,
    IntoArray, // qqq : xxx : cover by test
    ArrayRef,
    ArrayMut,
    VectorIterator,
    VectorIteratorRef,
    VectorIter,
    VectorIterMut,
  };

}

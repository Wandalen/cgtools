/// Internal namespace.
mod private
{
  // use crate::*;

  /// Trait for converting a value, reference, or mutable reference to an immutable reference.
  ///
  /// This trait provides a unified interface to obtain an immutable reference from
  /// different types of ownership. It enables functions to work seamlessly with values,
  /// references, and mutable references by abstracting the conversion to a reference.
  ///
  /// # Purpose
  ///
  /// The `ToRef` trait is useful in scenarios where a function or method needs to operate
  /// on an immutable reference, regardless of whether the input is an owned value, a reference,
  /// or a mutable reference. This abstraction simplifies the implementation of functions
  /// that need to handle different ownership types uniformly.
  ///
  /// # Example Usage
  ///
  /// ```rust
  /// use mdmath_core::ToRef;
  /// fn print_length< T : ToRef< String > >( input : T )
  /// {
  ///   let reference = input.to_ref();
  ///   println!( "Length: {}", reference.len() );
  /// }
  ///
  /// let mut owned = String::from( "Hello" );
  ///
  /// let borrowed = &owned;
  /// print_length( borrowed );
  ///
  /// let mut mutable_borrowed = &mut owned;
  /// print_length( mutable_borrowed );
  ///
  /// print_length( owned );
  /// ```
  pub trait ToRef< T : ?Sized >
  {
    /// Converts the implementing type to an immutable reference.
    ///
    /// # Returns
    /// - `&T`: An immutable reference to the underlying value.
    fn to_ref( &self ) -> &T;
  }

  // Implement ToRef for immutable references
  impl< T : ?Sized > ToRef< T > for &T
  {
    fn to_ref( &self ) -> &T
    {
      self
    }
  }

  // Implement ToRef for mutable references
  impl< T : ?Sized > ToRef< T > for &mut T
  {
    fn to_ref( &self ) -> &T
    {
      self
    }
  }

  // Implement ToRef for owned values
  impl< T : ?Sized > ToRef< T > for T
  {
    fn to_ref( &self ) -> &T
    {
      self
    }
  }

  /// Trait for obtaining a value from a reference or mutable reference.
  ///
  /// This trait provides a unified interface to clone the underlying value from
  /// references and mutable references. For owned values, it simply returns the value
  /// itself without cloning.
  ///
  /// # Purpose
  ///
  /// The `ToValue` trait is useful in scenarios where a function or method needs to
  /// obtain a value, regardless of whether the input is an owned value, a reference,
  /// or a mutable reference. This abstraction simplifies the implementation of functions
  /// that need to handle different ownership types uniformly.
  ///
  /// # Example Usage
  ///
  /// ```rust
  /// use mdmath_core::ToValue;
  /// fn duplicate_and_print< T : ToValue< String > >( input : T )
  /// {
  ///   let value = input.to_value();
  ///   println!( "Duplicated: {}", value );
  /// }
  ///
  /// let owned = String::from( "Hello" );
  ///
  /// let borrowed = &owned;
  /// duplicate_and_print( borrowed );
  ///
  /// let mut mutable_borrowed = &mut owned.clone();
  /// duplicate_and_print( mutable_borrowed );
  ///
  /// duplicate_and_print( owned );
  /// ```
  pub trait ToValue< T : ?Sized >
  {
    /// Obtains the value from the implementing type.
    ///
    /// # Returns
    /// - `T`: The value of the underlying type.
    fn to_value( self ) -> T;
  }

  // Implement ToValue for immutable references
  impl< T : ?Sized > ToValue< T > for &T
  where
    T : Clone,
  {
    fn to_value( self ) -> T
    {
      < T as Clone >::clone( self )
    }
  }

  // Implement ToValue for mutable references
  impl< T : ?Sized > ToValue< T > for &mut T
  where
    T : Clone,
  {
    fn to_value( self ) -> T
    {
      < T as Clone >::clone( self )
    }
  }

  // Implement ToValue for owned values
  impl< T > ToValue< T > for T
  {
    fn to_value( self ) -> T
    {
      self
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    ToRef,
    ToValue,
  };
}

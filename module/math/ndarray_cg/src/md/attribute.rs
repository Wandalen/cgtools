/// Internal namespace.
mod private
{
  // use crate::*;
  pub use ::num_traits::identities::Zero;
  pub use ::mdmath_core::Collection;

  /// Trait for objects that have an indexable dimension.
  ///
  /// This trait is used to define objects that have a specific dimensionality,
  /// allowing for operations that depend on the size and shape of the object.
  pub trait Indexable
  {
    /// The type representing the dimension of the object.
    type Index : ndarray::Dimension;

    /// Returns the dimension of the object.
    ///
    /// # Returns
    /// - `Self::Index`: The dimension of the object.
    fn dim( &self ) -> Self::Index;
  }

  /// Trait for objects that have a stride.
  ///
  /// This trait extends `Indexable` to include stride information, which is useful
  /// for operations that involve traversing or iterating over the object.
  pub trait StrideTrait : Indexable
  {
    /// Returns the stride of the object.
    ///
    /// # Returns
    /// - `<Self as Indexable>::Index`: The stride of the object.
    fn stride( &self ) -> <Self as Indexable>::Index;
  }
}

crate::mod_interface!
{
  exposed use
  {
    Zero,
    Collection,
    Indexable,
    StrideTrait,
  };
}

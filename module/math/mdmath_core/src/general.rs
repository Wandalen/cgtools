//! General math traits for handling zero-related operations.

/// Internal namespace.
mod private
{

//   /// Trait for zero-related functions.
//   ///
//   /// Unlike [`num_traits::identities::Zero`], `ZeroIdentity` does not require the implementation of `Add`.
//   /// This trait provides methods for creating, checking, and setting zero values.
//   pub trait ZeroIdentity : Sized
//   {
//     /// Creates a zero value of the implementing type.
//     ///
//     /// # Returns
//     /// - `Self`: A zero value.
//     fn make_zero() -> Self;
//
//     /// Checks if the value is zero.
//     ///
//     /// # Returns
//     /// - `bool`: `true` if the value is zero, otherwise `false`.
//     fn is_zero( &self ) -> bool;
//
//     /// Sets the value to zero.
//     fn zero_set( &mut self );
//   }

}

crate::mod_interface!
{
  exposed use
  {
    // ZeroIdentity,
  };
}

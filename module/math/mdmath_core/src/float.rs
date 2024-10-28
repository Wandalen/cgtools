//! Describe general floats and operations on them.

/// Internal namespace.
mod private
{
  // pub use ::num_traits::Float as _Float;

//   pub trait Float
//   where
//     Self : _Float
//   {
//   }
//
//   // impl< T > Float for T
//   // where
//   //   T : _Float
//   // {
//   // }
//
//   impl< T > Float for &T
//   where
//     T : Float + _Float,
//   {
//   }

}

crate::mod_interface!
{
  // exposed use
  // {
  //   _Float,
  //   Float,
  // };
  exposed use ::num_traits::Float;
  exposed use ::ndarray::NdFloat;
}

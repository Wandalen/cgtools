/// Internal namespace.
mod private
{
  // use crate::*;

}

crate::mod_interface!
{
  own use ::ndarray::*;
  exposed use ::ndarray::
  {
    LinalgScalar,
    NdFloat,
  };
}

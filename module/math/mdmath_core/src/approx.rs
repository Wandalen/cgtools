//! Approximate equality for floating-point types can be determined using either relative difference
//! or comparisons based on units in the last place (ULPs).

/// Internal namespace.
mod private
{
}

crate::mod_interface!
{
  exposed use ::approx::*;
}

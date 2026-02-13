//! Set of easing functions and related stuff.

mod private
{

}

crate::mod_interface!
{
  /// Base easing structs, traits, macros etc
  layer base;

  /// Collection of cubic spline easing functions
  layer cubic;

  /// Quaternion interpolation easing
  layer squad;
}

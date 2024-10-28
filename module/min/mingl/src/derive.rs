/// Internal namespace.
mod private
{
  // use crate::*;

}

crate::mod_interface!
{
  reuse ::derive_tools;
  reuse ::former;
  exposed use ::former; // xxx : make it unncecessary
  // exposed use ::former::Former;
}

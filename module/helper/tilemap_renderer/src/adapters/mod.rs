//! Backend adapter implementations.

mod private {}

mod_interface::mod_interface!
{
  #[ cfg( feature = "adapter-svg" ) ]
  layer svg;

  #[ cfg( feature = "adapter-terminal" ) ]
  layer terminal;

  #[ cfg( feature = "adapter-webgl" ) ]
  layer webgl;
}

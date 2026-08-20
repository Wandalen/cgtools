//! Backend adapter implementations.

mod private {}

mod_interface::mod_interface!
{
  #[ cfg( feature = "adapter-svg" ) ]
  layer svg;

  #[ cfg( feature = "adapter-terminal" ) ]
  layer terminal;

  #[ cfg( all( feature = "adapter-webgl" ) ) ]
  layer webgl;

  #[ cfg( all( feature = "adapter-webgpu", target_arch = "wasm32" ) ) ]
  layer webgpu;

  #[ cfg( all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ]
  layer native;

  #[ cfg( feature = "adapter-none" ) ]
  layer none;
}

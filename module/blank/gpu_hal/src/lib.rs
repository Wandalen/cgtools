//! GPU hardware abstraction layer over the `min*` drivers (reserved).
#![ doc( html_root_url = "https://docs.rs/gpu_hal/latest/gpu_hal/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "GPU hardware abstraction layer over the min* drivers (reserved)" ) ]

#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

#[ cfg( feature = "enabled" ) ]
mod private
{
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
}

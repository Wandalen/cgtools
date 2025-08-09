//! Browser utilities and tools for web-based graphics development.
#![ doc( html_root_url = "https://docs.rs/browser_tools/latest/browser_tools/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Browser utilities and tools for web-based graphics development" ) ]

#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

#[ cfg( feature = "enabled" ) ]
mod private
{
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  reuse ::browser_log;
}

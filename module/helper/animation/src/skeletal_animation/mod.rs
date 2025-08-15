//! Renderer for opaque 2D objects
#![ doc( html_root_url = "https://docs.rs/canvas_renderer/latest/canvas_renderer/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Renderer for opaque 2D objects" ) ]

#![allow(clippy::std_instead_of_core)]
#![allow(clippy::implicit_return)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::doc_markdown)]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  layer renderer;
}
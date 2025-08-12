//! Line drawing and manipulation utilities for 2D and 3D graphics.
#![ doc( html_root_url = "https://docs.rs/line_tools/latest/line_tools/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Line drawing and manipulation utilities for 2D and 3D graphics" ) ]

#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::similar_names)]
#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::len_zero)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::range_plus_one)]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// A layer for 2D graphics-related functionalities.
  layer d2;
  /// A layer for 3D graphics-related functionalities.
  layer d3;

  /// A layer dedicated to line join styles (e.g., miter, bevel, round).
  layer joins;
  /// A layer dedicated to line cap styles (e.g., butt, round, square).
  layer caps;

  /// A layer for mesh generation and manipulation.
  layer mesh;
  /// A layer for shader programs and related functionality.
  layer program;

  /// A layer for helper functions and utilities used by other modules.
  layer helpers;
}
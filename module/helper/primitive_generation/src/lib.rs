//! Set of tools for generating 3D geometry
#![ doc( html_root_url = "https://docs.rs/primitive_generation/latest/primitive_generation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Set of tools for generating 3D geometry" ) ]

#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::similar_names)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::zero_sized_map_values)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::std_instead_of_alloc)]
#![allow(clippy::question_mark)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::unused_async)]
#![allow(clippy::identity_op)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::float_cmp)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cloned_instead_of_copied)]
#![allow(clippy::for_kv_map)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::from_iter_instead_of_collect)]
#![allow(clippy::drain_collect)]
#![allow(clippy::used_underscore_binding)]
#![allow(clippy::needless_for_each)]
#![allow(clippy::get_first)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::incompatible_msrv)]
#![allow(clippy::inefficient_to_string)]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Text generation and font processing utilities.
  layer text;

  /// Basic geometric primitive creation.
  layer primitive;

  /// Data structures for primitive attributes and transformations.
  layer primitive_data;
}
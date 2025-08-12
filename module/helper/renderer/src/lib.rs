//! Graphics PBR renderer

#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::exhaustive_enums)]
#![allow(clippy::exhaustive_structs)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::needless_continue)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::unnecessary_semicolon)]
#![allow(clippy::min_ident_chars)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::std_instead_of_core)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::iter_kv_map)]
#![allow(clippy::format_push_string)]
#![allow(clippy::len_zero)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::doc_overindented_list_items)]
#![allow(clippy::single_match)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::unused_self)]
#![allow(clippy::type_complexity)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::inconsistent_struct_constructor)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ref_binding_to_reference)]
#![allow(clippy::option_as_ref_cloned)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::incompatible_msrv)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::needless_return)]
#![allow(clippy::if_not_else)]
#![allow(clippy::for_kv_map)]
#![allow(clippy::cloned_instead_of_copied)]
#![allow(clippy::map_flatten)]

mod private
{


}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Webgl implementation of the renderer
  //#[ cfg( feature = "webgl" ) ]
  layer webgl;
}

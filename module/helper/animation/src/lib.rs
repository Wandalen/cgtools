//! A general purpose library for working with animatable values.
#![ doc( html_root_url = "https://docs.rs/animation/latest/animation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "" ) ]

#![ allow( clippy::return_self_not_must_use ) ]
#![ allow( clippy::must_use_candidate ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::new_ret_no_self ) ]
#![ allow( dead_code ) ]
#![ allow( macro_expanded_macro_exports_accessed_by_absolute_paths ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Set of easing functions and related stuff.
  layer easing;

  /// Tweening system for smooth entity movement in tile-based games.
  layer interpolation;

  /// Tools for managing [`AnimatableValue`] playback in every time moment 
  layer sequencer;
}
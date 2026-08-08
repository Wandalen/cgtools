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

mod private
{

}

/// Implements the `EasingBuilder` trait for a specified easing function.
///
/// This macro generates a new public struct that acts as a builder for
/// a specific easing function, allowing you to create a boxed instance
/// of the function.
///
/// Defined at crate root, above the `mod_interface!` invocation below,
/// deliberately without `#[ macro_export ]`: call sites reach it via plain
/// textual macro scope instead of an absolute path, which avoids the
/// `macro_expanded_macro_exports_accessed_by_absolute_paths` future-incompat
/// lint that an exported macro triggers when invoked from a module generated
/// by the `layer` mechanism below.
macro_rules! impl_easing_function
{
  ( $builder_ty:ident, $function_ty:ty, $value:expr ) =>
  {
    /// A builder for the `EasingFunction` of type [`$function_ty`].
    ///
    /// This struct provides a way to create a boxed instance of the
    /// associated easing function.
    #[ non_exhaustive ]
    pub struct $builder_ty< A >( core::marker::PhantomData< A > );

    impl< A > crate::easing::base::EasingBuilder< $function_ty, A > for $builder_ty< A >
    where A : crate::Animatable
    {
      /// Creates a new `Box` containing an instance of the easing function.
      fn new() -> Box< $function_ty >
      {
        Box::new( $value )
      }
    }
  };
}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Set of easing functions and related stuff.
  layer easing;

  /// Set of animation traits
  layer traits;

  /// Tweening system for smooth entity movement in tile-based games.
  layer interpolation;

  /// Tools for managing [`AnimatablePlayer`] playback in every time moment
  layer sequencer;
}

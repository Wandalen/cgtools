//! A general purpose library for working with animatable values.
#![ doc( html_root_url = "https://docs.rs/animation/latest/animation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "" ) ]

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
    #[ doc = concat!( "A builder for the `EasingFunction` of type `", stringify!( $function_ty ), "`." ) ]
    ///
    /// This struct provides a way to create a boxed instance of the
    /// associated easing function.
    #[ non_exhaustive ]
    pub struct $builder_ty< A >( core::marker::PhantomData< A > );

    impl< A > crate::easing::base::EasingBuilder< $function_ty, A > for $builder_ty< A >
    where A : crate::Animatable
    {
      /// Builds a `Box` containing an instance of the easing function.
      // `$builder_ty` is a zero-sized phantom marker, never itself constructed by callers —
      // `build()` returns the boxed easing function it selects.
      fn build() -> Box< $function_ty >
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

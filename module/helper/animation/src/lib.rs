//! A general purpose library for working with animatable values.
#![ doc( html_root_url = "https://docs.rs/animation/latest/animation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "" ) ]

// Fix(TASK-042): this crate carried 8 blanket #![allow(...)] attributes with no explanatory
// comments. Investigating each by temporarily removing it and re-running
// `cargo clippy -p animation --all-targets --all-features -- -D warnings` (rather than assuming)
// found 3 that suppressed zero real hits and were deleted outright: `clippy::implicit_return` is
// a `clippy::restriction`-tier lint never enabled anywhere in this workspace's
// `[workspace.lints.clippy]` table, so it can never fire regardless of this crate's code;
// `clippy::cast_precision_loss` is active workspace-wide via `pedantic = "warn"` but this crate
// has zero integer-to-float `as` casts to trigger it; `dead_code` is rustc's own always-on lint
// and a clean rebuild (`cargo clean -p animation`) found no dead items. The 5 remaining below each
// have a verified nonzero hit count from that same investigation and are justified individually.
// Root cause: allows added defensively without confirming each one matched an actual triggering
// lint. Pitfall: an allow with zero real hits reads as "this crate needs this suppressed" when it
// actually means nothing here depends on it — leaving it in invites the next reader to assume a
// justification exists that was never verified.

// Builder-style `with_*` methods (`Tween`, `CubicBezier`) return `Self` by design; annotating
// each with #[must_use] individually is disproportionate for a crate-wide convention (5 hits).
#![ allow( clippy::return_self_not_must_use ) ]
// Pervasive across simple getters (`value_get`, `time`, `state`, `progress`, `duration_get`,
// `delay_get`, etc.) — annotating each individually is disproportionate noise (18 hits).
#![ allow( clippy::must_use_candidate ) ]
// Workspace-enabled restriction lint requiring #[inline] on every public item; left to the
// compiler/LTO instead of per-function annotation — the crate's most common hit (116 hits).
#![ allow( clippy::missing_inline_in_public_items ) ]
// Covers intentional narrowing casts (f64 repeat counts / normalized values truncated to
// i32/f32) where the animation domain makes the truncation safe (4 hits).
#![ allow( clippy::cast_possible_truncation ) ]
// The `EasingBuilder` pattern's generated `new()` (see `impl_easing_function!` below)
// intentionally returns `Box<AnimatableType>`, not `Box<Self>` — `Self` is a phantom builder
// marker, not the constructed value, so this violates the naming convention by design (1 hit).
#![ allow( clippy::new_ret_no_self ) ]

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

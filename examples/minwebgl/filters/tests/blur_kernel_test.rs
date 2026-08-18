//! Regression tests verifying each `filters::blur::Blur<T>` variant's fragment shader implements
//! a kernel shape distinct from its siblings.
//!
//! `filters` is a binary-only example crate (no `[lib]` target) whose fragment shaders are Rust
//! string literals rather than separate `.frag` files, so this test reads `blur.rs`'s own real
//! source text via `include_str!` rather than exercising library/GL code.

const BLUR_RS : &str = include_str!( "../src/filters/blur.rs" );

/// Extracts the body text of `impl Filter for Blur< T >` starting at `marker`, up to the next
/// `impl Filter for Blur` occurrence (or end of file for the last impl block).
fn impl_block< 'a >( src : &'a str, marker : &str ) -> &'a str
{
  let start = src.find( marker ).unwrap_or_else( || panic!( "marker {marker:?} not found in blur.rs" ) );
  let rest = &src[ start + marker.len().. ];
  match rest.find( "impl Filter for Blur" )
  {
    Some( end ) => &rest[ ..end ],
    None => rest,
  }
}

/// ## Root Cause
/// `Blur< Stack >`'s fragment shader used the exact same uniform-weight box-average kernel as
/// `Blur< Box >` (`sum` of equal-weight taps divided by tap count) — only the uniform name
/// (`u_radius` vs `u_box_size`) and loop bounds differed, which is a reparametrization of the same
/// average, not a different kernel shape. A real stack blur (Mario Klingemann's algorithm, which
/// the "Stack Blur" UI card name references) applies a triangular/tent-shaped weight that falls
/// off linearly from the center tap, producing visibly softer results than a uniform box average.
/// With the old shader, "Stack Blur" and "Box Blur" were pixel-identical for equivalent sizes.
///
/// ## Why Not Caught
/// Both filters visibly blur the image when selected — a skim of the demo's actual behavior looks
/// correct for both UI cards, since both do reduce high-frequency detail. Nothing ever compared
/// the two kernels' actual per-tap weighting to notice they were the same algorithm under two
/// different names/sliders.
///
/// ## Fix Applied
/// Changed `Blur< Stack >`'s fragment shader to weight each tap by `radius + 1 - abs(i)`
/// (a triangular kernel), normalizing by the accumulated weight sum instead of the flat tap count.
///
/// ## Prevention
/// This test extracts each `impl Filter for Blur< T >` block's shader source text and asserts the
/// Stack variant's kernel contains a per-tap `weight` term, distinguishing it from the Box
/// variant's flat average — catches this specific kernel collapsing back to a uniform average.
///
/// ## Pitfall
/// Three near-identical `impl Filter for Blur< T >` blocks in the same file/struct is exactly the
/// shape that invites a copy-pasted kernel body with only the uniform name swapped — the resulting
/// bug produces a visibly-working filter (it does blur), so it is invisible without explicitly
/// comparing kernel shapes across sibling variants.
#[ test ]
fn bug_reproducer_bug_xxx_stack_blur_uses_distinct_triangular_kernel_not_box_average()
{
  let box_block = impl_block( BLUR_RS, "impl Filter for Blur< Box >" );
  let stack_block = impl_block( BLUR_RS, "impl Filter for Blur< Stack >" );

  assert!( box_block.contains( "u_box_size" ), "sanity: box block should reference u_box_size" );
  assert!( stack_block.contains( "u_radius" ), "sanity: stack block should reference u_radius" );

  assert!
  (
    stack_block.contains( "weight" ),
    "Stack Blur's fragment shader has no per-tap weight term — it uses the same uniform \
    box-average kernel as Box Blur (sum of equal-weight taps / tap count), making \"Stack Blur\" \
    and \"Box Blur\" produce identical output despite being offered as two distinct filter \
    choices in the UI (BUG-XXX). A real stack blur uses a triangular (linearly-decreasing) \
    weight kernel that this test's fix introduces."
  );
}

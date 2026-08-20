//! Verifies `texture::d2`'s sprite-upload pure helpers ( `mip_levels_for_dimensions`,
//! `sprite_position` ), extracted from the GL-context-bound `sprite_upload` for testability —
//! per the all-tests-in-tests/ convention and the same private-pure-helper-extraction pattern
//! used for BUG-051/BUG-052 (see `clean_test.rs`/`geometry_test.rs`).

use minwebgl::{ texture::d2::{ mip_levels_for_dimensions, sprite_position }, WebglError };

/// Companion happy-path case: the sole real caller's exact sprite size ( 128x128 ) still
/// computes the pre-fix hardcoded value of 8 — the one boundary value where that constant
/// happened to be correct.
#[ test ]
fn mip_levels_matches_the_hardcoded_value_at_its_only_valid_boundary()
{
  assert_eq!( mip_levels_for_dimensions( 128, 128 ), 8 );
}

// test_kind: bug_reproducer(BUG-160)
/// ## Root Cause
/// `sprite_upload`'s `tex_storage_3d` call hardcoded `levels` to `8`, but WebGL2/GLES3.0's spec
/// requires `levels <= floor(log2(max(width,height))) + 1` — only satisfied when
/// `max(sprite_width,sprite_height) >= 128`. For smaller sprites the call raised
/// `INVALID_OPERATION` (never checked anywhere in the function) and allocated no storage, so
/// every subsequent `tex_sub_image_3d` call silently no-op'd against a texture that was never
/// actually created — a silent, undetectable failure with no error signal at all.
///
/// ## Why Not Caught
/// The only real caller ( `examples/minwebgl/sprite_animation` ) happens to use exactly
/// 128x128 sprites, the precise boundary value where 8 levels is still valid — coincidentally
/// masking the bug for every dimension actually exercised in this repo. WebGL errors are also
/// not surfaced as JS exceptions/`Result::Err` by wasm-bindgen, so nothing short of an explicit
/// `gl.get_error()` call could have caught this live either.
///
/// ## Fix Applied
/// Extracted the level-count computation into `mip_levels_for_dimensions`, replacing the
/// hardcoded `8` with `mip_levels_for_dimensions(sprite_width, sprite_height)`.
///
/// ## Prevention
/// This test independently computes each case's spec-mandated maximum level count ( not by
/// calling the function under test ) and asserts the two agree, so it is a genuine spec check,
/// not a tautology; the final assertion pins that these cases actually differ from the pre-fix
/// hardcoded `8`, so the test would have failed pre-fix.
///
/// ## Pitfall
/// A hardcoded constant that happens to match a spec-derived formula's value at one particular
/// input ( 128 -> 8 ) reads as correct until a different, equally ordinary input is tried.
#[ test ]
fn mip_levels_stays_within_the_spec_limit_for_common_sub_128_sprite_sizes()
{
  for ( width, height ) in [ ( 16u32, 16u32 ), ( 32, 32 ), ( 32, 64 ), ( 64, 32 ), ( 96, 96 ) ]
  {
    let spec_max_levels = width.max( height ).ilog2() + 1;
    let computed = mip_levels_for_dimensions( width, height );
    assert_eq!( computed, spec_max_levels, "mip_levels_for_dimensions({width},{height}) must equal the spec's own max level count" );
    assert!( computed < 8, "sanity: this case must actually differ from the pre-fix hardcoded 8, got {computed}" );
  }
}

/// A degenerate zero-sized sprite is out of scope to reject here ( `SpriteSheet` has no
/// constructor to validate at ); this only pins that the level-count math itself can never
/// panic via `ilog2(0)`.
#[ test ]
fn mip_levels_never_panics_on_a_zero_dimension()
{
  assert_eq!( mip_levels_for_dimensions( 0, 0 ), 1 );
}

/// Companion happy-path case: a 4-sprite sheet, 2 per row, each 10x20.
#[ test ]
fn sprite_position_accepts_an_ordinary_sheet()
{
  assert_eq!( sprite_position( 0, 2, 10, 20 ).unwrap(), ( 0, 0 ) );
  assert_eq!( sprite_position( 1, 2, 10, 20 ).unwrap(), ( 10, 0 ) );
  assert_eq!( sprite_position( 2, 2, 10, 20 ).unwrap(), ( 0, 20 ) );
  assert_eq!( sprite_position( 3, 2, 10, 20 ).unwrap(), ( 10, 20 ) );
}

// test_kind: bug_reproducer(BUG-161)
/// ## Root Cause
/// `sprite_upload`'s row/column computation divided and modulo'd by the caller-supplied
/// `sprites_in_row` field with no zero-guard — `SpriteSheet` has no constructor and is
/// deliberately kept exhaustive for external struct-literal construction, so a caller-computed
/// `sprites_in_row: 0` ( e.g. derived from a sprite wider than the source image ) panicked via
/// integer division-by-zero.
///
/// ## Why Not Caught
/// The computation lived inline in `sprite_upload` with no standalone function to call
/// directly, and this crate's only real caller always supplies a nonzero, hand-written literal.
///
/// ## Fix Applied
/// Extracted the computation into `sprite_position`, returning
/// `Result< (u32,u32), WebglError >` ( `WebglError::NotSupportedForType` ) instead of panicking
/// when `sprites_in_row == 0`; `sprite_upload` now propagates via `?` since it already returns
/// `Result`.
///
/// ## Prevention
/// RED state (empirically confirmed): reverting this helper's body to the pre-fix unguarded
/// `index % sprites_in_row` / `index / sprites_in_row` and re-running this test genuinely
/// panics with "attempt to calculate the remainder with a divisor of zero" — verified via a
/// temporary probe before this fix was finalized.
///
/// ## Pitfall
/// A fully-public, constructor-less struct ( kept exhaustive so external crates can
/// struct-literal-construct it, see `SpriteSheet`'s own doc comment in `src/texture/d2.rs` )
/// has no single choke point to validate fields at construction time — every consumer of a
/// field must guard independently.
#[ test ]
fn sprite_position_rejects_zero_sprites_in_row()
{
  let result = sprite_position( 0, 0, 10, 20 );
  assert!
  (
    matches!( &result, Err( WebglError::NotSupportedForType( _ ) ) ),
    "expected Err( WebglError::NotSupportedForType ), got {result:?}"
  );
}

//! Verifies `texture::d2`'s sprite-upload pure helpers ( `mip_levels_for_dimensions`,
//! `sprite_position` ), extracted from the GL-context-bound `sprite_upload` for testability —
//! per the all-tests-in-tests/ convention and the same private-pure-helper-extraction pattern
//! used for BUG-051/BUG-052 (see `clean_test.rs`/`geometry_test.rs`). Also verifies, via
//! source-inspection ( see `Fix(BUG-290)`'s precedent, `minvulkan/tests/context_test.rs` ),
//! that `sprite_upload` itself propagates a rejected image-load promise instead of panicking
//! (BUG-425) — genuinely unreachable from a native test since it needs a live GL context and a
//! real browser loading a broken image URL.

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

// test_kind: bug_reproducer(BUG-425)
/// ## Root Cause
/// `sprite_upload` wires an `on_error` handler that rejects `load_promise` when the image
/// element's `error` event fires ( a broken/unreachable image URL, e.g. ), but then awaited
/// that same promise via `JsFuture::from( load_promise ).await.unwrap()` -- discarding the
/// rejection and panicking instead of returning it through the `Result< WebGlTexture,
/// WebglError >` this `async fn` already declares and every other fallible step inside it
/// already propagates through.
///
/// ## Why Not Caught
/// `sprite_upload` needs a live `WebGl2RenderingContext` and a real `HtmlImageElement`, neither
/// constructible from a native `cargo test` run ( no JS engine ) ; reproducing the panic live
/// would need a real browser loading a real broken image URL through a dedicated test page. No
/// such page exists within this crate's own `tests/manual/` browser procedure ( which currently
/// covers only `context::from_canvas` + a draw call, via `examples/minwebgl/context_triangle_smoke`
/// ), and adding one is out of reach here : it would require creating or modifying an
/// `examples/` crate, which is outside this fix's edit scope. Source-inspection is therefore
/// the same fallback `Fix(BUG-290)`/`Fix(BUG-424)` ( `minvulkan/tests/context_test.rs`,
/// `swapchain_test.rs` ) already established in this workspace for defects that are real but
/// structurally unreachable from the available native/in-scope test surface.
///
/// ## Fix Applied
/// Replaced `.unwrap()` with
/// `.map_err( | _ | WebglError::Other( "image failed to load" ) )?`, so a rejected
/// `load_promise` now returns `Err( WebglError::Other( .. ) )` through the function's existing
/// `Result` return type instead of panicking.
///
/// ## Prevention
/// Before adding a rejection/error handler to a `Promise`/`JsFuture` bridge, check what the
/// `.await` on the *other* end of that bridge actually does with the rejection it produces --
/// wiring the handler is not the same as propagating what it delivers ; a stray `.unwrap()`
/// downstream silently converts every wired rejection back into a panic.
///
/// ## Pitfall
/// An `async fn` already returning `Result< _, WebglError >`, with every *other* fallible step
/// inside it correctly using `?`, can still hide a single `.unwrap()` on one particular
/// await -- the function's overall shape looks fallible-safe, but a per-call-site audit is
/// still needed since Rust does not require every await in a `Result`-returning `async fn` to
/// use `?`.
#[ test ]
fn sprite_upload_propagates_image_load_rejection_instead_of_panicking()
{
  let src = include_str!( "../src/texture/d2.rs" );

  let unwrap_count = src.matches( "JsFuture::from( load_promise ).await.unwrap()" ).count();
  assert_eq!
  (
    unwrap_count, 0,
    "sprite_upload (BUG-425) must not .unwrap() the image-load promise's rejection, found \
    {unwrap_count} remaining occurrences"
  );

  let propagated_count = src
  .matches( "JsFuture::from( load_promise ).await.map_err( | _ | WebglError::Other( \"image failed to load\" ) )?;" )
  .count();
  assert_eq!
  (
    propagated_count, 1,
    "sprite_upload (BUG-425) must propagate a rejected image-load promise via \
    .map_err(..)?, found {propagated_count} occurrences of the expected fix"
  );
}

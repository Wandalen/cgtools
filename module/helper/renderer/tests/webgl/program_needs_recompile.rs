//! Regression coverage for BUG-258: `Renderer`'s per-material shader-program cache
//! ( `renderer::webgl::program_needs_recompile` ) ignored changes to Image-Based Lighting
//! availability once a material had already been registered once, silently keeping stale
//! ( IBL-less, or stale-IBL ) programs bound forever.
//
// test_kind: bug_reproducer(BUG-258)
//
// ## Root Cause
// `Renderer::primitive_register` only ever invalidated a material's cached
// `( program UUID, defines )` mapping when `material.needs_recompile()` -- a purely
// material-intrinsic "my own defines changed" flag -- was set. The renderer-level
// `use_ibl = self.ibl.is_some() && material.ibl_base_texture_unit().is_some()` value was
// recomputed every call, but only ever *consulted* on a cache miss ( first-time
// registration ); once a mapping existed, `material_program_map.get( &material_id )`
// short-circuited straight past it. So a material registered before `Renderer::ibl_set` was
// ever called compiled without `#define USE_IBL` and without any IBL uniform bindings, and
// kept using that exact program on every subsequent frame no matter how many times
// `ibl_set` was called afterward.
//
// ## Why Not Caught
// No test exercised `primitive_register`'s cache-reuse path at all, let alone across a
// change in IBL availability -- and every first-party example in this workspace happens to
// call `ibl_set` ( awaited ) before its render loop ever starts, so the divergent order that
// triggers the bug never occurs in existing example code, only in the public API contract
// ( `ibl_set`'s own doc comment promises the IBL "will be used for rendering" with no
// caveat about materials registered before the call ).
//
// ## Fix Applied
// Extracted the invalidation decision into its own pure `pub fn program_needs_recompile(
// material_needs_recompile : bool, cached_use_ibl : Option< bool >, current_use_ibl : bool )
// -> bool`, and changed `material_program_map`'s value type from a bare program UUID to
// `( program UUID, use_ibl the program was compiled with )` so the cached IBL state is
// available to compare against. `primitive_register` now invalidates the cache entry
// whenever *either* the material's own flag is set *or* the cached IBL state differs from
// the freshly computed one.
//
// ## Prevention
// Any renderer-level ( not material-level ) input that gets baked into a shader's `#define`
// set must be part of that shader program's cache invalidation key or check -- a
// material-owned dirty flag alone cannot detect a change in state it has no visibility into.
//
// ## Pitfall
// The steady-state case ( IBL state unchanged across frames ) must remain `false`, or every
// material would be needlessly recompiled every single frame -- the fix is specifically
// "invalidate on a state *change*", not "always recompile when IBL is active".

use super::*;
use the_module::program_needs_recompile;

/// The exact real-world trigger: a material is registered ( and its program cached ) while
/// `Renderer::ibl.is_none()`, then `Renderer::ibl_set` is called before the material's next
/// registration. Pre-fix, this returned `false` ( no recompile ), leaving the material's
/// non-IBL program bound forever regardless of how many times `ibl_set` was called.
#[ test ]
fn ibl_becoming_available_after_registration_forces_a_recompile()
{
  let needs_recompile = program_needs_recompile( false, Some( false ), true );

  assert!( needs_recompile, "a material cached without IBL must be recompiled once IBL becomes available" );
}

/// Symmetric case : IBL was available when the material was first cached, but is no longer
/// reflected by the current computation ( e.g. the material's own `ibl_base_texture_unit()`
/// started returning `None` ). Must also force a recompile, not just the IBL-added direction.
#[ test ]
fn ibl_becoming_unavailable_after_registration_forces_a_recompile()
{
  let needs_recompile = program_needs_recompile( false, Some( true ), false );

  assert!( needs_recompile, "a material cached with IBL must be recompiled once IBL is no longer available to it" );
}

/// The steady-state case that must NOT regress : once the cached program's IBL state already
/// matches the current one, no recompile should be forced on the material's own account --
/// otherwise every material would be recompiled on every single frame.
#[ test ]
fn unchanged_ibl_state_does_not_force_a_recompile()
{
  assert!( !program_needs_recompile( false, Some( true ), true ), "IBL state unchanged (both true) must not force a recompile" );
  assert!( !program_needs_recompile( false, Some( false ), false ), "IBL state unchanged (both false) must not force a recompile" );
}

/// A material's own `needs_recompile()` flag must still force a recompile on its own,
/// independent of IBL state -- the fix adds a second trigger, it must not remove the first.
#[ test ]
fn material_owned_recompile_flag_still_forces_a_recompile()
{
  let needs_recompile = program_needs_recompile( true, Some( true ), true );

  assert!( needs_recompile, "material.needs_recompile() must still force a recompile even when IBL state is unchanged" );
}

/// First-time registration ( no cached program yet ) must never be treated as "needs
/// recompile" -- that's the normal "compile a fresh program" path, not an invalidation.
#[ test ]
fn no_cached_program_is_not_treated_as_needing_recompile()
{
  assert!( !program_needs_recompile( false, None, true ), "first-time registration ( no cache entry ) must take the normal compile path, not the invalidation path" );
  assert!( !program_needs_recompile( false, None, false ), "first-time registration ( no cache entry ) must take the normal compile path, not the invalidation path" );
}

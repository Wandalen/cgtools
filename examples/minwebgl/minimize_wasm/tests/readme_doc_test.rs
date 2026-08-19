//! Regression tests verifying `minimize_wasm`'s readme claims match its actual build pipeline.
//!
//! `minimize_wasm` is a binary-only example crate (no `[lib]` target), so these tests read the
//! crate's own real source text via `include_str!` rather than exercising library code.

const README : &str = include_str!( "../readme.md" );
const MAKEFILE : &str = include_str!( "../Makefile" );
const CARGO_TOML : &str = include_str!( "../Cargo.toml" );

/// ## Root Cause
/// `readme.md` claimed this demo "demonstrates optimization strategies including dead code
/// elimination, LTO, and size-focused compilation." LTO (link-time optimization) is a specific,
/// well-defined Cargo `[profile.release] lto = true` setting — and no `[profile.*]` section
/// exists anywhere reachable from this crate. Per-crate profile overrides are structurally
/// impossible for workspace members (Cargo only honors `[profile]` in the workspace root), and
/// the real workspace root `Cargo.toml` has zero `[profile.*]` sections. `trunk build --release`
/// therefore always uses Cargo's stock release profile, where `lto` defaults to `false`.
///
/// ## Why Not Caught
/// The crate genuinely does demonstrate real, working size-optimization techniques (`wee_alloc`
/// as a minimal global allocator, `wasm-opt -Os` + `wasm-strip` in the `Makefile`'s
/// `optimize-wasm` target) — so a skim of the demo's actual behavior looks consistent with the
/// readme's general thrust, and nothing ever exercises the specific named technique "LTO" against
/// the actual build configuration.
///
/// ## Fix Applied
/// Reworded `readme.md` to name the three techniques actually configured (`wee_alloc`,
/// `wasm-opt -Os`, `wasm-strip`) instead of the unconfigured "LTO".
///
/// ## Prevention
/// This test greps the readme for the word "lto" and fails if present, while sanity-asserting the
/// three real techniques are still actually configured (`wee_alloc` dependency, `wasm-opt` +
/// `wasm-strip` in the Makefile) — catches either the false claim reappearing or the real
/// techniques silently regressing out from under an unchanged readme.
///
/// ## Pitfall
/// A demo whose own purpose IS "show optimization techniques" is exactly the place a stale/wrong
/// named technique is most likely to go unnoticed — the demo still visibly "does its job" (small
/// wasm output) even when one of the specifically named techniques was never real.
#[ test ]
fn bug_reproducer_bug_329_readme_does_not_claim_unconfigured_lto()
{
  assert!( CARGO_TOML.contains( "wee_alloc" ), "sanity: wee_alloc should still be a dependency" );
  assert!( MAKEFILE.contains( "wasm-opt" ), "sanity: Makefile should still run wasm-opt" );
  assert!( MAKEFILE.contains( "wasm-strip" ), "sanity: Makefile should still run wasm-strip" );

  assert!
  (
    !README.to_lowercase().contains( "lto" ),
    "readme claims LTO but no [profile.release] lto = true exists anywhere reachable from this \
    crate (workspace members can't override profiles, and the real workspace root has no \
    [profile.*] section at all) (BUG-329)"
  );
}

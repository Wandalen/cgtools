//! Doc-text regression test for `tactical_rpg/src/main.rs`'s module doc
//! comment.
//!
//! Pure `include_str!` + substring assertion -- no library target needed
//! (this crate is binary-only), matching this session's established
//! `include_str!` precedent for doc-only defects that resist black-box
//! runtime testing.

// test_kind: bug_reproducer(BUG-305)
/// ## Root Cause
/// `src/main.rs`'s module doc comment claimed "Line-of-sight and
/// area-of-effect attacks" as a feature. `attack_execute()` takes exactly
/// one `attacker: hecs::Entity, target: hecs::Entity` pair -- single
/// target only, no multi-target/splash logic anywhere in the file.
/// Targeting is gated by a flat `Position::distance_to() <= N` hex-distance
/// check (`ai_turn_handle`/`player_turn_handle`), with no occlusion,
/// shadowcasting, or any other line-of-sight/visibility check performed
/// before an attack is allowed. Neither claimed capability is implemented.
/// ## Why Not Caught
/// This crate is binary-only (`src/main.rs`, no `src/lib.rs`) and had zero
/// pre-existing test coverage, so nothing tied the module doc comment's
/// feature list to what `attack_execute`/the turn-handling functions
/// actually implement. `readme.md` itself only claims "attack ranges"
/// (accurate), so the false claim was confined to `main.rs`'s doc comment
/// and easy to miss without directly cross-checking it against the combat
/// code.
/// ## Fix Applied
/// Removed the false "Line-of-sight and area-of-effect attacks" bullet.
/// Its accurate half (attack ranges) was already covered by the existing
/// "Movement and attack ranges on hexagonal grid" bullet; no replacement
/// bullet was added, since equipment's effect on attack damage was already
/// covered by the existing "Equipment and inventory management" bullet.
/// ## Prevention
/// A module doc comment's feature list is a set of falsifiable claims --
/// each one needs to be checked against the functions that would implement
/// it (here: `attack_execute`, `ai_turn_handle`, `player_turn_handle`)
/// before being trusted, not assumed accurate because it reads
/// plausibly alongside genuinely-implemented neighbors (Experience,
/// Equipment) in the same list.
/// ## Pitfall
/// A claimed-but-unimplemented feature is a more severe doc-drift defect
/// than an undercounted list (missing a real item) -- it asserts something
/// exists that a reader could reasonably expect to find and build on, with
/// no corresponding code anywhere to fall back on.
#[ test ]
fn main_rs_module_doc_comment_does_not_claim_los_or_aoe_attacks()
{
  let main_rs = include_str!( "../src/main.rs" );
  assert!
  (
    !main_rs.contains( "Line-of-sight" ) && !main_rs.contains( "area-of-effect" ),
    "tactical_rpg/src/main.rs's module doc comment must not claim line-of-sight or \
    area-of-effect attacks -- attack_execute() is single-target only, and targeting uses a flat \
    distance_to() range check with no occlusion/visibility check anywhere in this file (BUG-305)"
  );
  assert!
  (
    main_rs.contains( "Movement and attack ranges on hexagonal grid" )
    && main_rs.contains( "Equipment and inventory management" ),
    "tactical_rpg/src/main.rs's module doc comment must still describe the genuinely-implemented \
    attack-range and equipment features (BUG-305)"
  );
}

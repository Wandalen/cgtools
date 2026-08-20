//! Source-structure regression test for `game_systems_demo/src/main.rs`'s
//! scripted combat match arms.
//!
//! Pure `include_str!` + substring assertion -- no library target needed
//! (this crate is binary-only), matching this session's established
//! `include_str!` precedent for defects that resist black-box runtime
//! testing. A behavioral test would require extracting the combat match
//! statement into a library target purely to reach it -- disproportionate
//! restructuring for a currently-dormant, non-crashing inconsistency in a
//! demo binary; this structural check is the proportionate alternative.

// test_kind: bug_reproducer(BUG-301)
/// ## Root Cause
/// The scripted combat `match` in `main.rs` has 3 monster-turn arms (entity
/// IDs 11 "Orc Warrior 1", 12 "Orc Warrior 2", 13 "Orc Shaman"). Arms 11 and
/// 13 both guard on the attacker's own liveness
/// (`resources_get(N).unwrap().health.current > 0.0`) before attacking; arm
/// 12 was authored without that same guard, attacking unconditionally.
/// ## Why Not Caught
/// This crate is binary-only with zero pre-existing test coverage.
/// Currently dormant: a full-file grep confirms `health_modify(12` never
/// appears anywhere else in this file, so entity 12 is never damaged by
/// this demo's own fixed scripted sequence and the missing guard cannot
/// yet be observed to fire -- but a copy-paste omission in the middle of 3
/// sibling arms is exactly the kind of inconsistency that is easy to miss
/// during review and easy to reintroduce during a future edit.
/// ## Fix Applied
/// Added the same liveness guard entities 11 and 13 already carry to
/// entity 12's arm, so all 3 monster-turn arms share the identical pattern.
/// ## Prevention
/// When 3+ match arms share a guard pattern, check every sibling arm for
/// the same guard, not just the immediately adjacent one -- the omission
/// here was in the middle arm, easy to miss when only checking neighbors.
/// ## Pitfall
/// A guard that never fires under the current fixed test inputs is not the
/// same as a guard that is unnecessary -- entity 12's arm was reachable
/// and exploitable the moment any future change caused entity 12 to take
/// damage, exactly like entities 11 and 13 already can.
#[ test ]
fn orc_warrior_two_arm_guards_own_liveness_like_sibling_monster_arms()
{
  let main_rs = include_str!( "../src/main.rs" );
  let arm_12_start = main_rs.find( "12 => { // Orc Warrior 2" )
  .expect( "entity 12's match arm must still exist with its identifying comment (BUG-301)" );
  let arm_12_end = main_rs[ arm_12_start.. ].find( "13 if resources.resources_get" )
  .map( | rel | arm_12_start + rel )
  .expect( "entity 13's match arm must still follow entity 12's (BUG-301)" );
  let arm_12_body = &main_rs[ arm_12_start..arm_12_end ];
  assert!
  (
    arm_12_body.contains( "resources.resources_get(12).unwrap().health.current > 0.0" ),
    "game_systems_demo/src/main.rs's entity-12 (\"Orc Warrior 2\") combat arm must guard on its \
    own liveness before attacking, matching sibling arms 11 (\"Orc Warrior 1\") and 13 \
    (\"Orc Shaman\"), which both already carry this guard (BUG-301)"
  );
}

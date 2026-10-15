# BUG-305: `tactical_rpg`'s module doc comment falsely claims "Line-of-sight and area-of-effect attacks" -- combat is single-target only, with no occlusion/visibility check anywhere

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/tiles_tools/tactical_rpg/src/main.rs`
- **Component:** examples/tiles_tools/tactical_rpg
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`src/main.rs`'s module doc comment claimed "Line-of-sight and area-of-effect attacks" as a
feature. `attack_execute(&mut self, attacker: hecs::Entity, target: hecs::Entity)` takes exactly
one attacker/target pair -- single target only, no multi-target/splash logic anywhere in the
file. Targeting is gated by a flat `Position::distance_to() <= N` hex-distance check
(`ai_turn_handle`/`player_turn_handle`), with no occlusion, shadowcasting, or any other
line-of-sight/visibility check performed before an attack is allowed. Neither claimed capability
is implemented.

## Impact

**Who is affected:** any reader trusting the module doc comment's feature list to understand
combat mechanics, or expecting to build on either claimed capability.

**What breaks:** a claimed-but-unimplemented feature is a more severe doc-drift defect than an
undercounted list (missing a real item) -- it asserts something exists that a reader could
reasonably expect to find and build on, with no corresponding code anywhere to fall back on.

**Entity Scope:** `None` -- documentation-only defect.

## How Discovered

Disclosed by a fork bug-hunting `tiles_tools`'s 12 native example crates (task #183).
Independently verified: `attack_execute`'s signature confirmed single-target, `readme.md` itself
only claims "attack ranges" (accurate), and no occlusion/visibility logic exists anywhere in the
file.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "fn attack_execute" examples/tiles_tools/tactical_rpg/src/main.rs
grep -c "Line-of-sight\|area-of-effect" examples/tiles_tools/tactical_rpg/src/main.rs
```
**Expected** (fixed): `attack_execute` takes a single `attacker`/`target` pair, and the
Line-of-sight/area-of-effect count is 0. **Actual** (pre-fix): same single-target signature, but
the count was >= 1.

## Root Cause

The module doc comment's feature list included 2 capabilities that were either planned and never
implemented, or aspirationally copy-pasted alongside genuinely-implemented neighbors (Experience,
Equipment) without being individually verified against the actual combat code.

## Why Not Caught

This crate is binary-only (`src/main.rs`, no `src/lib.rs`) and had zero pre-existing test
coverage, so nothing tied the module doc comment's feature list to what `attack_execute`/the
turn-handling functions actually implement. `readme.md` itself only claims "attack ranges"
(accurate), so the false claim was confined to `main.rs`'s doc comment and easy to miss without
directly cross-checking it against the combat code.

## Fix Applied (2026-08-18)

Removed the false "Line-of-sight and area-of-effect attacks" bullet. Its accurate half (attack
ranges) was already covered by the existing "Movement and attack ranges on hexagonal grid"
bullet; no replacement bullet was added, since equipment's effect on attack damage was already
covered by the existing "Equipment and inventory management" bullet.

Added `tests/readme_doc_test.rs` (`main_rs_module_doc_comment_does_not_claim_los_or_aoe_attacks`):
pure `include_str!` + substring assertions confirming the module doc comment no longer claims
either capability while still describing the genuinely-implemented attack-range and equipment
features.

## Verification

- **Pre-fix (RED):** module doc comment claimed both capabilities -- test would fail against the
  pristine text.
- **Post-fix (GREEN):** `cargo test -p tactical_rpg --test readme_doc_test` → 1 passed. `cargo
  clippy -p tactical_rpg --all-targets --all-features -- -D warnings` → clean. Independently
  re-run by the orchestrating session as part of this task's combined confirming sweep.

## Generalized Version

A module doc comment's feature list is a set of falsifiable claims -- each one needs to be
checked against the functions that would implement it before being trusted, not assumed accurate
because it reads plausibly alongside genuinely-implemented neighbors in the same list.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting `tiles_tools`'s 12 native crates (task #183, one of 3 parallel forks covering 27 `examples/` crates); fixed and tested with a `BUG-XXX` placeholder marker since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session (diff read, `attack_execute` signature and absence of occlusion logic cross-checked in source, test independently re-run) before this report and its real ID were assigned; placeholder replaced with BUG-305 after a fresh on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor. |

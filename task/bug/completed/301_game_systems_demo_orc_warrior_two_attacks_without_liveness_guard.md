# BUG-301: `game_systems_demo`'s "Orc Warrior 2" combat arm attacks unconditionally, unlike its 2 sibling monster arms which both guard on their own liveness

- **Severity:** Low (currently dormant -- unreachable under this demo's own fixed scripted sequence)
- **state:** Completed
- **Affects:** `examples/tiles_tools/game_systems_demo/src/main.rs`
- **Component:** examples/tiles_tools/game_systems_demo
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

The scripted combat `match` in `main.rs` has 3 monster-turn arms (entity IDs 11 "Orc Warrior 1",
12 "Orc Warrior 2", 13 "Orc Shaman"). Arms 11 and 13 both guard on the attacker's own liveness
(`resources.resources_get(N).unwrap().health.current > 0.0`) before attacking; arm 12 was
authored without that same guard, attacking unconditionally regardless of whether entity 12 is
still alive.

## Impact

**Who is affected:** anyone reading or extending this demo's scripted combat sequence, and any
future change that causes entity 12 ("Orc Warrior 2") to take damage.

**What breaks:** once entity 12 can be damaged by any future scripted event, a dead Orc Warrior 2
would still take its turn and attack -- the exact bug the sibling guards on entities 11/13 exist
to prevent.

**Entity Scope:** `None` -- currently dormant, no live scripted event damages entity 12.

## How Discovered

Disclosed by a fork bug-hunting `tiles_tools`'s 12 native example crates (task #183); the fork
found and fixed 4 other findings in this same crate family but explicitly flagged this one as
unfixed, since the demo binary has no library target and a full behavioral test would require
disproportionate restructuring. Independently confirmed genuinely real (not just plausible) via
an exhaustive `grep` for `health_modify(12` across the whole file -- zero other occurrences,
confirming entity 12 is never damaged by this demo's own fixed scripted sequence -- then fixed
directly.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "health_modify(12" examples/tiles_tools/game_systems_demo/src/main.rs
grep -A2 '12 => { // Orc Warrior 2' examples/tiles_tools/game_systems_demo/src/main.rs
```
**Expected** (fixed): the second command's arm-12 body opens with
`if resources.resources_get(12).unwrap().health.current > 0.0 {`, matching sibling arms 11/13.
**Actual** (pre-fix): arm 12 attacked directly with no guard, while arms 11/13 both had one.

## Root Cause

Arm 12 was authored without copying the liveness guard its 2 siblings (arms 11 and 13) both
carry -- a copy-paste omission in the middle of 3 near-identical match arms.

## Why Not Caught

This crate is binary-only with zero pre-existing test coverage of any kind, and the omission is
currently dormant: no existing scripted event in this demo's fixed sequence ever damages entity
12, so the missing guard cannot yet be observed to fire in a normal run.

## Fix Applied (2026-08-18)

Added the same liveness guard entities 11 and 13 already carry to entity 12's arm
(`if resources.resources_get(12).unwrap().health.current > 0.0 { .. }`), so all 3 monster-turn
arms share the identical pattern.

Added `tests/combat_guard_test.rs`, a source-structure regression test: pure `include_str!` +
substring assertion (no library target needed, matching this session's established
`include_str!` precedent for defects that resist black-box runtime testing in a binary-only demo
crate) confirming entity 12's match-arm body contains the liveness-guard string. A full
behavioral test would require extracting the combat match statement into a library target purely
to reach it -- disproportionate restructuring for a currently-dormant, non-crashing
inconsistency in a demo binary; this structural check is the proportionate alternative.

## Verification

RED-before-GREEN proof via a scratchpad-backed `git show HEAD` restore (not `git stash`): copied
the fixed `main.rs` to a scratchpad backup, overwrote `main.rs` with `git show HEAD:...`
(pristine pre-fix), ran the new test (RED: 1 failed with the exact expected assertion message),
restored the fix via `cp` from the backup, confirmed byte-identical via `diff -q`.

- **Pre-fix (RED):** `cargo test -p game_systems_demo --test combat_guard_test` → 1 failed.
- **Post-fix (GREEN):** same command → 1 passed. `cargo clippy -p game_systems_demo --all-targets
  --all-features -- -D warnings` → clean.

## Generalized Version

When 3+ match arms share a guard pattern, check every sibling arm for the same guard during
review, not just the immediately adjacent one -- the omission here was in the middle arm, easy
to miss when only checking neighbors. A guard that never fires under the current fixed test
inputs is not the same as a guard that is unnecessary: entity 12's arm was reachable and
exploitable the moment any future change caused entity 12 to take damage, exactly like entities
11 and 13 already can.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Disclosed-but-unfixed finding from a fork bug-hunting `tiles_tools`'s 12 native crates (task #183, one of 3 parallel forks covering 27 `examples/` crates). Independently confirmed genuinely dormant via exhaustive grep, then fixed and tested directly by the orchestrating session (not the fork) with a proportionate lightweight structural test, avoiding the disproportionate-restructuring trap the fork itself flagged. Originally implemented under a placeholder ID before this session's own on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor (BUG-298 `quat_invert`, TASK-299 `renderer_gltf_loader`, BUG-300 `texture_descriptor_default_format`) -- renumbered to 301 before filing, with all inline `Fix()`/`bug_reproducer()` markers updated to match. |

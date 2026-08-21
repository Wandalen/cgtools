# BUG-427: `CameraOrbitControls::update` applies smoothed rotation even when `rotation.enabled` is false

- **Severity:** Medium (no crash -- camera silently keeps rotating after the consumer explicitly
  disables rotation input, as long as movement smoothing is on and any rotation velocity was queued
  beforehand)
- **state:** Completed
- **Affects:** Any consumer of `mingl::controls::camera_orbit_controls::CameraOrbitControls` that
  toggles `rotation.enabled` off at runtime (e.g. to hand focus to a UI overlay, or to freeze the
  camera during a cutscene) while `rotation.movement_smoothing_enabled` is also on.
- **Component:** `module/min/mingl` (`src/controls/camera_orbit_controls.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- isolated logic-gap defect in this one guard, no shared root cause with
  any other bug filed this sweep.

## Symptom

```rust
// pre-fix -- src/controls/camera_orbit_controls.rs, update()
if self.rotation.movement_smoothing_enabled
{
  self.rotation_apply( dt );
}
```

`rotation_apply` (which advances `eye`/`up`/`center` toward the smoothed target) ran on every
`update()` call whenever smoothing was on, with no check of `self.rotation.enabled` at all --
disabling rotation only stopped `rotate()` from queuing *new* velocity, it never stopped
already-queued smoothed velocity from continuing to apply.

## Impact

**Who is affected:** Any consumer that flips `rotation.enabled = false` at runtime as a way to
freeze camera rotation (e.g. while a modal UI has input focus) rather than only at construction
time, provided `movement_smoothing_enabled` is also on and the user was still dragging (or had
recently released a drag) at the moment `enabled` was cleared.

**What breaks:** The camera keeps rotating for one or more additional frames after rotation was
supposed to be fully disabled -- the smoothed velocity queued before the disable continues to decay
toward zero via `rotation_apply` instead of stopping immediately, since nothing gates that call on
`enabled`. Purely a camera-motion correctness bug, not a crash or data corruption.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`,
comparing `update()`'s two gated code paths (instant rotation vs. smoothed rotation) against each
other -- the instant-rotation path had no separate `enabled` gate to compare (it's gated entirely at
`rotate()`'s call site, which already checks `enabled`), but the smoothed path's `rotation_apply`
call is reachable independently of `rotate()` on every single `update()` tick, and was the one path
missing the `enabled` check.

## Minimum Reproducible Example

```rust
// module/min/mingl/tests/tests/camera_orbit_controls.rs
let mut controls = CameraOrbitControls::default();
controls.rotation.movement_smoothing_enabled = true;
controls.rotation.speed = 1.0;
controls.rotate( [ 1.0, 0.0 ] );   // queues smoothed rotation velocity
controls.rotation.enabled = false; // disable rotation
let before = ( controls.eye, controls.up, controls.center );
controls.update( 0.016 );          // pre-fix: still rotates despite `enabled == false`
assert_eq!( before, ( controls.eye, controls.up, controls.center ) );
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/mingl && cargo nextest run -p mingl --features camera_orbit_controls -E 'test(test_update_rotation_disabled_prevents_smoothed_rotation)'
```

## Root Cause

`update()`'s smoothed-rotation branch was gated solely on `self.rotation.movement_smoothing_enabled`,
never on `self.rotation.enabled` -- the two flags read as independent knobs (one for "is rotation on
at all", one for "should rotation be smoothed") but the smoothed-apply code path only ever consulted
the second, so a queued smoothing velocity kept applying every frame regardless of the first flag's
state.

## Why Not Caught

No existing test exercised the specific sequence of queuing smoothed rotation velocity via `rotate()`
and then disabling rotation before the velocity had fully decayed -- existing coverage tested
`rotation.enabled` gating `rotate()` itself, and separately tested smoothing's decay behavior with
`enabled` left on throughout, but never the interaction of both together.

## Fix Location

`module/min/mingl/src/controls/camera_orbit_controls.rs`, `update()`: guard widened from
`if self.rotation.movement_smoothing_enabled` to
`if self.rotation.enabled && self.rotation.movement_smoothing_enabled`.

## Prevention

New test `test_update_rotation_disabled_prevents_smoothed_rotation` in
`module/min/mingl/tests/tests/camera_orbit_controls.rs`: queues smoothed rotation velocity via
`rotate()`, disables `rotation.enabled`, calls `update()`, and asserts `eye`/`up`/`center` are
unchanged (within floating-point tolerance) from immediately before the `update()` call.

## Pitfall

Two independent-looking boolean flags gating the same code path (`enabled`, `..._smoothing_enabled`)
can silently combine incorrectly when only one branch checks both -- reviewing each flag's own gate
in isolation ("does `rotate()` check `enabled`? yes" / "does smoothing decay correctly? yes") missed
that the *smoothed-apply* branch specifically only checked one of the two, since that check was
never exercised by either flag's own dedicated test in isolation.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}` (L0 driver crates), comparing the instant-rotation and smoothed-rotation code paths in `update()` against each other. |
| 2026-08-20 | fixed | Widened the smoothed-rotation guard to also require `rotation.enabled`; added `Fix(BUG-427)`/`Root cause`/`Pitfall` source comment. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted the guard back to `if self.rotation.movement_smoothing_enabled` alone (via a `/* TEMP-RED-PROBE */` marker), re-ran the new test -- genuinely failed (nextest exit 100, 1 failed). Restored the fix, re-ran the full `camera_orbit_controls`-feature suite -- 41/41 pass, confirming no collateral regression from the revert-and-restore cycle. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-427)`/`Root cause`/`Pitfall` 3-field source comment applied at the guard site; test carries the mandated 5-section doc block (`bug_reproducer(BUG-427)`). | — |
| D3 | Scope containment | — | 🟢 | Only `camera_orbit_controls.rs` (fix) and `tests/tests/camera_orbit_controls.rs` (test) touched -- both within the `module/min/mingl` edit-scope boundary for this sweep. | — |

**Reproduced:** YES -- temporary revert of the `enabled &&` clause caused the new test to fail with
an eye/up/center-mismatch assertion; restoring the fix passes. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/mingl/src/controls/camera_orbit_controls.rs` | Widened `update()`'s smoothed-rotation guard from `movement_smoothing_enabled` alone to `enabled && movement_smoothing_enabled`; added `Fix(BUG-427)`/`Root cause`/`Pitfall` source comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/mingl/tests/tests/camera_orbit_controls.rs` | Added `test_update_rotation_disabled_prevents_smoothed_rotation`, RED/GREEN-confirmed against a temporary revert of the fix. |

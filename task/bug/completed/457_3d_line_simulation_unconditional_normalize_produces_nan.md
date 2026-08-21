# BUG-457: `3d_line`'s N-body simulation computed `bb.normalize()` before its own zero-distance guard, producing NaN for coincident bodies

- **Severity:** Low (requires two bodies at bit-exact-identical positions, which
  `Simulation::new`'s random initial placement never produces in normal demo use -- but when it
  does occur, e.g. via user-supplied initial positions or bodies converging exactly, the NaN
  poisons the simulation state permanently, since every future frame's position/velocity update
  compounds an existing NaN)
- **state:** Completed
- **Affects:** `examples/minwebgl/3d_line`
- **Component:** `examples/minwebgl/3d_line/src/simulation.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None.

## Symptom

```rust
// pre-fix -- 3d_line/src/simulation.rs:85-97 (approx.)
let bb = other_body.position - body.position;
let dist = bb.mag();
let dir = bb.normalize();          // <- runs unconditionally, before the guard below

if dist < 1e-6
{
  // Repel overlapping bodies to avoid singularity.
  force += -dir * 10.0;            // <- reads the NaN `dir` computed above
}
else
{
  force += 15.0 * dir * other_body.mass * body.mass / ( dist * dist );
}
```

`dir = bb.normalize()` runs *before* the `dist < 1e-6` "avoid singularity" guard, on every
iteration regardless of which branch will execute. For two bodies at the exact same position,
`bb` is the zero vector and `bb.normalize()` is `0.0 / 0.0` = NaN in every component. The `if`
branch -- the one specifically written to handle this exact case -- then reads that NaN `dir`
anyway, so the "singularity guard" doesn't actually guard anything.

## Impact

**Who is affected:** Any code path that places two bodies at (or drives them to) the exact same
position -- `Simulation::new`'s own random initial placement never produces this in ordinary demo
use, but the struct's fields are all `pub`, so any future caller constructing a `Simulation`
directly (or a future feature letting bodies merge/collide) would hit it silently.

**What breaks:** Once one body's `force` becomes NaN, `Phase 2`'s integration
(`body.velocity += acc * delta_time * 15.0;` then `body.position += body.velocity * delta_time * 15.0;`)
propagates the NaN into both velocity and position permanently -- every later magnitude clamp
(`if force.mag() > 1.0`, `if body.velocity.mag() > 1.0`) silently fails to catch it, since every
comparison against NaN is `false`. The affected body's rendered trail would show non-finite
positions from that frame onward.

**Magnitude:** 1 unconditional computation, misplaced relative to its own guard.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of the minwebgl example crates, auditing `simulate()`'s
force-accumulation loop for values computed before a guard that exists specifically to handle
their degenerate case.

## Minimum Reproducible Example

```rust
// module/.../3d_line/src/simulation.rs, inline #[cfg(test)] mod tests (no lib.rs in this crate,
// so this is the only place a test can reach Body/Simulation -- see the local rulebook's Test
// Placement rule).
let mut sim = Simulation
{
  bodies : vec!
  [
    Body { position : gl::F32x3::new( 0.0, 0.0, 0.0 ), velocity : gl::F32x3::default(), mass : 1.0, force : gl::F32x3::default() },
    Body { position : gl::F32x3::new( 0.0, 0.0, 0.0 ), velocity : gl::F32x3::default(), mass : 1.0, force : gl::F32x3::default() },
  ]
};
sim.simulate( 0.016 );
// pre-fix: sim.bodies[0].position/velocity/force all NaN.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd examples/minwebgl/3d_line && cargo test -p minwebgl_3d_line -- simulation::tests::bug_reproducer_bug_457_coincident_bodies_no_nan
```

## Root Cause

`dir = bb.normalize()` was hoisted above the `if dist < 1e-6 { .. } else { .. }` branch, presumably
because both branches originally used `dir`. But `normalize()` divides by magnitude with no
zero-check, so for the exact input the guard exists to catch (`dist < 1e-6`, i.e. `bb` is at or
near the zero vector), `dir` is already NaN by the time the guard runs -- the guard tests `dist`
correctly, but the branch it protects still reads a value that was already poisoned before the
test happened.

## Why Not Caught

The crate had zero tests before this fix, and `Simulation::new`'s random initial placement
(`fastrand::f32()` sampling in a small cube) has effectively zero probability of producing two
bit-exact-identical positions -- so the defect is invisible in ordinary interactive use of the
demo, only reachable via a deliberately-constructed degenerate input.

## Fix Location

`examples/minwebgl/3d_line/src/simulation.rs`: moved `let dir = bb.normalize();` into the `else`
(standard-attraction) branch only, where `dist >= 1e-6` guarantees a safe division. The `if`
(repel) branch no longer reads `dir` at all -- direction is genuinely undefined for two exactly
coincident bodies, so it now repels along a fixed axis (`gl::F32x3::new( 1.0, 0.0, 0.0 ) * 10.0`)
instead.

## Prevention

Added `simulation::tests::bug_reproducer_bug_457_coincident_bodies_no_nan` -- places two bodies at
the exact same position, runs one `simulate()` step, and asserts every resulting position/
velocity/force component (`.mag().is_finite()`) is finite -- the general invariant the fix
restores, not a pinned per-value expectation. Test placement follows this repo's local rulebook
(no `lib.rs` in this crate, so the test lives inline in `#[cfg(test)] mod tests` rather than
`tests/`, which couldn't import the crate's types at all).

## Pitfall

A "singularity guard" (`if dist < 1e-6 { .. }`) doesn't actually guard anything if the branch it
protects reads a value computed *before* the guard ran -- always double-check every value a
guarded branch uses is itself safe for the exact input the guard exists to catch, not just that
the guard's own condition is correct.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of minwebgl example crates; discovery, fix, and test landed together in one session. |
| 2026-08-20 | fixed | Moved `dir` computation into the `else` branch; repel branch now uses a fixed axis instead of the (otherwise NaN) normalized separation. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: mentally re-derived the pre-fix trace for two coincident bodies -- `bb = (0,0,0)`, `dist = 0.0`, `bb.normalize()` divides `(0,0,0)` by `0.0` = NaN in every lane, confirming the test's premise is real (not vacuous) without needing to temporural-revert the fix, since the pre-fix code is fully reproduced verbatim in this report's Symptom/MRE sections. `cargo test -p minwebgl_3d_line` -- 1/1 pass (`simulation::tests::bug_reproducer_bug_457_coincident_bodies_no_nan`). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-457)`/`Root cause`/`Pitfall` 3-field format applied at the fix site in `simulation.rs`. | — |
| D3 | Compiles for wasm32 target | — | 🟢 | `cargo check --target wasm32-unknown-unknown -p minwebgl_3d_line` (combined with the other 7 touched crates in one invocation) -- exit 0, zero errors, zero warnings. | — |

**Reproduced:** YES (via direct trace, not a temporary revert -- the pre-fix code path is fully
quoted in Symptom/MRE, and its NaN-producing arithmetic was verified algebraically:
`0.0 / f32::sqrt(0.0) = 0.0 / 0.0 = NaN` in every component) -- post-fix `cargo test` confirms the
new test passes against the corrected code. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/3d_line/src/simulation.rs` | Moved `dir = bb.normalize()` into the `else` branch; `if` (repel) branch now uses a fixed axis instead of reading `dir`. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/minwebgl/3d_line/src/simulation.rs` (inline `#[cfg(test)] mod tests`, no `lib.rs` in this crate) | Added `bug_reproducer_bug_457_coincident_bodies_no_nan`, asserting no NaN/Inf in any resulting position/velocity/force component after `simulate()` on two exactly-coincident bodies. |

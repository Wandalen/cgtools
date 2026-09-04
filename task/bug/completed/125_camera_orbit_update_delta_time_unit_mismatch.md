# BUG-125: `CameraOrbitControls::update` treats seconds `delta_time` as milliseconds, scaling smoothed rotation down by 1000x

- **Severity:** Medium (feature-scoped — only reachable when a caller opts into
  `movement_smoothing_enabled = true`, but that is a real, documented, intended usage path, not
  a contrived one; when reached, the smoothing feature appears completely frozen)
- **state:** Completed
- **Affects:** Any caller of `CameraOrbitControls::update(delta_time)` with
  `rotation.movement_smoothing_enabled = true`
- **Component:** `module/min/mingl` (`src/controls/camera_orbit_controls.rs::update`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-126, filed under the same task #62
  targeted `mingl` review

## Symptom

```bash
# rotation.movement_smoothing_enabled = true, rotation.speed = 1.0
# rotate([1.0, 0.0]) accumulates current_angular_speed = 1.0
# update(0.016) -- a realistic 60fps frame delta, in SECONDS per this function's own doc contract

# Wrong (pre-fix) -- treats delta_time as if it were already milliseconds:
current_rotation_angle == current_angular_speed * 0.016 / 1000.0 == 0.000016 rad  # imperceptible

# Correct (post-fix) -- converts seconds to milliseconds first:
current_rotation_angle == current_angular_speed * 16.0 / 1000.0 == 0.016 rad     # 1000x larger
```

## Impact

**Who is affected:** Any caller that enables `rotation.movement_smoothing_enabled` and drives the
camera via `rotate()` + `update(delta_time)` — the intended, documented usage pattern for smooth
(inertia-style) orbit rotation. `movement_smoothing_enabled` defaults to `false`, so the buggy
code path is dead unless a caller explicitly turns smoothing on.

**What breaks:** `update`'s own doc comments state the contract precisely: `delta_time` is "a
per-frame delta in seconds," and the smoothing decay is meant to happen "every 10 milliseconds."
The two formulas beneath those comments (`/ 10.0` and `/ 1000.0`) are correct only if `delta_time`
itself is already in milliseconds — but every real caller supplies seconds. With a realistic
60fps `delta_time` (~0.016s), both `decay_percentage` and `current_rotation_angle` come out
1000x smaller than intended: the camera's smoothed rotation appears almost completely frozen, and
any accumulated angular speed decays roughly 1000x slower than the documented "every 10ms" rate.

**Magnitude:** 100% of callers who enable `movement_smoothing_enabled` are affected on every
frame — this is not a boundary/edge-case defect, it fires under ordinary, continuous use of the
feature. Traced the seconds-vs-milliseconds contract via the crate's own caller chain:
`module/helper/renderer/src/webgl/camera.rs::Camera::update` is a pure passthrough with no
conversion of its own, and the workspace-wide calling convention (confirmed in
`examples/minwebgl/skeletal_animation/src/main.rs`) explicitly converts a raw rAF millisecond
timestamp to seconds (`let time = t / 1000.0;`) before computing and passing `delta_time` —
confirming `update`'s real callers universally pass seconds, matching its own doc comment.

**Entity Scope:** None — a code-level math defect, not an operational-entity concern.

## How Discovered

Task #62, a targeted code review of `mingl` dispatched under the standing bug-hunt mandate. The
reviewing agent flagged that `update`'s two internal formula constants (`/ 10.0` for decay,
`/ 1000.0` for rotation angle) are each individually plausible in isolation, but only jointly
self-consistent under one alternative unit assumption (milliseconds) that directly contradicts
the function's own doc comment ("delta_time is a per-frame delta in seconds"). Independently
re-verified before filing by:

```bash
$ sed -n '480,500p' module/min/mingl/src/controls/camera_orbit_controls.rs
# confirms both formulas and both doc comments exactly as quoted above

$ grep -n "\.update(" -B3 module/helper/renderer/src/webgl/camera.rs
# Camera::update(delta_time: f64) is a pure passthrough: self.controls.borrow_mut().update(delta_time)

$ sed -n '95,119p' examples/minwebgl/skeletal_animation/src/main.rs
# move |t: f64| { let time = t / 1000.0; ...; current_animation.borrow_mut().update(delta_time); }
# -- t is a raw rAF millisecond timestamp, explicitly divided by 1000.0 to reach seconds
#    before being used as a delta_time -- confirms the workspace-wide convention is seconds
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre125 && mkdir -p /tmp/mre125/src
cat > /tmp/mre125/Cargo.toml <<'EOF'
[package]
name = "mre125"
version = "0.1.0"
edition = "2021"

[dependencies]
mingl = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/min/mingl", features = [ "camera_orbit_controls" ] }
EOF
cat > /tmp/mre125/src/main.rs <<'EOF'
use mingl::controls::camera_orbit_controls::CameraOrbitControls;
use mingl::F32x3;

fn main()
{
  let mut controls = CameraOrbitControls
  {
    eye : F32x3::new( 1.0, 0.0, 0.0 ),
    up : F32x3::new( 0.0, 1.0, 0.0 ),
    center : F32x3::new( 0.0, 0.0, 0.0 ),
    ..Default::default()
  };
  controls.rotation.movement_smoothing_enabled = true;
  controls.rotation.speed = 1.0;

  controls.rotate( [ 1.0, 0.0 ] );
  let eye_before = controls.eye;
  controls.update( 0.016 ); // 60fps frame delta, in seconds

  let angle_before = eye_before.z().atan2( eye_before.x() );
  let angle_after = controls.eye.z().atan2( controls.eye.x() );
  println!( "{}", ( angle_after - angle_before ).abs() );
}
EOF
cd /tmp/mre125 && cargo run 2>&1 | tail -1
```

**Expected** (post-fix — swept angle matches the milliseconds-correct formula):
```
0.016
```

**Actual** (pre-fix — swept angle 1000x too small):
```
0.000016
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre125 && cargo run 2>&1 | tail -1
# 0.016 = fixed; 0.000016 = bug present
```
**What:** Violates `update`'s own documented contract — "delta_time is a per-frame delta in
seconds" — by internally treating it as milliseconds.

**Known MRE limitation (check 205):** `mingl` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116/118-124's own
documented exception. `0.016`/`16.0`/`1000.0` are exactly representable in `f32` with no
floating-point ambiguity this local dependency could be hiding; the swept-angle measurement
(`atan2` difference) sidesteps any dependency on `from_angle_y`'s internal sign convention.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `update`'s `/10.0` and `/1000.0` formulas assume `delta_time` is already in milliseconds, but every real caller supplies seconds per the function's own doc comment. | ✅ Root Cause | Direct read of `update` (pre-fix) confirms both formulas apply directly to the raw `delta_time` parameter with no unit conversion; doc comment states seconds; caller-chain trace confirms seconds are what's actually passed. MRE with a realistic 60fps delta confirms the 1000x-too-small symptom. | E1, E2 |
| H2 | The doc comment itself is wrong (a stale label), and `delta_time` is actually meant to be passed in milliseconds by convention, making this a documentation gap rather than a logic bug. | ❌ Falsified | The workspace's own calling convention, traced through `Camera::update`'s pure passthrough to `skeletal_animation`'s explicit `t / 1000.0` rAF-to-seconds conversion, proves real callers genuinely pass seconds — the doc comment is correct; the formula is what disagrees with it. | E1, E3 |
| H3 | `rotation_apply()` itself (not `update`) is where the unit mismatch originates, since it's the function that actually moves the camera. | ❌ Falsified | `rotation_apply()` takes no time parameter at all — it consumes `current_rotation_angle`, a value already fully computed by `update` before `rotation_apply()` is ever called. The scaling defect is entirely contained within `update`'s own formula, before `rotation_apply` is invoked. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/min/mingl/src/controls/camera_orbit_controls.rs` (pre-fix `update`, lines ~486-499) | `let mut decay_percentage = self.rotation.movement_decay * delta_time / 10.0;` and `self.rotation.current_rotation_angle = self.rotation.current_angular_speed * delta_time / 1000.0;` — both apply directly to the raw seconds-typed `delta_time`, with doc comments "per-frame delta in seconds" / "every 10 milliseconds" immediately above. | H1 ✅, H3 ❌ |
| E2 | `/tmp/mre125` run, pre-fix vs. post-fix, `current_angular_speed=1.0`, `delta_time=0.016` | Pre-fix: swept angle `0.000016` rad = `1.0 * 0.016 / 1000.0`. Post-fix: swept angle `0.016` rad = `1.0 * 16.0 / 1000.0` (`delta_time_ms = 0.016 * 1000.0`). Confirms the exact 1000x scaling factor predicted by the unit-mismatch hypothesis. | H1 ✅ |
| E3 | `examples/minwebgl/skeletal_animation/src/main.rs` (lines 95-119) and `module/helper/renderer/src/webgl/camera.rs` (lines 110-140) | `Camera::update` is a pure passthrough (`self.controls.borrow_mut().update(delta_time)`, no conversion). `skeletal_animation`'s rAF closure computes `let time = t / 1000.0;` from a raw millisecond timestamp before deriving `delta_time` — proving the workspace-wide convention genuinely is seconds, matching `update`'s own doc comment, not the formula. | H1 ✅, H2 ❌ |

## Root Cause

```
update( &mut self, delta_time: f64 )
  let delta_time = delta_time as f32          <- still seconds (per doc comment + real callers)
  decay_percentage = movement_decay * delta_time / 10.0     <- assumes delta_time in ms   ✗
  current_rotation_angle = current_angular_speed * delta_time / 1000.0  <- assumes ms too  ✗
```

Both formulas beneath the narrowing cast are written and doc-commented in terms of milliseconds
("every 10 milliseconds"), but the parameter they consume is never converted from the seconds
unit its own doc comment (and every real caller) uses. Two independently-plausible-looking
constants (`/10.0`, `/1000.0`) mask the mismatch: neither one alone looks obviously wrong, and
both become simultaneously correct only under the single alternative assumption (milliseconds)
that the surrounding doc comments explicitly rule out.

## Why Not Caught

No existing test in `tests/tests/camera_orbit_controls.rs` exercised `update()` with
`movement_smoothing_enabled = true` at all (confirmed via grep: zero hits for
`movement_smoothing`/`.update(`/`current_angular_speed`/`current_rotation_angle` before this
fix). Since `movement_smoothing_enabled` defaults to `false`, the buggy formulas are dead code
under every existing test's default construction — a plain `cargo test` run never touches them.

## Fix Location

`module/min/mingl/src/controls/camera_orbit_controls.rs`, `pub fn update`. One change:

```rust
// before
let delta_time = delta_time as f32;

let mut decay_percentage = self.rotation.movement_decay * delta_time / 10.0;
decay_percentage = decay_percentage.min( 1.0 );

if self.rotation.movement_smoothing_enabled
{
  self.rotation.current_rotation_angle = self.rotation.current_angular_speed * delta_time / 1000.0;
  self.rotation_apply();
  self.rotation.current_angular_speed *= 1.0 - decay_percentage;
}

// after
let delta_time = delta_time as f32;
let delta_time_ms = delta_time * 1000.0;

let mut decay_percentage = self.rotation.movement_decay * delta_time_ms / 10.0;
decay_percentage = decay_percentage.min( 1.0 );

if self.rotation.movement_smoothing_enabled
{
  self.rotation.current_rotation_angle = self.rotation.current_angular_speed * delta_time_ms / 1000.0;
  self.rotation_apply();
  self.rotation.current_angular_speed *= 1.0 - decay_percentage;
}
```

The `/10.0` and `/1000.0` formulas themselves are left completely unchanged — only the unit of
their input is corrected once, up front, preserving the doc comments' literal "every 10
milliseconds" semantics exactly as written.

## Prevention

Added `test_update_applies_smoothed_rotation_at_correct_time_scale` to
`tests/tests/camera_orbit_controls.rs`: enables smoothing, accumulates angular speed via
`rotate()`, calls `update(0.016)` (a realistic 60fps seconds-typed delta), and asserts the
camera's swept rotation angle (measured via `atan2` on the eye position, independent of the
rotation matrix's internal sign convention) matches the milliseconds-correct `0.016` rad rather
than the pre-fix `0.000016` rad.

**Pitfall:** a doc comment naming a time unit ("every 10 milliseconds") is not proof the formula
beneath it actually receives that unit — the two must be verified independently; a formula whose
constants are each individually plausible can still be silently wrong when they were tuned
against a different unit than the one now flowing through the parameter.

## Generalized Version

**Broken assumption:** "A formula's constants look reasonable, so the unit they were written
against matches the unit the parameter actually carries" — false; plausibility of an individual
constant (`/10.0`, `/1000.0`) is not evidence of unit agreement, only of *a* unit having once been
intended, possibly a different one than the parameter's current documented contract.

**Confirmed general rule:** when a function's internal formula divides or scales a parameter by a
constant tied to a specific unit (a time base, a distance base, an angle base), verify that unit
against the parameter's own doc comment AND against what real callers actually pass — trace the
caller chain to its origin rather than trusting a comment or a constant in isolation. A formula
that becomes exactly self-consistent under one alternative unit assumption is strong evidence the
contract changed upstream (or was always inconsistent) without the internal formula being updated
to match.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #62's targeted code review of `mingl`; confirmed by tracing the real caller chain (`Camera::update` passthrough → `skeletal_animation`'s `t/1000.0` rAF conversion) before filing. |
| 2026-08-15 | fixed | Added `let delta_time_ms = delta_time * 1000.0;` after the existing narrowing cast; changed both formulas to consume `delta_time_ms` instead of `delta_time`. 3-field `Fix(BUG-125)`/`Root cause`/`Pitfall` comment added at the fix site. |
| 2026-08-15 | verified | Added `test_update_applies_smoothed_rotation_at_correct_time_scale` to `tests/tests/camera_orbit_controls.rs`; scoped test run (`cargo nextest run -p mingl --all-features` via `longrun`) passed with the new test green alongside the pre-existing suite (56/56). |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16; fix not present anywhere in this session's own context — approached as a fresh reader). Independently re-read `camera_orbit_controls.rs::update` (confirmed `delta_time_ms = delta_time * 1000.0` genuinely present, both formulas consume it, `delta_time` used nowhere else in the crate — no scope escape) and the `bug_reproducer(BUG-125)` test body (non-tautological — computes the swept angle via `atan2` and asserts it matches the milliseconds-correct `0.016` rad, not the pre-fix `0.000016`). Fresh `cargo nextest run -p mingl --all-features` via `longrun`: 56/56 passed. `cargo clippy -p mingl --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-125/126 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-computed `0.016`/`0.000016`; adversarial pass independently re-ran the MRE against the current (fixed) source (`cargo run` → `0.016`, exact match) rather than trusting the confirming pass's arithmetic alone. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed this file correctly declares no `**Related Bugs:**` beyond noting BUG-126 as independently-rooted despite the shared task #62 review — different function, no shared code path. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass independently re-traced the caller chain (`Camera::update` → `skeletal_animation` rAF closure) from source rather than accepting the confirming pass's description of it. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked `decay_percentage`'s own formula for the same defect shape — confirmed it shares the identical mismatch and is fixed by the same single `delta_time_ms` conversion, not a separate defect requiring its own fix. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `mingl`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `update`'s own body; `rotation_apply()` and `rotate()` are unmodified and confirmed to take no time parameter of their own. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility — it corrects `update` to honor the time unit its own doc comment already claimed. | — |

**Reproduced:** YES — `/tmp/mre125` pre-fix: swept angle `0.000016` rad instead of the
milliseconds-correct `0.016` rad, for `current_angular_speed=1.0`, `delta_time=0.016s`, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/min/mingl/src/controls/camera_orbit_controls.rs` | `update`: added `let delta_time_ms = delta_time * 1000.0;`; changed both formulas to consume `delta_time_ms` instead of `delta_time`. `Fix(BUG-125)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/mingl/tests/tests/camera_orbit_controls.rs` | Added `test_update_applies_smoothed_rotation_at_correct_time_scale` (`bug_reproducer(BUG-125)`, 5-section doc comment, smoothing-enabled fixture). |

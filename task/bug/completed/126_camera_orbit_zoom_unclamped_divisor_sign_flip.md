# BUG-126: `CameraOrbitControls::zoom`'s zoom-out divisor can reach zero or go negative, corrupting or flipping the camera through its pivot

- **Severity:** Medium (requires a single-event `|delta_y| >= zoom.speed`; genuinely reachable via
  a fast pinch gesture's raw screen-pixel distance or a high-precision mouse wheel, not a
  contrived edge case; default `zoom.max_distance`/`min_distance` are both `None`, so nothing
  downstream masks the corruption in default configuration)
- **state:** Completed
- **Affects:** Any caller of `CameraOrbitControls::zoom(delta_y)` — including the crate's own
  built-in `Pinch`-gesture and wheel-scroll event wiring — where a single event's `delta_y`
  reaches or exceeds `zoom.speed` in magnitude
- **Component:** `module/min/mingl` (`src/controls/camera_orbit_controls.rs::zoom`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-125, filed under the same task #62
  targeted `mingl` review

## Symptom

```bash
# eye = (1,0,0), center = (0,0,0), zoom.speed = 1.0, zoom.max_distance/min_distance = None

# Wrong (pre-fix) -- delta_y == 1.0 drives the divisor to exactly 0.0:
zoom(1.0);  eye == (Inf or NaN, 0.0, 0.0)               # division by zero

# Wrong (pre-fix) -- delta_y == 1.5 drives the divisor negative (-0.5):
zoom(1.5);  eye == (-2.0, 0.0, 0.0)                     # flipped through the pivot to -x

# Correct (post-fix) -- divisor floored at f32::EPSILON, never reaches zero or negative:
zoom(1.5);  eye == (8388608.0, 0.0, 0.0)                # finite, same side as original direction
```

## Impact

**Who is affected:** Any caller of `zoom(delta_y)` whose single-event `|delta_y|` reaches or
exceeds `zoom.speed` (default `1000.0`) — including the crate's own built-in gesture wiring: the
`Pinch` pointer-event handler calls `camera.borrow_mut().zoom( old_dist - new_dist )` with
`old_dist`/`new_dist` computed directly from raw `e.screen_x()`/`e.screen_y()` browser screen
coordinates (not viewport-relative, and completely unclamped), and the plain wheel handler calls
`camera.borrow_mut().zoom( e.delta_y() as f32 )` directly on the raw wheel event delta.

**What breaks:** `zoom`'s zoom-out branch computes `k = 1.0 - delta_y.abs()` (post `/speed`) with
no lower bound, then divides `eye_new /= k`. This is only safe while `|delta_y| < zoom.speed`. At
exactly `|delta_y| == zoom.speed`, `k == 0.0` and the division produces `Inf`/`NaN`, corrupting
`self.eye` permanently (no self-recovery — every subsequent frame's `eye - center` starts from
the corrupted value). Beyond that, `k` goes negative, and dividing by a negative number flips
`eye_new`'s sign — the camera jumps through the `center` pivot to the geometrically opposite side,
the exact opposite of what a "zoom out" gesture should ever do.

**Magnitude:** Reachable via realistic real-world input, not just direct API misuse: a fast pinch
gesture on a large or high-resolution touchscreen can easily move a finger's raw screen-pixel
position by hundreds to over a thousand pixels between two consecutive `pointermove` samples
(especially under event coalescing/throttling), and some high-precision mice/trackpads emit
`DOM_DELTA_PIXEL` wheel events with per-event deltas in the hundreds. Both call sites feed their
raw, unbounded values straight into `zoom()` with zero pre-clamping anywhere in the pipeline.

**Entity Scope:** None — a code-level math defect, not an operational-entity concern.

## How Discovered

Task #62, a targeted code review of `mingl` dispatched under the standing bug-hunt mandate. The
reviewing agent flagged that `k = 1.0 - delta_y.abs()` has no lower bound, unlike the zoom-in
branch's `k = 1.0 + delta_y.abs()` (which is safe by construction — always `> 1.0`). Independently
re-verified before filing by direct source read and by tracing both real call sites:

```bash
$ sed -n '440,476p' module/min/mingl/src/controls/camera_orbit_controls.rs
# confirms the unclamped k = 1.0 - delta_y.abs() formula and the unconditional eye_new /= k

$ grep -n "Pinch\|\.zoom(\|old_dist\|new_dist" module/min/mingl/src/controls/camera_orbit_controls.rs
# CameraState::Pinch => ... camera.borrow_mut().zoom( old_dist - new_dist );   (line 671)
# CameraState::None  => ... camera.borrow_mut().zoom( delta_y );              (line 709, wheel)

$ sed -n '626,672p' module/min/mingl/src/controls/camera_orbit_controls.rs
# old_dist/new_dist computed from raw e.screen_x()/e.screen_y() via active_pointers tracking --
# no clamping anywhere between the pointer event and the zoom() call

$ grep -n "speed :" module/min/mingl/src/controls/camera_orbit_controls.rs
# zoom.speed default: 1000.0; zoom.max_distance/min_distance default: None (Default impl)
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre126 && mkdir -p /tmp/mre126/src
cat > /tmp/mre126/Cargo.toml <<'EOF'
[package]
name = "mre126"
version = "0.1.0"
edition = "2021"

[dependencies]
mingl = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/min/mingl", features = [ "camera_orbit_controls" ] }
EOF
cat > /tmp/mre126/src/main.rs <<'EOF'
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
  controls.zoom.speed = 1.0;

  controls.zoom( 1.5 ); // raw divisor: 1.0 - 1.5 = -0.5 (negative, pre-fix)

  println!( "{} {}", controls.eye.x(), controls.eye.x().is_finite() );
}
EOF
cd /tmp/mre126 && cargo run 2>&1 | tail -1
```

**Expected** (post-fix — divisor floored, camera stays on the original side, finite):
```
8388608 true
```

**Actual** (pre-fix — negative divisor flips the camera through the pivot):
```
-2 true
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre126 && cargo run 2>&1 | tail -1
# a large positive x = fixed; a negative x (or non-finite) = bug present
```
**What:** Violates `zoom`'s own doc contract — "A negative value zooms in, and a positive value
zooms out" — by letting an in-range-looking positive (zoom-out) input flip the camera to behave
as if it zoomed in past the pivot and out the other side, or corrupt it to non-finite.

**Known MRE limitation (check 205):** `mingl` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116/118-125's own
documented exception. `1.0`/`1.5`/`-0.5`/`-2.0` are exactly representable in `f32` with no
floating-point ambiguity this local dependency could be hiding.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The zoom-out branch's `k = 1.0 - delta_y.abs()` has no lower bound, letting a single event with `\|delta_y\| >= speed` produce `k <= 0`, corrupting or flipping `eye`. | ✅ Root Cause | Direct read of `zoom` (pre-fix) confirms no clamp exists between the raw formula and `eye_new /= k`. MRE confirms both the `k == 0` (non-finite) and `k < 0` (sign-flip) symptoms exactly as hand-derived. | E1, E2 |
| H2 | `min_distance`/`max_distance` clamping downstream of the division already prevents any observable corruption, making this a non-issue in practice. | ❌ Falsified | Both fields default to `None` (confirmed in `Default for CameraOrbitControls`), and even when set, the clamp only fires on `length < min_distance` / `length > max_distance` — a `NaN` `length` fails both comparisons (NaN comparisons are always `false` in IEEE 754), so neither clamp branch executes and the corrupted value passes through unmodified regardless of configuration. | E3 |
| H3 | This is unreachable in practice because no real caller ever passes `delta_y` that large — a pure API-misuse scenario, not a real defect. | ❌ Falsified | The crate's own built-in `Pinch` gesture handler feeds raw, unclamped `screen_x`/`screen_y`-derived pixel distances directly into `zoom()`, and the wheel handler feeds raw `e.delta_y()` directly — both are real, intended, already-wired call sites, not hypothetical misuse. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/min/mingl/src/controls/camera_orbit_controls.rs` (pre-fix `zoom`, lines ~440-453) | `delta_y /= self.zoom.speed; let k = if delta_y < 0.0 { 1.0 + delta_y.abs() } else { 1.0 - delta_y.abs() }; ...; eye_new /= k;` — the zoom-out branch's `k` has no floor, unlike the zoom-in branch's always-safe `1.0 + delta_y.abs()`. | H1 ✅ |
| E2 | `/tmp/mre126` run, pre-fix vs. post-fix, `eye=(1,0,0)`, `center=(0,0,0)`, `speed=1.0`, `delta_y=1.5` | Pre-fix: `k = 1.0 - 1.5 = -0.5`, `eye_new = (1,0,0)/-0.5 = (-2,0,0)` — flipped through the pivot. Post-fix: `k = (1.0-1.5).max(EPSILON) = EPSILON`, `eye_new = (1,0,0)/EPSILON ≈ (8388608,0,0)` — finite, same side. | H1 ✅ |
| E3 | `module/min/mingl/src/controls/camera_orbit_controls.rs` (`Default for CameraOrbitControls`, lines ~516-522, and the `zoom` clamp block, lines ~457-471) | `max_distance: None, min_distance: None` in the default config — both `if let Some(...)` clamp blocks are skipped entirely when unset; even when set, a `NaN` `length` fails both `<`/`>` comparisons under IEEE 754, so the clamp cannot rescue a `k == 0.0` corruption regardless of configuration. | H2 ❌ |
| E4 | `module/min/mingl/src/controls/camera_orbit_controls.rs` (pointer-move closure, lines ~648-671, and wheel closure, lines ~703-712) | `CameraState::Pinch => ... camera.borrow_mut().zoom( old_dist - new_dist );` with `old_dist`/`new_dist` computed from raw `e.screen_x()`/`e.screen_y()` via `active_pointers` tracking, no clamp. `CameraState::None => ... camera.borrow_mut().zoom( delta_y );` with `delta_y = e.delta_y() as f32`, also no clamp. Both are real, already-wired production call sites. | H1 ✅, H3 ❌ |

## Root Cause

```
zoom( &mut self, delta_y: f32 )
  delta_y /= self.zoom.speed
  k = if delta_y < 0.0 { 1.0 + delta_y.abs() }       <- zoom-in: always > 1.0, safe by construction
      else             { 1.0 - delta_y.abs() }       <- zoom-out: UNBOUNDED BELOW    ✗
  eye_new = (eye - center) / k                       <- k == 0 -> Inf/NaN; k < 0 -> sign flip
```

The zoom-out branch's divisor is only valid while its input, `delta_y.abs()`, is known to stay
strictly less than `1.0` (i.e. the raw event's magnitude stays under `zoom.speed`). Nothing in
the function enforces that — `delta_y` arrives directly from caller-supplied, externally-sourced
values (pointer pixel deltas, wheel deltas) with no inherent upper bound.

## Why Not Caught

Every existing zoom-out test kept `|delta_y| < speed` (the maximum tested was `delta_y=0.8`
against `speed=1.0`, in `test_zoom_with_non_origin_center`), never approaching the `k <= 0`
boundary. `zoom.max_distance`/`min_distance` both default to `None`, so no test using default
construction could have observed a masking clamp either. The two real call sites that do feed
unbounded raw input (`Pinch` gesture, wheel scroll) are behind `#[cfg(feature = "web")]` browser
event closures with no corresponding unit-test coverage of their numeric behavior.

## Fix Location

`module/min/mingl/src/controls/camera_orbit_controls.rs`, `pub fn zoom`. One change:

```rust
// before
let k = if delta_y < 0.0 { 1.0 + delta_y.abs() } else { 1.0 - delta_y.abs() };

// after
let k = if delta_y < 0.0 { 1.0 + delta_y.abs() } else { ( 1.0 - delta_y.abs() ).max( f32::EPSILON ) };
```

The floor only changes behavior in the previously-broken `|delta_y| >= 1.0` (post-`/speed`)
region — every already-correct case, including all pre-existing passing tests (max tested
`delta_y.abs()` of `0.8`, `0.9` in normalized-by-speed terms across all zoom tests), is
bit-for-bit unchanged, since `f32::EPSILON` (~1.19e-7) is many orders of magnitude below any
previously-exercised value.

## Prevention

Added `test_zoom_out_extreme_delta_does_not_corrupt_or_flip_eye` to
`tests/tests/camera_orbit_controls.rs`: drives `delta_y` to exactly the `k == 0.0` boundary
(`delta_y = 1.0` with `speed = 1.0`) and past it into `k < 0.0` (`delta_y = 1.5`), asserting in
both cases that the resulting `eye.x()` stays finite and positive (same side of the pivot as the
original position) — both assertions fail under the pre-fix formula.

**Pitfall:** a divisor derived as `1.0 - x.abs()` is only safe while `x` is known to stay inside
the unit interval — an external, unbounded input (screen pixels, wheel events) can never be
assumed to satisfy that on its own; clamp at the boundary the formula actually has, don't trust
the caller to stay within it. A sibling branch that is "safe by construction"
(`1.0 + delta_y.abs()`, always `> 1.0`) is not evidence the other branch is safe too — each
branch's own safety must be checked independently.

## Generalized Version

**Broken assumption:** "A formula that looks structurally symmetric to its sibling branch (`1.0 +
x` vs `1.0 - x`) shares that sibling's safety properties" — false; `1.0 + x.abs()` and
`1.0 - x.abs()` are only symmetric in form, not in range: the former is bounded away from zero
for all real `x`, the latter is not.

**Confirmed general rule:** for any divisor computed as `constant - unbounded_input.abs()` (or
equivalent), verify the input's actual bound at the call site(s), not just at the formula's own
definition — an input that "looks like it stays small" in test fixtures may be raw, externally
sourced, and genuinely unbounded in production (pixel deltas, wheel deltas, network payloads).
When the input's true bound cannot be guaranteed, clamp the divisor itself at its unsafe boundary
rather than trusting every caller to pre-validate.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #62's targeted code review of `mingl`; confirmed reachable by tracing both the `Pinch` gesture handler and the wheel-scroll handler back to their raw, unclamped browser-event sources before filing. |
| 2026-08-15 | fixed | Changed the zoom-out branch's divisor to `( 1.0 - delta_y.abs() ).max( f32::EPSILON )`. 3-field `Fix(BUG-126)`/`Root cause`/`Pitfall` comment added at the fix site. |
| 2026-08-15 | verified | Added `test_zoom_out_extreme_delta_does_not_corrupt_or_flip_eye` to `tests/tests/camera_orbit_controls.rs`; scoped test run (`cargo nextest run -p mingl --all-features` via `longrun`) passed with the new test green alongside the pre-existing suite (56/56). |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16; fix not present anywhere in this session's own context — approached as a fresh reader). Independently re-read `camera_orbit_controls.rs::zoom` (confirmed `( 1.0 - delta_y.abs() ).max( f32::EPSILON )` genuinely present on the zoom-out branch; confirmed the zoom-in branch `1.0 + delta_y.abs()` is safe by construction; confirmed `eye_new /= k` — the only unbounded-divisor division in the file — appears nowhere else, and `pan()` uses an unrelated, bounded `window_size`-derived multiplier, not a shared defect shape) and the `bug_reproducer(BUG-126)` test body (non-tautological — drives `delta_y` to both the `k == 0.0` boundary and past it into `k < 0.0`, asserting `eye.x()` stays finite and positive in both cases). Fresh `cargo nextest run -p mingl --all-features` via `longrun`: 56/56 passed. `cargo clippy -p mingl --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-125/126 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-computed `-2.0`/`8388608.0`; adversarial pass independently re-ran the MRE against the current (fixed) source (`cargo run` → `8388608 true`, exact match) rather than trusting the confirming pass's arithmetic alone. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed this file correctly declares no `**Related Bugs:**` beyond noting BUG-125 as independently-rooted despite the shared task #62 review — different function, no shared code path. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass independently re-read the `Pinch` handler body (lines 648-671) directly from source rather than accepting the confirming pass's description of `old_dist`/`new_dist`'s provenance. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked the zoom-in branch (`1.0 + delta_y.abs()`) independently for the same defect shape — confirmed it is safe by construction (always `> 1.0` for any real `delta_y`), so no matching fix is needed there. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `mingl`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `zoom`'s own `k` computation; the `Pinch`/wheel event closures that call `zoom()` are unmodified — the fix corrects the callee rather than attempting to clamp every caller individually. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility — it corrects `zoom` to honor its own doc contract ("zooms the camera... along its viewing direction") instead of occasionally flipping through the pivot. | — |

**Reproduced:** YES — `/tmp/mre126` pre-fix: `eye.x() == -2.0` (flipped through the pivot)
instead of remaining positive/finite, for `eye=(1,0,0)`, `center=(0,0,0)`, `speed=1.0`,
`delta_y=1.5`, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/min/mingl/src/controls/camera_orbit_controls.rs` | `zoom`: changed the zoom-out branch's divisor from `1.0 - delta_y.abs()` to `( 1.0 - delta_y.abs() ).max( f32::EPSILON )`. `Fix(BUG-126)`/`Root cause`/`Pitfall` comment added. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/mingl/tests/tests/camera_orbit_controls.rs` | Added `test_zoom_out_extreme_delta_does_not_corrupt_or_flip_eye` (`bug_reproducer(BUG-126)`, 5-section doc comment, `k == 0` and `k < 0` boundary fixtures). |

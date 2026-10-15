# BUG-278: `CharacterControls::rotate()`'s pitch delta uses the wrong sign, inverting vertical mouselook

- **Severity:** High (no crash -- a silent behavioral inversion of the primary interactive
  control of a public, documented character controller, with no workaround exposed by the
  public API)
- **state:** Completed
- **Affects:** `CharacterControls::rotate()` (`src/controls/character_controls.rs`) -- the
  vertical (pitch) mouselook axis
- **Component:** `module/min/mingl` (`src/controls/character_controls.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`CharacterControls::rotate()`, the mouse-delta-to-orientation handler for the WASD+mouselook
character controller, applies `self.pitch -= delta_y * self.rotation_sensitivity;` for the
vertical (pitch) axis. `delta_y` is fed verbatim from `MouseEvent.movementY` (DOM convention:
positive when the pointer moves down) via `mouse_move_closure_make`. Because increasing `pitch`
already rotates the character's `forward` vector to point more downward (`forward.y` more
negative, confirmed via the underlying `Quat::from_angle_x` Hamilton product), subtracting a
positive `delta_y` on mouse-down *decreases* pitch and makes the character look UP instead of
DOWN -- the vertical mouselook axis is inverted relative to the near-universal FPS/character-
controller convention ("mouse down looks down"). The horizontal (yaw) axis, by contrast, is
correct: `self.yaw -= delta_x * self.rotation_sensitivity;` is the right sign because the
character's own `right_xz()` base vector is `-X` while increasing yaw rotates `forward` toward
`+X`, so subtracting is required there to make "mouse right" turn the character right.

## Impact

**Who is affected:** any consumer of `mingl::controls::character_controls::CharacterControls`
(feature `character_controls`) that binds `controls_bind_to_input` to a canvas and lets the user
look around with the mouse while pointer-locked -- i.e. every real usage of this public,
documented WASD+mouselook character controller.

**What breaks:** moving the mouse down tilts the view up, and moving the mouse up tilts the view
down -- backwards from the documented behavior ("Mouse: Rotate the character (yaw and pitch)")
and from the conventional behavior of essentially every FPS-style mouselook implementation. This
is a silent behavioral defect, not a crash: the code compiles and runs without error, but the
interactive control scheme is disorienting/broken for any user of the resulting application, with
no public API to override just this one axis's sign.

**Entity Scope:** `None` -- source-level sign defect, not entity directory instances.

## How Discovered

During this session's dedicated review of `module/min/mingl`'s `controls/`+`web/` facade file
list (12 files), cross-checked the mouse-delta sign convention in `CharacterControls::rotate()`
against the underlying math crate's actual quaternion rotation formulas (`Quat::from_angle_x`/
`from_angle_y`, and `Quat::multiply`'s Hamilton product, both read directly from
`module/math/ndarray_cg/src/quaternion/{arithmetics,operator/mul}.rs`) rather than assuming the
doc comments were accurate. Derived by hand (and cross-verified via direct quaternion arithmetic)
that increasing `pitch` rotates `forward` downward, then found the mouse-down delta was wired to
*decrease* pitch -- the opposite of the yaw axis's (correct) analogous relationship one line
above it.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p mingl --all-features character_controls
```
**Expected** (fixed): compiles, all 3 new tests pass.
**Actual** (pre-fix, confirmed via temporary `git stash push -- module/min/mingl/src/controls/character_controls.rs`, real run):
```
failures:
    tests::character_controls::test_rotate_pitch_matches_mouse_down_looks_down_convention
    tests::character_controls::test_rotate_pitch_mouse_up_looks_up

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.00s
```

## Root Cause

`src/controls/character_controls.rs`, `CharacterControls::rotate()` (pre-fix):
```rust
pub fn rotate( &mut self, delta_x : f64, delta_y : f64 )
{
  // Update yaw (left/right rotation around Y axis)
  self.yaw -= delta_x * self.rotation_sensitivity;

  // Update pitch (up/down rotation around X axis)
  self.pitch -= delta_y * self.rotation_sensitivity;
  // ...
```
Both lines share the same `-=` operator, but they need different signs:
- **Yaw** is correct with `-=`: `right_xz()`'s base vector is `QuatF64::from([-1.0,0.0,0.0,0.0])`
  (i.e. `-X`), while `Quat::from_angle_y(yaw)` applied to the base `forward = (0,0,1)` rotates it
  toward `+X` as `yaw` increases (verified via `Quat::multiply`'s Hamilton product:
  `from_angle_y(PI/2) * (0,0,1,0) * conj = (1,0,0,0)`). Since `forward` moving toward `+X` is
  moving *away* from the character's own `right` (`-X`), increasing yaw turns the character
  *left*; subtracting `delta_x` (DOM `movementX`, positive = pointer right) is therefore required
  to make "mouse right" produce a right turn.
- **Pitch** needed `+=`, not `-=`: `Quat::from_angle_x(pitch)` applied to `forward = (0,0,1)`
  yields `(0, -sin(pitch), cos(pitch))` (verified by hand at `pitch = PI/2`: result `(0,-1,0)`,
  straight down). So increasing `pitch` *already* means "look down" -- there is no compensating
  inversion analogous to yaw's right-vector relationship. Mapping `delta_y` (DOM `movementY`,
  positive = pointer down) onto pitch therefore needed `+=` to reproduce "mouse down -> look
  down"; the `-=` was evidently copy-pasted from the yaw line immediately above without
  re-deriving pitch's own sign relationship, inverting the axis.

## Why Not Caught

No test file existed for `character_controls.rs` prior to this session -- only its sibling
`camera_orbit_controls.rs` (a different controller, with its own already-audited/fixed sign
conventions) had coverage. The inversion has no compile-time signature and is invisible to casual
manual testing unless someone explicitly checks that dragging the mouse down tilts the view down
rather than up; a quick play-test that only confirms "the view moves when I move the mouse"
(without checking the *direction*) would not catch it.

## Fix Applied (2026-08-17)

**`src/controls/character_controls.rs`:** changed
`self.pitch -= delta_y * self.rotation_sensitivity;` to
`self.pitch += delta_y * self.rotation_sensitivity;` in `CharacterControls::rotate()`, with an
inline `Fix(BUG-278)`/`Root cause`/`Pitfall` comment recording the yaw-vs-pitch sign-derivation
asymmetry.

**`tests/tests/character_controls.rs`** (new file, first test coverage for this source file):
- `test_rotate_pitch_matches_mouse_down_looks_down_convention` -- asserts the exact hand-computed
  `pitch` value and full `forward()` vector after a downward mouse delta.
- `test_rotate_pitch_mouse_up_looks_up` -- mirror-image check for an upward mouse delta.
- `test_rotate_yaw_matches_mouse_right_turns_right_convention` -- confirms the (already-correct)
  yaw axis is unaffected by the pitch fix.

Registered in `tests/tests.rs`
(`#[ cfg( all( feature = "character_controls", feature = "web" ) ) ] mod character_controls;`)
and added to `tests/readme.md`'s Responsibility Table.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p mingl --all-features character_controls` -- pre-fix (temporary
  `git stash push -- module/min/mingl/src/controls/character_controls.rs`, new tests left live):
  1 passed / 2 failed, exactly the two pitch tests, as diagnosed. Post-fix (`git stash pop`):
  3 passed / 0 failed.
- `cargo test -p mingl --all-features` (full suite, unfiltered): 62 passed / 0 failed
  (integration), 0/0 unit, 10 ignored / 0 failed doctests.
- `cargo clippy -p mingl --all-targets --all-features -- -D warnings`: clean, exit 0 (confirmed
  as a genuine fresh check, not a stale-cache false-clean, via `touch` on the changed files
  forcing a real ~35s "Checking mingl" recompile with zero diagnostics).

## Generalized Version

**Broken assumption:** when two sibling input axes (here, yaw and pitch) share an
identical-looking delta-application line (`angle -= delta * sensitivity`), the second line being
visually consistent with the first is not evidence the second is *correct* -- each axis's sign
must be independently re-derived from its own rotation-to-basis-vector relationship. Yaw needed
negation because increasing yaw rotates `forward` away from the character's own `right` vector;
pitch needed no such negation because increasing pitch already means "look down" directly.
Copying an established sibling's sign, rather than re-deriving it, is exactly how one axis of a
two-axis (or N-axis) rotation input ends up silently inverted while the other stays correct -- and
a test that only checks "the value changed" (not the hand-computed direction/vector) would not
catch it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's dedicated review of `module/min/mingl`'s `controls/`+`web/` facade file list (12 files: `controls/{camera_orbit_controls,character_controls,mod}.rs`, `web.rs`, `web/{canvas,dom,exec_loop,file,future,log,model,model/obj}.rs`; all others clean -- `camera_orbit_controls.rs`'s trig/sign math was independently re-derived and cross-checked against `mat3x3::from_angle_x/y`/`from_axis_angle` and found correct; the `web.rs`/`web/{dom,file}.rs` URL-resolution logic was likewise re-derived and found correct). Root cause: `CharacterControls::rotate()`'s pitch line copy-pasted the yaw line's `-=` operator without re-deriving pitch's own sign relationship, inverting vertical mouselook. Fixed by changing `-=` to `+=` for the pitch delta. Verified via 3 new native unit tests (confirmed 2 fail pre-fix / 3 pass post-fix via temporary `git stash` revert-and-rerun), the full `--all-features` suite (62/62 integration, 0/0 unit, 10 ignored doctests), and clean clippy (fresh-compile-confirmed, not cache-stale). Filed as BUG-278, not the provisionally-scanned BUG-273 (nor its first renumbering to BUG-275), after a fresh on-disk ID re-scan immediately before filing found the ID space had moved substantially further under heavy concurrent load from parallel session forks: BUG-273 had been independently claimed twice over, BUG-274 was taken, and by the time this fork returned to file, BUG-275 (`storage_texture_binding_layout_default_format_not_storage_capable`), BUG-276 (`render_target_2d_zero_size_panic`), and BUG-277 (`uniform_matrix_upload_copy_pasted_vector_error_message`) had *also* all been claimed by other concurrent forks in the interim. The in-source `Fix(BUG-NNN)` comment and the test file's `test_kind: bug_reproducer(...)` marker were renumbered from BUG-275 to BUG-278 accordingly before filing. |

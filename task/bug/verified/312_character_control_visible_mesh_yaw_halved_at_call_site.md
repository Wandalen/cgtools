# BUG-312: `character_control` example halves the visible character mesh's yaw at its `Quat::from_angle_y` call site, desyncing it from the camera's own (correctly unhalved) orbit

- **Severity:** Medium (active, visually-wrong behavior -- not latent -- but confined to 1
  non-critical example/demo crate, not library code; `CharacterControls` itself, the library
  type being called into, is correct and internally consistent)
- **state:** Verified
- **Affects:** the visible character mesh's rotation in the `character_control` example, which
  under-rotates to half the camera's own orbit angle while walking with WASD
- **Component:** `examples/minwebgl/character_control` (fix); `module/min/mingl` (reproducer
  test only -- `CharacterControls` itself is not buggy)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18

## Symptom

`mingl::controls::CharacterControls` stores `yaw`/`pitch` as raw radians (`yaw`'s own doc
comment: "in radians") and builds `rotation` internally, at all 4 of its own call sites
(`rotate()`, `rotation_set()`, `forward_xz()`, `right_xz()`), as
`QuatF64::from_angle_y( self.yaw ) * QuatF64::from_angle_x( self.pitch )` -- `self.yaw` passed
unmodified. `examples/minwebgl/character_control/src/main.rs` instead orients the *visible
character mesh* with `Quat::from_angle_y( character_controls.yaw() as f32 / 2.0 )` -- an extra
`/ 2.0` with no basis anywhere in `CharacterControls` itself.

## Impact

**Who is affected:** anyone running (or visually inspecting the rendered output of) the
`character_control` example.

**What breaks:** the camera orbits/looks via `character_controls.forward()` (unhalved,
correct), but the rendered character mesh's own facing direction is set from the halved
expression -- so while walking with WASD, the visible mesh only rotates at half the rate the
camera (and thus the player's actual intended facing) does. After turning 180°, the camera
faces fully backward but the mesh has only turned 90°; the character visibly desyncs from the
direction it is walking. Purely cosmetic: nothing panics, no data is corrupted, and it does not
affect any library API consumed elsewhere.

**Entity Scope:** `None` -- source-level call-site defect, not entity directory instances.

## How Discovered

Found during this session's workspace-wide bug-hunt pass, `module/math` + `module/min` review
stage, immediately after filing BUG-311 (a sibling `from_angle_y` misuse in 3 other example
crates). Grepping `examples/minwebgl/character_control` for `from_angle_y`/`from_angle_x` calls
surfaced this crate's own single call site (`main.rs:437`, pre-fix) alongside a division by
`2.0` with no accompanying comment or explanation. Cross-checking every one of
`CharacterControls`'s own 4 internal call sites (`module/min/mingl/src/controls/
character_controls.rs`) confirmed none of them halve `self.yaw` -- this call site is the only
place in the whole call graph that does.

## Minimum Reproducible Example

**Verify Command** (library-level reproducer -- `character_control` is a `fn main()`-only
binary with no test harness of its own; see Why Not Caught):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo nextest run -p mingl -E 'test(test_yaw_passed_unhalved_to_from_angle_y_matches_rotation)' --all-features
```
**Expected** (fixed -- this test is not itself a regression test on `CharacterControls`, since
it was never buggy; it locks in the correct-vs-wrong boundary the example's call site crossed):
1 passed / 0 failed.

**Actual** (the raw call-site expression's real behavior): the test sets a known yaw via
`rotation_set( yaw, 0.0 )` (at `pitch = 0.0`, `rotation()` equals `from_angle_y( yaw )` exactly,
since `from_angle_x( 0.0 )` is the identity quaternion), then asserts `controls.rotation()`
matches `from_angle_y( yaw )` (correct) via `assert_abs_diff_eq!`, and separately asserts (via
`assert_ne!`) it does NOT match `from_angle_y( yaw / 2.0 )` -- the exact expression the example's
pre-fix call site computed. Both assertions executed and passed, empirically confirming the
divergence between what the mesh was told to face and what the controller's own `rotation()`
actually is.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `CharacterControls` always passes `self.yaw` to `from_angle_y` unmodified, at every one of its own call sites | ✅ Root Cause | 4/4 internal call sites (`rotate`, `rotation_set`, `forward_xz`, `right_xz`) pass `self.yaw` directly, no scaling | E1-E4 |
| H2 | The example's call site is the sole place a `/ 2.0` is applied to yaw anywhere in the reachable call graph | ✅ Verified | `grep -rn "yaw()" examples/minwebgl/character_control` shows exactly 2 uses: line 437 (halved) and line 440 (unhalved, via `forward()`) | E5 |
| H3 | The camera (via `forward()`) and the mesh (via the buggy line) visibly desync as a result | ✅ Verified | `forward()` is built from `self.rotation` (unhalved internally); the mesh's `rotation_set` call receives half that angle -- different quaternions from the same underlying `yaw` | E1, E6 |
| H4 | Nothing in this crate could have caught this mechanically -- no test harness exists | ✅ Verified | `find examples/minwebgl/character_control -iname '*test*'` returns nothing | E7 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/min/mingl/src/controls/character_controls.rs:83,93,158-159` | `rotate()`/`forward_xz()`/`right_xz()`: `let quat_yaw = QuatF64::from_angle_y( self.yaw );` -- unmodified | H1 |
| E2 | `module/min/mingl/src/controls/character_controls.rs:228` | `rotation_set()`: same unmodified `QuatF64::from_angle_y( self.yaw )` | H1 |
| E3 | `module/min/mingl/src/controls/character_controls.rs:37` | Field doc comment: `/// Current yaw angle (rotation around Y axis) in radians` -- the value `from_angle_y` expects directly, no further scaling implied | H1 |
| E4 | `module/min/mingl/src/controls/character_controls.rs:56` | `pub fn rotation( &self ) -> QuatF64` returns `self.rotation`, built from the unmodified `self.yaw`/`self.pitch` pair | H1 |
| E5 | `examples/minwebgl/character_control/src/main.rs` (pre-fix lines 437, 440) | Line 437: `Quat::from_angle_y( character_controls.borrow().yaw() as f32 / 2.0 )`; line 440: `character_controls.borrow().forward()` (unhalved) | H2 |
| E6 | `module/min/mingl/src/controls/character_controls.rs:441` (pre-fix `main.rs`) | `camera.controls_get().borrow_mut().eye = center - forward * zoom` -- camera uses the unhalved `forward()` directly, confirming the asymmetry is call-site-local to the mesh orientation line only | H3 |
| E7 | Terminal output (this section, `find` command) | Empty output -- no `tests/` directory or test file anywhere in `character_control` | H4 |

## Root Cause

```
mingl::controls::CharacterControls  -- self.yaw stored + consumed as RADIANS, unmodified,
                                         at all 4 of its own internal call sites
  |
  |  examples/minwebgl/character_control/src/main.rs:437 (pre-fix)
  |
  +-- Quat::from_angle_y( character_controls.borrow().yaw() as f32 / 2.0 )
         intended: orient the mesh to face the same direction as the controller
         actual:   mesh yaw = controller yaw / 2  ==  half the camera's own rotation rate
```
`CharacterControls` is correct and internally consistent -- every one of its own methods that
builds a rotation from `yaw` uses the raw stored value directly. The defect is entirely at this
one example call site: a spurious `/ 2.0` with no corresponding halving anywhere else in the
type it's calling into.

## Why Not Caught

`character_control` has no `tests/` directory or test file (confirmed via `find`, E7) -- it is a
`fn main()`-only WebGL demo binary, verified only by running it in a browser and watching the
character walk. The desync is only visible by comparing the character mesh's facing direction
against the camera's own orbit while actively turning -- easy to miss in a quick smoke test
where the camera itself (unaffected) already looks correct, and the character model may not be
being watched closely relative to its own facing direction during casual play-testing.

## Fix Location

`examples/minwebgl/character_control/src/main.rs:437` (now :437-443 with the added comment):

```rust
// Before:
character.borrow_mut().rotation_set( Quat::from_angle_y( character_controls.borrow().yaw() as f32 / 2.0 ) );

// After:
character.borrow_mut().rotation_set( Quat::from_angle_y( character_controls.borrow().yaw() as f32 ) );
```
Source comment (`Fix(BUG-312)`/`Root cause`/`Pitfall`) added immediately above the call.

**`module/min/mingl/tests/tests/character_controls.rs`** (new test, added to the existing
crate's test file rather than to the untestable example binary -- see Why Not Caught and this
repo's own `rulebook.md` § Test placement): `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation`
sets a known yaw via `rotation_set( yaw, 0.0 )` and asserts `controls.rotation()` matches
`QuatF64::from_angle_y( yaw )` (correct) but not `QuatF64::from_angle_y( yaw / 2.0 )` (what the
example's pre-fix call site computed). This locks in the correct/incorrect boundary the call
site crossed; it does not itself test the example crate (which remains untestable without a
live browser), only the library contract it misused.

## Prevention

Detection command for this exact pattern (a caller-side arithmetic modification applied to a
value already documented as being in the target unit -- narrow by construction, catches this
one shape, not a general angle-unit checker):
```bash
grep -rn "\.yaw()\s*as\s*f32\s*/" examples/ --include=*.rs
```
Re-run after this fix (confirmed): zero matches workspace-wide -- this was the only call site of
this exact shape.

**Pitfall:** a field whose own doc comment states its unit ("in radians") gives no compile-time
or run-time signal when a caller applies an unexplained scaling factor before passing it
onward -- any arithmetic modification to a value pulled from a well-documented accessor should
be immediately suspicious and cross-checked against how the same value is used elsewhere in the
same call graph (here, the camera's own `forward()` use, two lines below, would have shown the
inconsistency immediately).

## Generalized Version

**Broken assumption:** a value already in the correct unit/scale for its destination API
benefits from an additional caller-side scaling factor.

Fails whenever:
1. A struct method/field is documented (or self-evidently, by its own internal usage) to already
   be in the unit a downstream API expects, AND
2. A caller applies an arithmetic transformation to that value before passing it to that API,
   with no comment or evidence justifying why

**Detection invariant:**
```
for every call site passing a struct's own field/accessor value into a rotation constructor:
  the argument expression must match how that same struct's own internal methods
  consume the same field, unless a documented reason justifies the divergence
```
Second confirmed instance of a `from_angle_y`-adjacent angle-scale defect in this workspace this
session, after BUG-311 (a degrees/radians unit confusion). Distinct root cause: BUG-311 was a
unit-of-measurement error (degrees passed where radians were expected); this one is a spurious
caller-side scaling factor applied to an already-correct radians value, with no unit confusion
involved at all.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during this session's workspace-wide bug-hunt task, `module/math`+`module/min` review stage, immediately after BUG-311, by cross-checking `character_control`'s own `from_angle_y` call against `CharacterControls`'s 4 internal call sites |
| 2026-08-18 | fix_applied | `Quat::from_angle_y( character_controls.borrow().yaw() as f32 / 2.0 )` -> `Quat::from_angle_y( character_controls.borrow().yaw() as f32 )` |
| 2026-08-18 | verified | `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation` (bug_reproducer) passes; full `mingl` suite (63 tests) and clippy (native for `mingl`, wasm32 for `character_control`, `-D warnings`) clean |

## Refs: src/

- `examples/minwebgl/character_control/src/main.rs` — removed the stray `/ 2.0` on the yaw passed to `Quat::from_angle_y`

## Refs: tests/

- `module/min/mingl/tests/tests/character_controls.rs` — added `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation` (bug_reproducer)

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass mechanically confirmed all 12+2 headers present (`grep -n "^## "`) and re-read each body for substantive, non-generic content -- none found thin | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Adversarial pass re-ran the documented verify command fresh from repo root (byte-for-byte, not the module-scoped form used during drafting) -- exit 0, 1 passed, matching "Expected" exactly; separately re-read the test body to confirm both assertions are unconditional and sequential (no early return), matching the "Actual" section's claim that both executed for real | — |
| D3 | Cross-Reference Integrity | — | 🟢 | `grep -rln "BUG-312" --include=*.rs --include=*.md .`: exactly 3 files (report + `main.rs` + `character_controls.rs`), matching the 2 `## Refs:` entries exactly in both directions | — |
| D4 | Root Cause Quality | 🟠 | 🟢 | Adversarial pass re-verified every cited line number (E1: 83,93,158-159; E2: 228; E4: 56) against current file content via `sed -n` -- all accurate; separately caught that E3 cited no line number (structural inconsistency vs. every other Evidence row) and, on checking, found the actual line (37) | Reworded E3 to cite `character_controls.rs:37` and quote the exact doc-comment text, matching the precision of every other Evidence row |
| D5 | Execution Scope | — | 🟢 | `git diff` (full content, not just `--stat`) re-read on both non-report touched files: `main.rs` shows only the intended comment block + one-line fix; `character_controls.rs` shows a pure append of exactly the one new test function -- no scope creep in either | — |
| D6 | Crate Scope Unity | — | 🟢 | Broader `grep -rn "yaw()" examples/ module/` (excluding the test file itself) re-confirmed exactly one call site workspace-wide -- no missed sibling instance requiring consolidation, unlike BUG-311's 3-crate case | — |
| D7 | Crate Locality | — | 🟢 | `git status --porcelain` on all touched paths re-checked immediately before this gate: `task/bug/readme.md`'s modified state is pre-existing (BUG-300/311 registration + concurrent actor's own prior additions), not new drift. Live highest ID re-verified via the corrected unbounded `find` command: 312 (this report itself) -- no concurrent actor filed 312+ during this investigation | — |
| D8 | Crate Single Responsibility | — | 🟢 | Re-read the surrounding closure context (`main.rs:395-444`) for any other latent defect near the fix site -- the initial `rotation_set( 0.0, 0.0 )` at line 395 and the rest of the update closure are all consistent and correct; fix stayed scoped to exactly the one reported defect | — |
| **Total** | | — | 🟢 | 0 open | 1/1 |

**Reproduced:** YES — `test_yaw_passed_unhalved_to_from_angle_y_matches_rotation` exit 0 (1 passed),
re-run fresh from repo root with the exact documented command. Full `mingl` suite (63 tests) and
`cargo clippy -p mingl --all-targets --all-features -- -D warnings` clean. `character_control`
re-checked via `cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -D
warnings`, exit 0.

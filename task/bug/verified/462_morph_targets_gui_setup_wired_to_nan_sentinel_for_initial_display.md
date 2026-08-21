# BUG-462: `morph_targets`' GUI sliders always display NaN on load instead of the animation's current weights

- **Severity:** Medium (no crash/panic -- `lil-gui`/`serde_wasm_bindgen` tolerate the NaN values,
  but every one of the 60 morph-weight sliders visually initializes wrong)
- **state:** Verified
- **Affects:** `examples/minwebgl/morph_targets`'s GUI panel -- all 60 weight sliders (`w0`..`w59`)
  on initial page load, before the user drags any of them.
- **Component:** `examples/minwebgl/morph_targets` (`src/main.rs`, `src/gui_setup.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Fix Task:** [506](../../verifying/506_register_morph_targets_gui_setup_initial_weights_display_fix_closes_bug462.md)
- **Related Bugs:** Directly downstream of BUG-330 (`gui_weights` filled with `f32::NAN` as a
  "not yet overridden by the user" sentinel, fixed 2026-08-18) -- this bug is a second, distinct
  defect at the *caller* of `gui_setup::setup` that only became visible because BUG-330 changed
  `gui_weights`'s untouched-value from `0.0` to `NaN`. Not a regression of BUG-330's own fix --
  `gui_weights` filled with NaN is correct; the bug is that `gui_setup::setup` was reading that same
  NaN-filled buffer for the sliders' *initial displayed value*, a role BUG-330 never intended it for.

## Symptom

```rust
// pre-fix -- src/main.rs
let gui_weights = Rc::new( RefCell::new( vec![ f32::NAN; 60 ] ) ); // BUG-330's sentinel buffer
// ...
gui_setup::setup( gltf.animations.clone(), &current_animation, &gui_weights );
//                                                               ^^^^^^^^^^^ single param, dual role
```

```rust
// pre-fix -- src/gui_setup.rs
pub fn setup( animations, current_animation, weights : &Rc< RefCell< Vec< f32 > > > )
{
  // ...
  weight_settings_init( &mut settings, &weights.borrow() ); // reads NaN for every slider's initial value
  // ...
  weight_sliders_bind( &gui, &object, weights ); // writes user overrides into the same buffer -- correct
}
```

`gui_setup::setup`'s single `weights` parameter served two different roles at once: seeding each
slider's *initial displayed value* (`weight_settings_init`) and being the *write target* for user
overrides (`weight_sliders_bind`). The caller had no buffer that was correct for both roles
simultaneously -- it passed `gui_weights` (all `NAN`, correct for the write-target role, wrong for
the initial-display role) rather than the real animation-driven `weights` buffer (which would have
been correct for display but gets overwritten every frame by the animation system, silently
discarding any slider drag on the very next frame if used as the write target instead).

## Impact

**Who is affected:** Anyone opening the `morph_targets` demo -- all 60 sliders show `NaN` on load.

**What breaks:** Visual/UX only -- `lil-gui` renders `NaN` in each numeric slider's text field
until the user drags it (at which point it becomes a real number and starts working correctly,
since `weight_sliders_bind`'s write-back was already wired to the correct buffer). No panic, no
render corruption -- the underlying animation itself renders correctly regardless, since the
per-frame merge logic in `main.rs` (`if !gui_weights[i].is_nan() { weights_mut[i] = gui_weights[i]; }`)
already only applies non-NaN overrides.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `examples/minwebgl/morph_targets`, tracing
`gui_setup::setup`'s single call site in `main.rs` against `weight_settings_init`'s read and
`weight_sliders_bind`'s write, and noticing both read from and wrote to the exact same NaN-filled
buffer for two semantically different purposes.

## Manual Reproduction / Verification

No dedicated automated MRE test was added -- this defect is purely about which buffer a UI setup
function is wired to at its one call site (a `lil-gui`/`wasm-bindgen` UI construction path with no
native-testable pure-logic core), consistent with this sweep's granted exception for example
crates. Verified instead by:

1. Hand-tracing the pre-fix data flow: `main.rs` passes `gui_weights` (all `NAN`) as `setup`'s
   single `weights` argument; `weight_settings_init(&mut settings, &weights.borrow())` then copies
   `NAN` into every one of `Settings`'s 60 `wN : f32` fields, which `lil-gui` displays verbatim.
2. Hand-tracing the post-fix data flow: `main.rs` now passes both `&weights.borrow()` (the real,
   animation-seeded buffer, for `initial_weights`) and `&gui_weights` (unchanged, for
   `gui_weights`) -- `weight_settings_init` now reads the real weights, `weight_sliders_bind` still
   writes to `gui_weights`, preserving BUG-330's override mechanism exactly as before.
3. `cargo check -p morph_targets --target wasm32-unknown-unknown` -- clean, no errors.

**Verify Command:**
```bash
cd examples/minwebgl/morph_targets && cargo check --target wasm32-unknown-unknown
```

## Root Cause

`gui_setup::setup` took a single `weights : &Rc<RefCell<Vec<f32>>>` parameter used for two
different roles -- "current displayed value" (read once, at setup) and "user override storage"
(written continuously, by slider drags). A single buffer can only be correct for one of those
roles at a time; the caller was forced to pick one (`gui_weights`, correct for the write role) at
the expense of the other (wrong initial display).

## Why Not Caught

This defect was only ever visible in-browser (the sliders' initial numeric display), with no
automated coverage of the GUI panel's initial values, and BUG-330's own fix (changing the
sentinel from `0.0` to `NAN`) is what turned this from "sliders start at 0" (silently plausible,
easy to miss) into "sliders start at NaN" (still requires visually inspecting the panel to notice).

## Fix Location

- `examples/minwebgl/morph_targets/src/gui_setup.rs`: `setup`'s single `weights` parameter split
  into `initial_weights : &[ f32 ]` (read once, for the sliders' initial displayed value) and
  `gui_weights : &Rc< RefCell< Vec< f32 > > >` (the override write-target) -- `weight_settings_init`
  now reads `initial_weights`, `weight_sliders_bind` now writes `gui_weights`.
- `examples/minwebgl/morph_targets/src/main.rs`: call site updated to
  `gui_setup::setup( gltf.animations.clone(), &current_animation, &weights.borrow(), &gui_weights )`,
  passing the real `weights` buffer for the new `initial_weights` role alongside the unchanged
  `gui_weights` for its own role.

## Prevention

None added beyond the fix itself and the wasm32 compile check, per this sweep's exception for
example crates -- the two-role split is now structurally enforced by the function signature itself
(two distinctly-named, distinctly-typed parameters instead of one overloaded one), which is the
most direct prevention available without introducing GUI-level test scaffolding this crate does
not have.

## Pitfall

One buffer used for two different roles ("current displayed value" vs. "user override storage")
can only ever be correct for one of them at a time -- the fix is to give each role its own
parameter, not to swap which single buffer is threaded through (swapping to the real `weights`
buffer alone would have "fixed" the display but broken `weight_sliders_bind`'s write-back, since
`weights` is overwritten every frame by the animation system, silently discarding slider drags).
When a bug report's suggested fix is "just pass X instead of Y", trace *every* existing use of the
parameter being changed, not just the one that prompted the report.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX sweep of `examples/minwebgl/morph_targets`. |
| 2026-08-20 | fixed | Split `gui_setup::setup`'s single `weights` parameter into `initial_weights`/`gui_weights`; documented with `Fix(BUG-462)`/`Root cause`/`Pitfall`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Fix correctness (hand-trace + compile + no BUG-330 regression) | — | 🟢 | Adversarial pass: specifically checked whether the naive "just pass `&weights` instead" fix (the literal wording a shallow reading of the finding might suggest) would have broken BUG-330's override mechanism -- confirmed it would have (see Pitfall), and confirmed the actual two-parameter fix preserves `weight_sliders_bind`'s write target unchanged. `cargo check -p morph_targets --target wasm32-unknown-unknown` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-462)`/`Root cause`/`Pitfall` 3-field format applied at the fix site in `gui_setup.rs`, cross-referencing BUG-330 by number. | — |

**Reproduced:** Confirmed via hand-trace of the pre-fix data flow (not a live browser render -- see
Manual Reproduction / Verification for why an automated MRE was not added). 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/morph_targets/src/gui_setup.rs` | `setup`: split `weights` into `initial_weights : &[f32]` / `gui_weights : &Rc<RefCell<Vec<f32>>>`; `Fix(BUG-462)`/`Root cause`/`Pitfall` comment. |
| `examples/minwebgl/morph_targets/src/main.rs` | Updated `gui_setup::setup` call site to pass both `&weights.borrow()` and `&gui_weights`. |

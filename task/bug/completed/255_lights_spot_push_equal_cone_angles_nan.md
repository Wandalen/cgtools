# BUG-255: `Lights::spot_push` accepts `inner_cone_angle == outer_cone_angle`, producing NaN in
every fragment lit by that spot light

- **Severity:** High (silently poisons rendered pixel output with NaN -- not a panic, so it can
  ship undetected; triggered by an input the function's own pre-fix doc comment explicitly
  documented as valid)
- **state:** Completed
- **Affects:** `Lights::spot_push`; any caller passing `inner_cone_angle == outer_cone_angle`
  (explicitly permitted by the pre-fix doc comment's `<=` wording)
- **Component:** `module/helper/renderer` (`src/webgpu/light.rs` + `src/webgpu/shaders/main.wgsl`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`shaders/main.wgsl`'s spot-light angular falloff computes
`smoothstep( light.outer.x, light.color_inner.w, angle )` -- a call of the form
`smoothstep(low=outer, high=inner, angle)`, which both WGSL and GLSL implement internally as
`clamp( ( angle - low ) / ( high - low ), 0, 1 )`. When `inner_cone_angle == outer_cone_angle`,
`( high - low )` is exactly `0.0`, producing a `0.0 / 0.0` division that yields NaN. That NaN then
multiplies into `attenuation`, poisoning the final lit color for every fragment inside that spot
light's cone -- with no panic, no error, just silently wrong (NaN) pixel output.

## Impact

**Who is affected:** Any consumer of `renderer::webgpu::Lights::spot_push` -- a fully public
method (`pub fn`, exported via `mod_interface!`) on a fully public type. Pre-fix, the doc comment
itself documented `inner_cone_angle <= outer_cone_angle` as the caller's obligation, meaning a
caller reading only the doc comment (not the consuming shader) would reasonably conclude equal
angles are a legitimate, degenerate-but-valid "hard-edged spotlight" configuration -- exactly the
input that breaks the shader formula.

**What breaks:** No crash, no error signal -- the light is packed into the uniform buffer as-is,
and the NaN only manifests as visually corrupted (NaN-propagated) pixels wherever that spot
light's cone reaches, on the native or wasm WebGPU backend at render time. This crate has no
native WebGPU pixel-readback path (per this session's earlier confirmed finding), so the visual
symptom itself isn't natively testable -- only the CPU-side contract that feeds it is.

**Entity Scope:** `None` -- source-level API validation gap, not entity directory instances.

## How Discovered

During this session's `renderer` crate scout (task #174), continuing past `camera.rs`
(BUG-253) into the `webgpu` module's own light-packing code. `Lights::spot_push`'s doc comment
documented `inner_cone_angle <= outer_cone_angle` as the caller obligation; cross-referencing
`shaders/main.wgsl`'s consuming `smoothstep` call revealed the `<=` boundary is exactly the input
that divides by zero in that formula -- the doc comment permitted the one case that breaks its own
consumer.

## Minimum Reproducible Example

Pure CPU-side reproduction, no GPU device needed -- `spot_push`'s pre-fix behavior can be observed
directly by inspecting its return value and the packed `outer`/`color_inner.w` fields it would
have written. See `tests/webgpu_light_test.rs`'s 3 new `spot_push_rejects_*` tests.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --features native --test webgpu_light_test
```
**Expected** (fixed): all 4 tests pass (3 reject, 1 accepts a valid spot light). **Actual**
(pre-fix, confirmed via temporary direct-edit revert-and-rerun of the validation block): the 3
`rejects_*` tests failed -- `spot_push` returned `true` (accepted) for equal cone angles,
`inner > outer`, and non-finite cone angles alike, since the pre-fix function had no check of any
kind on these two parameters.

## Root Cause

`spot_push` (pre-fix):
```rust
pub fn spot_push
(
  &mut self,
  position : [ f32; 3 ],
  direction : [ f32; 3 ],
  color : [ f32; 3 ],
  strength : f32,
  range : f32,
  inner_cone_angle : f32,
  outer_cone_angle : f32
) -> bool
{
  let i = self.raw.counts[ 2 ] as usize;
  if i >= MAX_SPOT_LIGHTS
  {
    return false;
  }
  // ... packs inner_cone_angle/outer_cone_angle straight into the uniform buffer, unvalidated
}
```
The doc comment documented a caller obligation (`inner_cone_angle <= outer_cone_angle`) but the
function never enforced it -- and the documented boundary itself (`==`) is exactly the input that
divides by zero in `shaders/main.wgsl`'s consuming `smoothstep( outer, inner, angle )` call. A
documented "caller obligation" invariant is not safe merely because it's documented; here the
documented invariant was itself insufficient to protect the consuming formula's actual domain.

## Why Not Caught

`spot_push` had zero test coverage of any kind prior to this bug, and no test ever exercised the
shader-side `smoothstep` call with equal cone angles -- this crate has no native WebGPU
pixel-readback path for the lit fragment shader (confirmed earlier this session), so the
shader-side symptom itself isn't natively testable, only the CPU-side contract that feeds it is.

## Fix Applied (2026-08-17)

**`src/webgpu/light.rs`:** `spot_push` now rejects non-finite cone angles and tightens the
documented invariant from `inner_cone_angle <= outer_cone_angle` to a strict
`inner_cone_angle < outer_cone_angle`, returning `false` -- the same "dropped" signal already used
for a full light array -- instead of packing a degenerate light into the uniform buffer:
```rust
pub fn spot_push( /* ... */ ) -> bool
{
  if !inner_cone_angle.is_finite() || !outer_cone_angle.is_finite() || inner_cone_angle >= outer_cone_angle
  {
    return false;
  }

  let i = self.raw.counts[ 2 ] as usize;
  if i >= MAX_SPOT_LIGHTS
  {
    return false;
  }
  // ... unchanged packing logic
}
```
No call-site changes were needed: a grep across the workspace confirmed every real call site
already wraps `spot_push` in `assert!( ... )` on its `bool` return (the array-capacity-full
semantics already required this), and the one real call site
(`examples/minwebgpu/renderer_pbr_scene/src/main.rs:139`) already passes `0.35, 0.55` --
comfortably satisfying the tightened strict invariant without modification. Widening the existing
`bool` "dropped" semantics to also cover "invalid cone angles" was chosen over introducing a
`Result` return type specifically because it required zero call-site changes, unlike BUG-253's
`Camera::projection_matrix_set` fix (which did need a `Result`, since that setter had no prior
fallible-return convention to widen).

`shaders/main.wgsl` was left unchanged -- the fix prevents the degenerate input at its CPU-side
source, matching this session's established pattern of CPU-side validation over shader-side
guards, and is sufficient since `spot_push` is the only way to populate a spot light's cone
angles.

**`tests/webgpu_light_test.rs`** (new file): 4 native `#[ test ]` functions --
`spot_push_accepts_valid_cone_angles` (happy path, mirrors the one real call site's own
arguments), `spot_push_rejects_equal_cone_angles`, `spot_push_rejects_inner_greater_than_outer`,
and `spot_push_rejects_non_finite_cone_angles` (NaN and Inf).

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --features native --test webgpu_light_test` -- pre-fix (temporary
  direct-source-edit revert of the validation block): the 3 `rejects_*` tests failed (all
  returned `true`/accepted instead of `false`/rejected), the `accepts_valid_cone_angles` test
  still passed. Post-fix (block restored): all 4 tests passed.
- `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: exit 0, clean.

## Generalized Version

**Broken assumption:** a documented caller-facing invariant (a doc comment stating "X must hold")
is safe simply because it's written down -- it isn't, when the documented boundary condition
itself is the exact input that breaks the formula the value feeds into downstream. Whenever a
CPU-side setter packs data consumed by a GPU formula with a division, validate against that
formula's actual mathematical domain directly (grep the shader source for how the value is used),
not just restate the doc comment's stated obligation as if writing it down were equivalent to
enforcing it.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by direct review of `webgpu/light.rs` during task #174's `renderer` crate scout, immediately after closing BUG-253 in the sibling `webgl` module. Root cause: `spot_push`'s own doc comment documented `inner_cone_angle <= outer_cone_angle` as a caller obligation, but never enforced it in code -- and `shaders/main.wgsl`'s consuming `smoothstep( outer, inner, angle )` call divides by `( inner - outer )` internally, exactly `0.0` at the documented `<=` boundary, producing NaN that propagates into every fragment lit by that spot light. Fixed by rejecting non-finite cone angles and tightening the invariant to strict `<`, widening the existing `bool` "dropped" return semantics (zero call-site changes needed, confirmed via grep of all real call sites). Verified via 4 new native unit tests (3 confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun) plus clean clippy. Filed as BUG-255, not BUG-254, after a fresh on-disk directory scan found 254 already claimed (by a task file) between this bug's initial drafting and filing. Closed same-session (Tier 2 Dual-Role Self-Check). |

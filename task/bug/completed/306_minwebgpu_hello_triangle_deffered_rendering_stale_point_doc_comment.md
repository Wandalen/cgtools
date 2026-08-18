# BUG-306: `hello_triangle` and `deffered_rendering` both carry the identical stale "draw a large point" module doc comment, copy-pasted from an unrelated example and never updated

- **Severity:** Low (documentation-only factual drift, no code/runtime behavior affected)
- **state:** Completed
- **Affects:** `examples/minwebgpu/hello_triangle/src/main.rs` (BUG-306-A),
  `examples/minwebgpu/deffered_rendering/src/main.rs` (BUG-306-B)
- **Component:** examples/minwebgpu/hello_triangle + examples/minwebgpu/deffered_rendering
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Both `hello_triangle/src/main.rs` and `deffered_rendering/src/main.rs` carried the identical
module doc comment: "Just draw a large point in the middle of the screen." Neither crate does
this: `hello_triangle` draws a single hardcoded 3-vertex triangle (`shaders/shader.wgsl`'s
`vs_main` returns one of 3 hardcoded clip-space positions indexed by `vertex_index`);
`deffered_rendering` renders a grid of models into a G-buffer (albedo, normal, position) then
composites it in a lighting pass against a set of point lights, each drawn with a small
visualization mesh.

## Impact

**Who is affected:** any reader using either crate's module doc comment to understand what the
demo does before reading further.

**What breaks:** both doc comments describe a capability (drawing a point) neither crate actually
has, giving a materially wrong first impression of what each demo renders.

**Entity Scope:** `None` -- documentation-only defect, 2 independent crates sharing one root
cause (copy-pasted boilerplate).

## How Discovered

Disclosed by a fork bug-hunting 8 `minwebgpu`/`minwgpu`/`gpu_hal` wasm example crates (task
#183; the other 6 crates were confirmed clean). Independently verified via grep: `hello_triangle`'s
`shaders/shader.wgsl` confirms `var positions = array<vec3f, 3>` (a real 3-vertex triangle);
`deffered_rendering`'s source confirms `albedo_tex`/`normal_tex`/`position_tex`, `NUM_LIGHTS`, and
`draw_with_instance_count(14, NUM_LIGHTS as u32)` are all genuinely present.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -c "Just draw a large point" examples/minwebgpu/hello_triangle/src/main.rs \
  examples/minwebgpu/deffered_rendering/src/main.rs
grep -n "var positions = array<vec3f, 3>" examples/minwebgpu/hello_triangle/shaders/shader.wgsl
```
**Expected** (fixed): both `main.rs` files' "Just draw a large point" count is 0, and the shader
confirms a real 3-vertex triangle. **Actual** (pre-fix): both counts were 1, despite neither
crate drawing a point.

## Root Cause

Stale copy-pasted doc comment, carried into both crates from an unrelated example and never
updated to describe either crate's own actual rendering behavior after being carried over.

## Why Not Caught

Neither crate had any automated test coverage at all prior to this fix -- nothing cross-checked
either doc comment's factual claim against the crate's own shader/render passes.

## Fix Applied (2026-08-18)

Corrected both module doc comments to describe each crate's actual behavior: `hello_triangle` →
"Classic 'Hello Triangle' -- draws a single hardcoded 3-vertex triangle to the canvas.";
`deffered_rendering` → "Deferred rendering demo -- renders a grid of models into a G-buffer
(albedo, normal, position), then a lighting pass composites the G-buffer with a set of point
lights...".

Added `tests/doc_comment_test.rs` to each crate (`main_doc_comment_describes_a_triangle_not_a_point`,
`main_doc_comment_describes_deferred_rendering_not_a_point`): pure `include_str!` + substring
assertions. Both crates are `wasm32`-only binaries in practice, but the doc comment defect is
plain text -- `include_str!` reads it as a string with zero dependency on the crate's own
(wasm32-gated) items, so both tests compile and run on any host target.

## Verification

- **Pre-fix (RED):** both doc comments claimed a point -- tests would fail against the pristine
  text.
- **Post-fix (GREEN):** `cargo test -p minwebgpu_deffered_rendering -p minwebgpu__ --tests` (note:
  `hello_triangle`'s real Cargo.toml package name is `minwebgpu__`, not
  `minwebgpu_hello_triangle`) → both tests passed. Independently re-run by the orchestrating
  session, confirmed passing.

## Generalized Version

Copy-pasted boilerplate doc comments across sibling example crates drift silently once one
crate's actual behavior diverges from the text -- each sibling needs its own cross-check, not
just the first one written. A demo crate's own top-of-file doc comment is a factual claim about
what the crate does, exactly like any other doc comment -- it must be cross-checked against the
crate's actual source (here, the shader/render passes it uses) rather than trusted at face value
just because it's "only an example".

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found and fixed by a fork bug-hunting 8 `minwebgpu`/`minwgpu`/`gpu_hal` wasm crates (task #183, one of 3 parallel forks covering 27 `examples/` crates); fixed and tested with `BUG-XXX-A`/`BUG-XXX-B` placeholder markers (2 files, one shared root cause) since forks running concurrently on a shared bug ledger must not self-file. Independently verified by the orchestrating session (both diffs read, shader/source claims cross-checked, both tests independently re-run) before this report and its real ID were assigned; placeholders replaced with BUG-306-A/BUG-306-B after a fresh on-disk collision scan found IDs 298/299/300 already claimed by a concurrent actor. |

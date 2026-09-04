# BUG-258: `Renderer`'s per-material shader-program cache ignores a change in
Image-Based Lighting availability once the material was already registered once

- **Severity:** Medium (silently wrong/incomplete lighting output for a realistic call-order --
  not a crash, no panic/NaN, but a visibly incorrect result the public API contract explicitly
  promises against)
- **state:** Completed
- **Affects:** `webgl::Renderer::primitive_register`, via the newly-extracted
  `program_needs_recompile`
- **Component:** `module/helper/renderer` (`src/webgl/renderer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`Renderer::primitive_register` caches each material's compiled shader program keyed only by
`material_id`, in `material_program_map : FxHashMap< uuid::Uuid, uuid::Uuid >`. On a cache hit
(material already registered before), the cached program UUID was reused unconditionally except
when `material.needs_recompile()` -- a purely material-intrinsic "my own `#define`s changed"
flag -- was set. The renderer-level `use_ibl = self.ibl.is_some() && material.ibl_base_texture_unit().is_some()`
value (which also feeds a `#define USE_IBL` into the compiled program) was recomputed on every
call but only ever *consulted* on a cache miss. So a material registered before `Renderer::ibl_set`
was ever called compiled without `#define USE_IBL` and without any IBL uniform bindings, and kept
using that exact stale program on every subsequent frame no matter how many times `ibl_set` was
called afterward -- and symmetrically, a material registered while IBL was available would keep
its IBL-enabled program bound even if IBL later became unavailable to it.

## Impact

**Who is affected:** Any caller that registers a primitive/material via `primitive_register`
before calling `Renderer::ibl_set` (or before an already-set IBL becomes unavailable to a
material) -- a legitimate, common call order the public API does not forbid.

**What breaks:** The material silently renders without any Image-Based Lighting contribution
(no diffuse irradiance, no specular prefilter, no BRDF LUT term) even after `ibl_set` has run
successfully, contradicting `ibl_set`'s own doc comment promise that the given IBL "will be used
for rendering" -- with no caveat about materials registered before the call. No panic, no NaN --
just silently incomplete lit output for as long as the process runs, since nothing ever
re-evaluates the cached program's `USE_IBL` state.

**Entity Scope:** `None` -- source-level rendering-correctness gap, not entity directory
instances.

## How Discovered

During this session's `renderer` crate scout, reviewing `src/webgl/renderer.rs` end-to-end
(also `scene.rs`, `light.rs`, `ibl.rs`, both found clean). `primitive_register`'s cache-hit branch
was traced against `ibl_set`'s doc comment guarantee and against `IBL::bind`'s texture-unit
wiring in `ibl.rs`; cross-referencing every first-party example in this workspace confirmed each
one happens to call `ibl_set` (awaited) strictly before its render loop starts, meaning no example
would ever exercise the divergent order that triggers the bug -- the doc comment's unconditional
promise was checked directly against `primitive_register`'s actual cache logic and found false
for the "material registered first" ordering.

## Minimum Reproducible Example

Pure CPU-side decision logic, no GPU context needed -- the missed-invalidation is observable
directly on the extracted `program_needs_recompile` function. See
`tests/webgl/program_needs_recompile.rs`'s 5 new tests.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --features webgl --test tests webgl::program_needs_recompile::
```
**Expected** (fixed): all 5 tests pass. **Actual** (pre-fix, confirmed via temporary
direct-source-edit revert-and-rerun): the 2 IBL-state-change tests
(`ibl_becoming_available_after_registration_forces_a_recompile`,
`ibl_becoming_unavailable_after_registration_forces_a_recompile`) failed; the 3 others (steady
state, material-owned flag, first-time registration) still passed.

## Root Cause

`primitive_register` (pre-fix, relevant excerpt):
```rust
let use_ibl = self.ibl.is_some() && material.ibl_base_texture_unit().is_some();
// ...
if material.needs_recompile()
{
  // invalidate cached program, recompile with current `use_ibl`
}
// else: reuse `material_program_map[ &material_id ]` unconditionally,
//       `use_ibl` computed above is only used on a fresh compile
```
`use_ibl` is a **renderer-level** input baked into the compiled program's `#define` set, but the
only invalidation trigger consulted on a cache hit was the **material-level** `needs_recompile()`
flag -- which has no visibility into renderer state at all. Any renderer-level input that affects
the compiled program must be part of the cache key or the invalidation check; `use_ibl` was
neither.

## Why Not Caught

No test exercised `primitive_register`'s cache-reuse path at all, let alone across a change in
IBL availability -- and every first-party example in this workspace happens to call `ibl_set`
before its render loop ever starts, so the divergent order that triggers the bug never occurs in
existing example code, only in the public API contract (`ibl_set`'s own doc comment promises the
IBL "will be used for rendering" with no caveat about materials registered before the call).

## Fix Applied (2026-08-17)

**`src/webgl/renderer.rs`:**
- Changed `material_program_map`'s value type from a bare program UUID to
  `( uuid::Uuid, bool )` -- program UUID plus the `use_ibl` state the program was compiled with.
- Extracted the invalidation decision into its own pure function:
  ```rust
  #[ must_use ]
  pub fn program_needs_recompile( material_needs_recompile : bool, cached_use_ibl : Option< bool >, current_use_ibl : bool ) -> bool
  {
    material_needs_recompile || cached_use_ibl.is_some_and( | cached | cached != current_use_ibl )
  }
  ```
- `primitive_register` now looks up `cached_use_ibl` from the map before deciding, and invalidates
  whenever *either* the material's own flag is set *or* the cached IBL state differs from the
  freshly computed `use_ibl` -- and stores the new `use_ibl` alongside the program UUID on every
  insert.
- Exported via `mod_interface!` (`orphan use { ..., program_needs_recompile }`) to make it
  directly unit-testable, matching this session's established `displacement_texture_size_compute`
  (BUG-252) / `normal_matrix_compute` (BUG-257) precedent for extracting pure decision logic out
  of GL-bound rendering methods for testability.

**`tests/webgl/program_needs_recompile.rs`** (new file, module registered as
`mod program_needs_recompile;` in `tests/webgl/mod.rs`): 5 native `#[ test ]` functions covering
IBL becoming available after registration, IBL becoming unavailable after registration, unchanged
IBL state (both directions) *not* forcing a recompile, the material-owned flag still forcing a
recompile independent of IBL state, and first-time registration (no cache entry) not being
mistaken for an invalidation.

Note: the file is deliberately **not** named `renderer.rs` -- an earlier draft used that name and
declared `mod renderer;` in `tests/webgl/mod.rs`, which silently shadows the external `renderer`
crate name in that same module's own `use renderer::webgl as the_module;` import (Rust's 2018+
path resolution prefers a same-named local item over an extern-prelude crate for an unqualified
leading path segment), breaking compilation of the *entire* shared `tests/webgl/mod.rs` for every
test in the module, not just this one. Renamed to `program_needs_recompile.rs` to avoid the
collision.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --features webgl --test tests webgl::program_needs_recompile::` --
  pre-fix (temporary direct-source-edit revert of `program_needs_recompile`'s body back to
  `material_needs_recompile` alone, ignoring the cached/current IBL state): the 2 IBL-state-change
  tests failed, the other 3 still passed. Post-fix (restored): all 5 passed.
- `cargo check -p renderer --features webgl`: exit 0, confirming the extracted function and the
  updated `material_program_map` value type compile cleanly.

## Generalized Version

**Broken assumption:** a cache keyed on a domain object's identity (here, `material_id`) is safe
to invalidate using only that object's own self-reported dirty flag. It isn't, whenever the
cached artifact also depends on external/ambient state the object itself cannot observe --
here, `Renderer::ibl`'s availability. Any external input folded into a cached artifact (a
compiled shader program's `#define` set, in this case) must itself be part of the cache's
invalidation check, not delegated entirely to the cached object's own change-tracking. This is
the same generalized lesson as BUG-257 (`webgpu::model_raw`'s comment wrongly assuming parity
with a sibling backend) and BUG-255 (`Lights::spot_push`'s doc comment permitting an input that
breaks its own shader consumer) applied to caching specifically: a cache's staleness key must
cover every input that shaped the cached value, not just the ones the cached object itself knows
how to report as changed.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by full-file review of `src/webgl/renderer.rs` during this session's `renderer` crate scout (alongside clean reviews of `scene.rs`, `light.rs`, `ibl.rs`). Root cause: `primitive_register`'s program cache only checked the material's own `needs_recompile()` flag on a cache hit, never the renderer-level `use_ibl` state baked into the same compiled program, so a material registered before `ibl_set` kept its IBL-less program bound forever. Fixed by storing `use_ibl` alongside the cached program UUID and extracting the invalidation decision into a pure, unit-tested `program_needs_recompile` function. Verified via 5 new native unit tests (2 confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun) plus a clean `cargo check`. An earlier draft test file named `tests/webgl/renderer.rs` was found to shadow the external `renderer` crate name and break the entire shared test binary's compilation (independently also hit by a sibling review agent testing an unrelated file); renamed to `program_needs_recompile.rs` to resolve. Filed as BUG-258 after a fresh on-disk scan (`find task -name "*.md" | grep -oE '[0-9]{3}_' ...`) found 257 as the highest existing ID. |

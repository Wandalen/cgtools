# BUG-177: `webgpu::Geometry::new` performs no vertex-count cross-validation

- **Severity:** High (native backend can read past a GPU buffer's end at draw time -- undefined
  behavior in `wgpu`'s validation model; the WebGPU browser backend clamps out-of-bounds reads
  per spec, but still renders silently-wrong geometry with no error signal)
- **state:** Completed
- **Affects:** Every caller of `renderer::webgpu::Geometry::new` that passes `normals`/`uvs`/
  `colors` arrays whose lengths don't match the vertex count implied by `positions` -- most
  notably any future glTF-on-webgpu loader (the `webgl` side already has one; `webgpu` does not
  yet, but shares the exact same untrusted-asset-data risk once added) or any caller handling
  partial/optional attribute data.
- **Component:** `module/helper/renderer` (`src/webgpu/geometry.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Pre-identified by task #98's review pass under the working title "renderer
  webgpu: Geometry::new no vertex-count cross-validation." Same general defect shape as BUG-102
  (`tilemap_renderer`'s `TriangleList mesh_def_generate` trusting a malformed index buffer) --
  parallel-array-length trust without cross-validation -- independent occurrence, disjoint code.
  Reuses the `Error::InvalidInput` variant added to `gpu_hal::Error` in BUG-176 (this function's
  `Result` type is `gpu_hal::Error` directly).

## Symptom

```rust
// pre-fix -- webgpu/geometry.rs, Geometry::new
pub fn new( device : &Device, positions : &[ f32 ], normals : &[ f32 ], uvs : &[ f32 ],
  colors : &[ f32 ], indices : Option< Vec< u32 > > ) -> Result< Self, Error >
{
  let vertex_count = ( positions.len() / 3 ) as u32;
  let position_buffer = device.buffer_init_create( bytemuck::cast_slice( positions ), BufferUsage::VERTEX )?;
  let normal_buffer = device.buffer_init_create( bytemuck::cast_slice( normals ), BufferUsage::VERTEX )?;
  let uv_buffer = device.buffer_init_create( bytemuck::cast_slice( uvs ), BufferUsage::VERTEX )?;
  let color_buffer = device.buffer_init_create( bytemuck::cast_slice( colors ), BufferUsage::VERTEX )?;
  // ...
}
```

`vertex_count` is derived from `positions` alone and never compared against `normals.len()`,
`uvs.len()`, or `colors.len()` -- despite the function's own doc comment stating all 4 arrays
share "the same vertex count" as an intended invariant.

## Impact

**Who is affected:** Any caller passing attribute arrays whose lengths don't all agree, most
plausibly a future glTF-on-webgpu asset loader (`webgl/loaders/gltf.rs` already exists for the
WebGL path; a WebGPU counterpart is a natural next addition given the two paths otherwise mirror
each other) resolving accessors that can legitimately have mismatched counts for malformed or
adversarial glTF files, or any caller substituting a placeholder/short array for an optional
attribute instead of padding it to the real vertex count.

**What breaks:** `Renderer::render` (`src/webgpu/renderer.rs:505-516`) binds all 4
`geometry.vertex_buffers` slots and issues `draw`/`draw_indexed` for `geometry.vertex_count`/
`index_count` vertices -- a count derived solely from `positions`. If e.g. `uvs` is shorter,
the GPU draw call reads past the end of the undersized `uv_buffer`: on the native `wgpu` backend
this is a validation error or undefined content depending on `wgpu`'s own bounds-checking mode;
on the WebGPU browser backend, out-of-bounds vertex reads are spec-clamped rather than a hard
fault, but still produce silently-wrong UV/normal/color data with zero error at the point of
failure -- far from where the mismatched array was actually supplied.

**Magnitude:** Every `Geometry::new` call site was exposed identically, since the defect is in
the single constructor, not any one caller. The 2 current real callers (`examples/minwebgpu/
renderer_pbr_scene`) happen to build internally-consistent arrays by construction, so this bug
was latent rather than actively triggered today -- but the constructor is a public, `orphan use`-
exported API with no enforcement of its own documented contract.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Pre-identified by task #98's review pass (this session) as "renderer webgpu: Geometry::new no
vertex-count cross-validation." Confirmed by reading `Geometry::new` directly (only `positions`
feeds `vertex_count`; the other 3 arrays are cast straight to buffers with no length check) and
tracing the downstream consumer (`Renderer::render`'s per-slot buffer binding and `draw`/
`draw_indexed` calls) to confirm a length mismatch is reachable all the way to an actual GPU draw,
not caught by any earlier validation layer.

## Minimum Reproducible Example

```rust
// module/helper/renderer/tests/webgpu_geometry_test.rs -- pre-fix, this returns Ok
let result = Geometry::new
(
  &device,
  &[ 0.0; 12 ], // 4 vertices' worth of positions
  &[ 0.0; 12 ], // 4 vertices' worth of normals
  &[ 0.0; 6 ],  // only 3 vertices' worth of uvs -- mismatched
  &[ 1.0; 16 ], // 4 vertices' worth of colors
  None
);
// pre-fix: Ok(Geometry { vertex_count: 4, .. }) with an undersized uv_buffer
// post-fix: Err(Error::InvalidInput(_))
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run -p renderer --features native new_rejects_uvs_shorter_than_vertex_count
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Geometry::new` never validates that `normals`/`uvs`/`colors` match the `positions`-derived vertex count, so a mismatched-length caller silently produces an undersized buffer that a later draw call reads past the end of. | ✅ Root Cause | Confirmed by direct read of `Geometry::new` (no length check anywhere) and of `Renderer::render`'s buffer-binding/draw-call site, which trusts `vertex_count`/`index_count` unconditionally. | E1, E2 |
| H2 | Every real caller always builds internally-consistent arrays (e.g. via a single per-vertex loop), so the mismatch is unreachable in practice and doesn't need a guard. | ❌ Falsified | True only for the 2 *current* hand-authored example callers -- `Geometry::new` is a public, `orphan use`-exported API with a documented "same vertex count" contract that the function itself does not enforce; any future caller (e.g. a glTF-on-webgpu loader, mirroring the existing `webgl` one) inherits the same untrusted-asset-data risk BUG-102 and BUG-173 already established as real for this codebase's glTF-adjacent code. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgpu/geometry.rs` (pre-fix, `Geometry::new`) | `vertex_count` computed from `positions` only; `normals`/`uvs`/`colors` cast straight to GPU buffers with no length check. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgpu/renderer.rs:505-516` | `Renderer::render` binds all 4 `vertex_buffers` slots and issues `draw( vertex_count )`/`draw_indexed( index_count )`, both sourced only from `positions`/`indices`. | H1 ✅ |
| E3 | `module/helper/renderer/src/webgl/loaders/gltf.rs` (existing WebGL-side glTF loader) + doc comment on `Geometry::new` itself ("... all per vertex, with the same vertex count") | A structurally analogous, already-shipped WebGL loader demonstrates untrusted multi-accessor mesh data is a real code path in this codebase; the doc comment's own explicit invariant statement confirms the constructor was *meant* to enforce this, not merely convention. | H2 ❌ |

## Root Cause

```rust
// before -- vertex_count derived from positions alone, never cross-checked
let vertex_count = ( positions.len() / 3 ) as u32;
let position_buffer = device.buffer_init_create( bytemuck::cast_slice( positions ), BufferUsage::VERTEX )?;
let normal_buffer = device.buffer_init_create( bytemuck::cast_slice( normals ), BufferUsage::VERTEX )?;
// normals/uvs/colors uploaded at whatever length the caller happened to pass
```

No validation existed between the 4 caller-supplied parallel arrays and the buffer uploads that
implicitly trust them to share one common vertex count -- a contract the function's own doc
comment states but never checks.

## Why Not Caught

No test ever exercised `Geometry::new` with mismatched-length attribute arrays prior to this bug;
the only 2 real callers happen to build consistent data by construction, so the gap was never
observed against a differing input.

## Fix Location

`module/helper/renderer/src/webgpu/geometry.rs`, `Geometry::new`: added two guards before any
buffer upload -- `positions.len() % 3 != 0` (guards against a truncated/malformed position
stream) and a length check on `normals`/`uvs`/`colors` against the `positions`-derived vertex
count (3/2/4 components per vertex respectively) -- both returning `Error::InvalidInput` (the
variant added in BUG-176, reused here since `Geometry::new`'s `Result` type is `gpu_hal::Error`
directly).

## Prevention

5 new tests added, `module/helper/renderer/tests/webgpu_geometry_test.rs`:
`new_rejects_uvs_shorter_than_vertex_count`, `new_rejects_normals_longer_than_vertex_count`,
`new_rejects_colors_mismatched_with_vertex_count` (one per attribute array, both shorter- and
longer-than-expected covered), `new_rejects_positions_not_a_multiple_of_three`, and
`new_accepts_consistent_attribute_lengths` (confirms the fix doesn't reject valid input).

## Pitfall

A doc comment stating a cross-array invariant ("all per vertex, with the same vertex count") is
not evidence the invariant is enforced -- it can just as easily describe the *intended* contract
of a function whose body silently trusts the caller. A public, exported constructor accepting
several independently-lengthed parallel arrays needs an explicit length cross-check at the point
they're first accepted together, not an implicit assumption inherited from whichever callers
happen to exist today.

## Generalized Version

**Broken assumption:** "the current callers all build this data consistently, so the constructor
itself doesn't need to enforce its own documented invariant."

**Confirmed general rule:** A public API's documented cross-parameter invariant (shared length,
shared count, shared unit) must be enforced at the API boundary itself, not left to caller
discipline -- today's callers being well-behaved says nothing about tomorrow's, especially once
the boundary starts accepting data from an external, untrusted source ( e.g. a future asset
loader ) rather than only hand-authored call sites.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified by task #98's review pass; confirmed this session by direct read of `Geometry::new` and its downstream draw-call consumer. |
| 2026-08-16 | fixed | Added a `positions.len() % 3` guard plus a `normals`/`uvs`/`colors` length cross-check against the `positions`-derived vertex count, both returning `gpu_hal::Error::InvalidInput`; 5 new regression tests added in a new file, `webgpu_geometry_test.rs` (kept separate from `geometry_tests.rs`, which tests the unrelated `webgl::Geometry` type of the same name). |
| 2026-08-16 | verified | `cargo check -p renderer --tests --features native`: clean (1.57s). `cargo check -p renderer --all-targets --all-features`: clean (7.31s). Full workspace `cargo check --workspace --all-targets --all-features`: clean. `cargo nextest run --workspace --all-features`: 1894/1894 passed, 0 skipped (includes all 5 new tests individually confirmed PASS; no `--exclude` needed this run -- the concurrent actor's `flecs_bouncing_circles` breakage seen during BUG-176's verification had resolved itself by this point, confirmed via a standalone `cargo check -p flecs_bouncing_circles` before the full run). `cargo test --doc --workspace --all-features`: all crates `test result: ok`, 0 failed. `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean, zero warnings. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote 5 differential tests against real mismatched-length inputs. Adversarial pass re-checked the log-collision risk flagged during BUG-176's own verification: this run's job (`-0104_longrun.log`, pid 714888) was again polled with explicit `log::`/`pid::` rather than bare auto-discovery, since the concurrent actor was confirmed still active (log counter had advanced from 93 to 103 between the two bugs' verification runs). No stale-content mismatch occurred this time. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-102 (same parallel-array-length-trust defect shape, disjoint code -- `tilemap_renderer` vs. `renderer`) and BUG-176 (this fix reuses BUG-176's `Error::InvalidInput` variant directly, correctly cited as a dependency, not a duplicate). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct read of both `Geometry::new` and its downstream draw-call consumer (`Renderer::render`), not inferred from the diff's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is 2 validation guards reusing an existing `Error` variant; no unrelated refactor attempted. Did not additionally validate index *values* against `vertex_count` (a related but distinct concern -- out-of-range index content vs. mismatched attribute-array length) since it falls outside this bug's named scope. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Entirely within `renderer`'s own `geometry.rs` + its own new test file; no downstream call sites needed updating (both real callers already pass consistent data, confirmed by the successful full-workspace build). | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via grep that `webgpu::Geometry::new` has exactly one definition site, already fixed; distinguished from the unrelated same-named `webgl::Geometry` (different file, different constructor signature, own pre-existing test file). | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix corrects the constructor's existing, self-documented responsibility (accept 4 per-vertex attribute arrays "with the same vertex count"); no responsibility added or removed. | — |

**Reproduced:** YES -- pre-fix, `new_rejects_uvs_shorter_than_vertex_count`'s equivalent call
returns `Ok` with a silently-undersized buffer; post-fix, all 5 new tests pass. Full workspace
suite (1894/1894, 0 skipped, +5 new), doctests (0 failed across every crate), and clippy all
clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgpu/geometry.rs` | `Geometry::new`: added a `positions.len() % 3` guard and a `normals`/`uvs`/`colors` length cross-check against the `positions`-derived vertex count, both returning `Error::InvalidInput` (full `Fix(BUG-177)` comment block); updated the `# Errors` doc section. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgpu_geometry_test.rs` | New file, 5 tests covering short/long/mismatched attribute arrays, a malformed `positions` length, and a consistent-input acceptance case. Native-gated (`#![cfg(all(feature = "native", not(target_arch = "wasm32")))]`), using `gpu_hal::Device::new_native` directly -- no full render pipeline needed, unlike `native_render_test.rs`. |
| `module/helper/renderer/tests/readme.md` | Added `webgpu_geometry_test.rs` Responsibility Table row. |

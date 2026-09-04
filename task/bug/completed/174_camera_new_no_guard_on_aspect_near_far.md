# BUG-174: `Camera::new` performs no validation on `aspect_ratio`/`fov`/`near`/`far`

- **Severity:** High (one specific degenerate combination -- `near == 0.0` xor `far == 0.0` --
  panics several frames away from its actual cause; every other degenerate combination silently
  bakes an `Inf`/`NaN`-poisoned projection matrix into the camera with no error signal at all)
- **state:** Completed
- **Affects:** Any consumer of `renderer::webgl::Camera::new`. `aspect_ratio` is routinely
  computed as `canvas.width() / canvas.height()` (this crate's own readme.md Quick Start example
  does exactly this) -- a transiently zero canvas height (hidden tab, canvas not yet laid out) was
  enough to trigger it with no malformed input required at all.
- **Component:** `module/helper/renderer` (`src/webgl/camera.rs`); the signature change also
  required updating every downstream call site across the workspace -- see Refs below.
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-16
- **Related Bugs:** Independent discovery from `task/readme.md`'s task #98 review pass (not found
  by investigating another bug this session). While investigating this bug, `Camera::new`'s doc
  comment was found to claim `fov` is in degrees while the only real call site in the workspace
  (this crate's own readme.md) passes radians, and the underlying formula is radians-only --
  a separate, independent documentation defect, deliberately left unfiled pending its own
  dedicated triage rather than folded into this bug's fix.

## Symptom

```rust
// pre-fix -- webgl/camera.rs, Camera::new
#[ must_use ]
pub fn new
(
  eye : gl::F32x3, up : gl::F32x3, look_at : gl::F32x3,
  aspect_ratio : f32, fov : f32, near : f32, far : f32
) -> Self
{
  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl( fov, aspect_ratio, near, far );
  // ... unconditionally builds and returns `Self` -- no check on any of the 4 parameters above
}
```

No `assert!`, `debug_assert!`, `if`-guard, or fallible return exists anywhere between the caller
and `perspective_rh_gl`. Every one of `aspect_ratio == 0`, `fov == 0`, `fov >= PI`, `near == far`,
`near > far`, and any non-finite value silently produces a broken matrix; `near == 0.0` xor
`far == 0.0` specifically produces a matrix whose determinant is exactly `0.0`.

## Impact

**Who is affected:** Every caller of `Camera::new` -- confirmed via workspace-wide search to be
17 example binaries plus `canvas_renderer`'s own doctested readme.md example, none of which
validated their inputs before this fix either.

**What breaks:**
- `near == 0.0` xor `far == 0.0` (other parameters finite/sane): construction itself does not
  panic, but the very next `.inverse()` call on the resulting `projection_matrix` -- reached in
  `Renderer::skybox_draw`, `src/webgl/renderer.rs:641`,
  `camera.projection_matrix_get().inverse().unwrap()` -- returns `None` per `Mat4::inverse`'s own
  documented "If the determinant is zero - return `None`" contract
  (`module/math/ndarray_cg/src/d2/mat4x4/general.rs:110-123`), and the `.unwrap()` immediately
  after panics. The panic message and location point at an unrelated skybox draw call, not at
  `Camera::new`, the actual root cause.
- `aspect_ratio == 0`, `fov == 0`, `fov >= PI`, or `near == far` (both, including both `0.0`):
  the matrix determinant is `Inf`- or `NaN`-poisoned, not exactly zero. IEEE-754 `NaN == 0.0` and
  `Inf == 0.0` are both `false`, so `Mat4::inverse`'s own zero-determinant guard never fires --
  `.inverse()` returns `Some` of an adjugate matrix that is itself `NaN`-poisoned, and this
  garbage is uploaded straight to the `invProjection` shader uniform with **no panic, no error,
  no diagnostic at all**. `Camera::upload`'s own direct `projectionMatrix` upload (`camera.rs`,
  no `.inverse()` involved) ships the same `Inf`/`NaN` values to the GPU unconditionally for every
  one of these cases.
- `near > far` (both finite, nonzero, unequal): the resulting matrix is fully finite and
  *looks* valid -- no panic, no NaN -- but the near/far depth mapping is silently swapped,
  producing wrong depth-test behavior with no error signal whatsoever.

**Magnitude:** Every one of the 17+ call sites in this workspace constructs a `Camera` with zero
validation upstream of them either -- the defect was universal across every real usage of this
constructor, not an edge case exercised by only one caller.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Pre-identified by an earlier review pass (task #98, this session) as "`Camera::new` no guard on
`aspect_ratio`/`near`/`far`"; this bug's work was to confirm, precisely characterize, and fix it.
Confirmation was empirical, not assumed: dispatched a research pass that read `camera.rs` in
full, confirmed (via exhaustive `grep`) zero pre-existing validation anywhere in the file or its
callee, confirmed zero pre-existing test coverage of `Camera::new` or `perspective_rh_gl` anywhere
in the workspace, and hand-derived the resulting matrix's determinant from this crate's own
`perspective_rh_gl` formula and `Mat4::determinant()`'s own cofactor-expansion implementation to
identify exactly which degenerate inputs panic (via a later `.inverse().unwrap()`) versus which
silently corrupt (`Inf`/`NaN` bypassing the zero-determinant guard) versus which produce a
fully-finite-but-wrong result (`near > far`). The downstream panic site
(`Renderer::skybox_draw`, `renderer.rs:641`) was located and confirmed by direct source read, not
inferred from the constructor's shape alone.

## Minimum Reproducible Example

Unlike a value-mismatch bug, the pre-fix `Camera::new` returns bare `Self` unconditionally --
there is no way to "run the new tests against the old code" (the old signature does not even
type-check against a `Result`-expecting test). The reproduction is instead the direct
mathematical trace, confirmed against this crate's own real formulas:

```text
perspective_rh_gl(fovy, aspect, near, far) row-major matrix M:
  [ f/aspect, 0, 0,      0     ]     where f = 1 / tan(fovy/2)
  [ 0,        f, 0,      0     ]           dz = near - far
  [ 0,        0, sz/dz,  mz/dz ]           sz = near + far
  [ 0,        0, -1,     0     ]           mz = 2*near*far

det(M) = (f^2 / aspect) * (mz / dz)

near = 0.0, far = 1000.0 (aspect, fovy sane):
  mz = 2 * 0.0 * 1000.0 = 0.0 (exact)   dz = 0.0 - 1000.0 = -1000.0 (nonzero)
  mz / dz = 0.0 (exact, clean division -- not NaN/Inf)
  => det(M) = 0.0 (exact)
  => Mat4::inverse() 's own `if det == E::zero() { return None; }` guard fires
  => the immediately-following `.unwrap()` in `Renderer::skybox_draw` panics
```

**Expected** (post-fix): `Camera::new(..., near: 0.0, ...)` returns
`Err(WebglError::Other("Camera::new: near must be finite and > 0.0"))` at the point of
construction, with a clear, attributable message.

**Actual** (pre-fix): `Camera::new` returns `Ok`-equivalent `Self` unconditionally; the panic
only surfaces later, at an unrelated call site, with a generic `Option::unwrap()` message.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo nextest run -p renderer webgl::camera::rejects_zero_near
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `Camera::new` has no validation of any kind on `aspect_ratio`/`fov`/`near`/`far`; specific degenerate combinations either panic downstream (`near`/`far` producing an exact zero determinant) or silently corrupt the projection matrix (`Inf`/`NaN`, bypassing `Mat4::inverse`'s zero-determinant guard). | ✅ Root Cause | Confirmed by exhaustive read of `camera.rs` (zero guards found) and hand-derivation of `det(M)` from this crate's own `perspective_rh_gl` and `Mat4::determinant()` formulas, cross-checked against `Mat4::inverse()`'s own documented zero-determinant contract and the real downstream `.unwrap()` call site. | E1, E2, E3 |
| H2 | Callers already validate `aspect_ratio`/`near`/`far` before calling `Camera::new`, so the missing guard is unreachable in practice. | ❌ Falsified | Workspace-wide search found 17 real call sites (12 example binaries with a simple one-line call, 3 with a dedicated `camera_setup`/`camera_init` helper, 2 inline in a `Renderer::new`-style constructor) plus `canvas_renderer`'s own doctested readme.md example -- none validate their inputs upstream; several compute `aspect_ratio` directly from live `canvas.width()/canvas.height()`, which can legitimately be zero. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/renderer/src/webgl/camera.rs` (pre-fix, full file read) | Zero `assert!`/`debug_assert!`/`if`-guard/fallible-return anywhere in `Camera::new` or between it and `perspective_rh_gl`. | H1 ✅ |
| E2 | `module/math/ndarray_cg/src/d2/mat3x3h/transformation.rs:101-128` (`perspective_rh_gl`) + `module/math/ndarray_cg/src/d2/mat4x4/general.rs:110-140` (`Mat4::inverse`, `"If the determinant is zero - return None"`) | Formula source for the determinant trace above; confirms `inverse()`'s own documented zero-determinant contract, and that `NaN`/`Inf` results bypass that guard (`NaN == 0.0` is `false`). | H1 ✅ |
| E3 | `module/helper/renderer/src/webgl/renderer.rs:641` (`Renderer::skybox_draw`) | `camera.projection_matrix_get().inverse().unwrap()` -- the real downstream panic site for the `near==0.0` xor `far==0.0` case, several frames from `Camera::new`. | H1 ✅ |
| E4 | Workspace-wide `grep -rn "Camera::new"` (17 example crates + `canvas_renderer/readme.md`) | Every real call site in the workspace, none validating inputs beforehand. | H2 ❌ |

## Root Cause

```rust
// before -- no validation of any parameter
pub fn new( eye, up, look_at, aspect_ratio : f32, fov : f32, near : f32, far : f32 ) -> Self
{
  let projection_matrix = gl::math::mat3x3h::perspective_rh_gl( fov, aspect_ratio, near, far );
  // ...
}
```

`perspective_rh_gl` divides by `aspect_ratio`, `tan(fov/2)`, and `near - far` with no guard of its
own either -- it is a pure math function, correctly scoped to trust its domain rather than
validate it. Responsibility for validating caller-supplied values before they reach a
division/tangent-based formula belongs at the constructor boundary (`Camera::new`), which had no
such boundary at all.

## Why Not Caught

`Camera::new` had zero test coverage of any kind prior to this bug -- confirmed via workspace-wide
search, no test anywhere called it, degenerate or otherwise. The underlying `perspective_rh_gl`
also has zero test coverage in `ndarray_cg`'s own test suite. No call site in the 17 real usages
across the workspace validated its own inputs either, so the gap was never exercised by any
existing caller either.

## Fix Location

`module/helper/renderer/src/webgl/camera.rs`: `Camera::new` now returns
`Result< Self, gl::WebglError >` (matching this crate's own established constructor idiom --
`Geometry::new`, `Renderer::new` already return `Result<Self, gl::WebglError>`) and rejects a
non-finite or out-of-domain `aspect_ratio` (`> 0.0`), `fov` (`(0.0, PI)` radians), `near`
(`> 0.0`), or `far` (`> near`) before calling `perspective_rh_gl`, via
`gl::WebglError::Other("...")` -- the same error-construction idiom `geometry.rs` already uses.

The signature change required updating every downstream call site:

- 3 sites already inside a `Result<(), gl::WebglError>`-returning `app_run` (`gltf_viewer`,
  `skeletal_animation`, `postprocessing`): `?` propagation, zero behavior change.
- 12 sites inside a bare `-> Camera`/`-> renderer::webgl::Camera` helper (`morph_targets`,
  `text_rendering`, `animation_amplitude_change`, `lottie_surface_rendering`, `pbr_lighting`,
  `character_control`, `curve_surface_rendering`, `renderer_with_outlines`,
  `animation_surface_rendering`, `shadowmap`, `area_light`, `deferred_shading`): `.expect("camera
  parameters are valid")` -- these all pass fixed literal `fov`/`near`/`far` constants with an
  `aspect_ratio` computed once at a point already confirmed non-degenerate for that example's own
  fixed canvas setup, so a signature-only cascading change through the helper's own return type
  was not warranted.
- 2 sites inside a `.unwrap()`-heavy `fn new() -> Self` (`outline`, `narrow_outline`): `.unwrap()`,
  matching every other fallible call already in that exact constructor.
- 2 sites in `canvas_renderer/readme.md`'s doctested Quick Start (inside
  `Result<(), gl::WebglError>`-returning `setup_and_render`): `?` propagation.
- `renderer/readme.md`'s own doctested Quick Start: `?` propagation (already inside
  `Result<(), gl::WebglError>`-returning `setup`).

## Prevention

10 new tests added, `module/helper/renderer/tests/webgl/camera.rs`: happy-path finite-matrix
assertion, plus one rejection test per degenerate case identified in the determinant trace above
(`near`/`far` == 0, `aspect_ratio` == 0 or negative, `near == far`, `near > far`, `fov` == 0 or
>= PI, and 4 non-finite-parameter cases in one test). `rejects_zero_near`/`rejects_zero_far`
carry the full 5-section bug-fix doc comment, matching this crate's own established convention
(`geometry_tests.rs`) since they reproduce the confirmed downstream-panic case specifically.

## Pitfall

Any constructor that feeds caller-supplied scalars into a division/tangent-based formula must
validate the formula's mathematical domain itself -- a pure-math callee (`perspective_rh_gl`) is
correctly scoped to trust its inputs, so nothing upstream of it did. `aspect_ratio` in particular
is routinely computed from live DOM/canvas state (`canvas.width() / canvas.height()`), which can
legitimately be transiently zero (hidden tab, canvas not yet laid out) with no malformed caller
input required at all -- unlike `near`/`far`/`fov`, which are more often fixed literals, this is a
parameter a real, correctly-written caller can pass a degenerate value for under entirely
ordinary conditions.

## Generalized Version

**Broken assumption:** "a pure constructor that only assembles a struct from its parameters
cannot need input validation -- validation belongs in the math functions it calls, if anywhere."

**Confirmed general rule:** a math function scoped to trust its domain (no guard, by design) pushes
the validation responsibility to its caller by construction -- if the caller is itself a
constructor with an infallible signature, that responsibility silently has nowhere left to land.
Any constructor forwarding caller-supplied scalars into a domain-sensitive formula (division,
trigonometric, logarithmic) must itself validate that domain, or explicitly document that it
does not and why.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Pre-identified by task #98's review pass; confirmed and precisely characterized this session via direct source read and hand-derivation of the determinant trace against this crate's own real formulas. |
| 2026-08-16 | fixed | `Camera::new` returns `Result<Self, gl::WebglError>`, validates `aspect_ratio`/`fov`/`near`/`far` before calling `perspective_rh_gl`; all 17+ downstream call sites across the workspace updated to match their own local error-handling convention. |
| 2026-08-16 | verified | Native `cargo check --workspace --all-targets --all-features`: clean (confirms every call site compiles). `cargo nextest run -p renderer --all-features`: 101/101 passed (10 new). `cargo test --doc -p renderer -p canvas_renderer --all-features`: 4/4 passed. `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass wrote the determinant trace directly from this crate's own `perspective_rh_gl`/`Mat4::determinant()`/`Mat4::inverse()` source. Adversarial pass checked whether the old signature could instead be exercised by a same-shape test (H2-style "maybe it's fine in practice") -- ruled out by the workspace-wide call-site search finding zero pre-existing validation anywhere, and by confirming a pre-fix/post-fix "run the same test twice" reproduction is structurally impossible here (signature-level fix, not a value-mismatch), documented explicitly rather than glossed over. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Cross-referenced against BUG-171/172/173/189 (same review pass area, confirmed disjoint code paths); noted but deliberately did not file the separate fov degrees/radians doc mismatch discovered during this investigation. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Backed by direct source reads of `perspective_rh_gl`, `Mat4::determinant()`, `Mat4::inverse()`, and the real downstream panic call site (`renderer.rs:641`), not inferred from the constructor's shape alone. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Fix is validation logic added to one constructor plus mechanical call-site updates required by the resulting signature change -- no unrelated refactor attempted. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | `renderer`'s own `camera.rs`/`readme.md`/tests touched for the fix itself; the 17 downstream call-site edits are a direct, required consequence of this exact signature change, not scope creep -- each left otherwise unchanged. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Confirmed via workspace-wide `grep` that every `Camera::new` call site was found and updated -- none left on the old signature. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The fix is purely parameter validation at construction; `Camera::new`'s own responsibility (assemble a `Camera` from validated inputs) is unchanged in kind, only completed. | — |

**Reproduced:** YES (by direct mathematical derivation against this crate's own real formulas,
not by running the same test against pre-fix code, which is structurally impossible for a
signature-level fix) -- `near == 0.0` produces an exact-zero determinant via the traced formula,
matching `Mat4::inverse()`'s own documented zero-determinant contract and the real
`.unwrap()` panic site it feeds. Full scoped suite (101/101), both doctests (4/4), and clippy all
clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/camera.rs` | `Camera::new` returns `Result<Self, gl::WebglError>`; validates `aspect_ratio > 0`, `fov` in `(0, PI)`, `near > 0`, `far > near`, all finite, before calling `perspective_rh_gl` (full `Fix(BUG-174)` comment block). |
| `module/helper/renderer/readme.md` | Quick Start's `Camera::new(...)` call gains `?` (already inside a `Result`-returning `setup`). |
| `module/helper/canvas_renderer/readme.md` | Both Quick Start `Camera::new(...)` calls gain `?` (already inside a `Result`-returning `setup_and_render`). |
| 12 `examples/minwebgl/*/src/main.rs` (`gltf_viewer`, `morph_targets`, `text_rendering`, `skeletal_animation`, `animation_amplitude_change`, `lottie_surface_rendering`, `pbr_lighting`, `character_control`, `curve_surface_rendering`, `postprocessing`, `renderer_with_outlines`, `animation_surface_rendering`) | `Camera::new(...)` call gains `?` (3 sites, already `Result`-returning `app_run`) or `.expect("camera parameters are valid")` (9 sites, bare `-> Camera` helper). |
| 3 `examples/minwebgl/{shadowmap,area_light,deferred_shading}/src/main.rs` | Multi-line `renderer::webgl::Camera::new(...)` call gains `.expect("camera parameters are valid")` (bare `-> renderer::webgl::Camera` helper). |
| 2 `examples/minwebgl/{outline,narrow_outline}/src/main.rs` | Multi-line `Camera::new(...)` call gains `.unwrap()`, matching the surrounding `fn new() -> Self` constructor's own existing `.unwrap()`-heavy style. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/camera.rs` | New file, 10 tests: happy-path finite-matrix check plus one rejection test per degenerate `aspect_ratio`/`fov`/`near`/`far` case identified in the determinant trace, plus a combined non-finite-parameter test. Registered via `mod camera;` in `tests/webgl/mod.rs`. |
| `module/helper/renderer/tests/readme.md` | Added `webgl/camera.rs` Responsibility Table row; also added the previously-missing `gltf_light_parsing_test.rs` row (BUG-172/BUG-189's own test file, omitted from this table when created earlier this session -- corrected as part of this same edit). |

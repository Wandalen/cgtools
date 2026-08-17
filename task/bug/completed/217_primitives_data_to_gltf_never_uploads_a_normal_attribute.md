# BUG-217: `primitives_data_to_gltf` never generates or uploads a normal attribute, causing NaN lighting

- **Severity:** Medium (silently produces `NaN` lighting for every primitive this crate generates
  when rendered with a shader that reads the geometric `normal` attribute -- no crash, no error, the
  mesh still renders in the correct shape, but every shaded pixel is `NaN`-corrupted)
- **state:** Completed
- **Affects:** Every consumer of `primitive_generation::primitives_data_to_gltf` whose resulting
  `GLTF` is rendered through a `Material` that reads a geometric `normal` vertex attribute (verified
  concretely for `renderer::webgl::material::PbrMaterial`, this crate's own primary/default
  material) -- i.e. every application using this crate's GLTF assembly pipeline for shaded
  rendering, not merely an edge case.
- **Component:** `module/helper/primitive_generation` (`src/primitive_data.rs`, `src/primitive.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Found via a fresh investigation into this crate's GLTF-assembly/rendering
  pipeline (distinct from the UFO/glif parsing pipeline audit that found BUG-215/BUG-216) -- no
  shared root cause or discovery context with either. Shares its general *defect class* -- an
  unguarded `normalize()` on data that can be exactly zero, producing `NaN` -- with BUG-158
  (`join_tangent_nan_at_180_degree_cusp` in `line_tools`), but that bug was a CPU-side geometry
  computation in an unrelated crate; this one is a missing GPU vertex attribute binding, an
  entirely different mechanism, so filed separately. Covers all 3 of this crate's geometry
  generators (`plane_to_geometry`, `curve_to_geometry`, `contours_to_fill_geometry`) under one ID
  since they share the identical root cause and fix shape (established multi-site precedent:
  BUG-181/193, BUG-207/208, BUG-209, BUG-213, BUG-216).

## Symptom

```rust
// pre-fix -- src/primitive_data.rs
pub struct AttributesData
{
  pub positions : Vec< [ f32; 3 ] >,
  pub indices : Vec< u32 >,
  // no normals field at all
}
```

```rust
// pre-fix -- src/primitive_data.rs, primitives_data_to_gltf
let attribute_infos =
[
  ( "positions", buffer_attribute_info_make( &position_buffer, .. ) ),
  // no "normal" attribute -- slot 1 is never bound
];
```

```glsl
// module/helper/renderer/src/webgl/shaders/main.vert (unmodified, the consumer)
layout( location = 1 ) in vec3 normal;   // REQUIRED, always declared
// ...
vNormal = normalize( normalMatrix * normal );   // unconditional, no guard
```

## Impact

**Who is affected:** Any application calling `primitive_generation::primitives_data_to_gltf` and
rendering the result through a shader that reads a geometric `normal` attribute -- confirmed
concretely for `PbrMaterial`, the crate's own default/primary material, so this is the common case,
not an edge case.

**What breaks:** WebGL2 leaves an unbound generic vertex attribute at its spec-mandated default
value `(0, 0, 0, 1)`. `main.vert` reads that as `vec3( 0, 0, 0 )` for `normal` and unconditionally
computes `normalize( normalMatrix * normal )` -- `normalize` of the zero vector is `NaN` in every
component (`dot( 0, 0 ) == 0`; `inversesqrt( 0 ) == +Inf`; `0 * Inf == NaN` per IEEE-754). `vNormal`
is `NaN` for every vertex of every primitive this crate ever generates, corrupting every downstream
lighting calculation that reads it. The mesh's *shape* is unaffected (positions/indices were always
correct) -- only shading is corrupted, silently, with no error or panic anywhere in the pipeline.

**Magnitude:** 1 new struct field, 1 new GPU buffer + attribute binding, and a per-vertex normal
computation added to all 3 geometry generators (`~90` lines across 2 files); 1 downstream example
crate (`animation_surface_rendering`) required a matching fix to keep compiling, since it
constructs `AttributesData` directly (`~35` lines across 1 file).

**Entity Scope:** None — a code-level defect.

## How Discovered

Continuing this session's primitive_generation bug-fixing pass (task tracking item scoped to 3
bugs in this crate, 2 already fixed via the UFO/glif pipeline), investigated the GLTF
assembly/rendering side of the crate as the remaining candidate. Read `primitives_data_to_gltf`
(`src/primitive_data.rs`) end to end and found `attribute_infos` bound only a `"positions"`
attribute, with no normal/tangent/uv counterpart. Cross-checked the consuming side by reading
`renderer::webgl::material::PbrMaterial`'s shader source directly
(`module/helper/renderer/src/webgl/shaders/main.vert`) and confirmed a required
`layout( location = 1 ) in vec3 normal` attribute is unconditionally read and normalized in
`main()`, with no conditional path that skips it. WebGL2's own spec-mandated unbound-attribute
default (`(0,0,0,1)`) closes the loop: this is a guaranteed, deterministic `NaN`, not a
possibility.

## Minimum Reproducible Example

```rust
// module/helper/primitive_generation/tests/geometry_normal_attribute_test.rs
let primitive = plane_to_geometry().expect( "plane_to_geometry always succeeds" );
let attributes = primitive.attributes.expect( "geometry must have attributes" );
// pre-fix: attributes.normals -- does not exist, AttributesData has no such field
// post-fix: attributes.borrow().normals == vec![ [ 0.0, 0.0, 1.0 ]; 4 ]
```

The `NaN` itself only manifests inside the GPU/shader pipeline once this attribute is bound and
read by `main.vert`, which is outside what a native `cargo nextest` run can observe (no live
`WebGl2RenderingContext` in this crate's test suite -- see the workspace's established Wasm
Native-Check Blind Spot awareness: a green native check proves nothing about wasm/shader-gated
code). The testable, native-side root cause is the missing attribute output itself, which is what
the MRE and regression tests target.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/primitive_generation && cargo nextest run --features font-processing -E 'binary(geometry_normal_attribute_test)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `primitives_data_to_gltf` never generates or uploads a normal attribute, and `PbrMaterial`'s vertex shader unconditionally normalizes a required `normal` attribute -- an unbound attribute reads WebGL's default `(0,0,0)`, and `normalize` of that is `NaN`. | ✅ Root Cause | Confirmed by direct read of both sides: `primitive_data.rs`'s `attribute_infos` only ever bound `"positions"`; `main.vert` unconditionally computes `vNormal = normalize( normalMatrix * normal )` from a required `layout( location = 1 )` attribute, with no conditional branch that skips it for the non-morph-target case. | E1, E2 |
| H2 | `main.vert` might guard the normal computation behind a conditional, or supply its own fallback/default normal, so a missing attribute wouldn't actually reach `normalize`. | ❌ Falsified | Read the full `main()` function (lines 245-274): the only conditional is `#ifdef USE_MORPH_TARGET`/`#else`, and *both* branches still unconditionally normalize the base geometric `normal` attribute -- there is no fallback path. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/primitive_generation/src/primitive_data.rs` (direct read, pre-fix) | `AttributesData` had no `normals` field; `attribute_infos` bound only `"positions"` at slot 0; no second buffer/attribute existed. | H1 ✅ |
| E2 | `module/helper/renderer/src/webgl/shaders/main.vert` lines 1-9 and 245-274 (direct read) | `layout( location = 1 ) in vec3 normal;` is required and always declared; `vNormal = normalize( normalMatrix * normal );` runs unconditionally in both the `USE_MORPH_TARGET` and non-morph-target branches. | H1 ✅, H2 ❌ |

## Root Cause

`primitives_data_to_gltf` and its 3 upstream geometry generators were written with only
`positions`/`indices` in mind -- `AttributesData` had no channel for any other per-vertex
attribute, so nothing in the producer side could have populated a normal even if someone had
thought to bind it. On the consumer side, `PbrMaterial`'s vertex shader was written assuming every
geometry source supplies a normal (a reasonable assumption for a general-purpose PBR material), and
GLSL/WebGL give no runtime signal when that assumption is violated -- an unbound attribute is not
an error, it is a silent, spec-defined zero.

## Why Not Caught

`AttributesData` had no `normals` field, so no test could have asserted on it -- there was nothing
to check. The actual `NaN` corruption happens entirely inside the GPU/shader pipeline, which this
crate's native `cargo nextest` suite cannot observe at all (no live `WebGl2RenderingContext`
anywhere in its tests); even a hypothetical pre-fix test asserting `attributes.normals.is_empty()`
would only have proven the symptom's *cause* was present, never the downstream shader-side `NaN`
itself, since that requires an actual WebGL context to reproduce.

## Fix Location

`module/helper/primitive_generation/src/primitive_data.rs`:
- Added `AttributesData::normals : Vec< [ f32; 3 ] >`, parallel to `positions`.
- `primitives_data_to_gltf`: added a second GPU buffer (`normal_buffer`) and a `"normal"`
  `attribute_infos` entry at slot 1, matching `main.vert`'s `layout( location = 1 )`; accumulate
  and upload it alongside `positions` in the existing per-primitive loop.

`module/helper/primitive_generation/src/primitive.rs` (all 3 geometry generators, each normal
derived from that function's own actual triangle winding, not assumed):
- `plane_to_geometry`: constant `(0,0,1)` -- its hardcoded `0,1,2` winding is CCW as seen from +Z,
  confirmed by direct cross product of its own edges.
- `curve_to_geometry`: constant `(0,0,-1)` -- its winding is direction-independent (proven
  algebraically for any unit segment direction: the quad's face normal always reduces to
  `(0, 0, -2 * half_width * segment_length)`, negative for any non-degenerate segment).
- `contours_to_fill_geometry`: computed at runtime from each body's own first triangle, since its
  winding depends on caller-supplied UFO/font contour data (`earcutr` preserves, never reorders,
  input winding) -- falls back to `(0,0,1)` for a zero-triangle or degenerate (zero-area) body to
  avoid reintroducing `NaN` via `normalize` of a zero vector.

`examples/minwebgl/animation_surface_rendering/src/primitive.rs` (downstream consumer, required to
keep the workspace compiling after `AttributesData` gained a new required field): its own local
duplicate `curve_to_geometry`/`contours_to_fill_geometry`/`bodies_triangulate` functions apply the
identical fix, mirroring the crate's own derivation exactly since the duplicated logic is
line-for-line identical.

## Prevention

New test file `tests/geometry_normal_attribute_test.rs`, one regression test per generator,
asserting: exactly one normal per vertex, every normal finite and unit-length (proving `normalize`
never saw a zero vector), and correctly Z-axis-oriented per that generator's own actual winding.
The GPU-side `NaN` symptom itself remains outside native test coverage (see Minimum Reproducible
Example) -- prevention is scoped to the testable root cause, the missing attribute output, not the
downstream shader behavior.

## Pitfall

A shader that unconditionally reads and normalizes a vertex attribute gives no signal -- no error,
no panic -- when the attribute was never bound; the defect only shows up as `NaN` in the final
shaded output, far from its actual cause. Any new geometry generator feeding a shared GLTF assembly
pipeline must populate every attribute its target shader unconditionally reads, not only the ones
its author happened to think about.

## Generalized Version

**Broken assumption:** "a geometry producer only needs to emit whatever attributes it considers
geometrically meaningful (positions and indices); a shader will simply not use attributes the
producer doesn't supply."

**Confirmed general rule:** In WebGL2 (and GL generally), an unbound generic vertex attribute does
not fail to be read -- it silently reads as its spec-mandated default `(0,0,0,1)`. For any attribute
a shader unconditionally `normalize()`s, a missing binding is not a missing feature; it is exact,
deterministic `NaN`, propagating into every calculation that reads it with no error anywhere in the
pipeline. A geometry producer's attribute contract must be driven by what its *target shader*
requires, not by what the producer's own author considered geometrically essential.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via a fresh investigation into `primitive_generation`'s GLTF-assembly/rendering pipeline (the 3rd of this session's 3 scoped bugs for this crate); confirmed by direct source read of both `primitives_data_to_gltf` (producer) and `PbrMaterial`'s `main.vert` (consumer), no assumptions. |
| 2026-08-17 | fixed | Added `AttributesData::normals`, wired a normal buffer/attribute into `primitives_data_to_gltf`, and populated it in all 3 geometry generators, each normal independently derived from that function's own actual triangle winding. Fixed 1 downstream consumer (`animation_surface_rendering` example) to keep the workspace compiling. 1 new test file added (3 tests). |
| 2026-08-17 | verified | `cargo nextest run -p primitive_generation --features font-processing`: 14/14 passed, 0 skipped. `cargo nextest run -p primitive_generation` (default features): 5/5 passed. `cargo clippy -p primitive_generation --all-targets --features font-processing -- -D warnings` and (default features): both clean. `cargo check -p animation_surface_rendering --target wasm32-unknown-unknown`: clean. Temporary direct-source-edit revert-and-rerun: all 3 new tests failed against a `(0,0,0)`-normal reverted source (the exact WebGL unbound-attribute default), passed after restoring the fix. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass checked all 3 generators were fixed, not just the ones directly exercised by the earliest-written test -- confirmed all 3 (`plane_to_geometry`, `curve_to_geometry`, `contours_to_fill_geometry`) have both a source fix and a dedicated regression test. Also checked for other `AttributesData` construction sites in the whole workspace (`grep -rln`), finding and fixing the one downstream break (`animation_surface_rendering`); a second local struct named `AttributesData` in `text_rendering` was confirmed to be an unrelated, pre-existing local type (already has its own `normals` field) via direct read, not a missed site. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Temporary direct-source-edit revert (setting all 3 generators' output to `(0,0,0)`, the literal WebGL unbound-attribute default) reproduced test failures in all 3 new tests before the fix was restored, confirming the tests actually catch the defect rather than passing vacuously. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly filed as independent from BUG-215/BUG-216 (different pipeline, different discovery context, no shared root cause); correctly scoped to cover all 3 generators under one ID per established multi-site precedent; correctly cross-referenced BUG-158 as a related-but-distinct defect class rather than conflated with it. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of both the producer (`primitive_data.rs`) and consumer (`main.vert`) sides, plus explicit falsification of the "shader has a fallback" alternative hypothesis (H2) via reading the full `main()` function -- not assumed from the symptom alone. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to attribute wiring and per-generator normal computation; the downstream `animation_surface_rendering` fix is a required, non-optional consequence of the struct change (the workspace would not compile otherwise), not scope creep -- confirmed via `cargo check` before and after. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives in `primitive_generation` plus the one downstream example crate whose compilation directly depends on `AttributesData`'s shape; `renderer`'s `main.vert`/`PbrMaterial` were read-only references (consumer-side evidence), never modified. | — |

**Reproduced:** YES — pre-fix (temporary revert to `(0,0,0)` normals, the literal WebGL
unbound-attribute default), all 3 new tests in `geometry_normal_attribute_test.rs` failed with
assertion messages matching each generator's expected orientation (e.g. `got [0, 0, 0]` where
`(0,0,-1)` was expected); post-fix all 3 pass. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/primitive_data.rs` | Added `AttributesData::normals`; wired a `normal_buffer`/`"normal"` attribute (slot 1) into `primitives_data_to_gltf`, accumulated and uploaded alongside `positions` (`Fix(BUG-217)` comment blocks). |
| `module/helper/primitive_generation/src/primitive.rs` | `plane_to_geometry`/`curve_to_geometry`/`contours_to_fill_geometry`: each populates `AttributesData::normals` with a per-vertex normal derived from that function's own actual triangle winding (`Fix(BUG-217)` comment blocks). |
| `examples/minwebgl/animation_surface_rendering/src/primitive.rs` | Downstream fix (required for the workspace to compile): local duplicate `curve_to_geometry`/`contours_to_fill_geometry`/`bodies_triangulate` mirror the crate's own BUG-217 fix. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/geometry_normal_attribute_test.rs` | New file: 3 tests, one per geometry generator, asserting one finite unit-length correctly-oriented normal per vertex. |

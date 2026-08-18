# Layer: L3 Stack Engine

Where a stack's invariants become running code: one engine per stack,
exposing that stack's rendering vocabulary upward and consuming foundation
layers downward. This is the highest layer that knows about GPU concepts
and the lowest that knows about stack concepts.

### Scope

- **Purpose**: Define the engine layer's role and record the current engines and their seams.
- **Responsibility**: Name each stack's engine, its upward vocabulary, and its downward dependencies (including the accepted pre-HAL violations).
- **In Scope**: `tilemap_renderer` (d2) and `renderer` (d3) as L3 engines.
- **Out of Scope**: The stacks' invariant tables (see [../render_stack/readme.md](../render_stack/readme.md)); scene data consumed from above (see [005_l4_scene_model.md](005_l4_scene_model.md)).

### Role

An engine turns its stack's vocabulary into GPU work (or into declarative
output, where the stack's invariants demand it). The *upward* API is the
stack's own language; the *downward* dependency should be L2/L1 — today it
is L0 directly, the accepted violation tracked in
[../pattern/002](../pattern/002_strict_layering_one_step_drilldown.md).

### Engines Today

| Engine | Stack | Upward vocabulary | Downward seam |
|--------|-------|-------------------|---------------|
| `renderer` | d3 | Legacy: `Node`/`Scene`/`Mesh` scene graph, PBR materials, cameras, lights. Canonical (`gpu_hal`-backed): `Geometry`/`PbrMaterial`/`Lights`/`Frame` | Legacy: direct `minwebgl` (`renderer::webgl::*` namespace). Canonical: `gpu_hal` on three constructible backends — `GpuContext::new_webgpu`, `new_webgl` (gpu_hal's own WebGL2 backend), `new_native` ( `webgpu`/`native` features, [L1](002_l1_gpu_hal.md) ) |
| `tilemap_renderer` | d2 | POD `RenderCommand` stream + assets | `Backend` trait — `adapter-none`/SVG need no GPU at all; `adapter-terminal` is a stub (no `Backend` impl yet, deferred to a follow-up PR); the WebGL2 adapter uses `minwebgl` directly; `adapter-webgpu` / `adapter-native` target `gpu_hal` ( [ADR-003](../adr/003_d2_stack_hal_adoption.md) ) |

The two demonstrate the two portability strategies ADR-001 weighs: a trait
seam at the command level (`tilemap_renderer` — backends multiply freely)
versus per-backend namespaces (`renderer` — each backend is a parallel
tree). The architecture keeps the first and dissolves the second onto the
HAL — now underway: `renderer`'s canonical opaque path (`src/webgpu/`)
already runs through `gpu_hal` on both `webgpu` and `native` features,
alongside the legacy `src/webgl/` tree it will eventually replace.
`tilemap_renderer`'s new adapters go further: rather than dissolving an
existing direct dependency, they adopt the HAL from the start — the same
trait seam now multiplying backends *through* L1 instead of around it.

### ADRs

| File | Relationship |
|------|--------------|
| [../adr/003_d2_stack_hal_adoption.md](../adr/003_d2_stack_hal_adoption.md) | Extends L1 adoption to `tilemap_renderer` — new `adapter-webgpu` / `adapter-native` / `adapter-none` |

### Layers

| File | Relationship |
|------|--------------|
| [003_l2_frame_orchestration.md](003_l2_frame_orchestration.md) | Machinery currently embedded in these engines |
| [005_l4_scene_model.md](005_l4_scene_model.md) | The declarative data engines receive from above |

### Render Stacks

| File | Relationship |
|------|--------------|
| [../render_stack/001_d2.md](../render_stack/001_d2.md) | The invariants `tilemap_renderer` realizes |
| [../render_stack/003_d3.md](../render_stack/003_d3.md) | The invariants `renderer` realizes |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/renderer/src/webgl/renderer.rs` | d3 engine core (legacy path) |
| `module/helper/renderer/src/webgpu/renderer.rs` | d3 engine core (canonical, `gpu_hal`-backed path) |
| `module/helper/renderer/tests/native_render_test.rs` | Canonical-path engine coverage, pixel-verified end-to-end (`opaque_path_renders_lit_quad`) |
| `module/helper/renderer/tests/webgpu_geometry_test.rs` | Canonical-path `Geometry::new` construction-validation coverage only — attribute-length cross-checks, no render pipeline or pixel readback (see `native_render_test.rs` for the pixel-verified path) |
| `module/helper/renderer/tests/webgpu_light_test.rs`, `tests/webgpu_normal_matrix_test.rs` | Canonical-path `Lights`/transform-vocabulary coverage |
| `module/helper/renderer/tests/skeleton_tests.rs`, `tests/animation_graph_tests.rs`, `tests/gltf_animation_loader_test.rs`, `tests/mirror_tests.rs`, `tests/scaler_tests.rs`, `tests/color_grading_tests.rs`, `tests/shader_validation_tests.rs`, `tests/webgl_frame_orchestration_test.rs` | Legacy-path native coverage of scene-graph, skeletal, animation-loading, post-processing, and frame-orchestration vocabulary — representative, not exhaustive |
| `module/helper/renderer/tests/geometry_tests.rs`, `tests/animation_tests.rs` | Legacy-path `wasm-bindgen-test` coverage for `Geometry`'s attribute API and glTF animation loading — browser-only, needs a live `WebGl2RenderingContext` |
| `module/helper/renderer/tests/webgl/` (`node.rs`, `mesh.rs`, `scene.rs`, `camera.rs`, `pass.rs`, `pbr_material.rs`, `shadow.rs`, `gbuffer.rs`, and others) | Legacy-path coverage for individual scene-graph, material, shadow, and G-buffer types — predominantly plain native `#[test]` (e.g. `shadow.rs`, `gbuffer.rs`), with a minority of `wasm-bindgen-test` (`ibl.rs`, `wide_outline.rs`) needing a live rendered surface |
| `module/helper/renderer/tests/fbo_pass_cycle_test.rs` | Legacy-path `wasm-bindgen-test` FBO pass-cycle coverage for `ShadowMap`/`GBuffer` — the bind/clear/render cycle itself (panics, incomplete-framebuffer, missing-uniform regressions), not just the pure helper functions cited above |
| `module/helper/tilemap_renderer/src/backend.rs` | d2 engine's `Backend` seam |
| `module/helper/tilemap_renderer/tests/backend_test.rs`, `tests/commands_test.rs` | `Backend` seam construct-level and command-shape coverage, shared across all adapters (per-adapter coverage cited in [003_l2_frame_orchestration.md](003_l2_frame_orchestration.md)) |

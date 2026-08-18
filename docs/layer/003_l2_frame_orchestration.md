# Layer: L2 Frame Orchestration

Pass scheduling, render-target lifecycles, and resolve/composite chains —
the machinery that turns "draw these things" into an ordered frame. Today
this layer exists only *embedded inside* each L3 engine; the blank crate
`frame_graph` reserves the slot for its extraction.

### Scope

- **Purpose**: Define the frame-orchestration layer's role and record where its logic currently lives.
- **Responsibility**: Name the embedded instances, the extraction trigger, and the reservation state.
- **In Scope**: Target allocation/lifecycle, pass ordering, resolve and composite steps.
- **Out of Scope**: What the passes draw (stack-engine concern, see [004_l3_stack_engine.md](004_l3_stack_engine.md)); the GPU API the targets are allocated through (see [002_l1_gpu_hal.md](002_l1_gpu_hal.md)).

### Role

An L2 crate owns *frame shape*: which targets exist (formats, multisampling),
in what order passes run, when multisampled targets resolve, and how partial
results composite. It is stack-agnostic machinery — a d2 engine and a d3
engine schedule very different passes, but allocation, dependency ordering,
and resolve mechanics are the same shape of problem.

### Embedded Instances Today

- `renderer` (`src/webgl/renderer.rs`, legacy path): the MSAA main /
  emission / transparent-accumulate target set (`RGBA16F`) plus a
  transparent-revealage target (`R16F`), the opaque → transparent →
  `resolve` → post-chain ordering, and the `post_processing/pass.rs` pass
  composition. The attachment-selection branch — which of `[0]` / `[0,1]` /
  `[0,2,3]` / `[0,1,2,3]` color attachments to enable per frame — is
  extracted into a pure `frame_attachments()` function, natively unit-tested
  across all 4 branch combinations by `webgl_frame_orchestration_test.rs`
  (no live `WebGl2RenderingContext` needed for this one piece; the rest of
  the embedded instance still has no test citation — the same
  browser-test-infrastructure gap named elsewhere in this layer).
- `renderer` (`src/webgl/shadow.rs`, legacy path): a separate shadow-map
  render-target and pass cycle, run before the main scene pass. Its
  `tests/webgl/shadow.rs` (3 tests) covers only the `SpotLight`→`Light`
  size-parameterization helper, not the FBO/pass-cycle machinery itself,
  which remains untested — the same browser-test-infrastructure gap named
  elsewhere in this layer.
- `renderer` (`src/webgl/post_processing/gbuffer.rs`, legacy path): a
  G-buffer target set and its own fill/composite pass cycle, feeding the
  post-processing chain. Its pure `GBufferAttachment::define_const` /
  `attribute_info` config-mapping methods are now natively tested
  (`tests/webgl/gbuffer.rs`, task 225) — the FBO fill/composite pass
  cycle itself remains untested, the same browser-test-infrastructure
  gap named elsewhere in this layer.
- `renderer` (`src/webgl/loaders/pmrem.rs`): a PMREM-prefiltering render
  cycle over a cubemap target set, run at load time rather than per frame.
  Structurally tested end-to-end against a real headless WebGL2 context by
  `tests/pmrem_tests.rs` (3 tests: full-output-set, single-mip, and
  non-power-of-two resolution) — signature regressions, panics, and
  incomplete-framebuffer failures are caught, though not pixel-level
  correctness (still visual-only, via the `gltf_viewer` example).
- `renderer` (`src/webgpu/renderer.rs`, canonical `gpu_hal`-backed path): a
  further independent embedded instance — `frame_targets_create()` builds
  the HDR target set and `render()` runs the opaque → tonemap ordering,
  pixel-verified end-to-end by `opaque_path_renders_lit_quad`.
- `tilemap_renderer` (WebGL2 adapter): per-batch VAO lifecycle and
  draw-time state management inside `src/adapters/webgl.rs`.
- `tilemap_renderer` (WebGPU adapter): `submit()` in `src/adapters/webgpu.rs`
  runs its own independent per-frame cycle — `command_encoder_create()` →
  `render_pass_begin()` → `pipeline_set()` → a per-command dispatch loop →
  `pass.end()` → `queue.submit()`.
- `tilemap_renderer` (native adapter): `submit()` in `src/adapters/native.rs`
  runs the same encoder/pass/pipeline/dispatch-loop/end/submit shape as the
  WebGPU adapter, over an offscreen surface with pixel readback.

### Extraction Trigger

Extract into `frame_graph` only when a second engine needs to *share* pass
logic — sharing, not symmetry, is the trigger. Until then, embedded is the
correct state (YAGNI); the slot exists so the eventual extraction has a
name and a documented home.

### Layers

| File | Relationship |
|------|--------------|
| [002_l1_gpu_hal.md](002_l1_gpu_hal.md) | The layer L2 will allocate targets and pipelines through |
| [004_l3_stack_engine.md](004_l3_stack_engine.md) | The engines currently embedding this layer's logic |

### Sources

| File | Relationship |
|------|--------------|
| `module/blank/frame_graph/` | The reserved crate slot |
| `module/helper/renderer/src/webgl/renderer.rs` | The richest embedded instance: target zoo + pass ordering + resolve |
| `module/helper/renderer/tests/webgl_frame_orchestration_test.rs` | Native unit coverage for the attachment-selection branch (task 247) |
| `module/helper/renderer/src/webgl/post_processing/pass.rs` | Pass composition machinery |
| `module/helper/renderer/src/webgl/shadow.rs` | Shadow-map target and pass cycle |
| `module/helper/renderer/tests/webgl/shadow.rs` | Covers only the `SpotLight`→`Light` size helper, not the FBO/pass-cycle machinery |
| `module/helper/renderer/src/webgl/post_processing/gbuffer.rs` | G-buffer target set and fill/composite pass cycle |
| `module/helper/renderer/tests/webgl/gbuffer.rs` | Native coverage for `GBufferAttachment::define_const`/`attribute_info` (task 225) |
| `module/helper/renderer/src/webgl/loaders/pmrem.rs` | PMREM-prefiltering render cycle over a cubemap target set |
| `module/helper/renderer/tests/pmrem_tests.rs` | Structural coverage of `pmrem::generate()` against a real headless WebGL2 context |
| `module/helper/tilemap_renderer/src/adapters/webgl.rs` | Per-batch VAO lifecycle and draw-time state management |
| `module/helper/tilemap_renderer/src/adapters/webgpu.rs` | Per-frame encoder/pass/pipeline/dispatch-loop/end/submit cycle in `submit()` |
| `module/helper/tilemap_renderer/src/adapters/native.rs` | Same per-frame cycle as the WebGPU adapter, over an offscreen surface with pixel readback |
| `module/helper/tilemap_renderer/tests/native_backend_test.rs` | Native adapter's own per-frame cycle, pixel-verified via readback (task 087) |
| `module/helper/tilemap_renderer/tests/webgpu_backend_test.rs` | WebGPU adapter's compile-and-construct-level coverage |
| `module/helper/tilemap_renderer/tests/webgl_backend_test.rs`, `tests/command_consistency_test.rs` | WebGL2 adapter's compile-and-construct-level coverage, plus a cross-backend (`none`/`svg`/`native`) `capabilities()`-honesty check (task 246) |

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
  the embedded instance still has no pass-cycle test citation beyond
  `tests/webgl/pass.rs`'s narrow `SwapFramebuffer::new` doc-comment
  regression test (BUG-259) — the same browser-test-infrastructure gap
  named elsewhere in this layer).
- `renderer` (`src/webgl/shadow.rs`, legacy path): a separate shadow-map
  render-target and pass cycle, run before the main scene pass. Its
  `tests/webgl/shadow.rs` (3 tests) covers only the `SpotLight`→`Light`
  size-parameterization helper (BUG-175); the FBO/pass-cycle machinery
  itself is now structurally tested against a real headless WebGL2 context
  by `tests/fbo_pass_cycle_test.rs`'s
  `shadow_map_bind_clear_render_completes_on_a_shadow_casting_mesh`.
- `renderer` (`src/webgl/post_processing/gbuffer.rs`, legacy path): a
  G-buffer target set and its own fill/composite pass cycle, feeding the
  post-processing chain. Its pure `GBufferAttachment::define_const` /
  `attribute_info` config-mapping methods are natively tested
  (`tests/webgl/gbuffer.rs`, task 225), and the FBO bind/render pass cycle
  itself is now structurally tested against a real headless WebGL2 context
  by `tests/fbo_pass_cycle_test.rs`'s
  `gbuffer_bind_render_completes_on_an_empty_scene`.
- `renderer` (`src/webgl/post_processing/unreal_bloom.rs`, legacy path): a
  10-target ping-pong bloom pass — `UnrealBloomPass` allocates 5 mip-level
  horizontal-blur and 5 mip-level vertical-blur targets
  (`horizontal_targets` / `vertical_targets`, `MIPS = 5`), and its
  `render()` alternates horizontal→vertical Gaussian blur per mip before
  compositing all 5 blurred mips into the output target. No test citation
  — the same browser-test-infrastructure gap named elsewhere in this
  layer.
- `renderer` (`src/webgl/post_processing/outline/wide_outline.rs`, legacy
  path): a JFA (Jump Flood Algorithm) ping-pong outline pass —
  `WideOutlinePass` allocates two step framebuffers
  (`jfa_step_fb_0`/`jfa_step_fb_1`) that its render cycle ping-pongs
  between across `num_passes` JFA step passes before a final
  outline-compositing pass. Five dedicated tests cover this pass and its
  shaders: `tests/webgl/wide_outline.rs` is a `wasm_bindgen_test` that
  constructs and renders two real `WideOutlinePass` instances against a
  live WebGL2 context (BUG-179); `tests/webgl/jfa_buffer_selection.rs`
  (2 tests) calls `WideOutlinePass::jfa_step_targets_fb0` directly
  (BUG-243); `tests/webgl/jfa_step_size.rs` (2 tests) ports the pass's
  `jfa_step_pass` step-size math (BUG-180); `tests/webgl/jfa_silhouette.rs`
  (4 tests) covers the silhouette check shared by
  `jfa_init.frag`/`outline.frag` (BUG-181, BUG-193); and
  `tests/webgl/outline_seed_sentinel.rs` (3 tests) covers
  `outline.frag`'s seed-validity check (BUG-182).
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
  pixel-verified end-to-end by `opaque_path_renders_lit_quad`. Also
  real-browser pixel-verified via `browsee` on both the `webgpu` and
  `webgl` backends, documented in
  `module/helper/renderer/tests/manual/readme.md` — confirmed readings of
  `rgb 205 46 41` (lit quad center) and `rgb 0 0 0` (background), identical
  on both backends.
- `tilemap_renderer` (WebGL2 adapter): per-batch VAO lifecycle and
  draw-time state management inside `src/adapters/webgl.rs`. Beyond
  `tests/webgl_backend_test.rs`'s compile-and-construct-level coverage,
  real-browser pixel verification via `browsee` is documented in
  `tests/manual/readme.md` — confirmed `rgb 255 0 0` sprite on
  `rgb 0 0 255` clear.
- `tilemap_renderer` (WebGPU adapter): `submit()` in `src/adapters/webgpu.rs`
  runs its own independent per-frame cycle — `command_encoder_create()` →
  `render_pass_begin()` → `pipeline_set()` → a per-command dispatch loop →
  `pass.end()` → `queue.submit()`. Beyond `tests/webgpu_backend_test.rs`'s
  compile-and-construct-level coverage, real-browser pixel verification via
  `browsee` is documented in `tests/manual/readme.md` — confirmed
  `rgb 255 0 0` sprite on `rgb 0 0 255` clear (Firefox, post-task-218).
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
| `module/helper/renderer/tests/webgl/pass.rs` | Narrow `SwapFramebuffer::new` doc-comment regression test (BUG-259) — not FBO/pass-cycle coverage |
| `module/helper/renderer/src/webgl/shadow.rs` | Shadow-map target and pass cycle |
| `module/helper/renderer/tests/webgl/shadow.rs` | Covers only the `SpotLight`→`Light` size helper (BUG-175), not the FBO/pass-cycle machinery |
| `module/helper/renderer/src/webgl/post_processing/gbuffer.rs` | G-buffer target set and fill/composite pass cycle |
| `module/helper/renderer/tests/webgl/gbuffer.rs` | Native coverage for `GBufferAttachment::define_const`/`attribute_info` (task 225) |
| `module/helper/renderer/tests/fbo_pass_cycle_test.rs` | Live headless-WebGL2 bind/clear/render coverage for both `ShadowMap::render` and `GBuffer::render` |
| `module/helper/renderer/src/webgl/post_processing/unreal_bloom.rs` | 10-target ping-pong bloom pass (5 mip-level horizontal/vertical blur targets) |
| `module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs` | JFA ping-pong outline pass (two step framebuffers) |
| `module/helper/renderer/tests/webgl/wide_outline.rs` | `wasm_bindgen_test` rendering two real `WideOutlinePass` instances against a live WebGL2 context (BUG-179) |
| `module/helper/renderer/tests/webgl/jfa_buffer_selection.rs` | Native coverage for `WideOutlinePass::jfa_step_targets_fb0` (BUG-243) |
| `module/helper/renderer/tests/webgl/jfa_step_size.rs` | Native coverage porting `jfa_step_pass`'s step-size math (BUG-180) |
| `module/helper/renderer/tests/webgl/jfa_silhouette.rs` | Native coverage for the silhouette check in `jfa_init.frag`/`outline.frag` (BUG-181, BUG-193) |
| `module/helper/renderer/tests/webgl/outline_seed_sentinel.rs` | Native coverage for `outline.frag`'s seed-validity check (BUG-182) |
| `module/helper/renderer/src/webgl/loaders/pmrem.rs` | PMREM-prefiltering render cycle over a cubemap target set |
| `module/helper/renderer/tests/pmrem_tests.rs` | Structural coverage of `pmrem::generate()` against a real headless WebGL2 context |
| `module/helper/renderer/src/webgpu/renderer.rs` | Canonical `gpu_hal`-backed embedded instance: HDR target set + opaque → tonemap ordering |
| `module/helper/renderer/tests/manual/readme.md` | Real-browser (`browsee`) pixel verification of the WebGPU/WebGL opaque path — `rgb 205 46 41` lit quad, `rgb 0 0 0` background, both backends |
| `module/helper/tilemap_renderer/src/adapters/webgl.rs` | Per-batch VAO lifecycle and draw-time state management |
| `module/helper/tilemap_renderer/src/adapters/webgpu.rs` | Per-frame encoder/pass/pipeline/dispatch-loop/end/submit cycle in `submit()` |
| `module/helper/tilemap_renderer/src/adapters/native.rs` | Same per-frame cycle as the WebGPU adapter, over an offscreen surface with pixel readback |
| `module/helper/tilemap_renderer/tests/native_backend_test.rs` | Native adapter's own per-frame cycle, pixel-verified via readback (task 087) |
| `module/helper/tilemap_renderer/tests/webgpu_backend_test.rs` | WebGPU adapter's compile-and-construct-level coverage |
| `module/helper/tilemap_renderer/tests/webgl_backend_test.rs`, `tests/command_consistency_test.rs` | WebGL2 adapter's compile-and-construct-level coverage, plus a cross-backend (`none`/`svg`/`native`) `capabilities()`-honesty check (task 246) |
| `module/helper/tilemap_renderer/tests/manual/readme.md` | Real-browser (`browsee`) pixel verification beyond compile-and-construct-level — `rgb 255 0 0` sprite on `rgb 0 0 255` clear, both backends (Firefox) |

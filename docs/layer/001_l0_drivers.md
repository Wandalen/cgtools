# Layer: L0 Drivers

The bottom of every stack: one thin, backend-faithful wrapper crate per
GPU API. A driver's job is to make its backend *usable from Rust/wasm*, not
to hide it — cross-backend abstraction is exactly what L0 must not do
(that is [L1](002_l1_gpu_hal.md)'s single responsibility).

### Scope

- **Purpose**: Define the driver layer's role, contract, and current occupants.
- **Responsibility**: Record what a driver may and may not abstract, and who currently depends on L0 directly.
- **In Scope**: `minwebgl`, `minwebgpu`, `minwgpu`, `minvulkan`, and the `mingl` substrate's relationship to them.
- **Out of Scope**: The abstraction over drivers (see [002_l1_gpu_hal.md](002_l1_gpu_hal.md)); layering rules (see [../pattern/002_strict_layering_one_step_drilldown.md](../pattern/002_strict_layering_one_step_drilldown.md)).

### Role and Contract

- **Backend-faithful**: a driver exposes its backend's own concepts and
  shader language (GLSL ES for `minwebgl`, WGSL for `minwebgpu`/`minwgpu`)
  truthfully — no cross-backend vocabulary, no lowest-common-denominator API.
- **Thin**: helpers for ergonomics (context setup, buffer upload, error
  surfacing), never policy (pass scheduling, materials, scenes).
- **Terminal drill-down target**: every drill-down chain from higher layers
  bottoms out at a driver handle; there is nothing below to expose except
  the raw web/native API itself.

### Occupants

| Crate | Backend | State |
|-------|---------|-------|
| `minwebgl` | WebGL2 (web) | Mature (primary driver) for pure-logic surface — its live-`WebGl2RenderingContext` entry point (`context::from_canvas` + a minimal shader/buffer/draw sequence) is now browser-pixel-verified too (proven by the `context_triangle_smoke` example via `browsee` — task 192, mirroring gpu_hal's own [002_l1_gpu_hal.md](002_l1_gpu_hal.md) coverage); broader GL-context/DOM surface (shaders, VAOs, textures, uniforms, file/fetch beyond this one path) remains a separate, not-yet-filed gap |
| `minwebgpu` | WebGPU (web) | Functional |
| `minwgpu` | `wgpu` (native) | Embryonic — helper/buffer/context/texture/surface/bind/pipeline/pass/readback/error layers exist; `tests/context_test.rs` deliberately uses `wgpu::Backends::empty()` for its own deterministic error-path coverage, but `tests/live_context_test.rs` now exercises a real adapter on `wgpu::Backends::PRIMARY` — `context_finish` producing a usable `Device`/`Queue`, and `texture::render_target_2d`'s actual `create_texture` call — skipped with a clear stderr reason (not a hard failure) when no adapter is reachable |
| `minvulkan` | Vulkan via `ash` (native, `wgpu`-free) | `Context::builder()` produces a real `ash::Instance`, `PhysicalDevice`, `Device`, and graphics `Queue` — tested against a live Vulkan ICD (task 201) ([ADR-004](../adr/004_native_vulkan_hal_backend.md)); `context::windowed` additionally produces a `VkSurfaceKHR` from raw handle traits and a real `VK_KHR_swapchain` over it, with a per-frame acquire/present pair and resize-driven rebuild ([ADR-006](../adr/006_vulkan_windowed_presentation.md)) — `gpu_hal`'s `Surface::VulkanWindow` is what consumes it, alongside the offscreen `Surface::Vulkan` (a `DEVICE_LOCAL` color image, no swapchain) that remains the readback path. Resource construction (buffers, images, pipelines) exists one layer up, directly in `gpu_hal`'s `vulkan` backend ([002_l1_gpu_hal.md](002_l1_gpu_hal.md)), not pushed down into this driver |

**`mingl` is not a layer.** All four drivers depend on it as a shared
substrate of backend-independent helpers — math, an orbit-camera controller
(`CameraOrbitControls`), and a WASD-plus-mouse-look character controller
(`CharacterControls`) among them — it sits *below* L0, which is why it
cannot become the HAL (dependency arrow points the wrong way; ADR-001,
alternatives).

### Current Direct Consumers (pre-HAL)

[L1](002_l1_gpu_hal.md) exists as v0 and `renderer`'s canonical opaque path
routes through it — but the same `webgpu`/`native` Cargo features that pull
in `gpu_hal` also declare a direct, optional dependency on `minwebgpu`
(`module/helper/renderer/Cargo.toml`), so even the canonical path reaches L0
directly alongside going through L1. The remaining code reaching L0 directly
with no L1 involvement at all is: `renderer`'s legacy `webgl` tree,
`tilemap_renderer`'s WebGL2 adapter (optional `dep:minwebgl`) — both L3
stack engines — and `line_tools` (optional `dep:minwebgl`; not an engine
itself, but its `line_tools::d2`/`line_tools::d3` submodules are already
split across the `d2`/`d3` stacks per
[rulebook.md](../../rulebook.md#rendering-layer-placement) and
[../adr/001](../adr/001_multi_stack_rendering_architecture.md)). These are
the accepted violations named in
[../pattern/002](../pattern/002_strict_layering_one_step_drilldown.md),
scheduled to strangle onto L1.

### Non-Stack Tooling Consumers

Not every L0 consumer is stack code awaiting HAL migration.
`shader_chunks_render_core` (`dep:minwgpu`) and `shader_chunks_preview_web`
(`dep:minwebgpu`) render individual WGSL shader chunks in isolation —
headless and browser-side respectively — as authoring/preview tooling, not
as part of any d2/tile/d3 stack. Single-backend access is intentional here:
the tooling's job is to exercise one exact chunk against one exact backend,
not to portray a stack-vocabulary scene across backends. These are **not**
scheduled to migrate onto L1 — see
[rulebook.md](../../rulebook.md#rendering-layer-placement)'s "beside the
ladder" list.

### Beside-the-Ladder Consumers

Some L0 consumers are neither stack code awaiting HAL migration nor
single-backend tooling — they are horizontal capabilities or cross-stack
bridges that
[rulebook.md](../../rulebook.md#rendering-layer-placement)'s placement
table places explicitly beside the ladder rather than on it:

- `canvas_renderer` (optional `dep:minwebgl`) — cross-stack bridge via
  textures, composing foundation resources across stack boundaries (see
  [../pattern/003](../pattern/003_cross_stack_bridge_via_foundation_resources.md)).
- `animation` (required `dep:minwebgl`) — value interpolation, easing, and
  multi-animation sequencing, feature-gated to `minwebgl`/`mingl`'s
  math/future/diagnostics utilities only, never their GL-context layers;
  feeds `scene_script`'s tween bindings.
- `gl_uniforms` (required `dep:minwebgl`) — program-scoped WebGL uniform
  upload wrapper: `ProgramUniforms` binds a `GL` context and a linked
  `WebGlProgram` once, collapsing the `get_uniform_location` + upload +
  `.expect()` boilerplate repeated at every uniform call site into a
  single `.upload()`/`.matrix_upload()` call. A thin ergonomic layer
  directly over `minwebgl`'s own uniform primitives, not a portability
  seam or orchestration layer — matches
  [rulebook.md](../../rulebook.md#rendering-layer-placement)'s own
  classification. No dependents yet; test coverage now exists in
  `tests/program_uniforms_test.rs` — four browser-run
  (`wasm_bindgen_test`, `run_in_browser`) tests exercise `.upload()` and
  `.matrix_upload()` against a live `GL`/`WebGlProgram`, covering both a
  present and an absent uniform name.
- `gpu_picking` (required `dep:minwebgl`) — GPU id-buffer object picking:
  `IdProgram` renders each `Pickable` part's small integer id into an
  off-screen `R32I` texture via `PickBuffer`, then `PickBuffer::pick` reads
  a single pixel back to find out what's there — no CPU-side ray/AABB
  math needed. A thin, WebGL2-only capability directly over `minwebgl`,
  same shape as `gl_uniforms` above, not a portability seam or
  orchestration layer — matches
  [rulebook.md](../../rulebook.md#rendering-layer-placement)'s own
  classification. Dependent: `examples/minwebgl/falling_frontier`
  (`HullPart` implements `Pickable`; `main.rs` drives `IdProgram`/
  `PickBuffer` directly). `IdProgram`/`PickBuffer`'s own
  methods all require a live `WebGl2RenderingContext` (framebuffers,
  textures, shader compilation) and stay untested natively — same Wasm
  Native-Check Blind Spot as the rest of this table; the one piece of
  context-free interpretive logic (`pick`'s `-1`
  background-sentinel-to-`None` mapping, pulled out as its own
  `readback_to_pick_id` function) is covered by two native `cargo
  nextest` tests inline in `src/lib.rs`.

`primitive_generation` (`dep:minwebgl` with the same `future`/`math`/
`diagnostics` feature gate as `animation`) is **not** a beside-the-ladder
consumer, despite that surface similarity — resolved via
[task/decisions.md Q-04](../../task/decisions.md#q-04--primitive_generations-l0-l5-ladder-placement).
Unlike `animation`, its `minwebgl` usage is not math-only: `src/primitive_data.rs`
unconditionally imports `WebGl2RenderingContext` and real `renderer` (L3)
scene types (`Mesh`, `Node`, `Scene`, `PbrMaterial`), and its
`primitives_data_to_gltf` function creates/uploads GL buffers directly and
returns a `renderer::webgl::loaders::gltf::GLTF` — the same struct type
produced by `renderer`'s own glTF loaders. It is now named in
[rulebook.md](../../rulebook.md#rendering-layer-placement)'s L4 (scene
model) row as a second, procedural producer of that same artifact type,
alongside `renderer`'s file-based loaders — not listed beside the ladder.

### Example / Reference-Implementation Consumers

Some L0 consumers are neither pre-HAL migration debt nor authoring
tooling nor beside-the-ladder library crates — they are `examples/`
crates that deliberately reach a driver directly as a permanent,
by-design reference or comparison implementation, not code awaiting
migration onto L1:

- `orrery_webgpu` (`examples/orrery/webgpu/`, `dep:minwebgpu`,
  wasm32-gated) — depends only on `minwebgpu`; no `gpu_hal` or
  `renderer` dependency at all. The `orrery` family's single-backend
  reference implementation, kept intentionally direct-to-L0 for
  comparison against `orrery_flexible`'s L1-mediated, multi-backend
  path (see [002_l1_gpu_hal.md](002_l1_gpu_hal.md)'s Sources table).

### Layers

| File | Relationship |
|------|--------------|
| [002_l1_gpu_hal.md](002_l1_gpu_hal.md) | The only layer that should depend on L0 once it exists |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/003_cross_stack_bridge_via_foundation_resources.md](../pattern/003_cross_stack_bridge_via_foundation_resources.md) | Foundation resources crossing a stack boundary are driver-level handles (textures, buffers) at this layer |

### Sources

| File | Relationship |
|------|--------------|
| `module/min/mingl/` | Shared substrate below the drivers |
| `module/min/minwebgl/` | WebGL2 driver |
| `module/min/minwebgpu/` | WebGPU driver |
| `module/min/minwgpu/` | Native `wgpu` driver (embryonic) |
| `module/min/minvulkan/` | Native Vulkan driver via `ash`, `wgpu`-free — real Instance/Device/Queue, tested against a live ICD, plus a window surface and real `VK_KHR_swapchain` with a per-frame acquire/present pair; resource construction (buffers, images, pipelines) not implemented at this layer (already exists in `gpu_hal`'s `vulkan` backend) |

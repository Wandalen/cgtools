# gpu_hal

The L1 GPU hardware abstraction layer of the cgtools rendering architecture —
one WebGPU-shaped API through which rendering engines reach the GPU without
knowing which backend they run on.

## Surface ( v0 )

- `Device` / `Queue` / `Surface` — backend selection happens once, at
  construction ( `Device::new_webgpu( canvas )`, `Device::new_webgl( canvas )`
  or `Device::new_native( width, height )` ); everything downstream is
  backend-agnostic.
- Resource handles ( `Buffer`, `Texture`, `TextureView`, `Sampler`,
  `ShaderModule`, `BindGroupLayout`, `BindGroup`, `RenderPipeline` ) — enum
  dispatch over backends, each with one-step `as_webgpu()` / `as_webgl()` /
  `as_native()` drill-downs to the raw driver object for anything the portable
  surface does not cover.
- `CommandEncoder` / `RenderPass` — one color attachment plus optional depth,
  always-clear load ops, the draw calls the opaque path needs.
- Plain-data descriptors ( `TextureDesc`, `SamplerDesc`,
  `RenderPipelineDesc`, `BindGroupLayoutEntry`, `VertexBufferLayout` ) and a
  byte-level buffer API — no marshalling crate dependency in the HAL.
- `ShaderSource` — canonical WGSL with a per-backend GLSL override slot.
- `Device::depth_range()` — the 0..1 vs -1..1 clip-space contract is owned
  here, not guessed by consumers.

## Backends

| Backend | Feature | Status |
| --- | --- | --- |
| WebGPU ( `minwebgpu` ) | `webgpu` | implemented |
| WebGL2 ( `minwebgl` ) | `webgl` | implemented |
| Native wgpu ( `minwgpu` ) | `native` | implemented |

Backends materialize per target : the browser pair exists only on `wasm32`,
the native backend only elsewhere. A build where no backend fits its target
( e.g. browser features on a native target ) still compiles — to just the
error and descriptor types — so cross-target feature unification never
breaks a consumer.

### WebGL2 backend notes

- `ShaderSource` must carry both GLSL override slots — there is no WGSL
  transpilation; `shader_module_create` returns `Unsupported` otherwise.
- Bindings resolve by name convention in the GLSL: uniform block
  `ub_{group}_{binding}`, sampler uniform `tex_{group}_{binding}`. Pipeline
  creation introspects these once; names the linker pruned are skipped.
- Inside a pass, `pipeline_set` must come before `bind_group_set` /
  `vertex_buffer_set` — both resolve through the active pipeline's
  introspected maps.
- The canvas backbuffer accepts no depth attachment; render to a texture for
  depth-tested passes.
- `new_webgl` requires `EXT_color_buffer_float`, keeping float color targets
  renderable on both backends.

### Native backend notes

- `Device::new_native( width, height )` builds its context through `minwgpu`
  and renders into an offscreen texture — there is no window; the machine
  needs a Vulkan ICD, and a software one ( lavapipe ) suffices.
- `Surface::pixels_read( &device, &queue )` is the native counterpart of
  presenting to a canvas : tightly-packed rgba8 bytes, top row first — the
  ground truth a pixel-asserting test reads. Browser surfaces return
  `Unsupported` there and present to their canvas instead.
- Recording is `&mut` : `render_pass_begin` and the `RenderPass` methods take
  `&mut self` on every backend, because the native backend records into its
  raw wgpu objects mutably.
- The WGSL slot of `ShaderSource` is consumed directly; the GLSL overrides
  are ignored.

## Verify

```bash
cargo nextest run -p gpu_hal --features native
```

`triangle_render_readback` draws through the full public surface and asserts
on pixels read back from the offscreen target.

## Context

- `docs/definition/readme.md` — this crate's own feature / invariant / pattern / pitfall documentation
- `docs/layer/002_l1_gpu_hal.md` — the layer's contract
- `docs/explorations/001_gpu_hal_buy_vs_build.md` — the build-vs-buy analysis behind building thin
- `docs/adr/001_multi_stack_rendering_architecture.md` — the architecture this crate serves

## Directory Layout

| Path | Responsibility |
|------|----------------|
| `src/` | Crate source — device/queue/surface, resource, pipeline, pass, and error wrappers over three backends |
| `docs/` | Design documentation as typed doc definitions — see [docs/definition/readme.md](docs/definition/readme.md) |
| `tests/` | Integration tests (native backend only) |
| `readme.md` | This file — user-facing entry point |

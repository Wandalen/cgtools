# gpu_hal

The L1 GPU hardware abstraction layer of the cgtools rendering architecture —
one WebGPU-shaped API through which rendering engines reach the GPU without
knowing which backend they run on.

## Surface ( v0 )

- `Device` / `Queue` / `Surface` — backend selection happens once, at
  construction ( `Device::new_webgpu( canvas )` or
  `Device::new_webgl( canvas )` ); everything downstream is backend-agnostic.
- Resource handles ( `Buffer`, `Texture`, `TextureView`, `Sampler`,
  `ShaderModule`, `BindGroupLayout`, `BindGroup`, `RenderPipeline` ) — enum
  dispatch over backends, each with one-step `as_webgpu()` / `as_webgl()`
  drill-downs to the raw driver object for anything the portable surface does
  not cover.
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

Browser-only, like the drivers it wraps: on native targets the crate compiles
to a stub, mirroring `minwebgpu`.

### WebGL2 backend notes

- `ShaderSource` must carry both GLSL override slots — there is no WGSL
  transpilation; `create_shader_module` returns `Unsupported` otherwise.
- Bindings resolve by name convention in the GLSL: uniform block
  `ub_{group}_{binding}`, sampler uniform `tex_{group}_{binding}`. Pipeline
  creation introspects these once; names the linker pruned are skipped.
- Inside a pass, `set_pipeline` must come before `set_bind_group` /
  `set_vertex_buffer` — both resolve through the active pipeline's
  introspected maps.
- The canvas backbuffer accepts no depth attachment; render to a texture for
  depth-tested passes.
- `new_webgl` requires `EXT_color_buffer_float`, keeping float color targets
  renderable on both backends.

## Context

- `docs/layer/002_l1_gpu_hal.md` — the layer's contract
- `docs/explorations/001_gpu_hal_buy_vs_build.md` — the build-vs-buy analysis behind building thin
- `docs/adr/001_multi_stack_rendering_architecture.md` — the architecture this crate serves

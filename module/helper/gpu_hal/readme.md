# gpu_hal

The L1 GPU hardware abstraction layer of the cgtools rendering architecture —
one WebGPU-shaped API through which rendering engines reach the GPU without
knowing which backend they run on.

## Surface ( v0 )

- `Device` / `Queue` / `Surface` — backend selection happens once, at
  construction ( `Device::new_webgpu( canvas )`, `Device::new_webgl( canvas )`,
  `Device::new_native( width, height )` or `Device::new_vulkan( width, height )`
  to pin one specific backend; the unified `Device::new( canvas )` /
  `Device::new( width, height )` overloads pick whichever browser/native
  feature is active for callers that don't need to name one ); everything
  downstream is backend-agnostic. `Device::backend_name()` reports which
  backend actually ran. Each non-browser backend also has a windowed
  constructor — `Device::new_native_windowed( window, size )` and
  `Device::new_vulkan_windowed( window, size )` — returning the same triple
  against a real presentable surface ( see *Windowed presentation* below ).
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
| Native Vulkan ( `minvulkan` / `ash`, no `wgpu` ) | `vulkan` | implemented |

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

### Windowed presentation

- Both non-browser backends present to a window as well as to an offscreen
  image : `Device::new_native_windowed( window, size )` goes through
  `minwgpu`'s surface, `Device::new_vulkan_windowed( window, size )` through a
  real `VK_KHR_swapchain` in `minvulkan`. The browser pair needs no
  counterpart — a canvas already *is* the presentable surface.
- A window enters as handle traits, never as a windowing type :
  `impl Into< wgpu::SurfaceTarget< 'static > >` for the former,
  `&( impl HasDisplayHandle + HasWindowHandle )` for the latter. `winit`,
  `sdl2` and `glfw` are equally usable and no crate under `module/` depends on
  any of them ( ADRs [005](../../../docs/adr/005_windowed_native_presentation.md),
  [006](../../../docs/adr/006_vulkan_windowed_presentation.md) ).
- What a windowed surface changes is the frame loop, not the render code :
  `current_view` acquires a swapchain image ( returning
  `Error::SurfaceNotReady` when the chain is out of date ), `present` shows it,
  `resize` rebuilds the chain. `pixels_read` is the one thing it gives up,
  returning `Unsupported` — offscreen and windowed are alternatives, not layers.
- `vulkan` windowed is the only configuration whose process links no `wgpu` at
  all : `examples/gpu_hal/triangle_vulkan_window`, verified with
  `cargo tree -p gpu_hal_triangle_vulkan_window | grep -c wgpu` → `0`.

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

The `vulkan` backend has its own offscreen readback test, mirroring `native`
— no browser, no `wgpu` dependency, just a Vulkan ICD (a software one, e.g.
lavapipe, suffices):

```bash
cargo nextest run -p gpu_hal --features vulkan
```

`triangle_render_readback` in `tests/vulkan_backend_test.rs` draws through
the same public surface and asserts on pixels read back the same way.

The windowed path is out of reach of every one of those tests — it needs a
real window handle, which no crate under `module/` can produce without
depending on a windowing library — so it is verified by running an example
and watching it:

```bash
cargo run -p gpu_hal_triangle_vulkan_window --release
```

Drag the window's edge : a triangle that stays correctly proportioned means
the swapchain rebuilt, and the color continuing to cycle means the
acquire/present loop is running rather than stuck on one stale image.

`new_native_windowed` has the same coverage, through
`examples/gpu_hal/triangle_native_window`:

```bash
cargo run -p gpu_hal_triangle_native_window --release
```

Same signal, same technique — a triangle that stays correctly proportioned
across a resize plus a continuously cycling color confirms the wgpu `Surface`
swapchain rebuilds rather than one stale image persisting. The one difference
from the Vulkan example is the window handle : `Arc<Window>` rather than
`&Window`, since `Device::new_native_windowed` takes
`impl Into<wgpu::SurfaceTarget<'static>>`, which only converts from an owned
handle — see that example's own readme.md.

The `webgpu` and `webgl` backends have no offscreen readback to assert on —
they present to a browser canvas instead — so they're verified with a real
browser via `browsee` against `examples/gpu_hal/triangle_browser/`:

```bash
cd examples/gpu_hal/triangle_browser
trunk serve --release --port 8080                                        # webgpu
# or: trunk serve --release --no-default-features --features webgl --port 8080
browsee .launch session::gpu_hal_tri url::http://127.0.0.1:8080/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::gpu_hal_tri
```

Full command sequence, exact pixel readings, and the `region::center` /
window-chrome caveat: `tests/manual/readme.md`.

Unlike `webgpu`, `webgl` device creation is synchronous ( `Device::new_webgl`
takes no `.await` ), so it can also be exercised by a real, `cargo
test`-driven browser instead of only a human eyeballing a screenshot.
`triangle_render_readback` in `tests/webgl_backend_test.rs` reads the canvas
backbuffer back through the live `GL` context directly ( `Surface::pixels_read`
is `Unsupported` on this backend, so this is not the same call the native and
vulkan tests above use ) and asserts on both the triangle's color and the
surrounding clear color, browser-driven via the same runner wired up in the
workspace's `.cargo/config.toml`:

```bash
cd module/helper/gpu_hal
cargo test --target wasm32-unknown-unknown --features webgl --test webgl_backend_test
```

## Context

- `docs/definition/readme.md` — this crate's own feature / invariant / pattern / pitfall documentation
- `docs/layer/002_l1_gpu_hal.md` — the layer's contract
- `docs/explorations/001_gpu_hal_buy_vs_build.md` — the build-vs-buy analysis behind building thin
- `docs/adr/001_multi_stack_rendering_architecture.md` — the architecture this crate serves

## Directory Layout

| Path | Responsibility |
|------|----------------|
| `src/` | Crate source — device/queue/surface, resource, pipeline, pass, and error wrappers over four backends |
| `docs/` | Design documentation as typed doc definitions — see [docs/definition/readme.md](docs/definition/readme.md) |
| `tests/` | Native integration tests, plus `manual/` for browser-side pixel verification |
| `readme.md` | This file — user-facing entry point |

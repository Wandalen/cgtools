# Layer: L1 GPU Hardware Abstraction

The keystone layer — one API over all drivers, so everything above is
written once per stack instead of once per backend. The *contract* is
decided and an in-house *v0 implementation* exists in `gpu_hal` ( WebGPU +
WebGL2 + native wgpu backends, plus a fourth `wgpu`-free native Vulkan
backend — see Status ); build-vs-buy is closed in-house by
[../adr/002_gpu_hal_in_house.md](../adr/002_gpu_hal_in_house.md).

### Scope

- **Purpose**: Define the HAL layer's contract so that any implementation (in-house or `wgpu`-backed facade) can be checked against it.
- **Responsibility**: Record what L1 must provide, what it must not contain, and its current reservation state.
- **In Scope**: The layer's contract and status.
- **Out of Scope**: The build-vs-buy decision itself (see [ADR-002](../adr/002_gpu_hal_in_house.md)); driver behavior (see [001_l0_drivers.md](001_l0_drivers.md)).

### Contract

- **WebGPU-shaped**: API mirrors WebGPU concepts (device, queue, pipeline,
  bind group) so the WebGPU path is near-zero-cost and the WebGL2 path
  emulates only what it must.
- **Shader access, not shader hiding**: canonical shader source in WGSL, with
  a per-backend override slot for hand-tuned sources — never a fixed pipeline
  that owns the shaders. Transpilation timing is backend-specific, not a
  uniform build-time step: the Vulkan backend has no override slot and
  instead compiles WGSL to SPIR-V itself, at RUNTIME inside `gpu_hal`, via
  `naga` (`shader_module_create` dispatching to `shader_compile_wgsl_to_spirv`,
  `device.rs:830` / `vulkan.rs:747`). WebGL sits at the opposite end
  of that timing spectrum: its override slots must already be populated by
  the caller, since `shader_module_create`'s WebGL arm performs no
  transpilation itself. `gpu_hal` ships a BUILD-TIME WGSL-to-GLSL ES 300
  translator for populating them, `webgl_build::wgsl_to_webgl_glsl()`
  (`src/webgl_build.rs`, 154 lines, `naga`-based like the Vulkan path, gated
  behind the `webgl-glsl-build` feature, its own 7-test suite in
  `tests/webgl_build_test.rs`) — meant to run once, in a downstream crate's
  own `build.rs`, not inside `gpu_hal` at runtime. `renderer`'s `build.rs`
  uses it as a required build-dependency instead of hand-porting the GLSL
  (`module/helper/renderer/build.rs:1-13,34`); `examples/orrery/flexible`
  depends on it optionally, under its own `webgl` feature.
- **One-step drill-down**: every HAL object exposes a handle to its raw
  driver counterpart ([../pattern/002](../pattern/002_strict_layering_one_step_drilldown.md)).
- **Stack-vocabulary-free**: no sprite, tile, camera, scene, or material
  concepts — those belong to stacks above (variance rule,
  [../pattern/001](../pattern/001_invariant_defined_stack.md)); L1 serves
  every stack equally.
- **Depth range is queryable, not uniform**: `Device::depth_range()`
  reports which NDC depth convention the active backend uses —
  `DepthRange::ZeroToOne` for WebGPU, native, and Vulkan;
  `DepthRange::NegOneToOne` for WebGL2 (`device.rs:575`, `types.rs:407`;
  asserted by `native_backend_test.rs`/`vulkan_backend_test.rs`). This is
  the one place the WebGPU-shaped contract above does not fully hide
  backend variance — projection math tuned to one convention must call
  this to stay correct across backends.

### Status

v0 implemented in `module/helper/gpu_hal` — the exploration's spike extracted
it from the webgl-vs-webgpu diff of `renderer`'s canonical opaque path, which
now builds against it on both browser backends ( `webgpu` / `webgl`
features ). `gpu_hal`'s own webgpu/webgl backends are now
browser-pixel-verified too ( proven by the `triangle_browser` example via
`browsee` — task 191 ); `renderer`'s own opaque-path browser-side pixel tests
are now covered the same way ( proven by the `opaque_path_browser` example via
`browsee` — task 197 ). A third, native backend ( `native`
feature, `minwgpu` + raw `wgpu` ) renders into an
offscreen texture with pixel readback, and is proven by an in-repo render
test ( `triangle_render_readback` ) that draws through the full public
surface and asserts on the bytes read back — no browser involved. The
renderer's canonical opaque path itself also runs on this backend
( renderer feature `native`, `GpuContext::new_native` ) and is
pixel-verified end-to-end in the terminal by `opaque_path_renders_lit_quad`. The v0 surface
covers the opaque path only: buffers, 2d textures, samplers, shader modules,
bind groups, one-color-attachment render passes, and a depth attachment
( `DepthState`, honored by all backends when rendering to a texture — the
WebGL2 canvas backbuffer itself accepts no depth attachment, so depth-tested
passes there must render to a texture first; `webgl_canvas_pass_begin`
returns `Error::Unsupported` if asked otherwise, `pass.rs:545-551` ). Texture upload is now
covered too ( `texture_write()`, proven by the `texture_write_readback`
render test — task 089 ). The same `native_backend_test.rs` file also
carries roughly 300 lines of `InvalidInput` guard-rail regression coverage —
10 tests spanning zero-size textures, zero-dimension `new_native` surfaces,
undersized/misaligned/oversized buffer and texture writes, and zero-size
vertex/index buffer binds ( fixes for BUG-165, BUG-176, BUG-199, BUG-204,
BUG-207, BUG-208 ). One same-defect-family fix sits outside this automated
coverage: BUG-200 ( `webgl_buffer_write`, `device.rs:2440-2456` ) fixes the
same silent-data-corruption class on the WebGL backend, which
`native_backend_test.rs` cannot exercise ( wasm32 + browser only ). `gpu_hal`
now has a real automated wasm test suite for it too — `webgl_backend_test.rs`
( 3 `wasm-bindgen-test`s: device creation, `pixels_read`'s expected
`Unsupported` on this surface, and `triangle_render_readback` asserting on
the canvas backbuffer read back through the live `GL` context ) — run via
`cargo test --target wasm32-unknown-unknown --features webgl --test
webgl_backend_test`, alongside the pre-existing manual colored-pixel check in
the `triangle_browser` example ( `main.rs:80-99` ), run via `browsee`. Not yet covered: mipmaps, MSAA, compute — accepted YAGNI scope boundary, tracked as a watch-item (task 291), revisited only if a real consumer emerges.
`renderer`'s legacy `webgl` tree keeps its accepted direct-to-L0 dependency
until strangled onto the HAL. `tilemap_renderer` (d2) is the second targeted
consumer — its `adapter-webgpu` / `adapter-native` adopt the HAL per
[../adr/003_d2_stack_hal_adoption.md](../adr/003_d2_stack_hal_adoption.md).
`adapter-webgpu` now builds and passes its own compile-and-construct-level
test suite, and is browser-pixel-verified via the `adapter_browser` example
and `browsee` ( task 251 ) — an initial pass proved the adapter's then-
unpopulated-texture opaque-**black** behavior; a re-run after `assets_load`
started uploading real pixel data ( task 218, sharing `to_rgba8` with
`adapter-native` rather than duplicating it ) confirms, in Firefox at the
sprite's exact configured location, a real solid-red sprite ( `rgb 255 0 0` )
on the blue clear color ( `rgb 0 0 255` ) — a pixel-exact match to
`adapter-webgl`'s own reading below, since both now upload the same asset
bytes. ( Chromium in this sandbox intermittently fails to present the canvas
frame at all — GPU-process/compositor-level errors, not a validation defect
in this crate's upload code; see
`module/helper/tilemap_renderer/tests/manual/readme.md` for the observed
symptoms and why Firefox is the proven browser for this check. )
`adapter-native` now also builds and passes an in-repo pixel-readback test
suite mirroring `gpu_hal`'s own `triangle_render_readback` precedent, proving
the offscreen-render-plus-readback path with no browser involved. Its existing
`adapter-webgl` keeps its direct `minwebgl` dependency for now, on the same
accepted-until-strangled posture — it now also has its own
compile-and-construct-level test suite (`webgl_backend_test.rs` +
`command_consistency_test.rs`, task 246) and is browser-pixel-verified too, via
the same `adapter_browser` example ( task 251 ) — proving a real solid-red
sprite paint, since `adapter-webgl` uploads real pixel bytes — the same shape
of coverage as `adapter-webgpu`'s, without adopting the HAL itself. The two
adapters' upload paths are no longer asymmetric in kind or in browser-
verification recency — both upload real pixel bytes, and both readings are
now current. The conversion path itself is not shared between them, though:
`adapter-webgpu` uses the same `crate::assets::to_rgba8` helper as
`adapter-native` ( `webgpu.rs:265`, `native.rs:166`, `assets.rs:579-588` ),
while `adapter-webgl`'s `bitmap_texture_upload` ( `webgl.rs:1334-1394` ) has
its own separate inline pixel-format match — notably uploading `Rgb8`
natively as `gl::RGB` rather than expanding it to RGBA the way `to_rgba8`
does.

A fourth backend, `vulkan` ( `minvulkan` via `ash`, no `wgpu` dependency ),
is now implemented — [ADR-004](../adr/004_native_vulkan_hal_backend.md) adds
it so `examples/orrery/flexible` can offer a Vulkan option that does not link
`wgpu`, distinct from `native`'s existing `wgpu`-picks-its-own-backend
behavior. Same v0 opaque-path surface as `native` ( buffers, 2d textures,
samplers, shader modules — WGSL compiled to SPIR-V via `naga` — bind groups,
one-color-attachment render passes ), proven the same way : an in-repo
offscreen-render-plus-readback test ( `triangle_render_readback` in
`tests/vulkan_backend_test.rs`, task 202 ) draws through the full public
surface and asserts on the bytes read back, no browser involved, mirroring
`native`'s own `triangle_render_readback` precedent. Resources use dedicated
( non-suballocated ) memory — one `vkAllocateMemory` call per buffer/image,
matching the crate's v0 "minimum resource support" tradeoff elsewhere.
Long-lived handles returned through the public HAL API ( buffers, textures,
pipelines, bind groups, render passes ) carry no `Drop`-based cleanup on this
backend — confirmed by zero `impl Drop` anywhere in the crate ( `BufferVulkan`
/ `TextureVulkan`, `vulkan.rs`, are plain structs of raw handles with no
destroy/free method ) — so every Vulkan-backend resource leaks for the
process lifetime. This was a known v0 tradeoff rather than an active bug for as long as
`cargo nextest`'s one-process-per-test isolation was the only consumer, so it
never accumulated across the test suite. [ADR-006](../adr/006_vulkan_windowed_presentation.md)'s
windowed loop invalidates that reasoning — it runs thousands of frames in one
process, each allocating a command pool, render pass and framebuffer that are
never destroyed — so on the windowed path this is now a genuine defect rather
than a documented simplification. The other three backends get cleanup for
free via `wgpu`'s / `web_sys`'s own `Drop` impls; only `vulkan`, built
directly on raw `ash` with no owning graphics crate underneath, lacks it.
Texture upload is covered on this backend too, mirroring `native`'s own
`texture_write()` proof above ( `vulkan_texture_write_readback` in
`tests/vulkan_backend_test.rs`, constructing a textured quad and asserting
two successive `texture_write` uploads both land via readback ).
Tracked by tasks 201 ( `minvulkan` driver ), 202 ( this crate's `vulkan`
backend variant ), 203 ( the consuming example ).

Presentation, added later, is what makes the four backends uniform in
capability : `Device::new_native_windowed` and `Device::new_vulkan_windowed`
return the same `( Device, Queue, Surface )` triple against a real presentable
surface — `minwgpu`'s surface and a real `VK_KHR_swapchain` respectively —
where `new_native`/`new_vulkan` return an offscreen image
( ADRs [005](../adr/005_windowed_native_presentation.md),
[006](../adr/006_vulkan_windowed_presentation.md) ). The browser pair needs no
counterpart : a canvas already is the presentable surface. What a windowed
surface changes is only the frame loop — `current_view` acquires,
`present` shows, `resize` rebuilds — at the cost of `pixels_read`, which
returns `Unsupported` there. Neither constructor takes a windowing type, so no
crate at this layer or below depends on `winit`/`sdl2`/`glfw`. Both windowed
paths now have a running consumer in `examples/` : `vulkan`'s is
`examples/gpu_hal/triangle_vulkan_window` ( the one configuration in this
workspace whose process links no `wgpu` at all ); `new_native_windowed`'s is
`examples/gpu_hal/triangle_native_window`, driving the same
`current_view`/`present`/`resize` triple through a real `wgpu::Surface`
swapchain in a `winit` window ( `Arc<Window>` handle, not `&Window` — the
one difference from its Vulkan sibling, which takes `&Window` because
`minvulkan` accepts the raw `HasWindowHandle`/`HasDisplayHandle` traits by
reference ).

### ADRs

| File | Relationship |
|------|--------------|
| [../adr/002_gpu_hal_in_house.md](../adr/002_gpu_hal_in_house.md) | Build-vs-buy decision — closed in-house; `gpu_hal` is the L1 HAL |
| [../adr/003_d2_stack_hal_adoption.md](../adr/003_d2_stack_hal_adoption.md) | Extends L1 adoption to the d2 stack ( `tilemap_renderer` ) |
| [../adr/004_native_vulkan_hal_backend.md](../adr/004_native_vulkan_hal_backend.md) | Adds a fourth, `wgpu`-free `vulkan` backend via `minvulkan` — implemented ( task 202 ) |
| [../adr/005_windowed_native_presentation.md](../adr/005_windowed_native_presentation.md) | Gives the `native` ( `wgpu` ) backend a real windowed swapchain path via handle traits |
| [../adr/006_vulkan_windowed_presentation.md](../adr/006_vulkan_windowed_presentation.md) | Gives the `vulkan` backend a real `VK_KHR_swapchain`, closing the asymmetry ADR-005 left open |

### Explorations

| File | Relationship |
|------|--------------|
| [../explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md) | The comparison and spike evidence behind ADR-002 ( closed ) |

### Layers

| File | Relationship |
|------|--------------|
| [001_l0_drivers.md](001_l0_drivers.md) | The only layer L1 may depend on |
| [003_l2_frame_orchestration.md](003_l2_frame_orchestration.md) | The first layer that should consume L1 |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/003_cross_stack_bridge_via_foundation_resources.md](../pattern/003_cross_stack_bridge_via_foundation_resources.md) | The HAL is the backend-agnostic layer a cross-stack bridge crate composes foundation resources through |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/gpu_hal/` | The v0 implementation |
| `module/helper/gpu_hal/src/webgl_build.rs` | Build-time WGSL→GLSL ES 300 translator (`webgl-glsl-build` feature) — required build-dependency of `renderer/build.rs`, optional for `examples/orrery/flexible` |
| `module/helper/renderer/src/webgpu/` | First consumer — the canonical opaque path on both backends |
| `module/helper/tilemap_renderer/src/adapters/webgpu.rs`, `src/adapters/native.rs` | Second targeted consumer — `adapter-webgpu` / `adapter-native` ( [ADR-003](../adr/003_d2_stack_hal_adoption.md) ) |
| `examples/orrery/flexible/src/main.rs` | Reference/comparison consumer, not an L3 stack engine — depends only on `gpu_hal` (no direct `minwebgl`/`minwebgpu`/`minwgpu`/`minvulkan`/`renderer`) and reaches all four backends through the unified `gpu_hal::Device::new(...)` constructor, which itself dispatches to whichever backend the crate's own Cargo feature ( `webgl`/`webgpu`/`wgpu`/`vulkan` ) selected — bypassing L3 entirely, unlike this table's other two rows |

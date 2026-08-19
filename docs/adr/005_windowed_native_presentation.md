# ADR-005: Windowed Native Presentation via Handle Traits, Not a Windowing Driver

- **Date**: 2026-08-18
- **Status**: Accepted
- **Deciders**: wandalen

## Context

Every native path in this workspace terminated in an offscreen texture. The
`native` backend rendered into a `wgpu::Texture` read back with `pixels_read`;
[ADR-004](004_native_vulkan_hal_backend.md)'s `vulkan` backend did the same
through `minvulkan`. `examples/orrery/flexible`'s own manifest records the
consequence directly — *"wgpu/vulkan have no windowing support in `gpu_hal`
today ... so the native paths render one frame offscreen and save it"* — and
`gpu_hal/src/vulkan.rs` states it in the code: *"there is no swapchain"*.

Nothing under `module/` referenced `winit` or any other windowing library.

This blocked a goal ADR-004 had already committed to. That ADR's stated
purpose was that `examples/orrery/flexible` *"can offer Vulkan without linking
`wgpu`"*, and its manifest enforces *"Only the `wgpu` feature may link the
`wgpu` crate."* But a consumer that wants a **window** had no cgtools-supplied
route at all: it had to create the `wgpu::Instance`, `wgpu::Surface` and
swapchain itself, which meant naming `wgpu` in its own manifest regardless of
which backend it nominally selected. The wgpu-free objective held only for
offscreen consumers.

A second, smaller gap compounded it: `minwgpu` did not re-export `wgpu`, while
its two sibling L0 drivers both re-export their own host API (`minwebgl` and
`minwebgpu` each carry `own use ::web_sys;`, and `renderer` relies on it —
`minwebgl::web_sys::WebGlTexture`). So even a consumer willing to name `wgpu`
types had to declare `wgpu` as a second, independently-versioned dependency
rather than reaching it through the driver.

## Decision

**1. cgtools accepts a window as handle traits, and never depends on a
windowing library.** The windowed entry points take
`impl Into< wgpu::SurfaceTarget< '_ > >` — in practice any type implementing
`raw_window_handle::HasWindowHandle + HasDisplayHandle`, which `wgpu`
blanket-converts. `winit` is not a dependency of any crate under `module/`,
and does not become one. A consumer may use `winit`, `sdl2`, `glfw`, or a raw
handle; the window stays the consumer's concern, as windowing is neither
computer graphics nor mathematics.

**2. `minwgpu` (L0) gains the windowed surface and frame lifecycle.**
`surface::from_window`, `context::windowed`/`windowed_with`, and a `Windowed`
type owning context + surface + configuration together. `surface::Frame`
collapses `wgpu::CurrentSurfaceTexture`'s seven acquisition outcomes into the
three a render loop acts on (`Ready`, `Skip`, `Reconfigure`) — deliberately
**exhaustive**, unlike the crate's `Error`, because its purpose is to be a
closed, stable simplification of an open-ended upstream status.

**3. `minwgpu` re-exports `wgpu`**, matching `minwebgl` and `minwebgpu`. This
corrects an inconsistency rather than establishing a new pattern, and is what
lets a consumer manifest drop its own `wgpu` entry.

**4. `gpu_hal` (L1) gains a `Surface::NativeWindow` variant** and
`Device::new_native_windowed`, following the enum-per-backend dispatch pattern
[ADR-002](002_gpu_hal_in_house.md) established. It carries the acquired
swapchain frame in a `RefCell` so `current_view` keeps the `&self` signature
its three sibling backends share; `Surface::present` and `Surface::resize` are
no-ops everywhere else. A transient acquisition failure surfaces as the new
`Error::SurfaceNotReady`, which is an expected render-loop condition, not a
fault.

**5. `TextureFormat` gains `Bgra8UnormSrgb`.** A desktop swapchain commonly
selects it, and the v0 surface could not name it — so the HAL could not
describe its own presentation format. Its reverse conversion
(`TryFrom< wgpu::TextureFormat >`) is added for the same reason: the HAL must
name what a driver *chose*, not only convert formats it selected itself.

**Scope.** This decision covers the `native` (`wgpu`) leg only. The `vulkan`
leg keeps its offscreen-only surface: a real `VK_KHR_swapchain` in `minvulkan`
is a substantially larger piece of work, and is deliberately left for its own
decision rather than folded in here.

## Alternatives Considered

- **A `minwindow` L0 driver wrapping `winit`.** Rejected: it would
  version-lock every consumer to one `winit` release, duplicate a solved
  problem, and take cgtools into a domain that is neither CG nor math. The
  handle traits are the standard interop boundary and cost nothing —
  `raw-window-handle` is already in the tree beneath `wgpu`.
- **Windowing at L1 (`gpu_hal`) only, leaving `minwgpu` offscreen-only.**
  Rejected: it would invert the layering — L0 drivers are the backend-faithful
  layer, and a swapchain is a plain `wgpu` concept that belongs there. It
  would also leave a direct `minwgpu` consumer with no windowed path at all.
- **Keep `minwgpu` non-re-exporting and let consumers depend on `wgpu`
  directly.** Rejected: it leaves the driver inconsistent with its two
  siblings for no stated benefit, and makes the wgpu-free objective
  unreachable by construction.
- **Do the Vulkan swapchain in the same change.** Rejected as scope: the
  `wgpu` leg delivers a windowed native path immediately, and bundling an
  unrelated multi-week Vulkan effort would delay it without making it better.

## Consequences

- **Positive**: A consumer can render to a native window through cgtools
  alone, with no `wgpu` entry in its own manifest — extending ADR-004's
  wgpu-free objective from offscreen consumers to windowed ones. `minwgpu`'s
  three L0 drivers are now consistent in re-exporting their host API.
- **Positive**: The seven-arm acquisition status and the wgpu 30 migration of
  presentation from `SurfaceTexture::present` to `Queue::present` are both
  absorbed once, in the driver, instead of being re-derived at every call
  site.
- **Negative**: `Surface::NativeWindow` introduces interior mutability into a
  previously plain enum, and `Surface` is consequently no longer `Sync`.
  Nothing in the workspace used it across threads, but this narrows what a
  future consumer may do with it.
- **Negative**: `Surface::current_view` can now fail transiently, on one
  backend only. Callers of the browser and offscreen backends are unaffected,
  but the shared signature is no longer "never fails except on WebGPU".
- **Neutral**: The `vulkan` backend stays offscreen-only, so the four backends
  are no longer uniform in presentation capability until that gap is closed
  separately.

## Related

- [002_gpu_hal_in_house.md](002_gpu_hal_in_house.md) — the enum-per-backend
  dispatch pattern the `NativeWindow` variant follows
- [004_native_vulkan_hal_backend.md](004_native_vulkan_hal_backend.md) — the
  wgpu-free objective this ADR extends from offscreen to windowed consumers;
  its `vulkan` leg is explicitly out of scope here
- [layer/001_l0_drivers.md](../layer/001_l0_drivers.md) — L0's driver
  contract, which the new `minwgpu` surface and frame API follows
- [layer/002_l1_gpu_hal.md](../layer/002_l1_gpu_hal.md) — L1's contract the
  `NativeWindow` variant extends

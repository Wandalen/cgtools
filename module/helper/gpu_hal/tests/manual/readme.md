# Manual Testing Guide for gpu_hal (presented-pixel paths)

This guide covers the two `gpu_hal` paths that end in a *presented* frame rather
than a readback, and so cannot be `cargo test`-automated: the browser backends
(`webgpu`, `webgl`), which need a real browser painting a real canvas
(Scenarios 1–4), and the windowed Vulkan swapchain, which needs a real window
handle no crate under `module/` can produce (Scenario 5). Both are counterparts
to `tests/native_backend_test.rs` / `tests/vulkan_backend_test.rs`'s
`triangle_render_readback`, which prove the same render path through an offscreen
readback instead.

## Prerequisites

**Scenarios 1–4 (browser):**

- `trunk` on `PATH` (serves the wasm32 build with hot reload)
- `browsee` on `PATH` (`command -v browsee && browsee .diagnose`)
- A reachable X display (`$DISPLAY`, or Xvfb — `browsee .diagnose` reports which)

**Scenario 5 (windowed Vulkan):**

- A Vulkan ICD exposing `VK_KHR_swapchain` for the current display server —
  a software rasterizer (lavapipe / `mesa-vulkan-drivers`) suffices:
  `vulkaninfo | grep -i swapchain`
- A reachable X display, plus `wmctrl` and `import`/`scrot` (or any screenshot
  tool) to drive and capture the window

## Test Application

`examples/gpu_hal/triangle_browser/` — one crate, two backend features, reusing
`triangle_render_readback`'s WGSL shader and vertex/uniform data (red triangle,
black clear). Build and serve one backend at a time:

```bash
cd examples/gpu_hal/triangle_browser
trunk serve --release --port 8080                                        # webgpu (default feature)
trunk serve --release --no-default-features --features webgl --port 8080 # webgl
```

Wait for `trunk`'s `📡 server listening at:` line before launching the browser —
`browsee .wait for::render` blocks on the *page* rendering, not on the dev server
being up yet.

## Test Scenarios

### 1. WebGPU backend renders the triangle

**Objective:** `Device::new_webgpu( canvas )` + one render pass paints a real frame.

**Steps:**
```bash
trunk serve --release --port 8080 &                    # webgpu build
browsee .launch session::gpu_hal_tri url::http://127.0.0.1:8080/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::gpu_hal_tri
browsee .shot out::./-gpu_hal_webgpu.png session::gpu_hal_tri
```

**Expected Behavior:**
- `.wait for::render` exits 0 (`rendered::rgb ...`), not a timeout
- The screenshot shows a solid red triangle on a solid black background — see
  "Reading exact pixel values" below for a pixel-level assertion

### 2. WebGL2 backend renders the triangle

**Objective:** `Device::new_webgl( canvas )` + one render pass paints a real frame,
consuming the GLSL ES override pair (`ShaderSource::glsl_vertex`/`glsl_fragment`)
since WebGL cannot execute WGSL directly.

**Steps:** identical to Scenario 1, against the `webgl` build:
```bash
trunk serve --release --no-default-features --features webgl --port 8080 &
browsee .launch session::gpu_hal_tri url::http://127.0.0.1:8080/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::gpu_hal_tri
browsee .shot out::./-gpu_hal_webgl.png session::gpu_hal_tri
```

**Expected Behavior:** same as Scenario 1 — solid red triangle on solid black,
proving the GLSL ES override renders identically to the WGSL path.

### 3. Bounded draw — clear color outside the triangle

**Objective:** confirm the render pass only painted the triangle, not the whole
canvas (guards against a fragment shader that ignores the mask entirely).

**Steps:** with either session from Scenario 1/2 still open, sample a corner:
```bash
browsee .pixel region::100x100x0,0 session::gpu_hal_tri
```

**Expected Behavior:** reads back the configured clear color (this example's
clear is `[0.0, 0.0, 0.0, 1.0]` — pure black), not the triangle's red.

### 4. `buffer_write` rejects oversized WebGL data (BUG-200)

**Objective:** confirm `Queue::buffer_write`'s WebGL arm returns `Err` instead of silently
no-op'ing when `data` exceeds the destination buffer's allocated size — `bufferSubData`
(`buffer_sub_data_with_i32_and_u8_array`) returns `()` and cannot surface the underlying
`INVALID_VALUE` itself, so this guard is the only signal available.

**Steps:** with the `webgl` build served (Scenario 2's `trunk serve` invocation), the example
itself performs the check on every load — it attempts a 16-byte write into a 4-byte buffer and
switches the clear color to cyan if that write does *not* return `Err`:
```bash
trunk serve --release --no-default-features --features webgl --port 8080 &
browsee .launch session::gpu_hal_tri url::http://127.0.0.1:8080/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::gpu_hal_tri
browsee .pixel region::40x40x300,150 session::gpu_hal_tri   # sample well inside the clear band, away from window chrome
```

**Expected Behavior:** the clear band reads pure black (`0, 0, 0`) — the guard fired, the
oversized write returned `Err`, and the example proceeded to its normal render. A cyan reading
(`0, 255, 255`) means the guard did not fire and `buffer_write` silently accepted data too large
for the buffer.

**Verified** 2026-08-16: guard reverted (temporary source edit) → clear band read pure cyan
`(0, 255, 255)`, reproducing the pre-fix defect in a real Firefox/WebGL2 context; guard restored
→ clear band read black (`4, 0, 0`, negligible antialiasing residue) with the triangle intact
(`255, 0, 0`), confirmed by both `.pixel` sampling and a full `.shot` screenshot.

### Reading exact pixel values (`region::center` caveat)

`browsee .pixel region::center` measures the center of the **window**, not the
**canvas** — on a normal (non-kiosk) browser launch the window includes its own
tab bar and address bar above the page content, so `region::center` and a naive
`region::100x100x0,0` both read a color blended with that chrome instead of the
canvas's true content. Confirmed on this build (Firefox via `browser::auto`,
`window::800x600`): `region::center` read `rgb 72 16 16` (should be pure red) and
a naive top-left `region::100x100x0,0` read `rgb 127 127 127` (should be pure
black) — both chrome-contaminated, not a rendering defect. A `--app=URL` /
chrome-less launch was tried as a fix and produced a blank page instead (`.wait
for::render` timed out) — not a viable workaround with the current browsee launch
flags, so this remains an open caveat rather than a resolved one.

Two reliable options once you have a `.shot` PNG open in front of you:

1. **Visual/PNG inspection** (most robust — no offset math): open the screenshot
   and confirm by eye, or scan a vertical center-column scanline with Pillow:
   ```bash
   python3 -c "
   from PIL import Image
   im = Image.open('./-gpu_hal_webgpu.png').convert('RGB')
   w, h = im.size
   for y in range(0, h, 5): print(y, im.getpixel((w//2, y)))
   "
   ```
   Content below the chrome band reads pure `(0, 0, 0)` (clear) then pure
   `(255, 0, 0)` (triangle) with a sharp, unblended transition — no antialiasing
   at this shape's edges on a center scanline.
2. **`browsee .pixel` with a chrome-corrected offset** — once you've confirmed the
   content boundary from a screenshot, target regions inside it directly. Verified
   working for `window::800x600` + `browser::auto` (Firefox) on this system:
   ```bash
   browsee .pixel region::40x40x300,120 session::gpu_hal_tri   # black clear band
   browsee .pixel region::40x40x300,270 session::gpu_hal_tri   # red triangle band
   ```
   Both read exactly `rgb 0 0 0` / `rgb 255 0 0` for both backends. These offsets
   are a snapshot of this browser/theme's chrome height, not a portable constant —
   re-derive them from a fresh `.shot` if the browser, window size, or OS theme
   changes.

### 5. Windowed Vulkan swapchain presents and rebuilds on resize

**Objective:** confirm `Device::new_vulkan_windowed` + `Surface::current_view` /
`present` / `resize` drive a real `VK_KHR_swapchain` — that frames actually cycle
rather than one stale image persisting, and that a resize rebuilds the chain
rather than stretching or tearing the old one.

**Steps:** the example animates its triangle's color continuously, which is what
makes "did a new frame present?" observable in a still screenshot at all:

```bash
cargo run -p gpu_hal_triangle_vulkan_window --release &
sleep 3
import -window "gpu_hal triangle -- Vulkan swapchain" ./-vulkan_triangle.png
wmctrl -r "gpu_hal triangle -- Vulkan swapchain" -e 0,330,360,420,700   # resize
sleep 2
wmctrl -l -G | grep "Vulkan swapchain"                                  # confirm new geometry
import -window "gpu_hal triangle -- Vulkan swapchain" ./-vulkan_triangle_resized.png
```

**Expected Behavior:**
- Both screenshots show one triangle on the dark clear color, **in two different
  colors** — the same color twice means frames are not cycling (a stale image, or
  `present` never reaching `vkQueuePresentKHR`)
- The second screenshot's window measures the requested size, and the triangle is
  centered and correctly proportioned in it — not stretched, letterboxed, or
  clipped, each of which means the swapchain was reconfigured without a rebuild
- No Vulkan validation output on stderr, and the process exits 0 when the window
  is closed — a `VK_ERROR_OUT_OF_DATE_KHR` escaping as a panic would surface here

**Verified** 2026-08-18 (lavapipe): 800x600 screenshot showed a magenta triangle;
after `wmctrl` resize, `wmctrl -l -G` reported geometry `420x700` and the second
screenshot showed a green triangle, correctly proportioned at the new aspect
ratio. Two distinct colors confirm the acquire/present loop cycled; the clean
geometry confirms `resize`'s `vkDeviceWaitIdle` + `oldSwapchain` rebuild. Log
carried no errors, panics, or validation messages; exit 0.

## Teardown

```bash
browsee .kill session::gpu_hal_tri purge::1        # scenarios 1-4
# then stop the trunk dev server (Ctrl-C, or kill the backgrounded PID)
# scenario 5: close the window, or kill the `cargo run` PID
```

## Test Matrix

| Scenario | webgpu | webgl | vulkan (windowed) |
|----------|--------|-------|-------------------|
| Render completes (`.wait for::render` exits 0) | ✓ | ✓ | n/a — no browser; observed as a painted window |
| Triangle reads configured color (`255, 0, 0`) | ✓ | ✓ | n/a — this example animates its color deliberately |
| Corner reads configured clear color (`0, 0, 0`) | ✓ | ✓ | ✓ |
| Oversized `buffer_write` rejected, no cyan clear (BUG-200) | n/a — WebGPU validates writes itself | ✓ | not covered |
| Successive frames differ (present loop is cycling) | n/a — static frame | n/a — static frame | ✓ |
| Resize rebuilds the swapchain, geometry stays proportioned | n/a — canvas is resized by the browser | n/a | ✓ |

Native and Vulkan offscreen regression coverage and the wasm32 compile check are
automated — `cargo nextest run -p gpu_hal --features native`,
`cargo nextest run -p gpu_hal --features vulkan` and
`cargo check -p gpu_hal --features webgpu,webgl --target wasm32-unknown-unknown`
(see `gpu_hal/readme.md`'s `## Verify` section). Only the two presented-pixel
paths above require this manual procedure.

`Device::new_native_windowed` has no scenario here: no example drives it, so the
`NativeWindow` variant's dispatch is currently unexercised by anything that runs
(`examples/minwgpu/flecs_bouncing_circles` covers the underlying
`minwgpu::surface::Windowed` directly, one layer below this HAL). Closing that is
a matter of adding an example, not of writing a scenario for one that exists.

## Reporting Issues

When reporting issues found during manual testing, please include:
- Browser name and version (`browsee .diagnose`)
- The exact `browsee`/`trunk` command sequence used
- The `.pixel`/`.wait` output and a `.shot` screenshot
- Whether the same defect reproduces on both backends or only one — a
  single-backend defect points at that backend's `Device`/`ShaderSource` path,
  a both-backends defect points at the example's shared render logic
- For Scenario 5 instead: `vulkaninfo | head -20` (driver and ICD), both
  screenshots, the `wmctrl -l -G` geometry line, and the process's full stderr —
  Vulkan validation output goes there and is the primary diagnostic

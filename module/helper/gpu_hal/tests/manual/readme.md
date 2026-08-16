# Manual Testing Guide for gpu_hal (browser backends)

This guide verifies `gpu_hal`'s `webgpu` and `webgl` backends against real painted
pixels in an actual browser, using the `browsee` CLI. It is the browser-side
counterpart to `tests/native_backend_test.rs`'s `triangle_render_readback`, which
proves the same render path on the `native` backend through an offscreen wgpu
readback instead of a browser canvas. This cannot be `cargo test`-automated — it
requires a real browser painting a real canvas.

## Prerequisites

- `trunk` on `PATH` (serves the wasm32 build with hot reload)
- `browsee` on `PATH` (`command -v browsee && browsee .diagnose`)
- A reachable X display (`$DISPLAY`, or Xvfb — `browsee .diagnose` reports which)

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

## Teardown

```bash
browsee .kill session::gpu_hal_tri purge::1
# then stop the trunk dev server (Ctrl-C, or kill the backgrounded PID)
```

## Test Matrix

| Scenario | webgpu | webgl |
|----------|--------|-------|
| Render completes (`.wait for::render` exits 0) | ✓ | ✓ |
| Triangle reads configured color (`255, 0, 0`) | ✓ | ✓ |
| Corner reads configured clear color (`0, 0, 0`) | ✓ | ✓ |

Native regression coverage and the wasm32 compile check are automated —
`cargo nextest run -p gpu_hal --features native` and
`cargo check -p gpu_hal --features webgpu,webgl --target wasm32-unknown-unknown`
(see `gpu_hal/readme.md`'s `## Verify` section) — only the browser-painted-pixel
assertion above requires this manual procedure.

## Reporting Issues

When reporting issues found during manual testing, please include:
- Browser name and version (`browsee .diagnose`)
- The exact `browsee`/`trunk` command sequence used
- The `.pixel`/`.wait` output and a `.shot` screenshot
- Whether the same defect reproduces on both backends or only one — a
  single-backend defect points at that backend's `Device`/`ShaderSource` path,
  a both-backends defect points at the example's shared render logic

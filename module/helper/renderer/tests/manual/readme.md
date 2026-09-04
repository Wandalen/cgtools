# Manual Testing Guide for renderer (browser opaque path)

This guide verifies `renderer`'s canonical opaque path (`src/webgpu/renderer.rs`,
`WebGpuRenderer`) against real painted pixels in an actual browser, using the
`browsee` CLI. It is the browser-side counterpart to
`tests/native_render_test.rs`'s `opaque_path_renders_lit_quad`, which proves the
same render path on the `native` backend through an offscreen wgpu readback
instead of a browser canvas. This cannot be `cargo test`-automated — it requires a
real browser painting a real canvas.

## Prerequisites

- `trunk` on `PATH` (serves the wasm32 build with hot reload)
- `browsee` on `PATH` (`command -v browsee && browsee .diagnose`)
- A reachable X display (`$DISPLAY`, or Xvfb — `browsee .diagnose` reports which)

## Test Application

`examples/renderer/opaque_path_browser/` — one crate, two backend features,
reusing `opaque_path_renders_lit_quad`'s exact scene: one unit quad `Geometry`,
one red `PbrMaterial` (`base_color_factor = [1,0,0,1]`), one directional light
(`[0,0,1]` direction, `[1,1,1]` color, intensity `3.0`), one `Frame` (camera at
`eye = [0,0,2.5]` via `look_at_rh`/`perspective_rh`). Build and serve one backend
at a time:

```bash
cd examples/renderer/opaque_path_browser
trunk serve --release --port 8080                                        # webgpu (default feature)
trunk serve --release --no-default-features --features webgl --port 8080 # webgl
```

Wait for `trunk`'s `📡 server listening at:` line before launching the browser —
`browsee .wait for::render` blocks on the *page* rendering, not on the dev server
being up yet.

## Test Scenarios

### 1. WebGPU backend renders the lit quad

**Objective:** `GpuContext::new_webgpu( &canvas )` + `WebGpuRenderer::render` paints
a real frame through the canonical opaque path.

**Steps:**
```bash
trunk serve --release --port 8080 &                     # webgpu build
browsee .launch session::renderer_opaque url::http://127.0.0.1:8080/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::renderer_opaque
browsee .pixel region::center session::renderer_opaque
browsee .shot out::./-renderer_opaque_webgpu.png session::renderer_opaque
```

**Expected Behavior:**
- `.wait for::render` exits 0 (`rendered::rgb ...`), not a timeout — note the
  reported color here is a generic paint-detection sample, not necessarily the
  canvas center; do not use it as the pass/fail reading, use `.pixel` below
- `.pixel region::center` reads a lit-red-dominant pixel (`r > 150, g < 80,
  b < 80`), matching `opaque_path_renders_lit_quad`'s bound. Verified reading on
  this build: `rgb 205 46 41`.

### 2. WebGL2 backend renders the lit quad

**Objective:** `GpuContext::new_webgl( &canvas )` + `WebGpuRenderer::render` paints
the same frame over a WebGL2 context.

**Steps:** identical to Scenario 1, against the `webgl` build:
```bash
trunk serve --release --no-default-features --features webgl --port 8080 &
browsee .launch session::renderer_opaque url::http://127.0.0.1:8080/ features::software_gl window::800x600
browsee .wait for::render timeout::60 session::renderer_opaque
browsee .pixel region::center session::renderer_opaque
browsee .shot out::./-renderer_opaque_webgl.png session::renderer_opaque
```

**Expected Behavior:** same bound as Scenario 1. Verified reading on this build:
`rgb 205 46 41` — identical to the webgpu backend, as expected since both drive
the same `WebGpuRenderer::render` call over the same scene data.

### 3. Bounded draw — clear color outside the quad

**Objective:** confirm the render pass only painted the quad, not the whole
canvas (guards against a test that would pass even if the draw call painted the
whole canvas).

**Steps:** with either session from Scenario 1/2 still open, sample a corner:
```bash
browsee .pixel region::20x20x5,5 session::renderer_opaque
```

**Expected Behavior:** reads background black (`rgb 0 0 0`) — the quad is a unit
square centered at the origin and does not cover the canvas corners at this
camera's field of view. Verified identical on both backends.

### `region::center` worked directly for this example

Unlike `gpu_hal/tests/manual/readme.md`'s own caveat (where `region::center`
read a chrome-contaminated blend on `triangle_browser`), `region::center` read
the correct in-bound lit-red pixel directly on this example, with no offset
correction needed — `retrieve_or_make()`'s canvas fills the full `html`/`body`
box (both set to `width:100%; height:100%` in `index.html`) and is resized via
`ResizeObserver` to match its own CSS box at device-pixel-ratio resolution, so
the canvas covers the entire viewport rather than a smaller centered region.
If a future browser/theme change reintroduces chrome contamination, fall back to
a `.shot` PNG + scanline inspection exactly as documented in `gpu_hal`'s guide,
then re-derive a corrected offset the same way.

## Teardown

```bash
browsee .kill session::renderer_opaque purge::1
# then stop the trunk dev server (Ctrl-C, or kill the backgrounded PID)
```

## Test Matrix

| Scenario | webgpu | webgl |
|----------|--------|-------|
| Render completes (`.wait for::render` exits 0) | ✓ | ✓ |
| Center reads lit-red-dominant (`r>150, g<80, b<80`) | ✓ (`205,46,41`) | ✓ (`205,46,41`) |
| Corner reads background black (`0,0,0`) | ✓ | ✓ |

Native regression coverage and the wasm32 compile check are automated —
`cargo nextest run -p renderer --features native` and
`cargo check -p renderer --features webgpu --target wasm32-unknown-unknown`
(see `renderer/readme.md`'s `## Verify`-equivalent section) — only the
browser-painted-pixel assertion above requires this manual procedure.

## Reporting Issues

When reporting issues found during manual testing, please include:
- Browser name and version (`browsee .diagnose`)
- The exact `browsee`/`trunk` command sequence used
- The `.pixel`/`.wait` output and a `.shot` screenshot
- Whether the same defect reproduces on both backends or only one — a
  single-backend defect points at that backend's `GpuContext` construction path,
  a both-backends defect points at `WebGpuRenderer`'s shared render logic

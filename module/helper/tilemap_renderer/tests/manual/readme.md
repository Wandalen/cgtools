# Manual Testing Guide for tilemap_renderer (browser adapters)

This guide verifies `tilemap_renderer`'s `adapter-webgpu` and `adapter-webgl`
`Backend` impls against real painted pixels in an actual browser, using the
`browsee` CLI. It is the browser-side counterpart to
`tests/native_backend_test.rs`'s `sprite_and_corner_pixels_match_configured_colors`,
which proves the same construct → assets_load → submit → output flow on
`adapter-native` through an offscreen GPU readback instead of a browser canvas.
This cannot be `cargo test`-automated — it requires a real browser painting a
real canvas.

## Prerequisites

- `trunk` on `PATH` (serves the wasm32 build with hot reload)
- `browsee` on `PATH` (`command -v browsee && browsee .diagnose`)
- A reachable X display (`$DISPLAY`, or Xvfb — `browsee .diagnose` reports which)

## Test Application

`examples/tilemap_renderer/adapter_browser/` — one crate, two backend
features, reusing `native_backend_test.rs`'s exact 8x8 solid-red sprite asset:
one `Clear` (blue, `[0,0,1,1]`) plus one `Sprite` centered in the viewport at
`SPRITE_PROPORTION` (`0.375`) of its extent. Build and serve one backend at a
time:

```bash
cd examples/tilemap_renderer/adapter_browser
trunk serve --release --port 8080                                        # webgpu (default feature)
trunk serve --release --no-default-features --features webgl --port 8080 # webgl
```

**The two backends are NOT expected to paint the same pixel.** `adapter-webgl`
uploads real pixel bytes (`tex_image_2d_with_...`) and paints the sprite's
configured solid red; `adapter-webgpu` has no texture-upload path wired yet
(see `src/adapters/webgpu.rs:9-13`'s module doc comment) and paints an opaque
**black** quad instead — this crate proves each backend's own honest, distinct
current behavior rather than a uniform claim neither could back up.

**Gotcha for anyone editing `centered_sprite_command`:** `Transform::position`
is the sprite quad's *starting corner*, not its center, and `Transform::scale`
multiplies the sprite's region size (`8.0` here), not its final on-screen
size — both `sprite.vert` and `webgpu.rs`'s WGSL shader compute
`world = transform * ( quad * region_size )` for a raw `[0,1]` unit `quad`.
Mirroring `native_backend_test.rs::centered_sprite_command`'s raw
`position`/`scale` numbers verbatim produces a quadrant-filling oversized quad
here, not a centered square — that native test's own two pixel assertions
happen to pass under either interpretation, so it never caught this. See
`examples/tilemap_renderer/adapter_browser/src/main.rs`'s own
`centered_sprite_command` doc comment for the corrected, actually-centering
formula.

## Test Scenarios

### 1. WebGL backend renders the solid-red sprite

**Objective:** `WebGlBackend::new` + `assets_load` + `submit` + `output`
paints a real, correctly-centered sprite through a real WebGL2 context.

**Steps:**
```bash
trunk serve --release --no-default-features --features webgl --port 8100 &
browsee .launch session::tmr_adapter_webgl browser::firefox url::http://127.0.0.1:8100/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::tmr_adapter_webgl
browsee .pixel region::20x20x314,270 session::tmr_adapter_webgl   # chrome-corrected — inside the sprite
browsee .shot out::./-tmr_webgl.png session::tmr_adapter_webgl
```

**Expected Behavior:**
- `.wait for::render` exits 0, not a timeout — its reported color is a generic
  paint-detection sample (window-center, chrome-contaminated here — see
  below), not the pass/fail reading; use the corrected `.pixel` offset above.
- The corrected offset reads pure solid red (`rgb 255 0 0`), matching
  `native_backend_test.rs`'s own `SPRITE_RGBA`. Verified reading on this build:
  `rgb 255 0 0`.

### 2. WebGPU backend renders an opaque black quad

**Objective:** `WebGpuBackend::new` + `assets_load` + `submit` + `output`
paints a real, correctly-centered — but unpopulated-texture — quad through a
real WebGPU context, confirming the round-trip bounds a real draw call rather
than silently no-op'ing.

**Steps:** identical shape, against the `webgpu` (default-feature) build:
```bash
trunk serve --release --port 8099 &
browsee .launch session::tmr_adapter browser::firefox url::http://127.0.0.1:8099/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::tmr_adapter
browsee .pixel region::20x20x314,270 session::tmr_adapter   # chrome-corrected — inside the sprite
browsee .shot out::./-tmr_webgpu.png session::tmr_adapter
```

**Expected Behavior:** the corrected offset reads pure opaque black
(`rgb 0 0 0`) — the adapter's own documented current behavior (zero-initialized
texture × tint through `gpu_hal`'s opaque, no-blend v0 pipeline), not the clear
color. Verified reading on this build: `rgb 0 0 0`.

### 3. Bounded draw — clear color outside the sprite

**Objective:** confirm each render pass only painted the sprite, not the whole
canvas (guards against a test that would pass even if the draw call painted the
whole canvas), and that `adapter-webgpu`'s black quad is a bounded shape, not a
full-canvas clear-to-black.

**Steps:** with either session from Scenario 1/2 still open, sample well
inside the canvas but away from the sprite:
```bash
browsee .pixel region::20x20x100,150 session::tmr_adapter_webgl
browsee .pixel region::20x20x100,150 session::tmr_adapter
```

**Expected Behavior:** both read the configured clear color (`rgb 0 0 255`),
distinct from both the sprite's solid red (webgl) and opaque black (webgpu).
Verified identical on both backends.

### `region::center` needs a chrome-corrected offset here

Like `gpu_hal/tests/manual/readme.md`'s own caveat (and unlike
`renderer/tests/manual/readme.md`'s example, where no correction was needed),
`browsee .pixel region::center` measures the center of the **window**, not the
**canvas** — the window includes its own tab bar and address bar above the
page content, so `region::center` reads a color blended with that chrome
instead of the canvas's true content. Confirmed on this build (Firefox,
`window::800x600`): `region::center` read `rgb 16 16 188` (a blend of the
sprite's black and the clear color's blue) instead of a clean unblended
reading of either.

The offset used throughout this guide (`region::20x20x314,270` for the
sprite, `region::20x20x100,150` for the background) was derived exactly as
`gpu_hal`'s guide documents: take a `.shot` PNG, scan a vertical and a
horizontal center-line with Pillow to find the sprite's exact on-canvas
boundary, then pick offsets safely inside each uniform-color region. These
offsets are a snapshot of this browser/theme's chrome height, not a portable
constant — re-derive them from a fresh `.shot` if the browser, window size, or
OS theme changes.

## Teardown

```bash
browsee .kill session::tmr_adapter purge::1
browsee .kill session::tmr_adapter_webgl purge::1
# then stop both trunk dev servers (Ctrl-C, or kill the backgrounded PIDs)
```

## Test Matrix

| Scenario | webgpu | webgl |
|----------|--------|-------|
| Render completes (`.wait for::render` exits 0) | ✓ | ✓ |
| Sprite-center reads configured color | ✓ opaque black (`0,0,0`) | ✓ solid red (`255,0,0`) |
| Background reads configured clear color, distinct from sprite | ✓ (`0,0,255`) | ✓ (`0,0,255`) |

Native regression coverage and the wasm32 compile check are automated —
`cargo nextest run -p tilemap_renderer --features adapter-native` and
`cargo check -p tilemap_renderer --features adapter-webgpu,adapter-webgl --target wasm32-unknown-unknown`
(see `tilemap_renderer/readme.md`'s `## Verify`-equivalent section) — only the
browser-painted-pixel assertion above requires this manual procedure.

## Reporting Issues

When reporting issues found during manual testing, please include:
- Browser name and version (`browsee .diagnose`)
- The exact `browsee`/`trunk` command sequence used
- The `.pixel`/`.wait` output and a `.shot` screenshot
- Whether the same defect reproduces on both backends or only one — a
  single-backend defect points at that backend's `Backend` impl, a
  both-backends defect points at `Transform::to_mat3()` or the shared command
  → scene-command construction in `examples/tilemap_renderer/adapter_browser`

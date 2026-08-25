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

**Historical note (superseded by task 218, re-verified live):** at the time
this guide was first written, the two backends were NOT expected to paint the
same pixel — `adapter-webgl` uploaded real pixel bytes
(`tex_image_2d_with_...`) and painted the sprite's configured solid red, while
`adapter-webgpu` had no texture-upload path wired and painted an opaque
**black** quad instead (see `src/adapters/webgpu.rs:9-13`'s then-current module
doc comment). Task 218 has since wired `adapter-webgpu`'s own real pixel
upload through `gpu_hal::Queue::texture_write`, sharing the same `to_rgba8`
conversion helper `adapter-native` uses, and this has now been re-verified
live in Firefox: both backends paint the identical solid red (`rgb 255 0 0`)
sprite on the identical blue (`rgb 0 0 255`) clear color. Scenario 2 and the
Test Matrix below have been updated with the confirmed reading — every
`rgb 0 0 0` reading remaining in this guide is now explicitly historical
(pre-218), not the adapter's current behavior.

**Chromium gotcha (this sandbox):** unlike Firefox, Chromium has been observed
to intermittently fail to present the `adapter-webgpu` canvas frame at all —
`.wait for::render` times out, and the session log shows GPU-process/compositor
-level errors (e.g. Dawn's `CopyTextureForBrowser`/`[Invalid Texture]`, or a
lower-level `vaInitialize failed`/`ContextResult::kTransientFailure` GPU-process
crash) with a different signature each time, consistent with this sandbox's
virtualized/software-GPU limitations rather than a defect in this crate's
texture-upload code — the same Rust/wasm path renders correctly in Firefox.
Use Firefox for this procedure; it is the proven, reliable browser here.

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

### 2. WebGPU backend renders its sprite (re-verified post-task-218)

**Objective:** `WebGpuBackend::new` + `assets_load` + `submit` + `output`
paints a real, correctly-centered quad through a real WebGPU context,
confirming the round-trip bounds a real draw call rather than silently
no-op'ing.

**Steps:** identical shape, against the `webgpu` (default-feature) build:
```bash
trunk serve --release --port 8099 &
browsee .launch session::tmr_adapter browser::firefox url::http://127.0.0.1:8099/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::tmr_adapter
browsee .pixel region::20x20x314,270 session::tmr_adapter   # chrome-corrected — inside the sprite
browsee .shot out::./-tmr_webgpu.png session::tmr_adapter
```

**Expected Behavior (pre-task-218, historical):** the corrected offset read
pure opaque black (`rgb 0 0 0`) — `WebGpuBackend` had no texture-upload path
wired at the time, so the fragment shader sampled a zero-initialized texture
through `gpu_hal`'s opaque, no-blend v0 pipeline.

**Expected Behavior (post-task-218, live-confirmed):**
`WebGpuBackend::assets_load` now uploads the sprite's real pixel bytes (same
solid-red 8x8 bitmap, same `to_rgba8` conversion `adapter-native` uses).
Re-run in Firefox against a fresh `trunk serve --release` build: the corrected
offset reads pure solid red (`rgb 255 0 0`), matching Scenario 1 exactly.
Chromium in this sandbox failed to present a frame at all on the same build
(see the Chromium gotcha note above) — treat that as an environment limitation
of this sandbox, not a re-opening of this reading; Firefox's reading is the
one this guide treats as authoritative.

### 3. Bounded draw — clear color outside the sprite

**Objective:** confirm each render pass only painted the sprite, not the whole
canvas (guards against a test that would pass even if the draw call painted the
whole canvas) — pre-task-218 this also confirmed `adapter-webgpu`'s black quad
was a bounded shape, not a full-canvas clear-to-black; post-task-218 it confirms
the same for whatever the sprite now actually paints.

**Steps:** with either session from Scenario 1/2 still open, sample well
inside the canvas but away from the sprite:
```bash
browsee .pixel region::20x20x100,150 session::tmr_adapter_webgl
browsee .pixel region::20x20x100,150 session::tmr_adapter
```

**Expected Behavior:** both should read the configured clear color
(`rgb 0 0 255`), distinct from the sprite's solid red. Verified identical on
both backends pre-task-218 (when `adapter-webgpu`'s own sprite color was
still black, also distinct from the clear color) and re-verified post-task-218
in Firefox — `rgb 0 0 255` on both, still distinct from the now-red sprite.

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
| Render completes (`.wait for::render` exits 0) | ✓ (Firefox; Chromium intermittently fails to present — see Chromium gotcha note) | ✓ |
| Sprite-center reads configured color | ✓ solid red (`255,0,0`), re-verified post-task-218 in Firefox | ✓ solid red (`255,0,0`) |
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

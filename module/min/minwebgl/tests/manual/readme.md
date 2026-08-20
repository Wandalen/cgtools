# Manual Testing Guide for minwebgl (browser-side context/draw path)

This guide verifies `minwebgl::context::from_canvas` and a minimal shader/buffer/draw
sequence against real painted pixels in an actual browser, using the `browsee` CLI.
`context::from_canvas` and everything downstream of it (shaders, VAOs, buffers) touch
a live GL context/the DOM and cannot be exercised by `cargo test` — they need a real
browser painting a real canvas. This is the browser-side counterpart to `tests/`'s
native suite, which covers only the pure-logic layer (`DataType` conversions,
validation helpers) that needs no GL context at all.

## Prerequisites

- `trunk` on `PATH` (serves the wasm32 build with hot reload)
- `browsee` on `PATH` (`command -v browsee && browsee .diagnose`)
- A reachable X display (`$DISPLAY`, or Xvfb — `browsee .diagnose` reports which)

## Test Application

`examples/minwebgl/context_triangle_smoke/` — one crate, one draw: `from_canvas` to
get a `WebGl2RenderingContext`, `Program::new` to compile+link a minimal shader pair,
`buffer::create`/`buffer::upload` plus a `BufferDescriptor` to upload and describe one
triangle's vertices, then a single `draw_arrays` call. Solid red triangle
(`vec4(1.0, 0.0, 0.0, 1.0)`), solid black clear (`clear_color(0.0, 0.0, 0.0, 1.0)`).

```bash
cd examples/minwebgl/context_triangle_smoke
trunk serve --release --port 8091
```

Wait for `trunk`'s `📡 server listening at:` line before launching the browser —
`browsee .wait for::render` blocks on the *page* rendering, not on the dev server
being up yet.

## Test Scenarios

### 1. `from_canvas` + draw paints the triangle

**Objective:** `context::from_canvas` plus one minimal shader/buffer/draw sequence
paints a real frame — the narrowest possible proof this entry point works end to end.

**Steps:**
```bash
browsee .launch session::minwebgl_smoke url::http://127.0.0.1:8091/ features::webgpu window::800x600
browsee .wait for::render timeout::60 session::minwebgl_smoke
browsee .shot out::./-context_smoke.png session::minwebgl_smoke
```

**Expected Behavior:**
- `.wait for::render` exits 0 (`rendered::rgb ...`), not a timeout
- The screenshot shows a solid red triangle on a solid black background — see
  "Reading exact pixel values" below for a pixel-level assertion

### 2. Bounded draw — clear color outside the triangle

**Objective:** confirm the draw only painted the triangle, not the whole canvas
(guards against a fragment shader or missing `clear`/scissor that paints everywhere).

**Steps:** with the session from Scenario 1 still open, sample a point above the
triangle (see the chrome-corrected offset below — a naive top-left crop reads browser
chrome, not canvas content).

**Expected Behavior:** reads back the configured clear color (pure black), not the
triangle's red.

### Reading exact pixel values (`region::center` caveat)

`browsee .pixel region::center` measures the center of the **window**, not the
**canvas** — on a normal (non-kiosk) browser launch the window includes its own tab
bar and address bar above the page content, so `region::center` reads a color blended
with (or entirely inside) that chrome instead of the canvas's true content. Confirmed
on this build (Firefox via `browser::auto`, `window::800x600`): `region::center` read
`rgb 248 248 250` — nowhere near either the triangle's red or the clear's black,
because the window's vertical center for this page's chrome height lands inside the
chrome band itself, not the canvas. Not a rendering defect — confirmed by screenshot.

Two reliable options once you have a `.shot` PNG open in front of you:

1. **Visual/PNG inspection** (most robust — no offset math): open the screenshot and
   confirm by eye, or scan a vertical center-column scanline with Pillow:
   ```bash
   python3 -c "
   from PIL import Image
   im = Image.open('./-context_smoke.png').convert('RGB')
   w, h = im.size
   for y in range(0, h, 20): print(y, im.getpixel((w//2, y)))
   "
   ```
   Content below the chrome band (this build: chrome ends ~y=100) reads pure
   `(0, 0, 0)` (clear, y=100-180) then pure `(255, 0, 0)` (triangle, y=200-360) then
   pure `(0, 0, 0)` again (clear, y=380+) — a sharp, unblended transition.
2. **`browsee .pixel` with a chrome-corrected offset** — once you've confirmed the
   content boundary from a screenshot, target regions inside it directly. Verified
   working for `window::800x600` + `browser::auto` (Firefox) on this system:
   ```bash
   browsee .pixel region::40x40x306,120 session::minwebgl_smoke   # black clear band
   browsee .pixel region::40x40x306,260 session::minwebgl_smoke   # red triangle band
   ```
   Read exactly `rgb 0 0 0` / `rgb 255 0 0`. These offsets are a snapshot of this
   browser/theme's chrome height, not a portable constant — re-derive them from a
   fresh `.shot` if the browser, window size, or OS theme changes.

## Teardown

```bash
browsee .kill session::minwebgl_smoke purge::1
# then stop the trunk dev server (Ctrl-C, or kill the backgrounded PID)
```

## Test Matrix

| Scenario | webgl2 |
|----------|--------|
| Render completes (`.wait for::render` exits 0) | ✓ |
| Triangle reads configured color (`255, 0, 0`) | ✓ |
| Clear band reads configured clear color (`0, 0, 0`) | ✓ |

Native regression coverage and the wasm32 compile check are automated —
`cargo test -p minwebgl --all-features` and
`cargo check -p minwebgl --target wasm32-unknown-unknown` (see `minwebgl/readme.md`'s
Testing section) — only the browser-painted-pixel assertion above requires this
manual procedure.

## Reporting Issues

When reporting issues found during manual testing, please include:
- Browser name and version (`browsee .diagnose`)
- The exact `browsee`/`trunk` command sequence used
- The `.pixel`/`.wait` output and a `.shot` screenshot
- Whether the defect is in `from_canvas` itself or downstream (shader compile, buffer
  upload, draw call) — helps localize the fix to `context.rs` vs `shader.rs`/`buffer.rs`

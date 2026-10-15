# Manual Browser Verification — orrery_flexible

The `webgl` and `webgpu` features present the shared orrery scene live to a
browser `<canvas>` — unlike the `wgpu`/`vulkan` features, there is no
offscreen surface to read pixels back from inside a `cargo test` process
(see `native_render_test.rs` / `vulkan_render_test.rs` for those). Real
in-browser rendering is verified here with a scripted browser (`browsee`)
against a real `trunk`-served page, following the same pattern as
`module/helper/gpu_hal/tests/manual/readme.md`.

## Prerequisites

- `browsee` on `PATH` (scripted Firefox/Chromium automation over X11)
- `trunk` on `PATH`
- ImageMagick (`convert`/`identify`) for exact pixel sampling from a `.shot` PNG

## Test Application

`examples/orrery/flexible` itself, served by `trunk` with exactly one of the
two browser features enabled — `index.html` is the trunk shell shared by
both.

## Test Scenario — webgl

**Objective:** confirm the `webgl` feature renders the real orrery scene
(non-blank, correct landmark colors) in a browser canvas.

**Steps:**

```bash
cd examples/orrery/flexible
trunk serve --no-default-features --features webgl --port 8090
# in a second session:
browsee .launch session::orrery_flexible_webgl browser::firefox url::http://localhost:8090/ window::800x600
browsee .wait for::render timeout::25 session::orrery_flexible_webgl
browsee .shot session::orrery_flexible_webgl out::./-webgl_shot.png
identify ./-webgl_shot.png
convert ./-webgl_shot.png -format "%[pixel:p{326,251}]" info:   # sun-disc landmark
convert ./-webgl_shot.png -format "%[pixel:p{326,95}]" info:    # background landmark
browsee .kill session::orrery_flexible_webgl purge::1
```

**Expected Behavior:** the screenshot shows the orrery scene — a warm,
bright sun disc near the canvas center over a dark starfield/nebula
background — not a blank or solid-color canvas.

## Test Scenario — webgpu

**Objective:** confirm the `webgpu` feature renders the real orrery scene
in a browser canvas via the browser's native WebGPU implementation.

**Steps:**

```bash
cd examples/orrery/flexible
trunk serve --no-default-features --features webgpu --port 8092
# in a second session:
browsee .launch session::orrery_flexible_webgpu browser::firefox features::webgpu url::http://127.0.0.1:8092/ window::800x600
browsee .wait for::render timeout::25 session::orrery_flexible_webgpu
browsee .shot session::orrery_flexible_webgpu out::./-webgpu_shot.png
identify ./-webgpu_shot.png
convert ./-webgpu_shot.png -format "%[pixel:p{326,251}]" info:
convert ./-webgpu_shot.png -format "%[pixel:p{326,95}]" info:
browsee .kill session::orrery_flexible_webgpu purge::1
```

**Expected Behavior:** same as `webgl` — a real, non-blank rendered scene,
visually consistent with the `webgl` capture.

**`features::webgpu` is mandatory, not optional.** Firefox ships with
WebGPU disabled by default; the profile `browsee` creates only enables it
when the launch includes `features::webgpu` (`dom.webgpu.enabled` and
`gfx.webgpu.ignore-blocklist`, set in browsee's own `launch.rs`). Omitting
the flag doesn't error — it silently renders a solid black canvas that
looks exactly like a real WebGPU init failure, so a black screenshot here
means "check the launch flags" before "check the shader."

## Caveats Discovered This Session

- **`.pixel`'s coordinate space did not match the `.shot` screenshot's
  coordinate space** in this environment — sampling `region::center` or a
  `region::WxHxOFFX,OFFY` window through `.pixel` returned colors
  inconsistent with what the same location visibly shows in the
  screenshot. Do not trust `.pixel` readings on their own; take a `.shot`
  and sample it with `identify`/`convert` instead, as in the Steps above.
- **Screenshot dimensions don't equal the requested `window::` size.** A
  `window::800x600` launch produced a 652×502 `.shot` in this environment
  (compositor/DPI scaling, not a `browsee` bug). Always re-derive landmark
  coordinates from a fresh `identify` on the actual PNG rather than
  reusing pixel coordinates from a previous run or a different machine.
- **A `trunk serve` process can die silently mid-session** from a
  redundant hostname-alias bind race (`Address already in use (os error
  98)` on one of its several loopback aliases cascading into the whole
  process exiting) — Firefox then reports "Unable to connect" for every
  subsequent navigation, which looks identical to a browser/proxy/WebGPU
  profile problem. Before chasing the browser side, confirm the server is
  actually still alive: `ps aux | grep trunk` and `ss -ltnp | grep <port>`
  (a `curl` that succeeded moments earlier proves nothing — it can catch a
  narrow window before a delayed crash). If it's gone, relaunch on a fresh
  port and confirm the process is still present a few seconds later before
  pointing the browser at it again.

## Verified

**2026-08-16:** both features render the real scene, not a blank canvas:

| Feature | Sun-disc landmark | Background landmark |
|---------|--------------------|-----------------------|
| `webgl`  | srgb(200,130,20) | srgb(13,27,41) |
| `webgpu` | srgb(209,148,40) | srgb(14,28,43) |

Both are warm/orange at the sun-disc landmark and dark at the background
landmark, closely matching each other across backends. They aren't
byte-identical to `native_render_test.rs`/`vulkan_render_test.rs`'s
`[254,136,28]`/`[9,19,29]` — those offscreen tests render a single frame
frozen at `time = 0.0`, while the browser captures are a live-animating
scene sampled at an arbitrary moment, plus whatever color management the
browser's canvas presentation applies on top of the raw framebuffer. This
is a structural/visual consistency check (real scene, right landmarks,
cross-backend agreement), not a byte-exact match against the offscreen
tests.

## Teardown

```bash
browsee .list
browsee .kill session::orrery_flexible_webgl purge::1
browsee .kill session::orrery_flexible_webgpu purge::1
# stop both trunk serve processes (Ctrl-C, or kill the exact PID from `ps`)
```

## Test Matrix

| Feature | Method | Result |
|---------|--------|--------|
| `webgl` | `browsee` + ImageMagick pixel sampling | Pass — see Verified above |
| `webgpu` | `browsee` (`features::webgpu`) + ImageMagick pixel sampling | Pass — see Verified above |

## Reporting Issues

If a scenario fails, capture the `.shot` PNG, the exact `convert`/`identify`
output, and the trunk server log (`trunk serve ... 2>&1 | tee ./-NNNN_trunk.log`)
before filing a bug — screenshot plus exact sampled values plus server log
is what makes a rendering regression here reproducible.

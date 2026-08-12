# Sun Grid Lines (WebGPU)

**Keywords:** WebGPU, WGSL, Fragment Shader, Procedural Generation, Noise, Fullscreen Pass

A browser-WebGPU port of [`minwebgl_sun_grid_lines`](../../minwebgl/sun_grid_lines/readme.md)'s sci-fi HUD-style solar-system diagram, since expanded into a richer scene: a dark gradient background; three drifting nebula bands; two twinkling star-field layers; a grid overlay; a granulated, flickering, gently-pulsating corona-wrapped star; three glowing, pulsing orbit rings; six authored orbiting planets/moons; a handful of keyboard-driven procedural nodes; and a vignette/glow/scanline effects pass — all rendered by a single fullscreen fragment shader written in WGSL instead of GLSL. `scene.rhai` (see below) has diverged from the WebGL sibling's own `scene.rhai`, which still describes the original single-layer scene unchanged.

The vertex shader uses the same "big triangle" trick (three vertices, no buffer, positions derived from `vertex_index`) to cover the viewport in a single draw call. All visual structure is procedural, built from the same compact hash-based value-noise/fbm function, ported line-for-line from the GLSL version.

**No `showcase.webp` — headless verification only got as far as execution proof.** `cargo check`/`clippy --all-features -D warnings` pass clean, `trunk build` succeeds, and a diagnostic pass with console breadcrumbs confirmed the full pipeline runs end-to-end in headless Chromium (canvas → adapter → device → context configure → pipeline creation → first frame submitted) with no JS error or panic. But the screenshot itself came back blank: this sandbox's headless Chromium (software-rendered, `swiftshader`/`virtio-gpu`) fails to back the WebGPU swap-chain texture with a real shared image (`SharedImageBackingFactory` / `shared_image_stub` errors appear right after submit, in the GPU process, after every one of my own API calls already succeeded) — a browser/environment limitation, not a defect in this code. No pixel-level screenshot was produced, so no showcase image is included rather than fabricating one.

**No multi-pass bloom.** The WebGL2 version composites a real `UnrealBloomPass` (5-mip Gaussian blur) over an MRT emission target. No equivalent post-processing library exists for `minwebgpu` in this workspace, and building one from scratch was judged out of scope for a backend port. This example instead uses the same analytic radial-falloff / `exp()` glow terms the WebGL version used before its bloom pass was added — a single-pass approximation, not real bloom.

**Live parameterization.** The same three uniforms drive the shader and can be changed at runtime with the keyboard:

| Key | Effect |
| :--- | :--- |
| `↑` / `↓` | Increase / decrease the number of orbiting nodes (1–8) |
| `←` / `→` | Decrease / increase grid density |
| `Space` | Reshuffle the seed — reshuffles the star field and every node's orbital phase/radius |

**Scene file.** Everything not listed above as keyboard-live is data, not a shader constant: [`scene.rhai`](scene.rhai), a `scene_script`-based Rhai script evaluated by [`src/scene.rs`](src/scene.rs)'s `SceneConfig::load()` and written into the same uniform buffer the keyboard-driven parameters already share. It describes the whole scene: the background gradient; `nebula_bands` (three drifting fog layers, each its own height, hue, noise scale, and drift speed/direction); `star_layers` (two hashed star fields at different densities, sizes, and twinkle speeds); `grid` (color, opacity, line width, glow); `sun_corona` and `sun_disc` (three-stop radial colors and radii, plus a slow brightness flicker and a gentle breathing pulsation); `orbit_rings` (three concentric rails, each its own radius, glow, stroke width, and pulse speed); `nodes` (six authored planets/moons, each with its own orbit radius, angular speed and direction, phase, size, and color — independent of the keyboard-driven procedural nodes in `scene.wgsl`, which keep working unchanged); and `effects` (vignette strength/radius, a global glow-intensity multiplier, and a scanline texture). `nebula_bands`, `star_layers`, `orbit_rings`, and `nodes` are lists so the diagram can carry more than one instance of a kind — `scene.rs` asserts each list's length against a matching `NEBULA_BAND_COUNT`/`STAR_LAYER_COUNT`/`ORBIT_RING_COUNT`/`NODE_COUNT` constant at load time, since `scene.wgsl`'s corresponding `array<vec4f, N>` uniform fields must be a compile-time fixed size.

`load()` runs the script through `scene_script::build_engine()` and extracts the returned value into `SceneConfig` via `rhai`'s serde bridge (`rhai::serde::from_dynamic`) — the same crate and convention documented in [`scene_script`](../../../module/helper/scene_script/readme.md) and demonstrated in [`examples/scene_script/`](../f32x2_vector_arithmetic/readme.md). `scene.rhai` sticks to the declarative half of that convention: top-level `let` bindings and a single trailing map-literal expression, no `fn main()` and no host callbacks. That holds even though the scene now has real dynamics and animation (nebula drift, star twinkle, corona flicker, disc pulsation, ring pulse, orbiting nodes): the script itself still runs exactly once, at native load time via `include_str!`, not once per frame, so every dynamic is expressed as data — a speed, a phase, an amplitude — rather than imperative logic. `scene.wgsl`'s `fs_main()` is what actually animates these, evaluating each one against the live `time` uniform every frame. Edit `scene.rhai` and rebuild to restyle or re-time the diagram without touching `main.rs` or `scene.wgsl`. Two things it deliberately leaves alone: the script is bundled via `include_str!` at compile time like the shader itself, so it is not hot-reloadable — a rebuild is required, same as editing the WGSL; and the noise/hash internals, anti-aliasing epsilons, and orbiting-node jitter formula stay shader constants, since they're generation internals rather than author-facing content.

**[How to run](../../how_to_run.md)**

**References:**

* [WebGPU Fundamentals]
* [WGSL Specification]
* [inigo quilez — fbm]

[WebGPU Fundamentals]: https://webgpufundamentals.org/
[WGSL Specification]: https://www.w3.org/TR/WGSL/
[inigo quilez — fbm]: https://iquilezles.org/articles/fbm/

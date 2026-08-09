# Sun Grid Lines (WebGPU)

**Keywords:** WebGPU, WGSL, Fragment Shader, Procedural Generation, Noise, Fullscreen Pass

A browser-WebGPU port of [`minwebgl_sun_grid_lines`](../../minwebgl/sun_grid_lines/readme.md): the same sci-fi HUD-style solar-system diagram — dark gradient background, noise-modulated nebula band, twinkling star field, grid overlay, granulated corona-wrapped star, glowing orbit ring, and one or more orbiting nodes — rendered by a single fullscreen fragment shader written in WGSL instead of GLSL.

The vertex shader uses the same "big triangle" trick (three vertices, no buffer, positions derived from `vertex_index`) to cover the viewport in a single draw call. All visual structure is procedural, built from the same compact hash-based value-noise/fbm function, ported line-for-line from the GLSL version.

**No `showcase.webp` — headless verification only got as far as execution proof.** `cargo check`/`clippy --all-features -D warnings` pass clean, `trunk build` succeeds, and a diagnostic pass with console breadcrumbs confirmed the full pipeline runs end-to-end in headless Chromium (canvas → adapter → device → context configure → pipeline creation → first frame submitted) with no JS error or panic. But the screenshot itself came back blank: this sandbox's headless Chromium (software-rendered, `swiftshader`/`virtio-gpu`) fails to back the WebGPU swap-chain texture with a real shared image (`SharedImageBackingFactory` / `shared_image_stub` errors appear right after submit, in the GPU process, after every one of my own API calls already succeeded) — a browser/environment limitation, not a defect in this code. No pixel-level screenshot was produced, so no showcase image is included rather than fabricating one.

**No multi-pass bloom.** The WebGL2 version composites a real `UnrealBloomPass` (5-mip Gaussian blur) over an MRT emission target. No equivalent post-processing library exists for `minwebgpu` in this workspace, and building one from scratch was judged out of scope for a backend port. This example instead uses the same analytic radial-falloff / `exp()` glow terms the WebGL version used before its bloom pass was added — a single-pass approximation, not real bloom.

**Live parameterization.** The same three uniforms drive the shader and can be changed at runtime with the keyboard:

| Key | Effect |
| :--- | :--- |
| `↑` / `↓` | Increase / decrease the number of orbiting nodes (1–8) |
| `←` / `→` | Decrease / increase grid density |
| `Space` | Reshuffle the seed — reshuffles the star field and every node's orbital phase/radius |

**[How to run](../how_to_run.md)**

**References:**

* [WebGPU Fundamentals]
* [WGSL Specification]
* [inigo quilez — fbm]

[WebGPU Fundamentals]: https://webgpufundamentals.org/
[WGSL Specification]: https://www.w3.org/TR/WGSL/
[inigo quilez — fbm]: https://iquilezles.org/articles/fbm/

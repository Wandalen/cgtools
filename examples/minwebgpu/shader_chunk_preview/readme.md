# Shader Chunk Preview (WebGPU)

**Keywords:** WebGPU, WGSL, Shader Chunks, Tunable Parameters, Live UI Controls, Fullscreen Pass

This example answers the question of what command opens a window with one shader chunk rendered, tunable by UI: `action/run shader_chunk_preview` (see [How to run](../../how_to_run.md)) opens a browser window rendering a single composed chunk set, with a slider panel wired live to its `//@ param:`-declared uniforms — drag a slider, the shader redraws that frame with the new value, no rebuild.

This example renders a domain-warped `fbm3` noise field: `shader_chunks_core`'s `hash21`/`value_noise`/`fbm3` stack is sampled three times per pixel (twice to build a 2D warp offset, once more at the warped point) via the fullscreen-triangle vertex trick, same composition pattern as [`orrery/webgpu`](../../orrery/webgpu/readme.md). The three `//@ param:` uniforms this shader declares — `noise_scale`, `warp_strength`, `brightness` — all visibly change the picture; none is decorative.

**No `showcase.webp`.** Same environment limitation documented in [`orrery/webgpu`'s readme](../../orrery/webgpu/readme.md): this sandbox's headless Chromium cannot back a WebGPU swap-chain texture with a real shared image, so no pixel-level screenshot is available here either.

## The chunk being previewed

[`shader/preview_fragment.wgsl`](shader/preview_fragment.wgsl) is this example's own local chunk — not one of `shader_chunks_core`'s bundled `shader/*.wgsl` rows. Annotating a *bundled* chunk with `//@ param:` lines was ruled out of scope by decision Q-03 (see [`shader_chunks_params`'s readme](../../../module/shader/shader_chunks_params/readme.md)), so tunables live here instead, exactly mirroring how `orrery/webgpu`'s `scene_fragment.wgsl` is a local, non-reusable fragment chunk. [`src/shader_source.rs`](src/shader_source.rs) selects `hash21`, `value_noise`, `fbm3`, `fullscreen_triangle` from `shader_chunks_core` by name at compile time, composes them with the local `preview_fragment` chunk via `set_compose()`, and asserts the set's dependency-closure at compile time.

To preview a *different* chunk set, edit `PREVIEW_CHUNKS` in `src/shader_source.rs` and the local chunk's manifest/body in `shader/preview_fragment.wgsl` — the UI panel and uniform wiring in `src/main.rs` follow whatever `//@ param:` lines the local chunk declares (see [`shader_chunks` CLI's `tunables` command](../../../module/shader/shader_chunks/docs/cli/command/06_tunables.md) to inspect a chunk's declared parameters from the terminal first, before wiring them into `main.rs` by hand — this example does not read `chunk_discover`'s output at runtime, since a browser-side UI's sliders need to exist before the shader that reads their values compiles).

## Files

| File | Responsibility |
| :--- | :--- |
| `Cargo.toml` | Declare crate metadata and dependencies |
| `readme.md` | Document the example for users |
| `index.html` | Host the canvas and control panel DOM |
| `style.css` | Style the slider control panel |
| `controls.js` | Render sliders and forward their values to Rust |
| `verb/run` | Launch the example via the browser dev server |
| `src/main.rs` | Wire the WebGPU render loop to live slider uniforms |
| `src/lib.rs` | Expose `shader_source` for native testing |
| `src/shader_source.rs` | Assemble the previewed chunk set into WGSL |
| `src/uniforms.rs` | Define the GPU-side uniform buffer layout |
| `src/controls.rs` | Bind the slider panel's JS functions into Rust |
| `shader/preview_fragment.wgsl` | Declare the local tunable fragment chunk |
| `tests/shader_source_test.rs` | Verify chunk composition and parameter parity |

## Controls

| Slider | Uniform | Range | Effect |
| :--- | :--- | :--- | :--- |
| Noise scale | `noise_scale` | 0.5 – 20.0 | Spatial frequency of the noise field |
| Warp strength | `warp_strength` | 0.0 – 2.0 | How far the domain warp displaces the sample point |
| Brightness | `brightness` | 0.0 – 3.0 | Output color multiplier |

Every slider's label/property/range in [`src/main.rs`](src/main.rs) is this crate's UI-side copy of `preview_fragment.wgsl`'s own `//@ param:` declarations — kept honest by `tests/shader_source_test.rs`'s `discovered_tunable_parameters_match_params_uniform_fields` test, which runs `shader_chunks_params::chunk_discover` against the local chunk and asserts every discovered parameter names a real `Params` struct field.

**[How to run](../../how_to_run.md)**

**References:**

* [WebGPU Fundamentals]
* [WGSL Specification]
* [inigo quilez — fbm]

[WebGPU Fundamentals]: https://webgpufundamentals.org/
[WGSL Specification]: https://www.w3.org/TR/WGSL/
[inigo quilez — fbm]: https://iquilezles.org/articles/fbm/

# Sun Grid Lines (wgpu)

**Keywords:** wgpu, Rust, WGSL, Fragment Shader, Procedural Generation, Noise, Offscreen Rendering

A native-wgpu port of [`minwebgl_sun_grid_lines`](../../minwebgl/sun_grid_lines/readme.md): the same sci-fi HUD-style solar-system diagram — dark gradient background, noise-modulated nebula band, twinkling star field, grid overlay, granulated corona-wrapped star, glowing orbit ring, and one or more orbiting nodes — rendered by a single fullscreen fragment shader. It reuses the identical WGSL shader (`shaders/scene.wgsl`) written for the [browser WebGPU port](../../scene_script/sun_grid_lines/readme.md), byte-for-byte — WGSL is portable across both.

There is no windowing anywhere in this workspace (no `winit`, no `wgpu::Surface`), so this example follows the same pattern as [`hello_triangle`](../hello_triangle/readme.md) and [`grid_render`](../grid_render/readme.md): render once into an offscreen texture, copy it into a `MAP_READ` buffer, and save the result as `-sun_grid_lines.png`. No live loop, no animation, no keyboard interactivity — the `time`/`seed`/`node_count`/`grid_density` uniforms that the browser versions drive live from an animation loop and the keyboard are instead baked into a single fixed uniform buffer at buffer-creation time (`node_count = 4`, to show off the orbiting-node parameterization in the one frame this example produces).

**No multi-pass bloom**, for the same reason as the WebGPU port: no post-processing library exists for `minwgpu` in this workspace. Glow uses the shader's analytic radial-falloff / `exp()` terms.

![image](showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [wgpu Documentation]
* [WGSL Specification]
* [inigo quilez — fbm]

[wgpu Documentation]: https://docs.rs/wgpu/latest/wgpu/
[WGSL Specification]: https://www.w3.org/TR/WGSL/
[inigo quilez — fbm]: https://iquilezles.org/articles/fbm/

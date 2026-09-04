# renderer Opaque Path (Browser)

**Keywords:** renderer, gpu_hal, WebGPU, WebGL2, HAL, Pixel Verification, Tutorial, Basics

Renders the same lit-quad scene as `renderer/tests/native_render_test.rs`'s
`opaque_path_renders_lit_quad` — one `Geometry` quad, one red `PbrMaterial`, one
directional light, one `Frame` — through `renderer::webgpu`'s canonical opaque path
(`GpuContext`, `WebGpuRenderer`), proving both browser backends paint real pixels. The
native test proves the same render path through an offscreen `wgpu` readback instead of
a browser canvas; this crate reuses its exact vertex data, material, light, and camera.

One crate, two backends, one feature each — build/run one at a time:

```bash
trunk serve --release                                         # webgpu (default feature)
trunk serve --release --no-default-features --features webgl  # webgl
```

**[How to run](../../how_to_run.md)**

**References:**

* [WebGPU Specification]
* [WebGL2 Specification]

[WebGPU Specification]: https://www.w3.org/TR/webgpu/
[WebGL2 Specification]: https://registry.khronos.org/webgl/specs/latest/2.0/

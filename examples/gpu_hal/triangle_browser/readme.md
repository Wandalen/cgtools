# gpu_hal Triangle (Browser)

**Keywords:** gpu_hal, WebGPU, WebGL2, HAL, Pixel Verification, Tutorial, Basics

Draws one uniform-colored triangle through `gpu_hal`'s public surface — `Device::new_webgpu`
or `Device::new_webgl`, one shader module, one pipeline, one render pass — proving both
browser backends paint real pixels. Mirrors `gpu_hal/tests/native_backend_test.rs`'s
`triangle_render_readback`, which proves the same render path on the native backend through
an offscreen readback instead of a browser canvas; this crate reuses that test's WGSL shader
and vertex/uniform data, adding only the GLSL ES override the WebGL backend requires
(`Device::shader_module_create`).

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

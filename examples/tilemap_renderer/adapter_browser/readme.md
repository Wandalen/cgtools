# tilemap_renderer Adapter Browser Pixel Verification

**Keywords:** tilemap_renderer, WebGPU, WebGL2, Backend, Pixel Verification, Tutorial, Basics

Drives `tilemap_renderer`'s `adapter-webgpu` / `adapter-webgl` `Backend` impls
through a real browser canvas — one `Clear` plus one centered `Sprite`,
mirroring `tilemap_renderer/tests/native_backend_test.rs`'s exact 8x8
solid-red sprite asset. That test proves the same construct → assets_load →
submit → output flow on the native backend through an offscreen GPU
readback; this crate proves it through a real canvas instead.

The two backends are **not** expected to paint the same pixel:
`adapter-webgl` uploads real pixel bytes and paints the sprite's configured
solid red; `adapter-webgpu` has no texture-upload path wired yet (see its own
module doc comment) and paints an opaque **black** quad instead — this crate
proves each backend's own honest, distinct current behavior rather than a
uniform claim neither could back up.

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

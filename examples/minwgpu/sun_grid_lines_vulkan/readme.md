# Sun Grid Lines (wgpu / Vulkan backend)

**Keywords:** wgpu, Vulkan, Rust, WGSL, Fragment Shader, Procedural Generation, Noise, Offscreen Rendering

The same [native-wgpu `sun_grid_lines`](../sun_grid_lines/readme.md) example, with the backend pinned explicitly to Vulkan instead of letting `wgpu` pick whatever `PRIMARY` backend the host prefers. Identical source otherwise: same WGSL shader (`shaders/scene.wgsl`, byte-for-byte copy), same single-shot offscreen-texture-to-PNG pattern, same fixed uniforms (`node_count = 4`).

**No raw Vulkan crate.** This workspace has no direct Vulkan bindings anywhere — `ash` (the Vulkan bindings `wgpu-hal` uses internally) appears only as a transitive dependency pulled in by `wgpu-hal` itself (confirmed via `cargo tree -i ash`), never as a direct dependency of any crate here. "Vulkan" in this example means `wgpu` with its `wgpu::Backends::VULKAN` flag forced in the `InstanceDescriptor`, not a hand-written Vulkan renderer:

```rust
let instance = wgpu::Instance::new( &wgpu::InstanceDescriptor
{
  backends : wgpu::Backends::VULKAN,
  ..Default::default()
} );
```

Every other line of `src/main.rs` is unchanged from the `PRIMARY`-backend version — same adapter/device request, same texture/buffer/pipeline setup, same render-and-readback sequence.

**No multi-pass bloom**, for the same reason as the other ports in this family: no post-processing library exists for `minwgpu` in this workspace. Glow uses the shader's analytic radial-falloff / `exp()` terms.

![image](showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [wgpu Documentation]
* [Vulkan backend (wgpu-hal)]
* [WGSL Specification]
* [inigo quilez — fbm]

[wgpu Documentation]: https://docs.rs/wgpu/latest/wgpu/
[Vulkan backend (wgpu-hal)]: https://docs.rs/wgpu-hal/latest/wgpu_hal/vulkan/index.html
[WGSL Specification]: https://www.w3.org/TR/WGSL/
[inigo quilez — fbm]: https://iquilezles.org/articles/fbm/

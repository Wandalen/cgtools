# 🔬 WebGPU Examples with `minwebgpu`

Examples targeting the browser's WebGPU API through the `minwebgpu` driver crate — and, for the canonical renderer path, through the `gpu_hal` abstraction on top of it.

## 🚀 How to Run

All examples are built and served with [trunk](https://trunkrs.dev). Trunk resolves the crate from its working directory, so run it from the example's own directory:

```bash
rustup target add wasm32-unknown-unknown   # once
cd <example>
trunk serve --release
```

Then open `http://127.0.0.1:8080` (pass `--port <n>` if 8080 is taken) in a WebGPU-capable browser. On Linux, Chromium may need WebGPU enabled explicitly:

```bash
chromium --enable-unsafe-webgpu --enable-features=Vulkan http://127.0.0.1:8080
```

## 📂 Examples

| Example | Responsibility |
|---------|----------------|
| `deffered_rendering/` | Deferred shading: G-buffer pass, lighting pass, compute-updated lights |
| `hello_triangle/` | Minimal WebGPU pipeline drawing one shape to the canvas |
| `hello_triangle_quickstart/` | Same triangle via minwebgpu's `context::setup`/`render_pass::draw_to` helpers |
| `renderer_pbr_scene/` | Canonical `gpu_hal` PBR scene via `renderer::webgpu` — WebGPU with WebGL2 fallback |
| `sun_grid_lines/` | Procedural sci-fi HUD diagram: animated star, orbit ring, Cartesian grid — styled via `scene.rhai` |

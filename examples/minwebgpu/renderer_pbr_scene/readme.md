# 🧪 renderer_pbr_scene

**Keywords:** PBR, gpu_hal, Renderer, WebGPU, WebGL2, HDR, Tone Mapping

The runtime demo of the canonical `gpu_hal` opaque path ( `renderer::webgpu` ): a metallic-roughness sphere grid over a ground plane — gold metallic row and red dielectric row, roughness rising left to right — lit by one directional, one point and one spot light, rendered through the HDR opaque pass + ACES tone mapping, with a slowly orbiting camera.

The scene is written once against the `gpu_hal` API and runs on **both backends**:

- **WebGPU** — picked automatically when the browser exposes `navigator.gpu`.
- **WebGL2** — fallback, or forced with `?webgl` in the URL.

The page title shows which backend is active. The projection matrix is chosen per backend from `Device::depth_range()` ( 0..1 vs -1..1 ) — the one per-backend divergence the HAL deliberately surfaces.

## 🚀 Run

```bash
cd examples/minwebgpu/renderer_pbr_scene
trunk serve --release
```

Open `http://127.0.0.1:8080` ( add `--port <n>` if taken ). To compare backends side by side, open a second tab at `http://127.0.0.1:8080/?webgl`.

On Linux, Chromium may need WebGPU enabled explicitly:

```bash
chromium --enable-unsafe-webgpu --enable-features=Vulkan http://127.0.0.1:8080
```

Verify which backend rendered: the browser tab title reads `renderer PBR scene — WebGPU` or `— WebGL2`.

## 📐 Scope

Only the canonical opaque slice is exercised — direct lighting, no IBL, no shadows, no skinning, no loaders ( those remain with `renderer::webgl` until strangled onto the HAL ). Canvas size is fixed at startup; there is no resize handling in the v0 renderer.

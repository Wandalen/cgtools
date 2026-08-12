# orrery

One scene, many implementations. An **orrery** — a sun with a granulated, corona-wrapped disc; concentric orbit rings; orbiting planets/moons; a drifting nebula; twinkling star fields; a sci-fi HUD grid — implemented once per rendering backend/layer, so the same visual contract can be compared across the workspace's stack. The scene data lives in a `scene_script` Rhai document; each implementation renders that same scene with its own driver and shading language.

| Directory | Responsibility |
|-----------|----------------|
| `webgpu/` | Reference implementation: browser WebGPU via `minwebgpu`, WGSL fullscreen shader |

## Family conventions

- **Package naming:** `orrery_<implementation>` — dir short (`webgpu/`), package prefixed (`orrery_webgpu`), same convention the API-group dirs use.
- **Tags:** every member declares `scene:orrery` in `[package.metadata.action]`, alongside its `runtime:`/`api:` tags. `action/run list scene:orrery` lists the whole family.
- **API-group symlinks:** each member is also reachable from its API group dir via a symlink (`examples/minwebgpu/orrery -> ../orrery/webgpu`), so browsing by API still finds it. Symlinked paths are `exclude`d in the root `Cargo.toml` — the member globs must resolve each package exactly once.
- **Scene contract:** the scene definition currently lives in `webgpu/scene/scene.rhai`. When a second member lands, it is promoted to this directory so every implementation consumes the identical document — that, not lookalike output, is what makes the family "the same scene". Counts (`NEBULA_BAND_COUNT`, `STAR_LAYER_COUNT`, `ORBIT_RING_COUNT`, `NODE_COUNT`) and field semantics are pinned by `webgpu/src/scene.rs` and its tests.
- **Planned members** (not yet built): `webgl/` (browser WebGL2 via `minwebgl`, with real multi-pass bloom), `wgpu/` (native via `minwgpu`; forcing a specific backend such as Vulkan is a run mode of this member, not a separate crate), `gpu_hal/` (one body of code targeting all backends through the L1 hardware abstraction layer — see `docs/adr/002_gpu_hal_in_house.md`).

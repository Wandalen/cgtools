# Orrery (Flexible)

**Keywords:** Orrery, gpu_hal, WebGL2, WebGPU, wgpu, Vulkan, Multi-backend

One crate, four selectable backends — `webgl`, `webgpu`, `wgpu`, `vulkan` —
chosen at compile time via Cargo features, all rendering the same orrery
scene (see the [family readme](../readme.md) for the shared scene
contract). Governed by an explicit product principle: **only the `wgpu`
feature may link the `wgpu` crate; the other three do not, even
transitively.** See
[ADR-004](../../../docs/adr/004_native_vulkan_hal_backend.md) for the full
rationale, including why `vulkan` needed a new, dedicated `minvulkan`
driver instead of reusing `wgpu` forced onto its own Vulkan backend.

## Backend selection

| Feature | Runtime | Routes through |
|---------|---------|-----------------|
| `webgl` | Browser (wasm32) | `gpu_hal/webgl` → `minwebgl` |
| `webgpu` | Browser (wasm32) | `gpu_hal/webgpu` → `minwebgpu` |
| `wgpu` (default) | Native | `gpu_hal/native` → `minwgpu` + `wgpu` |
| `vulkan` | Native | `gpu_hal/vulkan` → `minvulkan` ( `ash`, no `wgpu` ) |

Exactly one backend feature must be selected — the crate fails to compile
otherwise, with a `compile_error!` naming the problem (zero selected, or
more than one). The guard is duplicated in both `src/lib.rs` and
`src/main.rs`: `cargo check`/`build` compiles the `lib` target first, so
`lib.rs` needs its own copy to produce this message instead of a confusing
`E0432` from its unconditional `gpu_hal` imports.

**Status:** implemented — all 4 features load the shared `scene.rhai` orrery
scene and render it (`webgl`/`webgpu` present live to a browser canvas;
`wgpu`/`vulkan` render one offscreen frame and save it as a PNG, since
`gpu_hal`'s native/Vulkan backends have no windowing support). Task 203
tracks final verification and lifecycle closure.

## Verify

```bash
cargo nextest run -p orrery_flexible --features wgpu
cargo nextest run -p orrery_flexible --no-default-features --features vulkan
```

`wgpu` is the crate's default feature (see `[features]` in `Cargo.toml`) — the
second command needs `--no-default-features`, otherwise `wgpu` stays enabled
alongside `vulkan` and the build fails with the "more than one backend
feature" `compile_error!` described above.

Both draw the shared scene through `scene_render` and assert on pixels read
back from the offscreen surface (sun-disc-center and background-corner
landmarks).

The `webgpu` and `webgl` backends have no offscreen readback — they present
to a browser canvas instead — so they're verified with a real browser via
`browsee`:

```bash
trunk serve --no-default-features --features webgl --port 8090   # or --features webgpu
browsee .launch session::orrery_flexible url::http://127.0.0.1:8090/ features::webgpu window::800x600
browsee .wait for::render timeout::25 session::orrery_flexible
```

Full command sequence and exact pixel readings: `tests/manual/readme.md`.

## Directory Layout

| Path | Responsibility |
|------|-----------------|
| `src/lib.rs` | Shared render path (`scene_render`) reused by every backend |
| `src/uniforms.rs` | Backend-agnostic uniform buffer layout, packed from `SceneConfig` |
| `src/main.rs` | Per-feature entry points — browser live loop / native offscreen render |
| `build.rs` | Translates the shared WGSL to GLSL ES 300 for the `webgl` feature |
| `index.html` | trunk shell for the `webgl`/`webgpu` browser features |
| `tests/` | Native pixel-asserting tests, plus `manual/` for browser-side pixel verification |
| `readme.md` | This file — user-facing entry point |
| `Cargo.toml` | Package manifest — 4-way backend feature wiring |

**[How to run](../../how_to_run.md)**

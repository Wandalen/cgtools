# agnostic 2d render engine - development roadmap

- **Project:** Agnostic 2D Render Engine
- **Version:** 0.2.0
- **Status:** Active Development

## current state

The core library is functional and the WebGL2 adapter is partially implemented. The engine uses a flat command stream architecture with POD commands and a `Backend` trait.

### completed

- **Core types** — `Transform`, `ResourceId<T>`, `RenderConfig` (incl. configurable `max_depth`), blend modes, topology, coordinate system (Y-up)
- **Command system** — all POD commands: Clear, Path (moveto/lineto/quad/cubic/arc/close), Text, Mesh, Sprite, Batch lifecycle, Groups with effects
- **Asset system** — images (bitmap/encoded/path), sprites, geometries, gradients, patterns, clip masks, paths, validation
- **Backend trait** — `load_assets`, `submit`, `output`, `resize`, `capabilities`
- **Test suite** — 39 tests covering types, commands, assets validation, and backend trait
- **WebGL2 adapter (partial)** — hardware-accelerated sprites, meshes, and instanced batches on wasm32:
  - Split across `adapters/webgl.rs` (backend + renderers + async image loader) and
    `adapters/webgl/webgl_helpers.rs` (self-contained helpers wired via `mod_interface::layer`)
    to stay under the per-file size budget
  - `ArrayBuffer<T>` — GPU-side Vec with 2× grow via `copy_buffer_sub_data` (no CPU readback);
    `swap_remove` uses a persistent scratch buffer to avoid binding the same buffer to both
    `COPY_READ_BUFFER` and `COPY_WRITE_BUFFER` (WebGL2 spec violation)
  - `SpriteInstanceData` (72B) and `MeshInstanceData` (40B) with compile-time layout assertions
  - Single-draw: `Clear`, `Mesh` (with texture + topology), `Sprite` (with tint)
  - Batch lifecycle: `Create`, `Bind`, `Add/Set/Remove` instances, `Draw`, `Delete` — for both sprite and mesh batches
  - Per-batch VAO setup at create/unbind time; bind-only at draw time
  - Asset loading: images (Bitmap sync + Path async via `spawn_local`), sprites, geometries (sync + async path); async handlers use `Closure::once_into_js` so the browser drops the Rust closures (and captured `Rc<RefCell<GpuResources>>`) after `onload` / `onerror` fires, letting `WebGlBackend` drop actually free GPU resources
  - `Transform::depth` — honored via depth buffer (`DEPTH_TEST`, `LEQUAL`). Per-field range `[-RenderConfig::max_depth, max_depth]` (default `1.0`); shader divides by `u_max_depth`, GPU clips out-of-range values. Batch sum `parent_depth + instance_depth` is subject to the same range. Reliable for fully opaque draws (translucent must be back-to-front)
  - Blend modes: Normal, Add, Multiply, Screen (hardware-accelerated); Overlay falls back to Normal. `Capabilities::supported_blend_modes` advertises the correct set; `blend_modes: bool` means "all variants correct" and is `false` until Overlay is implemented
  - Shaders: `sprite.vert/frag`, `sprite_batch.vert/frag`, `mesh.vert/frag`, `mesh_batch.vert`

### deferred to follow-up PRs

- **SVG adapter** — stub only (`adapter-svg` feature gate exists; implementation pending)
- **Terminal adapter** — stub only (`adapter-terminal` feature gate exists; implementation pending)

### project structure

```
tilemap_renderer/           # Single crate with feature-gated adapters
├── src/
│   ├── lib.rs              # Module exports, feature gates
│   ├── types.rs            # Core types (Transform, ResourceId, enums)
│   ├── commands.rs         # All render commands (POD, Copy)
│   ├── assets.rs           # Asset definitions and validation
│   ├── backend.rs          # Backend trait, Output, Capabilities
│   └── adapters/
│       ├── mod.rs          # Feature-gated re-exports
│       ├── svg.rs          # SVG 1.1 backend
│       ├── webgl.rs        # WebGL2 backend entry point (WebGlBackend + renderers)
│       ├── webgl/          # WebGL submodule layer
│       │   └── webgl_helpers.rs  # ArrayBuffer, GPU handles, GL mappers, batch types
│       ├── terminal.rs     # Terminal backend
│       └── shaders/        # GLSL shaders for WebGL
├── Cargo.toml
├── readme.md
├── spec.md
└── roadmap.md
```

## remaining work

### webgl adapter gaps

- Path rendering (tessellation or GPU-based curves) — path commands are currently silent no-ops
- Text rendering (glyph atlas or SDF fonts) — text commands are currently silent no-ops
- Group commands (`BeginGroup`/`EndGroup`) — currently ignored
- `ImageSource::Encoded` decoding — skipped with TODO
- Gradient/pattern/clip-mask asset loading — not loaded into GPU resources
- Effects (blur, drop shadow — requires FBO post-processing)
- `BlendMode::Overlay` — Photoshop-style (Multiply where dst<0.5, Screen where dst>0.5) cannot be expressed as a single `blend_func` call; currently falls back to Normal; requires a custom shader or separate FBO read-back pass
- WebGL context loss handling (`webglcontextlost` / `webglcontextrestored` events)

### svg adapter gaps

- Font loading and rendering (currently no font resolution)
- Image flip for Y-up (SVG images are Y-down natively)

### terminal adapter gaps

- Sprite/mesh/batch support
- Effect support
- Gradient approximation

### infrastructure

- Visual regression testing (reference image comparison)
- wasm-pack test runner for WebGL tests
- CI with feature matrix testing
- Performance benchmarks (target: 10,000 commands < 16ms)

### future backends

- WebGPU via `minwebgpu` (compute shaders, advanced instancing)
- Interactive SVG with JavaScript events

## design decisions

| Decision | Rationale |
|----------|-----------|
| Flat command stream, no Scene object | Simpler, zero-allocation, lets users manage their own scene graph |
| All commands are POD (Copy) | Cache-friendly, no allocations, trivial to serialize |
| Y-up coordinate system | Standard in graphics (OpenGL, math), consistent across backends |
| Feature-gated adapters in one crate | Simpler dependency management vs separate crates |
| `Backend` trait (not `Renderer`/`PrimitiveRenderer`) | Single trait with `submit(&[RenderCommand])` is simpler than per-command dispatch |
| Instanced batches in WebGL | Essential for tilemap performance (thousands of sprites) |
| SVG uses `<g>` for batch parent transform | Natural SVG composition, avoids double Y-flip on instances |

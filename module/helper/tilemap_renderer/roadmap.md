# agnostic 2d render engine - development roadmap

- **Project:** Agnostic 2D Render Engine
- **Version:** 0.2.0
- **Status:** Active Development

## current state

The core library and SVG adapter are functional. The engine uses a flat command stream architecture with POD commands and a `Backend` trait.

### completed

- **Core types** — `Transform`, `ResourceId<T>`, `RenderConfig`, blend modes, topology, coordinate system (Y-up)
- **Command system** — all POD commands: Clear, Path (moveto/lineto/quad/cubic/arc/close), Text, Mesh, Sprite, Batch lifecycle, Groups with effects
- **Asset system** — images (bitmap/encoded/path), sprites, geometries, gradients, patterns, clip masks, paths, validation
- **Backend trait** — `load_assets`, `submit`, `output`, `resize`, `capabilities`
- **SVG adapter** — full implementation: paths, text, sprites, meshes, batches, groups, effects, gradients, patterns, blend modes, bitmap PNG encoding, viewport pan/zoom wrapper
- **Test suite** — 69 tests covering types, commands, assets, backend trait, and SVG adapter

### deferred to follow-up PRs

- **WebGL2 adapter** — stub only (`adapter-webgl` feature gate exists; implementation pending)
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
│       ├── webgl.rs        # WebGL2 backend (wasm32)
│       ├── terminal.rs     # Terminal backend
│       └── shaders/        # GLSL shaders for WebGL
├── Cargo.toml
├── readme.md
├── spec.md
└── roadmap.md
```

## remaining work

### webgl adapter gaps

- Path rendering (tessellation or GPU-based curves)
- Text rendering (glyph atlas or SDF fonts)
- Gradient/pattern fill support
- Effects (blur, drop shadow — requires FBO post-processing)
- WebGL context loss handling

### svg adapter gaps

- Font loading and rendering (currently no font resolution)
- `Source::Path` geometries silently skipped — no file loader; callers must supply `Source::Bytes` (future: `std::fs` on native, `fetch()` on wasm32)
- Image Y-flip: SVG `<image>` elements are Y-down natively; sprites rendered from them may appear flipped

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
| SVG viewport in top-level `<g transform>` | Single `replace_range` updates pan/zoom without re-submitting commands |
| Mesh `<symbol>` defs generated lazily | Only topologies actually used appear in `<defs>`, keeping output lean |
| Bitmap images encoded to PNG via `image` crate | Browsers require real PNG format inside `data:image/png` URIs, not raw bytes |

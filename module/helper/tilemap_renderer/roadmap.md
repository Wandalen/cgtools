# agnostic 2d render engine - development roadmap

- **Project:** Agnostic 2D Render Engine
- **Version:** 0.2.0
- **Status:** Active Development

## current state

The core library and SVG adapter are functional; the WebGL2 adapter is partially implemented. The engine uses a flat command stream architecture with POD commands and a `Backend` trait.

### completed

- **Core types** — `Transform`, `ResourceId<T>`, `RenderConfig` (incl. configurable `max_depth`), blend modes, topology, coordinate system (Y-up)
- **Command system** — all POD commands: Clear, Path (moveto/lineto/quad/cubic/arc/close), Text, Mesh, Sprite, Batch lifecycle, Groups with effects
- **Asset system** — images (bitmap/encoded/path), sprites, geometries, gradients, patterns, clip masks, paths, validation
- **Backend trait** — `assets_load`, `submit`, `output`, `resize`, `capabilities`
- **SVG adapter** — implemented across every command and asset family: paths, text, sprites, meshes, batches, groups, effects, gradients, patterns, blend modes, bitmap PNG encoding, viewport pan/zoom wrapper, `Source::Path` geometry loading via blocking `std::fs` (loud skip with stderr warning + diagnostic comment on read failure, incl. on wasm32 where no filesystem exists). Not complete, though — see "svg adapter gaps" below (font selection unimplemented, no `Transform::depth` ordering)
- **WebGL2 adapter (partial)** — hardware-accelerated sprites, meshes, and instanced batches on wasm32:
  - Split across `adapters/webgl.rs` (backend + renderers + async image loader) and
    `adapters/webgl/webgl_helpers.rs` (self-contained helpers wired via `mod_interface::layer`)
    to stay under the per-file size budget
  - `ArrayBuffer<T>` — GPU-side Vec with 2× grow via `copy_buffer_sub_data` (no CPU readback);
    `swap_remove` uses a persistent scratch buffer to avoid binding the same buffer to both
    `COPY_READ_BUFFER` and `COPY_WRITE_BUFFER` (WebGL2 spec violation)
  - `SpriteInstanceData` (72B, includes per-instance tint) and `MeshInstanceData` (56B, includes per-instance tint) with compile-time layout assertions
  - Single-draw: `Clear`, `Mesh` (with texture + topology), `Sprite` (with tint)
  - Batch lifecycle: `Create`, `Bind`, `Add/Set/Remove` instances, `Draw`, `Delete` — for both sprite and mesh batches
  - Per-batch VAO setup at create/unbind time; bind-only at draw time
  - Asset loading: images (Bitmap sync + Path async via `spawn_local`), sprites, geometries (sync + async path); async handlers use `Closure::once_into_js` so the browser drops the Rust closures (and captured `Rc<RefCell<GpuResources>>`) after `onload` / `onerror` fires, letting `WebGlBackend` drop actually free GPU resources
  - `Transform::depth` — honored via depth buffer (`DEPTH_TEST`, `LEQUAL`). Per-field range `[-RenderConfig::max_depth, max_depth]` (default `1.0`); shader divides by `u_max_depth`, GPU clips out-of-range values. Batch sum `parent_depth + instance_depth` is subject to the same range. Reliable for fully opaque draws (translucent must be back-to-front)
  - Blend modes: Normal, Add, Multiply, Screen (hardware-accelerated); Overlay falls back to Normal. `Capabilities::supported_blend_modes` advertises the correct set; `blend_modes: bool` means "all variants correct" and is `false` until Overlay is implemented
  - Shaders: `sprite.vert/frag`, `sprite_batch.vert/frag`, `mesh.vert/frag`, `mesh_batch.vert/frag`
- **WebGPU / native / no-op adapters** — `adapter-webgpu`, `adapter-native`, `adapter-none` via
  `gpu_hal`, adopted in `docs/adr/003_d2_stack_hal_adoption.md` (supersedes the earlier
  direct-`minwebgpu` idea); `task/completed/084_tilemap_renderer_adapter_none_backend.md`,
  `task/completed/086_tilemap_renderer_adapter_webgpu_backend.md`,
  `task/completed/087_tilemap_renderer_adapter_native_backend.md`
- **Terminal adapter** — coarse ANSI-truecolor character-cell rendering, no external dependencies:
  - World-space commands downsample onto a fixed cell grid (`CELL_PX_WIDTH`×`CELL_PX_HEIGHT` =
    16×32 world units per cell, chosen to approximate a monospace glyph's ~1:2 aspect ratio);
    `resize` reallocates the grid
  - Single-draw: `Clear`, `Mesh` (`FillRef::Solid` only), `Sprite`/`ScreenSpaceSprite` (tint) — each
    paints one cell at its transform's resolved position
  - Paths: `BeginPath`..`EndPath` rasterized as connected straight segments between accumulated
    points via true Bresenham line rasterization; curves (`QuadTo`/`CubicTo`/`ArcTo`) flatten into
    16 straight segments before rasterization (`CURVE_SEGMENTS`), including true SVG-style
    endpoint-to-center arc parameterization for `ArcTo`
  - Text: the native medium here — `BeginText`/`Char`/`EndText` places real glyphs with horizontal
    anchor support (left/center/right) and vertical anchor support (top/center/bottom)
  - Batches: full lifecycle (`Create`/`Bind`/`Add`/`Set`/`Remove`/`Draw`/`Delete`, both sprite and
    mesh) composing instance transforms through the batch's own parent transform
  - Groups: `BeginGroup`/`EndGroup` transform stack composed via `Transform::to_mat3`; clip masks
    and visual effects are accepted but not honored
  - `output()` encodes the grid as 24-bit ANSI SGR truecolor escape sequences, one line per row
  - Blending: source-over (Porter-Duff "over") alpha compositing on straight RGBA via
    `composite_over` — the only evaluated `BlendMode` (`capabilities().supported_blend_modes`
    is `&[BlendMode::Normal]`; other variants fall back to it); no gradients/patterns (`Mesh`
    only draws a `FillRef::Solid` fill)
- **Test suite** — covers types, commands, assets, backend trait, and SVG adapter

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
│       ├── terminal.rs     # Terminal backend (ANSI-truecolor character-cell grid)
│       └── shaders/        # GLSL shaders for WebGL
├── Cargo.toml
├── readme.md
├── roadmap.md
└── docs/                   # Design documentation (typed doc definitions)
```

## remaining work

### webgl adapter gaps

- Path rendering (tessellation or GPU-based curves) — path commands are currently silent no-ops
- Text rendering (glyph atlas or SDF fonts) — text commands are currently silent no-ops
- Group commands (`BeginGroup`/`EndGroup`) — currently ignored
- `ImageSource::Encoded` decoding — skipped with a console warning; decision made (`task/decisions.md` Q-02) to decode via browser-native Blob/object-URL mechanisms rather than a bundled Rust decoder, tracked as `task/completed/092_tilemap_renderer_webgl_encoded_image_decode.md`
- Gradient/pattern/clip-mask asset loading — not loaded into GPU resources
- Effects (blur, drop shadow — requires FBO post-processing)
- `BlendMode::Overlay` — Photoshop-style (Multiply where dst<0.5, Screen where dst>0.5) cannot be expressed as a single `blend_func` call; currently falls back to Normal; requires a custom shader or separate FBO read-back pass. The Multiply/Screen approximations likewise diverge from the reference formulas when `src_alpha < 1` (see the `BlendMode::Multiply` doc); the same FBO / custom-shader pass would make them exact

### svg adapter gaps

- Font loading and rendering (currently no font resolution)
- `Source::Path` geometry loading is blocking `std::fs` only — works natively; on wasm32 the read fails at runtime and the geometry is skipped loudly (stderr warning + diagnostic SVG comment). An async `fetch()` path would need a redesign of the sync `assets_load` contract
- `Transform::depth` ordering — the adapter emits in submission order and ignores `depth`; callers must pre-sort (future: stable sort by `depth` before emission)
- Interactive SVG with JavaScript events

### terminal adapter gaps

- Gradient/pattern fills — `Mesh` only resolves `FillRef::Solid`; `FillRef::Gradient`/
  `FillRef::Pattern` are accepted but paint nothing
- Group clip masks and visual effects (blur, drop shadow, color matrix, opacity) — accepted but
  ignored; only a `BeginGroup`'s `transform` is honored

### infrastructure

- Visual regression testing (reference image comparison)
- wasm-pack test runner for WebGL tests
- CI with feature matrix testing
- Performance benchmarks (target: 10,000 commands < 16ms)

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
| Bitmap images encoded to PNG via `png` crate | Browsers require real PNG format inside `data:image/png` URIs, not raw bytes; `png` is a minimal single-format dependency vs the full multi-format `image` crate (Q-02) |

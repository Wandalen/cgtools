# spec

- **Name:** Agnostic 2D Render Engine
- **Version:** 0.2.0
- **Date:** 2026-04-22

### table of contents

1. [Project Goal](#1-project-goal)
2. [Problem Solved](#2-problem-solved)
3. [Ubiquitous Language](#3-ubiquitous-language)
4. [Architecture](#4-architecture)
5. [Coordinate System](#5-coordinate-system)
6. [Core Modules](#6-core-modules)
7. [Backend Adapters](#7-backend-adapters)
8. [Functional Requirements](#8-functional-requirements)
9. [Non-Functional Requirements](#9-non-functional-requirements)
10. [Conformance Checklist](#10-conformance-checklist)

---

### 1. Project Goal

High-performance, backend-agnostic 2D rendering engine for Rust. Decouples rendering command definition from backend implementation — define once, render anywhere.

### 2. Problem Solved

Developers are often locked into a single rendering backend. This engine provides a clean abstraction: a flat stream of POD commands that any backend can process. One command stream can produce SVG files, WebGL frames, or terminal output.

### 3. Ubiquitous Language

| Term | Definition |
| :--- | :--- |
| **Backend** | A concrete implementation of the `Backend` trait for a specific rendering technology. |
| **Command** | A lightweight POD struct describing a single rendering operation. All commands are `Copy`. |
| **Command Stream** | An ordered `&[RenderCommand]` slice processed sequentially by a backend. |
| **Assets** | Resources (images, geometries, gradients, etc.) loaded once before rendering. |
| **Batch** | A collection of instances (sprite or mesh) drawn in a single call. |
| **Transform** | Position, rotation, scale, skew, depth — applied to any drawable. |
| **ResourceId\<T\>** | Type-safe handle referencing a loaded asset. Phantom-typed for compile-time safety. |

### 4. Architecture

**Ports & Adapters** (hexagonal):

```
              ┌─────────────────────────┐
              │       Core Library       │
              │  types, commands, assets │
              │    backend (trait)       │
              └────────┬────────────────┘
                       │ Backend trait
          ┌────────────┼────────────────┐
          │            │                │
    ┌─────▼──┐   ┌────▼────┐   ┌──────▼───┐
    │  SVG   │   │  WebGL2 │   │ Terminal  │
    │adapter │   │ adapter │   │  adapter  │
    └────────┘   └─────────┘   └──────────┘
```

- **Core** is platform-independent, zero graphics dependencies
- **Adapters** are feature-gated (`adapter-svg`, `adapter-webgl`, `adapter-terminal`)
- Single crate, not separate adapter crates

### 5. Coordinate System

All backends use **Y-up**:

- `(0, 0)` = bottom-left corner
- Positive Y = up
- Positive rotation = counter-clockwise (CCW)
- SVG adapter converts internally (Y-flip, rotation negation, scale-Y negation)
- WebGL adapter uses standard OpenGL convention natively

### 6. Core Modules

#### 6.1. types (`types.rs`)

- `Transform` — position `[f32; 2]`, rotation `f32`, scale `[f32; 2]`, skew `[f32; 2]`, depth `f32`
- `Transform::to_mat3()` — column-major 3×3 affine matrix
- `ResourceId<T>` — phantom-typed u32 handle, nohash for IntMap/IntSet
- `RenderConfig` — width, height, antialias, background, max_depth
- Enums: `BlendMode`, `LineCap`, `LineJoin`, `TextAnchor`, `Topology`, `SamplerFilter`, `Antialias`
- `FillRef` — None, Solid, Gradient, Pattern
- `DashStyle` — stroke dash pattern with offset

#### 6.2. commands (`commands.rs`)

All structs are `Copy + Clone + Debug`. No allocations.

| Command | Description |
|---------|-------------|
| `Clear` | Fill framebuffer with solid color |
| `BeginPath`..`EndPath` | Streaming path: MoveTo, LineTo, QuadTo, CubicTo, ArcTo, ClosePath |
| `BeginText`..`EndText` | Streaming text: Char-by-char with font, size, color, anchor |
| `Mesh` | Single geometry draw with fill/texture/topology |
| `Sprite` | Single sprite draw with tint |
| `CreateSpriteBatch` | Create instanced sprite batch |
| `CreateMeshBatch` | Create instanced mesh batch |
| `BindBatch`..`UnbindBatch` | Batch editing: Add/Set/Remove instances, update params |
| `DrawBatch` | Draw all instances in a batch |
| `DeleteBatch` | Free batch resources |
| `BeginGroup`..`EndGroup` | Nested group with transform, clip, effect |
| `Effect` | Blur, DropShadow, ColorMatrix, Opacity |

#### 6.3. assets (`assets.rs`)

- `Assets` container with validation (`validate()` checks for duplicate ResourceIds)
- `ImageAsset` — Bitmap, Encoded, or Path source
- `SpriteAsset` — sub-region of a sprite sheet
- `GeometryAsset` — positions, UVs, indices with data type
- `GradientAsset` — linear/radial with color stops
- `PatternAsset` — tiling pattern from image
- `ClipMaskAsset` — path-based clip mask
- `PathAsset` — named path for text-on-path

#### 6.4. backend (`backend.rs`)

```rust
pub trait Backend {
    fn load_assets(&mut self, assets: &Assets) -> Result<(), RenderError>;
    fn submit(&mut self, commands: &[RenderCommand]) -> Result<(), RenderError>;
    fn output(&self) -> Result<Output, RenderError>;
    fn resize(&mut self, width: u32, height: u32);
    fn capabilities(&self) -> Capabilities;
}
```

### 7. Backend Adapters

#### 7.1. SVG (`adapter-svg`)

- Generates SVG 1.1 documents with `<defs>` and body sections via `SvgContentManager`
- `SvgContentManager` tracks byte offsets for `<defs>`, body elements, and the viewport `<g>` transform so inserts/clears are O(1) splice operations on a contiguous `String`
- Viewport pan/zoom factored into a single top-level `<g transform="scale(s) translate(ox,-oy)">` wrapper — `set_viewport_offset`/`set_viewport_scale` update it in-place via `replace_range` without re-submitting commands
- Y-up → SVG Y-down conversion per element: position Y flipped by `height - y`, rotation negated, scale Y negated
- Effects via SVG `<filter>` elements (feGaussianBlur, feDropShadow, feColorMatrix, opacity)
- Sprite tint via feColorMatrix filter
- Mesh `<symbol>` defs generated lazily on first use of a (geometry, topology) pair
- Mesh texture approximated via `<pattern>` fill
- Batch drawing: parent transform in `<g>` wrapper, raw local transforms for instances
- Bitmap images encoded to PNG via the `image` crate and inlined as `data:image/png;base64` URIs; PNG dimensions extracted from IHDR for correct sprite sheet sizing
- `ImageSource::Encoded` — MIME type auto-detected from magic bytes (PNG, JPEG, GIF, WebP, SVG; fallback PNG); dimensions extracted via `image::ImageReader::with_guessed_format` for any format the `image` crate recognizes
- `ImageSource::Path` — dimensions unknown at load-assets time (no file I/O is performed). Sprites referencing a Path-sourced sheet are **skipped** with a stderr warning and a diagnostic HTML comment in the SVG output. Use `ImageSource::Bitmap` or `Encoded` when sprites require sheet dimensions
- `Source::Path` geometries are silently skipped — no file loader; callers must supply `Source::Bytes`
- XML-special characters in Char-stream text content are escaped (`&`, `<`, `>`, `"`, `'` → named entities) to prevent SVG injection / XSS via `<script>` or inline event handlers
- `ImageSource::Path` href values are percent-encoded (RFC 3986): every byte outside the unreserved set and `/` becomes `%XX`, and Windows backslashes are normalized to forward slashes. This yields a valid URI reference for browsers and simultaneously neutralizes attribute-injection payloads (`"`, `<`, `>`, `&` are percent-encoded rather than entity-escaped)
- Arc rotation values are emitted in **degrees** (per SVG 1.1 A-path spec); `ArcTo::rotation` is stored in radians and converted at emission time
- Colors are emitted as SVG 1.1 `rgb(r,g,b)` with a separate `fill-opacity` / `stroke-opacity` / `stop-opacity` / `flood-opacity` attribute for alpha; the CSS Color Level 4 `rgba()` notation is **not** used (Inkscape / strict SVG 1.1 parsers may reject it). Fully opaque colors (alpha = 1.0) omit the opacity attribute entirely
- `TriangleStrip` mesh emission alternates vertex order on odd-indexed triangles (OpenGL/D3D convention) so the SVG polygon sequence preserves consistent CCW winding

#### 7.2. WebGL2 (`adapter-webgl`)

- Hardware-accelerated via `minwebgl` crate (wasm32 target)
- Split across two files to stay under the per-file size budget:
  - `adapters/webgl.rs` — `WebGlBackend`, `SpriteRenderer`, `MeshRenderer`, async image loader
  - `adapters/webgl/webgl_helpers.rs` (submodule via `mod_interface::layer`) — self-contained helpers:
    `ArrayBuffer<T>`, `SpriteInstanceData` / `MeshInstanceData`, `GpuResources` / `GpuTexture` /
    `GpuSprite` / `GpuGeometry` / `GpuBatch`, VAO setup, async `Loadable`, GL mapping helpers
    (`index_format`, `apply_texture_filter`, `apply_blend`, `topology_to_gl`)
- `ArrayBuffer<T>` — GPU-side Vec with dynamic grow (copy_buffer_sub_data); uses a persistent
  scratch buffer to avoid the WebGL2 spec violation of binding the same buffer to both
  `COPY_READ_BUFFER` and `COPY_WRITE_BUFFER` in `swap_remove`
- Instanced rendering: `SpriteInstanceData` (72B), `MeshInstanceData` (56B) — each carries
  per-instance `depth` and `tint` (compile-time layout assertions enforce the sizes)
- Depth buffer enabled (`DEPTH_TEST`, `LEQUAL`); `Transform::depth` honored for fully opaque
  draws. Range `[-RenderConfig::max_depth, max_depth]` per field (default `1.0`); the shader
  divides by `u_max_depth` so out-of-range depths are clipped by the GPU rather than silently
  saturated
- Per-batch VAO with attrib setup at create/unbind time, just bind at draw time
- Async image loading uses `Closure::once_into_js` for one-shot `onload` / `onerror` handlers,
  so the browser drops the Rust closures (and their captured `Rc<RefCell<GpuResources>>`)
  after the event — a `WebGlBackend` drop actually releases its GPU resources
- Shaders: sprite.vert/frag, sprite_batch.vert/frag, mesh.vert/frag, mesh_batch.vert/frag
- Quad vertices generated in vertex shader via `gl_VertexID`

#### 7.3. Terminal (`adapter-terminal`)

- ASCII/Unicode rendering with Bresenham line drawing
- ANSI color support
- Configurable output dimensions

### 8. Functional Requirements

#### FR-A: Command System

- **FR-A1:** ✅ `RenderCommand` enum wraps all command types
- **FR-A2:** ✅ All command structs are POD (`Copy`, `Clone`)
- **FR-A3:** ✅ Commands form a flat `&[RenderCommand]` stream

#### FR-B: Rendering Primitives

- **FR-B1:** ✅ Path commands (line, quad, cubic, arc) with full styling
- **FR-B2:** ✅ Text commands with font, anchoring, text-on-path
- **FR-B3:** ✅ Mesh with geometry, topology, fill, texture
- **FR-B4:** ✅ Sprite with tint and blend mode
- **FR-B5:** ✅ Instanced batches (sprite and mesh) with CRUD lifecycle
- **FR-B6:** ✅ Groups with transform, clip mask, effects

#### FR-C: Backend Interface

- **FR-C1:** ✅ `Backend` trait with load_assets/submit/output/resize/capabilities
- **FR-C2:** ✅ `Capabilities` struct for runtime feature discovery (coarse `bool` flags plus `supported_blend_modes: &'static [BlendMode]` for per-variant queries)
- **FR-C3:** ✅ `RenderError` for graceful error handling

#### FR-D: SVG Backend

- **FR-D1:** ✅ Generates valid SVG 1.1 documents
- **FR-D2:** 🟡 Supports all rendering primitives (path, text, sprite, mesh, batch, group). **Known gap:** font assets (`Assets.fonts`) are ignored — `load_assets` does not emit `@font-face`/`<font-face>` definitions and `<text>` elements carry no `font-family`, so all text renders in the SVG viewer's default font. Text *rendering* works; font *selection* is not implemented.
- **FR-D3:** ✅ Converts RGBA colors to SVG `rgb()` format
- **FR-D4:** ✅ Supports all stroke styles (caps, joins, dash)
- **FR-D5:** ✅ Supports all text anchoring modes
- **FR-D6:** ✅ Effects: blur, drop shadow, color matrix, opacity
- **FR-D7:** ✅ Sprite tint via feColorMatrix
- **FR-D8:** ✅ Mesh texture via pattern fill
- **FR-D9:** ✅ Batch drawing with correct transform composition

#### FR-E: WebGL Backend

- **FR-E1:** ✅ Uses `minwebgl` crate
- **FR-E2:** ✅ Hardware-accelerated WASM rendering (sprites, meshes, batches)
- **FR-E3:** ✅ Instanced batching for sprites and meshes
- **FR-E4:** ✅ Per-batch VAO management (setup at create/unbind, bind-only at draw)
- **FR-E5:** ❌ Path rendering (tessellation/GPU curves)
- **FR-E6:** ❌ Text rendering (glyph atlas / SDF fonts)
- **FR-E7:** ❌ WebGL context loss handling
- **FR-E8:** ❌ Effects (blur, shadow — requires FBO post-processing)
- **FR-E9:** ⚠️ Blend modes — Normal/Add/Multiply/Screen hardware-accelerated; `Overlay` falls back to Normal (requires custom shader or FBO read-back). `Capabilities::blend_modes` is `false` (not all variants correct); `Capabilities::supported_blend_modes` advertises the precise set
- **FR-E10:** ⚠️ `Transform::depth` — honored via depth buffer (`DEPTH_TEST`, `LEQUAL`, higher = on top). Per-field range is `[-RenderConfig::max_depth, max_depth]` (default `1.0`); the shader divides by `max_depth` and the GPU clips values outside the range. For batches the **sum** `parent_depth + instance_depth` must satisfy the same constraint. Reliable only for fully opaque draws; translucent content must still be submitted back-to-front

#### FR-F: Terminal Backend

- **FR-F1:** ⏳ ASCII/Unicode line rendering (deferred to adapter-terminal PR)
- **FR-F2:** ⏳ Configurable dimensions (deferred to adapter-terminal PR)
- **FR-F3:** ⏳ ANSI color support (deferred to adapter-terminal PR)
- **FR-F4:** ❌ Sprite/mesh/batch support

### 9. Non-Functional Requirements

- **NFR-1:** Performance: 10,000 commands < 16ms (not yet benchmarked)
- **NFR-2:** ✅ Zero graphics dependencies in core (only `nohash-hasher`, `error_tools`, `mod_interface`; `base64`, `bytemuck`, `image` are optional behind `adapter-svg` feature)
- **NFR-3:** ✅ Feature-gated adapters for minimal builds
- **NFR-4:** ✅ Y-up coordinate system consistent across all backends
- **NFR-5:** ✅ 100% documentation coverage (zero warnings)
- **NFR-6:** ✅ All command types are POD (Copy, Clone)
- **NFR-7:** ✅ Test suite: core + SVG adapter covered; WebGL/terminal adapter tests deferred
- **NFR-8:** ✅ Compile-time layout assertions for GPU data structures (`SpriteInstanceData` 72B, `MeshInstanceData` 56B)
- **NFR-9:** ❌ Visual regression testing
- **NFR-10:** ❌ CI with feature matrix
- **NFR-11:** ✅ SVG output is injection-safe for text and attribute contexts: all caller-controlled strings flowing into text PCDATA or XML attributes (Char stream, `ImageSource::Path`) are entity-escaped. **Scope limitation:** this guarantee does **not** cover the contents of embedded SVG images supplied via `ImageSource::Encoded` with SVG bytes — those are base64-embedded as-is inside a `data:image/svg+xml` `<image>`, and browsers may execute scripts/event handlers inside them in some rendering contexts. Callers supplying SVG image bytes are responsible for trusting or sanitizing their source.

### 10. Conformance Checklist

| Status | ID | Summary |
| :--- | :--- | :--- |
| ✅ | FR-A1 | RenderCommand enum |
| ✅ | FR-A2 | All commands are POD |
| ✅ | FR-A3 | Flat command stream |
| ✅ | FR-B1 | Path commands |
| ✅ | FR-B2 | Text commands |
| ✅ | FR-B3 | Mesh rendering |
| ✅ | FR-B4 | Sprite rendering |
| ✅ | FR-B5 | Instanced batches |
| ✅ | FR-B6 | Groups with effects |
| ✅ | FR-C1 | Backend trait |
| ✅ | FR-C2 | Capabilities |
| ✅ | FR-C3 | RenderError |
| 🟡 | FR-D1–D9 | SVG backend — implemented; font assets ignored (see FR-D2 note) |
| ⚠️ | FR-E1–E4 | WebGL backend partial (sprites, meshes, batches work; paths, text, effects missing) |
| ❌ | FR-E5–E8 | WebGL: paths, text, context loss, effects — not implemented |
| ⚠️ | FR-E9 | WebGL blend modes partial — Overlay falls back to Normal; `supported_blend_modes` lists the correct set |
| ⚠️ | FR-E10 | WebGL depth honored for opaque draws only (range `[-max_depth, max_depth]`, out-of-range values clipped by the GPU; translucent must be back-to-front) |
| ⏳ | FR-F1–F3 | Terminal backend — deferred to adapter-terminal PR |
| ✅ | NFR-2 | Zero core graphics deps |
| ✅ | NFR-4 | Y-up coordinate system |
| ✅ | NFR-5 | 100% doc coverage |
| ✅ | NFR-7 | Test suite |
| ✅ | NFR-8 | Compile-time layout assertions for GPU structs (SpriteInstanceData 72B, MeshInstanceData 56B) |
| ✅ | NFR-11 | SVG injection-safe output (text + attributes only; embedded SVG image bytes are caller-trusted) |
| ❌ | NFR-1 | Performance benchmarks |
| ❌ | NFR-9 | Visual regression tests |
| ❌ | NFR-10 | CI feature matrix |

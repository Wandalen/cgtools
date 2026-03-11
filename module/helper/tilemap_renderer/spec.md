# spec

- **Name:** Agnostic 2D Render Engine
- **Version:** 0.2
- **Date:** 2026-03-11

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
- `RenderConfig` — width, height, antialias, background
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

- Generates SVG 1.1 documents with `<defs>` and body sections
- `SvgContentManager` for efficient string buffer management
- Y-up → SVG Y-down conversion in `transform_to_svg_static`
- Effects via SVG `<filter>` elements (feGaussianBlur, feDropShadow, feColorMatrix)
- Sprite tint via feColorMatrix filter
- Mesh texture approximated via `<pattern>` fill
- Batch drawing with `<g>` wrapper for parent transform, raw local transforms for instances

#### 7.2. WebGL2 (`adapter-webgl`)

- Hardware-accelerated via `minwebgl` crate (wasm32 target)
- `ArrayBuffer<T>` — GPU-side Vec with dynamic grow (copy_buffer_sub_data)
- Instanced rendering: `SpriteInstanceData` (68B), `MeshInstanceData` (36B)
- Per-batch VAO with attrib setup at create/unbind time, just bind at draw time
- Shaders: sprite.vert/frag, sprite_batch.vert, mesh.vert/frag, mesh_batch.vert
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
- **FR-C2:** ✅ `Capabilities` struct for runtime feature discovery
- **FR-C3:** ✅ `RenderError` for graceful error handling

#### FR-D: SVG Backend

- **FR-D1:** ✅ Generates valid SVG 1.1 documents
- **FR-D2:** ✅ Supports all rendering primitives
- **FR-D3:** ✅ Converts RGBA colors to SVG format
- **FR-D4:** ✅ Supports all stroke styles (caps, joins, dash)
- **FR-D5:** ✅ Supports all text anchoring modes
- **FR-D6:** ✅ Effects: blur, drop shadow, color matrix, opacity
- **FR-D7:** ✅ Sprite tint via feColorMatrix
- **FR-D8:** ✅ Mesh texture via pattern fill
- **FR-D9:** ✅ Batch drawing with correct transform composition

#### FR-E: WebGL Backend

- **FR-E1:** ✅ Uses `minwebgl` crate
- **FR-E2:** ✅ Hardware-accelerated WASM rendering
- **FR-E3:** ✅ Instanced batching for sprites and meshes
- **FR-E4:** ✅ Per-batch VAO management
- **FR-E5:** ❌ Path rendering (tessellation/GPU curves)
- **FR-E6:** ❌ Text rendering
- **FR-E7:** ❌ WebGL context loss handling
- **FR-E8:** ❌ Effects (blur, shadow — requires FBO)

#### FR-F: Terminal Backend

- **FR-F1:** ✅ ASCII/Unicode line rendering
- **FR-F2:** ✅ Configurable dimensions
- **FR-F3:** ✅ ANSI color support
- **FR-F4:** ❌ Sprite/mesh/batch support

### 9. Non-Functional Requirements

- **NFR-1:** Performance: 10,000 commands < 16ms (not yet benchmarked)
- **NFR-2:** ✅ Zero graphics dependencies in core (only `nohash-hasher`, `base64`)
- **NFR-3:** ✅ Feature-gated adapters for minimal builds
- **NFR-4:** ✅ Y-up coordinate system consistent across all backends
- **NFR-5:** ✅ 100% documentation coverage (zero warnings)
- **NFR-6:** ✅ All command types are POD (Copy, Clone)
- **NFR-7:** ✅ Test suite: 51 tests (types, commands, assets, SVG backend)
- **NFR-8:** ✅ Compile-time layout assertions for GPU data structures
- **NFR-9:** ❌ Visual regression testing
- **NFR-10:** ❌ CI with feature matrix

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
| ✅ | FR-D1–D9 | SVG backend complete |
| ⚠️ | FR-E1–E4 | WebGL backend partial (sprites, meshes, batches work; paths, text, effects missing) |
| ⚠️ | FR-F1–F3 | Terminal backend partial (lines, colors work; sprites, meshes missing) |
| ✅ | NFR-2 | Zero core graphics deps |
| ✅ | NFR-4 | Y-up coordinate system |
| ✅ | NFR-5 | 100% doc coverage |
| ✅ | NFR-7 | Test suite |
| ❌ | NFR-1 | Performance benchmarks |
| ❌ | NFR-9 | Visual regression tests |
| ❌ | NFR-10 | CI feature matrix |

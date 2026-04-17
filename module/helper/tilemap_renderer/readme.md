# tilemap_renderer

Backend-agnostic 2D rendering engine with adapter support.

Define rendering commands once, render to any backend — SVG, WebGL2, or terminal.

## coordinate system

All backends use a **Y-up** convention:

- `(0, 0)` is the bottom-left corner
- Positive Y points up
- Positive rotation is counter-clockwise (CCW)

## architecture

The crate follows **Ports & Adapters** (hexagonal) architecture:

- **Core** (`types`, `commands`, `assets`, `backend`) — platform-independent, no graphics dependencies
- **Adapters** (`adapters::SvgBackend`, `adapters::WebGlBackend`, `adapters::TerminalBackend`) — feature-gated backend implementations

All rendering commands are **POD** (`Copy`, `Clone`) — no allocations, no lifetimes. Commands form a flat sequential stream processed by backends.

```text
tilemap_renderer/
├── types.rs        # Transform, ResourceId, RenderConfig, enums
├── commands.rs     # Clear, Path, Text, Mesh, Sprite, Batch, Group
├── assets.rs       # Images, sprites, geometries, gradients, patterns, clip masks
├── backend.rs      # Backend trait, Output, Capabilities, RenderError
└── adapters/
    ├── svg.rs      # SVG 1.1 document generation
    ├── webgl.rs    # WebGL2 hardware-accelerated rendering (wasm32)
    └── terminal.rs # ASCII/Unicode terminal output
```

## features

| Feature | Status | Description |
|---------|--------|-------------|
| `adapter-svg` | stub | SVG backend — generates SVG 1.1 documents |
| `adapter-webgl` | partial | WebGL2 backend — sprites, meshes, instanced batches (wasm32); paths/text/effects pending |
| `adapter-terminal` | stub | Terminal backend — ASCII art output |

Default: no features enabled (core only, zero backend dependencies).

## usage

```toml
[dependencies]
tilemap_renderer = { version = "0.2", features = ["adapter-svg"] }
```

```rust,ignore
use tilemap_renderer::{ commands::*, types::*, assets::*, backend::* };
use tilemap_renderer::adapters::SvgBackend;

let config = RenderConfig { width : 800, height : 600, ..Default::default() };
let mut svg = SvgBackend::new( config );
svg.load_assets( &assets )?;
svg.submit( &[
  RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } ),
  // ... path, sprite, mesh, batch commands ...
])?;
let Output::String( doc ) = svg.output()? else { unreachable!() };
```

## rendering primitives

- **Paths** — moveto, lineto, quadratic/cubic bezier, arc, close (with fill, stroke, dash, blend)
- **Text** — styled text with anchoring, optional text-on-path
- **Sprites** — sub-regions of sprite sheets with tint
- **Meshes** — indexed geometry with topology (triangle list/strip, line list/strip)
- **Batches** — instanced sprite/mesh batches for high-performance rendering
- **Groups** — nested transforms with clip masks and effects (blur, drop shadow, color matrix, opacity)
- **Gradients & Patterns** — linear/radial gradients, tiling patterns as fills

## backend capabilities

| Feature | SVG | WebGL | Terminal |
|---------|-----|-------|----------|
| Paths | yes | — | yes |
| Text | yes | — | yes |
| Sprites | yes | yes | — |
| Meshes | yes | yes | — |
| Batches | yes | yes | — |
| Gradients | yes | — | — |
| Effects | yes | — | — |
| Blend modes | yes | partial¹ | — |

> **SVG** and **Terminal** adapters are currently stub implementations (deferred to follow-up PRs).
> **WebGL** adapter is partially implemented: sprites, meshes, and instanced batches work;
> paths, text, groups, gradients, patterns, and effects are not yet rendered.
>
> ¹ WebGL blend modes: Normal, Add, Multiply, Screen are hardware-accelerated.
> `BlendMode::Overlay` (Photoshop-style) cannot be expressed as a single `blend_func` call
> and currently falls back to Normal; a custom shader or FBO pass is required.

## license

Licensed under MIT license.

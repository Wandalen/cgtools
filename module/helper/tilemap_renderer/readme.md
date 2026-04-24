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
    ├── webgl/
    │   └── webgl_helpers.rs  # Self-contained WebGL types (ArrayBuffer, GPU handles, GL mappers)
    └── terminal.rs # ASCII/Unicode terminal output
```

## features

| Feature | Status | Description |
|---------|--------|-------------|
| `adapter-svg` | complete | SVG backend — generates SVG 1.1 documents |
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
| Sprites | yes | stub | — |
| Meshes | yes | stub | — |
| Batches | yes | stub | — |
| Gradients | yes | — | — |
| Effects | yes | — | — |
| Blend modes | yes | partial¹ | — |
| Viewport pan/zoom | yes | partial | — |

> **Terminal** adapter is currently a stub implementation (deferred to a follow-up PR).
> **WebGL** adapter is partially implemented: sprites, meshes, and instanced batches work;
> paths, text, groups, gradients, patterns, and effects are not yet rendered.
>
> ¹ WebGL blend modes: Normal, Add, Multiply, Screen are hardware-accelerated.
> `BlendMode::Overlay` (Photoshop-style) cannot be expressed as a single `blend_func` call
> and currently falls back to Normal; a custom shader or FBO pass is required. Because
> not all variants render correctly, `Capabilities::blend_modes` is `false` on this
> backend; query `Capabilities::supported_blend_modes: &'static [BlendMode]` for the
> precise set (`[Normal, Add, Multiply, Screen]`).
>
> **Depth (WebGL):** `Transform::depth` is honored via the depth buffer (`LEQUAL`, higher
> values drawn on top). Valid range is `[-RenderConfig::max_depth, RenderConfig::max_depth]`
> (default `1.0`, backwards-compatible); the shader divides by `max_depth` and lets the
> GPU clip values outside the range. In batches the **sum** `parent_depth + instance_depth`
> must stay within the range — out-of-range sums are clipped. Correct only for fully
> opaque draws — submit translucent content back-to-front as you would for a
> painter's-algorithm renderer. SVG and terminal adapters still emit in submission order
> and ignore `depth` / `max_depth`.

## known issues / TODO

### `HexConfig::grid_stride` — consider a bounding-box helper

The field is now named `grid_stride` and documented as the pixel spacing
between centres of adjacent cells (see `scene_model::pipeline::HexConfig`).
For equilateral hex sprites this coincides with the sprite's bounding
box; for stylised sprites (e.g. the Slay atlas used by
`examples/minwebgl/slay_map/`) the two diverge and callers must tune
`grid_stride` empirically.

Possible ergonomics follow-up: add a helper `HexConfig::from_hex_size( w, h )`
that computes the equilateral-hex stride for authors who *do* have a
bounding box (`(w * 0.75, h)` for flat-top, `(w, h * 0.75)` for pointy-top).
Not urgent — the rename alone removes the worst confusion.

Also worth exposing an explicit **stride override** for pixel-art hexes
that aren't perfectly equilateral — artists frequently tune the visual
hex shape away from exact `sqrt(3)/2` ratios.

### `ScreenSpaceSprite` — SVG / terminal adapter coverage

Slice 4 added [`crate::commands::RenderCommand::ScreenSpaceSprite`] for
`Anchor::Viewport` rendering. Only the WebGL adapter implements it end-to-
end; SVG silently skips the variant and terminal does nothing at all.
Follow-up: wire SVG to emit an un-transformed sprite (its `cmd_sprite` is
already screen-space pixels, so routing should be one line).

### `ViewportTiled::Repeat{2D|X|Y}` — tiled viewport modes

The tiled viewport variants are declared on `SpriteSource::ViewportTiled`
but currently rejected at compile time with `CompileError::UnsupportedSource`.
They need a `Mesh` command with `wrap=Repeat` UVs (or an emitted loop of
screen-space sprites for SVG). Deferred until a real game demo asks for
tiling backdrops.

### WebGL texture upload Y-flip asymmetry

The two image-upload paths in `adapters::webgl` flip differently:

- **`ImageSource::Path`** (async, via `HtmlImageElement`) — uploads through
  `minwebgl::texture::d2::upload`, which sets `UNPACK_FLIP_Y_WEBGL=1`. Images
  are stored vertically flipped in texture memory.
- **`ImageSource::Bitmap`** (sync, raw bytes) — uploads via
  `tex_image_2d_with_..._opt_u8_array` without touching `pixel_storei`, so
  `UNPACK_FLIP_Y_WEBGL` stays at its default `0`. Images are stored
  un-flipped.

The sprite shaders (`sprite.vert` / `sprite_batch.vert`) compensate for the
Path-path flip: `v_uv.y = 1 - ( region.y + ( 1 - quad.y ) * region.h ) / tex.y`.
This gives correct rendering for Path-loaded sprites but means the **same
image loaded via `Bitmap` renders upside-down** through sprite commands. The
`mesh.vert` shader passes `a_uv` through unchanged, so meshes "work" for both
upload paths only when callers author UVs in GL (Y-up) convention — which
matches the flipped Path upload but mismatches the un-flipped Bitmap upload.

**Fix**: pick one convention and enforce it in the upload path (simplest:
also set `UNPACK_FLIP_Y_WEBGL=1` in the sync `Bitmap` branch in
`webgl.rs`). If we instead drop the flip everywhere, every existing example
authoring UVs in GL convention (e.g. `examples/minwebgl/hexagonal_map`)
needs its UVs re-authored in image convention, and the sprite shader's
outer `1 - ...` can be removed.

## license

Licensed under MIT license.

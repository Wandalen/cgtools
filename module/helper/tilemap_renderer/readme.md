# tilemap_renderer

Backend-agnostic 2D rendering engine with adapter support.

Define rendering commands once, render to any backend — SVG and WebGL2 today; terminal planned.

## coordinate system

All backends use a **Y-up** convention:

- `(0, 0)` is the bottom-left corner
- Positive Y points up
- Positive rotation is counter-clockwise (CCW)

## architecture

The crate follows **Ports & Adapters** (hexagonal) architecture:

- **Core** (`types`, `commands`, `assets`, `backend`) — platform-independent, no graphics dependencies
- **Adapters** (`adapters::SvgBackend`, `adapters::WebGlBackend`) — feature-gated backend implementations; the `adapter-terminal` feature gate exists but is a stub with no backend type yet

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
    └── terminal.rs # stub — no implementation yet (planned: ASCII/Unicode output)
```

## features

| Feature | Status | Description |
|---------|--------|-------------|
| `adapter-svg` | partial | SVG backend — generates SVG 1.1 documents; every command/asset family implemented, but font selection is not (`Assets.fonts` ignored — viewer default font) |
| `adapter-webgl` | partial | WebGL2 backend — sprites, meshes, instanced batches (wasm32); paths/text/effects pending |
| `adapter-terminal` | stub | Terminal backend — feature gate compiles; no `Backend` implementation exists yet |

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
| Paths | yes | — | — |
| Text | yes¹ | — | — |
| Sprites | yes | yes | — |
| Meshes | yes | yes | — |
| Batches | yes | yes | — |
| Gradients | yes | — | — |
| Effects | yes | — | — |
| Blend modes | yes | partial² | — |
| Viewport pan/zoom | yes | partial | — |

> **Terminal** column is all-empty because the adapter is a stub — the `adapter-terminal`
> feature gate compiles an empty module; no `Backend` implementation or type exists yet.
> **WebGL** adapter is partially implemented: sprites, meshes, and instanced batches work;
> paths, text, groups, gradients, patterns, and effects are not yet rendered.
>
> ¹ SVG text renders, but font selection is not implemented: `Assets.fonts` is accepted
> by `load_assets` and then ignored — no `@font-face` is emitted and `<text>` elements
> carry no `font-family`, so text appears in the viewer's default font. This is why
> `adapter-svg` is tracked as partial. See `docs/feature/001_svg_backend_adapter.md`.
>
> ² WebGL blend modes: Normal, Add, Multiply, Screen are hardware-accelerated.
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
> painter's-algorithm renderer. The SVG adapter still emits in submission order
> and ignores `depth` / `max_depth`.

## known issues / TODO

### `ScreenSpaceSprite` — terminal adapter coverage

[`crate::commands::RenderCommand::ScreenSpaceSprite`] renders a sprite in
screen-space coordinates, bypassing world-to-screen projection. WebGL and
SVG implement this command end-to-end — both dispatch through their
existing `cmd_sprite` path since the compile layer already emits
screen-space coordinates. The terminal adapter has no implementation at
all yet (see the features table), so this variant — like every other
command — is not rendered there; cover it when the terminal backend is
actually built and a use-case needs HUD / overlay rendering.

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

## Directory Layout

| Path | Responsibility |
|------|----------------|
| `src/` | Crate source — core types/commands/assets/backend trait, feature-gated backend adapters |
| `tests/` | Integration tests (core types, commands, assets, `Backend` trait contract) |
| `docs/` | Design documentation as typed doc definitions — see [docs/definition/readme.md](docs/definition/readme.md) |
| `roadmap.md` | Future work and per-adapter gaps |
| `readme.md` | This file — user-facing entry point |

## license

Licensed under MIT license.

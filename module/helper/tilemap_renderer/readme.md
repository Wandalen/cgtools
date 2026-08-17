# tilemap_renderer

Backend-agnostic 2D rendering engine with adapter support.

Define rendering commands once, render to any backend — SVG, WebGL2, WebGPU, and native `wgpu` today (the latter two via the `gpu_hal` HAL), plus a no-op backend for math-only simulation; terminal planned.

## coordinate system

All backends use a **Y-up** convention:

- `(0, 0)` is the bottom-left corner
- Positive Y points up
- Positive rotation is counter-clockwise (CCW)

## architecture

The crate follows **Ports & Adapters** (hexagonal) architecture:

- **Core** (`types`, `commands`, `assets`, `backend`) — platform-independent, no graphics dependencies
- **Adapters** (`adapters::SvgBackend`, `adapters::WebGlBackend`, `adapters::WebGpuBackend`, `adapters::NativeBackend`, `adapters::NoneBackend`) — feature-gated backend implementations; the `adapter-terminal` feature gate exists but is a stub with no backend type yet

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
    ├── terminal.rs # stub — no implementation yet (planned: ASCII/Unicode output)
    ├── none.rs     # complete no-op — math-only simulation, no rendering
    ├── webgpu.rs   # WebGPU rendering via gpu_hal (browser, sprites only)
    └── native.rs   # offscreen native wgpu rendering via gpu_hal (sprites only, pixel-verified)
```

## features

| Feature | Status | Description |
|---------|--------|-------------|
| `adapter-svg` | partial | SVG backend — generates SVG 1.1 documents; every command/asset family implemented, but font selection is not (`Assets.fonts` ignored — viewer default font) |
| `adapter-webgl` | partial | WebGL2 backend — sprites, meshes, instanced batches (wasm32); paths/text/effects pending |
| `adapter-terminal` | stub | Terminal backend — feature gate compiles; no `Backend` implementation exists yet |
| `adapter-none` | complete | No-op backend — accepts and discards everything; for math-only simulation with no rendering |
| `adapter-webgpu` | partial | WebGPU backend via `gpu_hal` — sprites only (wasm32); real pixel upload not yet supported by `gpu_hal`'s WebGPU surface |
| `adapter-native` | complete | Native `wgpu` backend via `gpu_hal` — offscreen sprite rendering with pixel readback; sprites only, but pixel-verified end-to-end |

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
svg.assets_load( &assets )?;
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

| Feature | SVG | WebGL | Terminal | None | WebGPU | Native |
|---------|-----|-------|----------|------|--------|--------|
| Paths | yes | — | — | — | — | — |
| Text | yes¹ | — | — | — | — | — |
| Sprites | yes | yes | — | — | yes³ | yes⁴ |
| Meshes | yes | yes | — | — | — | — |
| Batches | yes | yes | — | — | — | — |
| Gradients | yes | — | — | — | — | — |
| Effects | yes | — | — | — | — | — |
| Blend modes | yes | partial² | — | — | —⁵ | —⁵ |
| Viewport pan/zoom | yes | partial | — | — | — | — |

> **Terminal** column is all-empty because the adapter is a stub — the `adapter-terminal`
> feature gate compiles an empty module; no `Backend` implementation or type exists yet.
> **WebGL** adapter is partially implemented: sprites, meshes, and instanced batches work;
> paths, text, groups, gradients, patterns, and effects are not yet rendered.
>
> ¹ SVG text renders, but font selection is not implemented: `Assets.fonts` is accepted
> by `assets_load` and then ignored — no `@font-face` is emitted and `<text>` elements
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
>
> **None** column is all-empty by design, not incompleteness — `NoneBackend` is a
> complete, working no-op (`Capabilities::default()`), used to drive a command stream
> for math-only simulation with no rendering at all. See
> `docs/feature/004_none_backend_adapter.md`.
>
> ³ WebGPU sprites: the pipeline, transform math, and command classification are real
> and tested, but `gpu_hal`'s WebGPU surface has no pixel-upload call yet (`Device`
> offers `texture_create` allocation only, no `texture_write`) — loaded images are
> allocated but never populated with real pixels. See
> `docs/feature/005_webgpu_backend_adapter.md`.
>
> ⁴ Native sprites: unlike WebGPU, `gpu_hal`'s native surface does support pixel
> upload (`Queue::texture_write`), so this path renders real image content — verified
> by exact-byte pixel readback tests (`tests/native_backend_test.rs`), the only
> pixel-verified adapter in this crate. See `docs/feature/006_native_backend_adapter.md`.
>
> ⁵ Neither the WebGPU nor the native adapter reads `Sprite::blend` — the field is
> accepted but not yet applied by either pipeline.

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

### WebGL texture upload Y-flip asymmetry (fixed, BUG-210)

The two image-upload paths in `adapters::webgl` used to flip differently:

- **`ImageSource::Path`** (async, via `HtmlImageElement`) — uploads through
  `minwebgl::texture::d2::upload`, which sets `UNPACK_FLIP_Y_WEBGL=1`. Images
  are stored vertically flipped in texture memory.
- **`ImageSource::Bitmap`** (sync, raw bytes, `bitmap_texture_upload`) —
  previously uploaded via `tex_image_2d_with_..._opt_u8_array` without
  touching `pixel_storei`, so `UNPACK_FLIP_Y_WEBGL` stayed at its default
  `0`. Images were stored un-flipped.

The sprite shaders (`sprite.vert` / `sprite_batch.vert`) compensate for the
Path-path flip: `v_uv.y = 1 - ( region.y + ( 1 - quad.y ) * region.h ) / tex.y`.
This gave correct rendering for Path-loaded sprites but meant the **same
image loaded via `Bitmap` rendered upside-down** through sprite commands. The
`mesh.vert` shader passes `a_uv` through unchanged, so meshes "work" for both
upload paths only when callers author UVs in GL (Y-up) convention — which
matches the flipped Path upload but mismatched the un-flipped Bitmap upload.

**Fixed**: `bitmap_texture_upload` now also sets `UNPACK_FLIP_Y_WEBGL=1`
before its `tex_image_2d` call (and restores it to `0` afterward, matching
the function's existing `UNPACK_ALIGNMENT` restore convention) — see
BUG-210. Both upload paths now agree on the flipped/GL-convention UV
contract the sprite shaders already assume.

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

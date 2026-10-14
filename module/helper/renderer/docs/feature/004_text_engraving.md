# Feature: Text Engraving

Runtime-editable engraved text on glTF mesh nodes: user text is rasterized
into a per-node mask texture, and a `USE_ENGRAVING` variant of the PBR shader
carves that mask into the surface as a beveled, matte, darkened groove.

### Scope

- **Purpose**: Navigational hub for the engraving subsystem — config, scene binding, text rasterization, the mask texture and the shader block.
- **Responsibility**: Describe how text gets from `EngravingSession::set_text` to the shaded groove, and what `EngravingSession::build` does to the scene.
- **In Scope**: `engraving_config.json` parsing and validation, `EngravingSession`, canvas rasterization and `SizingMode`s, the mask texture and its mip chain, the `USE_ENGRAVING` shader block.
- **Out of Scope**: Preparing engravable assets and writing their config (see [../format/001_engraving_asset_pipeline.md](../format/001_engraving_asset_pipeline.md)); the PBR shading the groove modifies (see [001_pbr_rendering_core.md](001_pbr_rendering_core.md)).

### Design

**Config.** `EngravingConfig::from_json` (`src/webgl/engraving/config.rs`)
parses `engraving_config.json` (schema and a worked example sit next to it,
`engraving_config.schema.json` / `engraving_config.example.json`). Each entry
maps one glTF node name to an engraving zone: the UV set dedicated to it, the
zone's aspect ratio, a character limit, a font whitelist, the mask's texture
height, the relief knobs and an optional `SizingMode`. Validation mirrors the
schema's ranges and rejects a node listed twice.

**Binding.** `EngravingSession::build` (`apply.rs`) resolves every configured
node first, so a missing node, a non-mesh node or a non-`PbrMaterial` leaves
the scene untouched. It then allocates one offscreen canvas and one mask
texture per node, rejecting a texture larger than the context's
`MAX_TEXTURE_SIZE` (`EngravingError::TextureTooLarge`): WebGL reports an
oversized allocation only through `getError`, which would leave the mask black.
Finally it gives each node its own copy of every material on its primitives
and sets the mask on the copies. The glTF loader shares one material between
every primitive using the same glTF material, so writing the mask into it in
place would engrave every other node too; a node's mesh that another node also
uses (mesh instancing) is first detached into a mesh of its own over the same
geometry and skeleton. A material handle taken from the node before `build`
is therefore stale afterwards.

**Rasterizing.** `EngravingSession::set_text` rejects a font outside the
node's whitelist, awaits `document.fonts.load()`, and has
`EngravingCanvasManager` (`canvas_text.rs`) draw white-on-black text once per
mip level, each at its own resolution rather than box-filtered down from
level 0: a binary mask filtered that way aliases at the glyph edges, and the
shader differentiates whichever level it samples. `mip_dimensions`
(`texture.rs`) sizes the chain for both the canvas and the texture.
`EngravingTexture::update` uploads the chain with `texSubImage2D` per level,
with no reallocation or shader recompile, but every call still re-rasterizes,
reads back and uploads each level, so editors should re-render when an edit is
committed rather than per keystroke.

**Sizing.** `SizingMode::Physical` and `Hybrid` derive the font size from
`defaultFontSizeMm / stripHeightMm`; `Physical` rejects text that overflows
the padded strip, `Hybrid` shrinks it toward `minFontSizeMm` first. `Relative`
uses `fontSizeRatio` of the canvas height or auto-fits the padded bounds; a
node with no physical fields resolves to it. Overflow is fatal only at the
full-size level: lower mips only anti-alias the minified result.
`EngravingSession::text_layout_mut` changes these at runtime; the geometry
fields that sized the canvas and texture are not reachable through it.

**Shader.** `main.frag`'s `USE_ENGRAVING` block reads the mask on its own UV
set, forced to 0 outside `[0, 1]`. It picks an LOD from the UV's screen-space
footprint and takes `textureLod` central differences one texel apart along U
and V at that LOD, then bends the normal along the shared tangent frame, with
the sign flipped so the mask reads as a groove carved into the surface.
Sampling in UV space at one explicit LOD keeps the bevel aligned with the
groove at any view angle and stable at any distance, which screen-space
`dFdx`/`dFdy` of the mask would not. Inside the mask, roughness moves toward
`engravingRoughness` and the diffuse color and `f0` are scaled by
`1 - engravingDarkening`, which darkens the groove without changing the
metal's hue. The tangent frame follows `TEXCOORD_0`, so the engraving UV set
must keep its orientation (see the format doc, §2).

### Sources

| File | Relationship |
|------|--------------|
| `src/webgl/engraving/config.rs` | `EngravingConfig` / `EngravingNodeConfig` parsing and validation, `EngravingTextLayout`, `SizingMode`, `TextAlign` |
| `src/webgl/engraving/apply.rs` | `EngravingSession`: node resolution, per-node material copies, font whitelist, `set_text` |
| `src/webgl/engraving/canvas_text.rs` | `EngravingCanvasManager`: font loading, per-level rasterization, font sizing |
| `src/webgl/engraving/texture.rs` | `EngravingTexture`: mip-chain allocation and in-place upload; `mip_dimensions` |
| `src/webgl/engraving/engraving_config.schema.json` | JSON Schema of `engraving_config.json` |
| `src/webgl/material/pbr.rs` | `PbrMaterial`'s engraving fields, the `USE_ENGRAVING` define and `PBR_ENGRAVING_UNIT` |
| `src/webgl/shaders/main.frag` | The `USE_ENGRAVING` relief, roughness and darkening block |

### Tests

| File | Relationship |
|------|--------------|
| `tests/engraving_config_tests.rs` | Config parsing, `SizingMode` resolution, validation ranges, `mip_dimensions` |
| `tests/engraving_session_tests.rs` | `build`'s per-node material copies, the font whitelist, `MAX_TEXTURE_SIZE` |
| `tests/engraving_shader_tests.rs` | The `USE_ENGRAVING` variants compile alone and alongside normal maps, clearcoat, anisotropy and IBL |

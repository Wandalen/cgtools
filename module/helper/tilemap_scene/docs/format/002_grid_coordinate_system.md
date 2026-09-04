# Format: Grid Coordinate System

### Scope

- **Purpose**: Define the tiling strategy declaration, its axial coordinate system, and the grid-to-pixel conversion it implies.
- **Responsibility**: Document `TilingStrategy`, `HexConfig`, direction ordering, and the coordinate-to-pixel mapping.
- **In Scope**: `tiling` selection, axial `(q, r)` coordinates, per-strategy direction enumeration order, `grid_stride`-based pixel conversion.
- **Out of Scope**: How coordinates combine into placements (see `format/003`); how pixel positions are culled/sorted at render time (see `algorithm/002`).

### Abstract

A scene picks exactly one **tiling strategy** for its grid, declared once on `RenderPipeline.hex` (see `format/007`). The strategy fixes four things at once: how many neighbours a cell has, in what clockwise order those neighbours are enumerated (which in turn fixes `NeighborBitmask` bit indices — see `format/005`), how many corners a dual-mesh vertex has, and the formula that converts an axial grid coordinate to a world-pixel point. Only the two hex variants are implemented; the two square variants are reserved schema surface for a future minor version.

### Data Model

`TilingStrategy` (declared on `HexConfig.tiling`):

| Variant | Neighbours | Direction order (clockwise from top) | Status |
|---------|-----------|----------------------------------------|--------|
| `HexFlatTop` | 6 | `[N, NE, SE, S, SW, NW]` | Implemented |
| `HexPointyTop` | 6 | `[NE, E, SE, SW, W, NW]` | Implemented |
| `Square4` | 4 | — | **Reserved, not implemented** |
| `Square8` | 8 | — | **Reserved, not implemented** |

Grid positions use axial coordinates `(q, r)` with `i32` components; cube coordinates `(q, r, -q - r)` are derived only where a computation needs them (e.g. hex-distance). `HexConfig` additionally carries `grid_stride: (u32, u32)` — the pixel spacing between the centres of adjacent cells along the primary axes, tuned per sprite art rather than derived from geometry (stylised/non-equilateral hex art needs empirical tuning to tile seamlessly).

### Encoding Structure

`tiling` is a bare RON enum tag (`tiling: HexFlatTop`). Bit `i` of any `NeighborBitmask`/`EdgeConnectedBitmask` value corresponds to the neighbour at index `i` of the active strategy's direction-order list above — the same bitmask byte means a different set of physical neighbours under `HexFlatTop` versus `HexPointyTop`, so a spec's bitmask sources are only meaningful relative to its own declared `tiling`.

**Pixel conversion**: axial `(q, r)` converts to a Y-up world-pixel centre via `crate::compile::coords::hex_to_world_pixel_flat`/`_pointy(q, r, grid_stride)`. Internally this delegates to `tiles_tools::coordinates::pixel::Pixel` for the underlying trigonometric unit-scale output (`1.5 * q`, `sqrt(3)/2 * q + sqrt(3) * r`, etc. — the exact per-strategy constants), then applies a per-axis compensating factor so the unit-scale output spans exactly `grid_stride`, then negates the Y axis once to convert `tiles_tools`' native Y-down convention to this crate's Y-up convention (all downstream consumers, including `crate::compile::camera::Camera::project`, operate Y-up).

### Version Compatibility

The specification's normative requirement — **`Square4`/`Square8` MUST be rejected at load time with a clear error** — and the implementation now agree: `src/validate.rs`'s tiling-whitelist check constructs `ValidationError::UnsupportedTiling(String)` (see `invariant/001`) whenever `pipeline.hex.tiling` names a reserved variant, so a spec naming `Square4`/`Square8` now fails `RenderSpec::load()` with that error instead of reaching render. New minor-version additions to this schema (a third hex orientation, actual square support) are expected to extend `TilingStrategy` without breaking existing `HexFlatTop`/`HexPointyTop` specs.

### Formats

| File | Relationship |
|------|--------------|
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | `Hex`/`Edge`/`Vertex`/`Multihex` anchors are positioned in this coordinate system |
| [format/007_render_pipeline.md](../format/007_render_pipeline.md) | `HexConfig` (tiling + grid_stride) is declared on `RenderPipeline.hex` |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | Formalizes the tiling-strategy whitelist and its enforcement (`ValidationError::UnsupportedTiling`) |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_load_time_validation_partially_enforced.md](../pitfall/001_load_time_validation_partially_enforced.md) | `Square4`/`Square8` — formerly this format's worked example of an unenforced MUST; `validate.rs`'s tiling whitelist now rejects both at load time, so this class of failure no longer reaches render |

### Sources

| File | Relationship |
|------|--------------|
| `src/pipeline.rs` | `TilingStrategy`, `HexConfig` |
| `src/compile/coords.rs` | Axial-to-world-pixel conversion, Y-up flip |

### Tests

| File | Relationship |
|------|--------------|
| `tests/hex_config_test.rs` | `HexConfig`/`from_hex_size` arithmetic |
| `src/compile/coords.rs` | Inline `#[cfg(test)]` coverage including `flat_top_y_flip_is_applied` |

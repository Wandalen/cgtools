# Format: Anchor Placement Types

### Scope

- **Purpose**: Define the six `Anchor` variants that determine what "position" means for an object instance.
- **Responsibility**: Document each anchor's position payload, canonicalization rule (where one applies), and sprite-source compatibility restrictions.
- **In Scope**: `Hex`, `Edge`, `Vertex`, `Multihex`, `FreePos`, `Viewport` — their position payloads, culling/sort implications, and which `SpriteSource` variants each permits.
- **Out of Scope**: The coordinate system anchors are positioned in (see `format/002`); the sprite source variants themselves (see `format/005`).

### Abstract

`Anchor` is the field on `Object` (see `format/001`) that determines three things at once: what a "position" payload looks like for an instance of this object, what neighbour/context information is visible to its sprite sources, and how it is culled and sorted at render time. Two anchors — `Edge` and `Vertex` — additionally carry a canonicalization rule: because an edge or vertex is shared between multiple cells, more than one `(hex, ...)` encoding can name the same physical location, and the format fixes which encoding is canonical so a renderer never emits a duplicate.

### Data Model

| Variant | Position payload | Neighbours | Used for |
|---------|-------------------|------------|----------|
| `Hex` | one `(q, r)` | 6 (hex) | Terrain, units, hex-anchored overlays |
| `Edge` | `(hex, direction)` | — | Fences, rivers, roads along cell boundaries |
| `Vertex` | tuple of corner cells (3 for hex) | — | Dual-mesh triangle blends |
| `Multihex` | anchor cell + `shape: Vec<(i32, i32)>` relative offsets | — | Multi-cell buildings (e.g. a 2×2 castle) |
| `FreePos` | `(x, y)` world-pixel point | — | Projectiles, particles, floating damage numbers |
| `Viewport` | screen-space, via `anchor_point` | — | Skyboxes, weather overlays, vignettes, letterboxing |

**Sprite-source compatibility** (which `SpriteSource` variants — see `format/005` — each anchor permits):

| Anchor | Permitted | Rejected |
|--------|-----------|----------|
| `Hex` | all variants | — |
| `Edge` | `Static`, `Variant`, `Animation`, `External`, `EdgeConnectedBitmask` | Hex-specific: `NeighborBitmask`, `NeighborCondition`, `VertexCorners` |
| `Vertex` | `VertexCorners`-oriented sources | Hex-specific neighbour sources not defined for a vertex context |
| `Multihex` | `Static`, `Animation` only | All neighbour-dependent sources: `NeighborBitmask`, `NeighborCondition`, `VertexCorners` |
| `FreePos` | non-neighbour-dependent sources | All neighbour-dependent sources |
| `Viewport` | `ViewportTiled` and non-neighbour-dependent leaf sources | Neighbour-dependent sources (no grid context exists) |

### Encoding Structure

`Multihex`'s `shape` is a list of offsets relative to the anchor cell, e.g. `Multihex(shape: [(0, 0), (1, 0), (0, 1), (1, 1)])` for a 2×2 footprint; its pixel position is the anchor cell's pixel position, a single sprite covers the shape's bounding box, it is culled visible if **any** cell in its shape is visible, and its Y-sort key is the anchor cell's Y unless the object sets `sort_y_source: BottomOfShape` (see `format/001`).

`Viewport`'s position is computed per layer from `anchor_point` (top-left, center, bottom-center, stretch, etc. — see `format/006` for `parallax`); it is never culled against the world grid and draws for as long as its instance is alive.

**Canonicalization** (`Edge` and `Vertex` only): an edge `(hex_A, dir_AB)` and its mirror `(hex_B, dir_BA)` name the same physical edge; the canonical encoding is the pair whose hex has the lexicographically smaller `(q, r)`. A vertex's corner-cell tuple is sorted lexicographically by the terrain id present at each corner (used during pattern matching), with a `rotation` integer (∈ {0, 1, 2} for hex) recording the permutation applied to reach that sorted order. Both rules exist so a renderer emits each edge/vertex exactly once even though two or three adjacent cells could each independently propose it (see `invariant/002`).

### Version Compatibility

Anchor variants are additive across minor versions — a new anchor kind is expected to extend this enum without invalidating specs that don't use it. The `Square4`/`Square8` tiling whitelist (`format/002`, enforced at load time) is orthogonal to any anchor variant's own schema; anchors are defined independent of which tiling strategy is active.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/002_scene_rendering_pass.md](../algorithm/002_scene_rendering_pass.md) | Culling ("if instance is culled: continue") is anchor-specific |

### APIs

| File | Relationship |
|------|--------------|
| [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) | `spawn(object_id, placement)`'s `placement` payload shape is anchor-specific, per the Data Model table above |

### Formats

| File | Relationship |
|------|--------------|
| [format/001_scene_object_model.md](../format/001_scene_object_model.md) | `Object.anchor` selects one of these variants |
| [format/002_grid_coordinate_system.md](../format/002_grid_coordinate_system.md) | `Hex`/`Edge`/`Vertex`/`Multihex` positions are expressed in this coordinate system |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | Sprite-source compatibility restrictions listed above |
| [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) | Each `SceneSnapshot` instance collection corresponds to one anchor kind |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | `connects_with` reserved-id resolution is enforced there; anchor↔source compatibility (Data Model table above) is the one rule it documents as still unenforced |
| [invariant/002_edge_and_vertex_canonical_uniqueness.md](../invariant/002_edge_and_vertex_canonical_uniqueness.md) | Formalizes the `Edge`/`Vertex` canonicalization rule stated above |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_load_time_validation_partially_enforced.md](../pitfall/001_load_time_validation_partially_enforced.md) | Anchor↔source compatibility restrictions above are declared but not yet enforced at load time |

### Sources

| File | Relationship |
|------|--------------|
| `src/anchor.rs` | `Anchor`, `EdgeDirection`, `SortYSource` |
| `src/compile/edges.rs` | `canonical_edge()` — Edge canonicalization |
| `src/compile/vertex.rs` | Vertex canonicalization |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_compile_test.rs` | Anchor × source combination coverage, including rejected combinations |
| `src/compile/edges.rs` | Inline `#[cfg(test)]` including `canonical_picks_smaller_hex` |

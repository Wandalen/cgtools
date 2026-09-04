# Format: Render Pipeline

### Scope

- **Purpose**: Define `RenderPipeline` — the ordered z-bucket list objects draw into, and each bucket's sort mode.
- **Responsibility**: Document `RenderPipeline`/`PipelineLayer`/`SortMode`/`HexConfig` fields and the per-layer bucket-override mechanism.
- **In Scope**: Pipeline-level fields (`hex`, `viewport_size`, `clear_color`, `layers`, `global_tint`), the 8 `SortMode` variants, `pipeline_layer` override semantics.
- **Out of Scope**: The tiling strategy `hex` declares (see `format/002`); how a bucket's draw calls are actually gathered, sorted, and submitted per frame (see `algorithm/002`).

### Abstract

A scene declares exactly one `RenderPipeline`: a bottom-to-top ordered list of named z-buckets (`PipelineLayer`s) that every object's layers draw into, plus grid geometry (`hex: HexConfig`, see `format/002`) and optional viewport/clear-color/global-tint settings. Bucket ids are entirely user-chosen and carry no semantic meaning to the renderer beyond their declared order — `"terrain"` versus `"units"` is a naming convention, not a reserved keyword. `format/001`'s `Object.global_layer` and `ObjectLayer.pipeline_layer` both name a `PipelineLayer.id` from this list.

### Data Model

`RenderPipeline`: `hex: HexConfig`, `layers: Vec<PipelineLayer>`, `global_tint: Option<TintRef>`, `viewport_size: Option<(u32, u32)>` (derived from the window when absent), `clear_color: Option<[f32; 4]>` (linear RGBA; `None` = transparent).

`PipelineLayer`: `id: String`, `sort: SortMode` (default `None`), `tint_mask: Option<TintRef>`.

`SortMode` (8 variants):

| Variant | Order |
|---------|-------|
| `None` | Spawn order (default; deterministic for static scenes). |
| `XAsc` / `XDesc` | Screen X ascending / descending. |
| `YAsc` | Screen Y ascending — objects lower on screen draw later, appearing in front. |
| `YDesc` | Reverse of `YAsc`. |
| `XAscYDesc` | X ascending primary, Y descending tiebreak — common for isometric stacks. |
| `XAscYAsc` | X ascending primary, Y ascending tiebreak. |
| `YDescXAsc` | Y descending primary (top-of-screen first), X ascending tiebreak — top-to-bottom painter's order; used for zigzag hex coasts where a screen-lower tile must paint over a screen-higher one regardless of column. |
| `YAscXAsc` | Y ascending primary, X ascending tiebreak. |

For `Multihex` instances, the Y used by any Y-based sort mode is `sort_y_source` (see `format/001`) — the anchor cell's Y by default, or the shape's bottom Y if the object overrides it.

### Encoding Structure

`layers` is a flat RON list, read bottom-to-top — the list's declaration order *is* the draw order between buckets. `HexConfig` additionally exposes an `from_hex_size(width, height, tiling)` convenience constructor that derives `grid_stride` from a sprite's pixel dimensions rather than requiring the stride to be computed by hand. `pipeline_layer` (declared per-`ObjectLayer`, see `format/001`) overrides `Object.global_layer` for exactly that one layer, letting a single object contribute draw calls to more than one bucket in the same frame — the canonical use is the Wesnoth-style edge-transition idiom (see `format/005`): a terrain object's base layer draws in `terrain`, its edge-blend layer draws in `terrain_edges`, so every hex's edge overlap lands on top of *all* hexes' base terrain rather than just its own. Within one bucket, draw calls from every contributing object are gathered together, the bucket's own `sort` mode is applied once across all of them, and `z_in_object` (see `format/001`) breaks ties when two draw calls share a sort key. `global_tint` applies multiplicatively to every draw call after all per-object tints and effects — the typical use is a time-of-day overlay.

### Version Compatibility

New `SortMode` variants are expected to be additive. `HexConfig.tiling` carries the same `Square4`/`Square8`-reserved status documented in `format/002` — a pipeline naming one of them is schema-valid but not currently renderable end-to-end (see `pitfall/001`).

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/002_scene_rendering_pass.md](../algorithm/002_scene_rendering_pass.md) | Consumes this declaration to gather, sort, and submit draw calls per bucket each frame |

### Formats

| File | Relationship |
|------|--------------|
| [format/001_scene_object_model.md](../format/001_scene_object_model.md) | `Object.global_layer` / `ObjectLayer.pipeline_layer` reference `PipelineLayer.id` |
| [format/002_grid_coordinate_system.md](../format/002_grid_coordinate_system.md) | `HexConfig` (tiling strategy + grid_stride) declared here |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | Per-layer bucket override is the mechanism behind the Wesnoth edge-blend idiom |
| [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) | `RenderSpec.pipeline` field of this doc's top-level structure |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | Pipeline-layer id uniqueness and reference resolution enforced here |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_load_time_validation_partially_enforced.md](../pitfall/001_load_time_validation_partially_enforced.md) | Square-tiling gap surfaces through a pipeline declaring it |

### Sources

| File | Relationship |
|------|--------------|
| `src/pipeline.rs` | `RenderPipeline`, `PipelineLayer`, `SortMode`, `HexConfig`, `from_hex_size` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/sorted_batching_test.rs` | Bucket gathering + all `SortMode` variants |
| `tests/hex_config_test.rs` | `HexConfig`/`from_hex_size` arithmetic |

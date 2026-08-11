# Format: Sprite Sources

### Scope

- **Purpose**: Define the `SpriteSource` sum type — the rule that produces a concrete sprite/frame for a layer at render time.
- **Responsibility**: Document all 9 `SpriteSource` variants, their leaf/composite split, anchor applicability, and rule-resolution order for ambiguous matches.
- **In Scope**: `Static`, `Variant`, `Animation`, `External` (leaf); `NeighborBitmask`, `NeighborCondition`, `VertexCorners`, `EdgeConnectedBitmask`, `ViewportTiled` (composite); specificity/priority/declaration-order resolution for multi-match rules.
- **Out of Scope**: The resources these sources reference by id (see `format/004`); how a selected frame is tinted/blended once chosen (see `format/006`); how an `Animation`'s current frame is computed (see `algorithm/001`).

### Abstract

A `SpriteSource` is a rule that, given a layer's render-time context, produces a concrete sprite or frame. Sources split into two categories: **leaf sources** (`Static`, `Variant`, `Animation`, `External`) look no further than the object's own position and compose freely — a `Variant` may wrap `Static` or `Animation` sub-sources, an autotile mapping's per-mask slot accepts any leaf source. **Composite sources** (`NeighborBitmask`, `NeighborCondition`, `VertexCorners`, `EdgeConnectedBitmask`, `ViewportTiled`) look at grid context beyond the object's own cell — neighbour cells, vertex corners, or the viewport — and cannot nest inside another composite source (no neighbour-of-neighbour lookups), though their internal "sprite per mask/variant" slots accept leaf sources. This is what lets an autotile mapping point at an `Animation` (an animated wall segment) or a `Variant` with `HashCoord` selection (visually-varied copies of the same wall shape) without the composite/leaf split becoming a combinatorial special case.

### Data Model

**Leaf sources**:

| Variant | Fields | Behaviour |
|---------|--------|-----------|
| `Static` | `SpriteRef` | Fixed sprite, no selection logic. |
| `Variant` | `variants: Vec<{sprite: SpriteSource, weight}>`, `selection: VariantSelection` | Picks one weighted entry per **object instance** (not per frame); the picked entry's own sub-source still runs every frame. |
| `Animation` | `AnimationRef` | Current frame selected per `algorithm/001`. |
| `External` | `slot: String` | Sprite supplied by game code at runtime via `set_external_sprite(instance, slot, SpriteRef)` (see `api/001`); unset slot silently skips the layer this frame — no warning, no placeholder; a set slot that doesn't resolve fails the pass with `CompileError::UnresolvedRef` (see `algorithm/002`). Applicable to all anchors. |

`VariantSelection`: `HashCoord` (default; deterministic hash of the anchor's grid coordinate — requires a grid-coordinate anchor) | `Random` (same deterministic hash, salted by `Scene.seed` instead of a fixed salt — "random" here means seed-reshuffleable and run-stable, not runtime entropy) | `Fixed(usize)` (forces one entry).

**Composite sources**:

| Variant | Applicable anchor | Fields | Behaviour |
|---------|--------------------|--------|-----------|
| `NeighborBitmask` | `Hex` only | `connects_with: Vec<ObjectId>`, `source: NeighborBitmaskSource` | Sets bit `i` (tiling-strategy direction order, see `format/002`) when neighbour `i` carries an object whose id is in `connects_with`; looks the mask up via `ByMapping{mapping, fallback}` (explicit bitmask→leaf-source map) or `ByAtlas{asset, layout}` (atlas grid pre-authored one sprite per bitmask index). |
| `NeighborCondition` | `Hex` only | `condition: Condition`, `sides: Vec<EdgeDirection>`, `sprite_pattern: String` (`{dir}` template), `asset` | For each side in `sides`, evaluates `condition` against that side's neighbour; on match, emits one sprite with `{dir}` substituted — up to `len(sides)` sprites per cell per pass. Covers both 3D-skirt and Wesnoth-style edge-blend idioms (see Design Notes below). Emitted sprites are positioned at the cell's own pixel center; their art may extend into the neighbour direction — the renderer does not clip. |
| `VertexCorners` | `Vertex` only | `patterns: Vec<{corners: (ObjectId|"*", ...), sprite_pattern: String ({rot} template), priority}>`, `asset` | Reads the object ids at the vertex's corners, builds the canonical sorted tuple + rotation (see `format/003`), matches against `patterns` by specificity then priority then declaration order (see Rule Resolution below), emits with `{rot}` substituted, or emits nothing if no pattern matches. `"*"` matches any single corner and sorts as lexicographically greater than any concrete id. |
| `EdgeConnectedBitmask` | `Edge` only | `connects_with`, `source: NeighborBitmaskSource`, `layout: EdgeConnectedLayout` (`EdgeHex`) | The edge analogue of `NeighborBitmask` — a 4-bit mask (2 potentially-connected neighbour edges at each of the edge's 2 endpoints in a hex grid), bit layout and sprite-rotation rules fixed by `EdgeHex` (see Design Notes below). |
| `ViewportTiled` | `Viewport` only | `content: SpriteSource` (typically `Static`/`Animation`), `tiling: ViewportTiling`, `anchor_point: ViewportAnchorPoint` | Non-tiled modes (`Center`, `Stretch`, `Fit`) emit one screen-space sprite; tiled modes (`Repeat2D`, `RepeatX`, `RepeatY`) emit a grid of screen-space sprites at native pixel size scaled by camera zoom, so background texels stay coherent with zoomed foreground sprites. `anchor_point` applies to non-tiled modes and to the non-repeating axis of `RepeatX`/`RepeatY`. |

`Condition` grammar (used by `NeighborCondition`): `NeighborIs([ObjectId, ...])` | `NoNeighbor` | `NeighborPriorityLower` (true when the current cell's `Object.priority` — see `format/001` — is strictly higher than the examined neighbour's) | `AnyOf([Condition, ...])` | `AllOf([Condition, ...])` | `Not(Condition)`.

### Encoding Structure

`NeighborBitmaskSource::ByMapping.mapping` is authored as `{ bitmask_literal: leaf_source, ... }` with missing entries falling back to `fallback`; `ByAtlas` instead pre-lays-out one sprite per bitmask index in a single atlas (`layout: Bitmask6` for hex's 64 combinations from 6 neighbour bits). `sprite_pattern` strings use brace-delimited placeholders resolved per emission — `{dir}` to the side's direction name (`NeighborCondition`), `{rot}` to the resolved rotation index (`VertexCorners`, `EdgeConnectedBitmask`'s own rotation-override convention).

**Design notes — idioms encoded via these sources**: `NeighborCondition` with `condition: NeighborPriorityLower` and a pipeline-layer override (see `format/001`, `format/007`) implements Wesnoth-style edge blending — a higher-priority terrain (e.g. grass, `priority: 10`) draws a thin overlap sprite into a lower-priority neighbour (water, `priority: 5`); routing the overlap layer into a later pipeline bucket (e.g. `terrain_edges` after `terrain`) guarantees every hex's base layer finishes before any hex's edge overlap draws, so the overlap always lands on top regardless of draw order within the base bucket. This idiom composes freely with skirts (a second `NeighborCondition` layer) and with `VertexCorners` triangle blends as independent layers, typically each in its own pipeline bucket. `EdgeConnectedBitmask`'s `EdgeHex` layout fixes `start_vertex`/`end_vertex` relative to the canonical edge's direction (the endpoints shared with the clockwise-previous and clockwise-next edges), each contributing a ccw/cw bit — a three-edge river junction sets both bits at the shared vertex on all three meeting edges (`0b11` at that end), which is why Y/T-shaped mask entries (`0b0011`, `0b1100`, `0b1111`) exist. Mirror-symmetric masks (e.g. `0b0001` vs `0b0010`) have no dedicated symmetry-declaration mechanism in 0.2.0 — duplicate `mapping` entries are the documented workaround.

**Rule resolution** (applies to `NeighborCondition` and `VertexCorners`, whose `patterns`/rules may multi-match one input): (1) fewer wildcards wins (specificity); (2) higher `priority` integer wins; (3) earlier declaration order wins. Implementations SHOULD warn when two rules tie on both specificity and priority.

### Version Compatibility

New leaf or composite variants are expected to be additive. The leaf/composite nesting restriction ("composite sources cannot nest inside composite sources") is stated as a format rule but is **not yet enforced by `validate.rs`** — see `pitfall/001` and `invariant/001`; a spec author who nests a composite inside another composite's per-mask slot today gets no load-time diagnostic. `format/003`'s anchor↔source applicability table (`NeighborBitmask`/`NeighborCondition`/`VertexCorners` restricted to `Hex`/`Vertex`, `EdgeConnectedBitmask` to `Edge`, all neighbour-dependent sources excluded from `Multihex`/`FreePos`/`Viewport`) is likewise declared but not yet checked at load time.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_animation_phase_and_frame_selection.md](../algorithm/001_animation_phase_and_frame_selection.md) | `Variant::HashCoord`/`Random` selection shares this algorithm's `hash_coord`/`hash_str` primitives |
| [algorithm/002_scene_rendering_pass.md](../algorithm/002_scene_rendering_pass.md) | `sample_source` step; composite sources may emit multiple draw calls per instance |

### APIs

| File | Relationship |
|------|--------------|
| [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) | `set_external_sprite` populates an `External` source slot |

### Formats

| File | Relationship |
|------|--------------|
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | Anchor↔source applicability restrictions summarized above |
| [format/004_declared_resources.md](../format/004_declared_resources.md) | Leaf sources resolve `SpriteRef`/`AnimationRef` against resources declared there |
| [format/006_layer_behaviour.md](../format/006_layer_behaviour.md) | A layer's `sprite_source` (documented here) is independent of its `behaviour` |
| [format/007_render_pipeline.md](../format/007_render_pipeline.md) | Per-layer bucket override is the mechanism behind the Wesnoth edge-blend idiom |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | Asset/animation reference resolution and composite-nesting enforcement gap |
| [invariant/002_edge_and_vertex_canonical_uniqueness.md](../invariant/002_edge_and_vertex_canonical_uniqueness.md) | `EdgeConnectedBitmask`/`VertexCorners` read neighbour state through the canonical edge/vertex form |

### Sources

| File | Relationship |
|------|--------------|
| `src/source.rs` | `SpriteSource`, `Variant`, `VariantSelection`, `Condition`, `NeighborBitmaskSource`, `AutotileLayout`, `EdgeConnectedLayout`, `TriBlendPattern`, `ViewportTiling`, `ViewportAnchorPoint` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_compile_test.rs` | Compile-time coverage of all 9 source variants across applicable anchors |

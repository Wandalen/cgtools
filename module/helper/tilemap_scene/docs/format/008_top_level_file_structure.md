# Format: Top-Level File Structure

### Scope

- **Purpose**: Define the two top-level documents — `RenderSpec` and `Scene` (`SceneSnapshot`) — and how they compose.
- **Responsibility**: Document `RenderSpec`'s top-level fields, `Scene`'s instance-list fields, the palette/map shorthand, and `RenderSpec.version`'s compatibility contract.
- **In Scope**: `RenderSpec` structure, `SceneSnapshot` structure (bounds, tiles, palette/map, edges, multihex/free/viewport instances, entities, players, seed), the RON-is-one-serialization design stance.
- **Out of Scope**: The schema of each nested type — objects (`format/001`), resources (`format/004`), pipeline (`format/007`); load-time validation of the cross-references this structure creates (see `invariant/001`).

### Abstract

Two plain Rust structs with `serde` derives form the entire persisted format: `RenderSpec` (declares reusable resources, objects, and the render pipeline — authored once per game or art set) and `SceneSnapshot` (declares one scene's instances — which objects are placed where). Both are format-agnostic: the RON payloads shown here are one serialization; JSON or any other `serde`-compatible format works identically, and games with their own data representation (their own JSON, a binary format, ECS queries) are expected to build a `Scene` directly in memory without ever going through a RON file at runtime. The format carries rendering information only — game mechanics (HP, AI, inventory) live in the game's own types, referenced only loosely here via `Entity.owner`/`Player`.

### Data Model

`RenderSpec`: `version: String`, `assets: Vec<Asset>`, `tints: Vec<Tint>`, `animations: Vec<Animation>`, `effects: Vec<Effect>`, `objects: Vec<Object>`, `pipeline: RenderPipeline` — all four resource collections and `objects` are flat lists (not maps), consistent with each other (see `format/004`, `format/001`, `format/007`).

`SceneSnapshot`: `meta: SceneMeta` (optional `name`, optional `render_spec` path — both `None` by default for runtime-constructed scenes, populated when loading from disk), `bounds: Bounds` (`min`/`max` grid coordinates, or `unbounded()`), plus one collection per placement kind:

| Field | Element type | Anchor kind (see `format/003`) |
|-------|--------------|----------------------------------|
| `tiles` | `Tile{pos, objects: Vec<ObjectId>}` | `Hex` — objects stacked bottom-to-top on one cell |
| `edges` | `EdgeInstance{at: EdgePosition, object}` | `Edge` — one entry per canonical edge |
| `multihex_instances` | `MultihexInstance{anchor, object}` | `Multihex` |
| `free_instances` | `FreeInstance{pos, object}` | `FreePos` |
| `viewport_instances` | `ViewportInstance{object, animation}` | `Viewport` |
| `entities` | `Entity{at, object, owner, animation}` | Same schema as `tiles` objects, tracked separately because they move at runtime |
| `players` | `Player{id, color, name}` | Referenced by `TeamColor` symbolic tint (see `format/004`) via `Entity.owner` |

Plus `initial_global_tint: Option<TintRef>` (runtime-changeable via `set_global_tint`, see `api/001`) and `seed: Option<u64>` (defaults to `0`; feeds `Variant::Random` selection, see `format/005`).

### Encoding Structure

`tiles` and the `(palette, map)` pair are mutually exclusive for hex cells — exactly one MUST be present. The palette form maps a single character to a stacked-objects list (`'#': ["grass", "stone_wall"]`) and an ASCII grid (`map: ["..v#..", ...]`) references those characters by position; `expand_palette()` converts this shorthand into the same `Vec<Tile>` the `tiles` field would hold directly, interpreting the grid in offset coordinates and converting to axial internally via the active tiling strategy (see `format/002`). Vertex-anchored objects are normally implicit — dual-mesh triangles emit automatically from terrain configuration via `VertexCorners` (see `format/005`) — the format has no top-level `vertices` list; only explicit overrides would need one, and 0.2.0 does not define that override path.

`RenderSpec.version` is a semver string (e.g. `"0.2.0"`).

### Version Compatibility

The specification this doc replaces states: *implementations MUST reject specs with a major version higher than supported; minor-version additions (new anchor types, new sources) SHOULD remain backward compatible; breaking changes require a major bump.* **This is not currently enforced** — `RenderSpec.version` is stored (`src/spec.rs`) but no code path in `src/load.rs` or `src/validate.rs` compares it against a supported range. A spec declaring a future-incompatible major version loads exactly as successfully as one declaring the current version; there is no load-time signal warning the caller their implementation may not understand the file's semantics (see `pitfall/001`).

### APIs

| File | Relationship |
|------|--------------|
| [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) | `RenderSpec::load` / `Scene::from_snapshot` are the runtime entry points consuming this structure |

### Formats

| File | Relationship |
|------|--------------|
| [format/001_scene_object_model.md](../format/001_scene_object_model.md) | `RenderSpec.objects` element type |
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | Each `SceneSnapshot` instance collection corresponds to one anchor kind |
| [format/004_declared_resources.md](../format/004_declared_resources.md) | `RenderSpec.assets`/`tints`/`animations`/`effects` element types |
| [format/007_render_pipeline.md](../format/007_render_pipeline.md) | `RenderSpec.pipeline` |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | Cross-references created by this structure (instance `object` fields, `Object.default_state`, etc.) are validated at load time |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_load_time_validation_partially_enforced.md](../pitfall/001_load_time_validation_partially_enforced.md) | `RenderSpec.version`'s unenforced compatibility contract |

### Sources

| File | Relationship |
|------|--------------|
| `src/spec.rs` | `RenderSpec` |
| `src/snapshot.rs` | `SceneSnapshot`, `SceneMeta`, `Bounds`, `Tile`, `EdgeInstance`, `EdgePosition`, `MultihexInstance`, `FreeInstance`, `ViewportInstance`, `Entity`, `Player`, `expand_palette` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_test.rs` | `RenderSpec`/`SceneSnapshot` RON round-trip, including palette/map expansion |

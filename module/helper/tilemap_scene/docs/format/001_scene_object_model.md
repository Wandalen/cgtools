# Format: Scene Object Model

### Scope

- **Purpose**: Define the `Object`/`Layer` schema that is the atomic renderable unit of a scene.
- **Responsibility**: Document the `Object` struct, its `states` map, and the object/state id namespace rules.
- **In Scope**: `Object` fields, `ObjectLayer` fields shared across all layers, state-map semantics, id uniqueness and cross-reference rules.
- **Out of Scope**: Anchor variants (see `format/003`), per-layer sprite selection rules (see `format/005`), per-layer rendering behaviour — tint/blend/effects (see `format/006`).

### Abstract

An `Object` is the declarative template for everything a scene can place: a knight, a wall, a floating damage number, a full-screen weather effect. It never appears in a scene directly — a scene instantiates it at a `Placement` (see `format/003`) to produce a runtime instance. An object's `states` map holds one or more named layer stacks (`"idle"`, `"walk"`, `"default"` — names are user-defined and opaque to the renderer); at most one state is active per instance at a time, and external game logic switches it. A `Layer` is one textured strip within a state's stack, combining a sprite-selection rule (`format/005`) with independent rendering behaviour (`format/006`) — the two are declared as separate fields precisely so any sprite-source/behaviour pairing is valid without special-casing.

### Data Model

`Object`:

| Field | Type | Meaning |
|-------|------|---------|
| `id` | `String` | Unique object id (see Cross-Reference Rules below). |
| `anchor` | `Anchor` | Placement kind — see `format/003`. |
| `global_layer` | `String` | Name of the pipeline z-bucket this object draws into by default (see `format/007`). |
| `priority` | `Option<i32>` | Used by `NeighborPriorityLower` condition comparisons (see `format/005`). |
| `sort_y_source` | `SortYSource` | `Anchor` (default) or `BottomOfShape` — Y-sort key source for `Multihex` instances. |
| `pivot` | `(f32, f32)` | Sprite anchor point within its bounding box, default `(0.5, 0.5)` (centered). Shifts the draw transform's screen position by `-(pivot.x * width, pivot.y * height) * zoom`. **Not present in the original format specification** — a real, source-verified addition (`src/object.rs`) absent from the spec text this doc replaces. |
| `default_state` | `String` | State name active until the game calls `set_state` (default `"default"`). |
| `states` | `HashMap<String, Vec<ObjectLayer>>` | Named layer stacks. |

`ObjectLayer` (one entry in a state's stack):

| Field | Type | Meaning |
|-------|------|---------|
| `id` | `Option<String>` | Optional layer id, informational only. |
| `sprite_source` | `SpriteSource` | Sprite/frame selection rule — see `format/005`. |
| `behaviour` | `LayerBehaviour` | Tint/blend/effects/parallax/alpha — see `format/006`. |
| `z_in_object` | `i32` | Draw order within the state's stack, ascending. |
| `pipeline_layer` | `Option<String>` | Override for which pipeline bucket this one layer draws into; inherits `Object.global_layer` when absent. |

**Design note**: `sprite_source` and `behaviour` are declared as independent fields with no coupling between them — a frame-animated sprite with a static tint, a static sprite with a frame-animated mask, and a frame-animated sprite with a frame-animated mask are all valid without any special-case schema.

### Encoding Structure

In RON, `states` is a map literal (`{ "idle": [ Layer(...), ... ], "walk": [...] }`), not a list — each key is a state name, each value an ordered `Vec<ObjectLayer>` read bottom-to-top by `z_in_object`. `Object.id` and `default_state` are plain strings; `default_state`'s value MUST be a key present in `states` (see `invariant/001`).

**Cross-reference and namespace rules**:

- Every object declared in `RenderSpec.objects[]` MUST have a unique `id` (enforced — see `invariant/001`).
- References from a scene (`tiles[].objects`, `entities[].object`, `edges[].object`, etc. — see `format/008`) MUST resolve to a declared object id.
- State names are scoped per-object — `"idle"` in one object's `states` map is unrelated to `"idle"` in another's.
- The reserved id `"void"` (see `format/003`'s neighbour-condition discussion) MUST NOT be declared as a user object id (enforced — see `invariant/001`).

### Version Compatibility

New `Object`/`ObjectLayer` fields are expected to be additive across minor versions, each defaulting to a value that preserves existing specs' behavior unchanged (e.g. `pivot` defaulting to `(0.5, 0.5)`, `sort_y_source` defaulting to `Anchor`). `pivot` itself is the concrete example already in the shipped implementation: a real field with no formal specification update behind it (see Data Model above) — evidence that this kind of addition is expected to land ahead of, not blocked on, a version bump. A change to `states`' map-of-named-stacks shape, or to what `default_state` means, would be a breaking change requiring a major version bump per `format/008`'s versioning contract.

### APIs

| File | Relationship |
|------|--------------|
| [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) | Runtime spawns/mutates instances of the object classes declared here |

### Formats

| File | Relationship |
|------|--------------|
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | `Object.anchor` selects one of these placement kinds |
| [format/007_render_pipeline.md](../format/007_render_pipeline.md) | `Object.global_layer` / `ObjectLayer.pipeline_layer` reference a declared pipeline bucket |
| [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) | `RenderSpec.objects` element type declared by this doc |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | Object id uniqueness, `default_state` existence, and reserved-id exclusion are enforced here |

### Sources

| File | Relationship |
|------|--------------|
| `src/object.rs` | `Object`, `SortYSource` |
| `src/layer.rs` | `ObjectLayer` (sprite_source/behaviour/z_in_object/pipeline_layer fields) |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_test.rs` | RON round-trip and `RenderSpec` load coverage for the object/state schema |
| `tests/scene_model_compile_test.rs` | Anchor × source × pipeline compile-time coverage exercising object/layer combinations |
| `tests/scene_state_test.rs` | Runtime `set_state`/`default_state` behaviour |

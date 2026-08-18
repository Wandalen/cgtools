# Invariant: RenderSpec Referential Integrity

### Scope

- **Purpose**: State the property that every id reference within a loaded `RenderSpec`/`SceneSnapshot` pair must resolve to a declaration, and how much of that property the loader actually checks today.
- **Responsibility**: Enumerate every referential-integrity rule the format declares (§16-equivalent checklist), and split it into what's enforced today — via `src/validate.rs`'s load-time `Validate` trait or `src/scene.rs`'s cross-file `Scene::from_snapshot` pass — versus the one rule that remains a declared-but-unconstructed `ValidationError` variant.
- **In Scope**: Id uniqueness within each resource/object collection, `*Ref` resolution, `default_state` existence, reserved-id exclusion, pipeline-layer reference resolution, composite-source nesting, anchor↔source compatibility, tiling-strategy whitelist.
- **Out of Scope**: The Edge/Vertex canonical-form uniqueness rule, which is a distinct invariant (see `invariant/002`); render-time missing-sprite semantics — the unset-`External` skip (deliberate leniency) and the hard `CompileError::UnresolvedRef` failures documented in `algorithm/002` — which are runtime behavior, not load-time validation.

### Invariant Statement

For a `RenderSpec` R and any `SceneSnapshot` loaded against it: every id reference anywhere in R or the snapshot resolves to exactly one declaration of the correct kind. Formally, for every reference site listed below, the referenced id MUST appear exactly once among the declarations of its own kind:

- Every `SpriteRef(asset_id, frame)` → a declared `Asset.id`, with `frame` itself resolving per `format/004`'s named-frame/numeric-index lookup.
- Every `TintRef`, `AnimationRef`, `EffectRef` → a declared `Tint.id` / `Animation.id` / `Effect.id`.
- Every `objects[].id`, `assets[].id`, `animations[].id`, `effects[].id`, `tints[].id` is unique within its own collection (no two `Asset`s share an id, independent of whether two `Object`s might).
- Every `NeighborBitmask.connects_with` / `EdgeConnectedBitmask.connects_with` entry is a declared object id or the reserved id `"void"` (see `format/003`).
- Every `PipelineLayer.id` is unique and non-empty; every `Object.global_layer` and `ObjectLayer.pipeline_layer` references a declared `PipelineLayer.id` (see `format/001`, `format/007`).
- For every object: `default_state` is a key present in `states` (see `format/001`); the reserved id `"void"` is never itself declared as an object id.
- Scene-side references (`tiles[].objects`, `entities[].object`, `edges[].object`, `multihex_instances[].object`, `free_instances[].object`, `viewport_instances[].object`) resolve to a declared object id (see `format/008`).
- Composite sources do not nest inside other composite sources (see `format/005`).
- Every sprite source is compatible with the anchor of the object declaring it (see `format/003`'s applicability table).
- `RenderPipeline.hex.tiling` is one of the tiling strategies supported by the loading implementation (see `format/002`).

### Enforcement Mechanism

Two independent mechanisms together enforce nearly every rule in the Invariant Statement above; only one — anchor↔source compatibility — is enforced by neither.

**`Validate::validate()`** (`src/validate.rs`) runs at `RenderSpec::load` / `SceneSnapshot::load` time and — per the format's own "MUST verify and report all violations, not stop at the first" contract — collects every violation into a `Vec<ValidationError>` rather than short-circuiting on the first hit.

`impl Validate for RenderSpec` enforces:

| Enforced | Rule |
|----------|------|
| ✅ | Pipeline-layer reference resolution — both `Object.global_layer` and `ObjectLayer.pipeline_layer` overrides. |
| ✅ | Asset reference resolution — recursive walk over `Static`/`Variant`/`NeighborCondition`/`VertexCorners`/`NeighborBitmask` (`ByMapping` recursively, `ByAtlas` directly)/`EdgeConnectedBitmask`/`ViewportTiled`, stopping at `Animation`/`External` leaves, plus every `AnimationTiming` frame asset. |
| ✅ | `default_state` existence in `states`. |
| ✅ | Reserved id `"void"` not used as a declared object id. |
| ✅ | Id uniqueness within `assets` / `tints` / `animations` / `effects` / `objects` (each its own collection). |
| ✅ | `TintRef` / `AnimationRef` / `EffectRef` resolution, including recursive occurrences inside `Variant` / `NeighborBitmask` / `EdgeConnectedBitmask` / `ViewportTiled`. |
| ✅ | `NeighborBitmask.connects_with` / `EdgeConnectedBitmask.connects_with` entry resolution (declared object id or reserved id `"void"`). |
| ✅ | Composite-in-composite nesting rejection (`IllegalSourceNesting`). |
| ✅ | Tiling whitelist — `pipeline.hex.tiling` restricted to `HexFlatTop`/`HexPointyTop`; `Square4`/`Square8` rejected (`UnsupportedTiling`). |
| ❌ | Anchor↔source compatibility. `AnchorSourceMismatch` is declared in `src/error.rs` but never constructed — `validate.rs` carries an explicit `// TODO SPEC §16` comment explaining the gap is deliberate, not an oversight: `format/003` and `format/005` disagree with each other and with the actual compile-time dispatch on what the rule even is (see `pitfall/001` for the full breakdown), so implementing it against the literal docs would flag `tests/scene_model_compile_test.rs`'s intentionally-passing `vertex_corners_three_way_blend` as invalid. This is the sole rule in this table without a checkmark. |

`impl Validate for SceneSnapshot` enforces two Scene-internal rules:

| Enforced | Rule |
|----------|------|
| ✅ | Tile-source exclusivity — `tiles` and the `(palette, map)` shorthand are mutually exclusive (`ConflictingTileSource`); catches the silent-data-loss case where `Scene::from_snapshot`/`palette_expand` would otherwise drop `map` when both are populated. |
| ✅ | Entity owner bounds — every `entities[*].owner` is a valid index into `players` (`UnresolvedRef { kind: "player", .. }`). |

The three cross-file rules a `SceneSnapshot` alone cannot check — palette-character coverage, scene-side object-id references, `initial_global_tint` resolution — are architecturally out of `Validate::validate(&self)`'s reach: the trait has no way to pass a second `&RenderSpec` document in. `validate.rs` documents this explicitly rather than leaving it unexplained.

**`Scene::from_snapshot`** (`src/scene.rs`) is the separate pass with access to both loaded documents, and is where those three rules are actually enforced instead:

| Enforced | Rule | Error |
|----------|------|-------|
| ✅ | Scene-side references (`tiles[].objects`, `edges[].object`, `multihex_instances[].object`, `free_instances[].object`, `viewport_instances[].object`, `entities[].object`) resolve to a declared object id. | `SnapshotLoadError::UnknownObject` |
| ✅ | `initial_global_tint` (if set) resolves to a declared `Tint.id`. | `SnapshotLoadError::UnknownTint` |
| ✅ | Every ASCII `map` character is present in `palette` (via `SceneSnapshot::palette_expand`, called from `from_snapshot` when `tiles` is empty). | `SnapshotLoadError::UnknownPaletteChar` |

Net result: every rule in the Invariant Statement above is enforced somewhere except anchor↔source compatibility, which remains genuinely open. (`src/source.rs`'s own module doc comment states "composite-inside-composite nesting is an error caught at validation time" — this is now accurate, not aspirational; the `IllegalSourceNesting` check above enforces it.)

### Violation Consequences

The two mechanisms differ in what a violation looks like to a caller, not just in which rules they cover:

- **`Validate::validate()` violations** (`RenderSpec::load` / `SceneSnapshot::load`): returns `Err(LoadError::Validation(errors))`, and `LoadError`'s manual `Display` impl formats every collected violation on its own line rather than surfacing only the first — a caller gets a complete violation list from one load attempt.
- **`Scene::from_snapshot` violations**: returns `Err(SnapshotLoadError)` — a single variant, not a `Vec`. Unlike the collect-everything contract above, `from_snapshot` fails fast on the first cross-file mismatch it encounters (tile/edge/multihex/free/viewport/entity object refs are checked in that order, then `initial_global_tint`) and does not continue checking the rest of the snapshot afterward.

Where a rule is NOT enforced by either mechanism (anchor↔source compatibility only): the malformed spec loads successfully (`Ok`), and the failure surfaces later, at a different layer, in a form that does not name the original schema mistake — e.g. an `External` source on an `Edge`-anchored object fails only at the first `Renderer::render()` call against that scene, as `CompileError::UnsupportedSource`. See `pitfall/001` for the concrete, disclosed trap this creates for spec authors.

### Formats

| File | Relationship |
|------|--------------|
| [format/001_scene_object_model.md](../format/001_scene_object_model.md) | Object id uniqueness, `default_state` existence, reserved-id exclusion |
| [format/002_grid_coordinate_system.md](../format/002_grid_coordinate_system.md) | Tiling-strategy whitelist |
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | `connects_with` reserved-id semantics; anchor↔source compatibility — the one unenforced rule |
| [format/004_declared_resources.md](../format/004_declared_resources.md) | Asset/tint/animation/effect id uniqueness and `*Ref` resolution |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | Composite-source nesting restriction |
| [format/007_render_pipeline.md](../format/007_render_pipeline.md) | Pipeline-layer id uniqueness and reference resolution |
| [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) | Scene-side instance-collection references |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_load_time_validation_partially_enforced.md](../pitfall/001_load_time_validation_partially_enforced.md) | The consumer-facing trap created by this invariant's partial enforcement |

### Sources

| File | Relationship |
|------|--------------|
| `src/validate.rs` | `impl Validate for RenderSpec`, `impl Validate for SceneSnapshot` |
| `src/error.rs` | `ValidationError`, `LoadError`, `SnapshotLoadError` |
| `src/scene.rs` | `Scene::from_snapshot` — the cross-file enforcement pass for the three rules `SceneSnapshot::validate()` cannot reach |
| `src/snapshot.rs` | `SceneSnapshot::palette_expand` — where `UnknownPaletteChar` is actually constructed |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_test.rs` | `validate_rejects_*` / `validate_accepts_*` (14 tests) exercise every enforced `RenderSpec` rule in the table above; `validates_minimal_scene`, `validate_rejects_conflicting_tile_source`, `validate_rejects_owner_out_of_range` cover both enforced `SceneSnapshot` rules |
| `tests/scene_model_compile_test.rs` | `from_snapshot_rejects_unknown_initial_global_tint` / `_accepts_known_initial_global_tint`, `from_snapshot_rejects_unknown_tile_object`, `from_snapshot_rejects_unknown_palette_char` cover the three `Scene::from_snapshot` cross-file rules; `edge_rejects_external_source` pins the one remaining gap — passes `validate()`, then fails at compile (see `pitfall/001`) |

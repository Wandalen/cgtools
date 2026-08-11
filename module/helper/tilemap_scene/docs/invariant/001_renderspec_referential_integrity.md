# Invariant: RenderSpec Referential Integrity

### Scope

- **Purpose**: State the property that every id reference within a loaded `RenderSpec`/`SceneSnapshot` pair must resolve to a declaration, and how much of that property the loader actually checks today.
- **Responsibility**: Enumerate every referential-integrity rule the format declares (§16-equivalent checklist), and split it into what `src/validate.rs` enforces now versus what remains a declared-but-unchecked `ValidationError` variant.
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

`src/validate.rs`'s `impl Validate for RenderSpec` runs at load time and — per the format's own "MUST verify and report all violations, not stop at the first" contract — collects every violation into a `Vec<ValidationError>` rather than short-circuiting on the first hit. As of this migration, it actually implements four of the rules above:

| Enforced | Rule |
|----------|------|
| ✅ | Pipeline-layer reference resolution — both `Object.global_layer` and `ObjectLayer.pipeline_layer` overrides. |
| ✅ | Asset reference resolution — recursive `visit_asset_refs` walk over `Static`/`Variant`/`NeighborCondition`/`VertexCorners`/`NeighborBitmask` (`ByMapping` recursively, `ByAtlas` directly)/`EdgeConnectedBitmask`, stopping at `Animation`/`External` leaves. |
| ✅ | `default_state` existence in `states`. |
| ✅ | Reserved id `"void"` not used as a declared object id. |

The remaining rules above are declared as `ValidationError` variants (`DuplicateId`, `UnresolvedRef`, `IllegalSourceNesting`, `UnsupportedTiling`, `AnchorSourceMismatch`) — see `src/error.rs` — but not yet constructed anywhere in `validate.rs`; each carries an explicit `// TODO SPEC §16` source comment marking it unimplemented: id-uniqueness across the five collections, `TintRef`/`AnimationRef`/`EffectRef` resolution, `NeighborBitmask.connects_with` entry validity, composite-in-composite nesting detection, anchor↔source compatibility, and the tiling whitelist. `impl Validate for SceneSnapshot` is fully unimplemented — it always returns `Ok(())`; all of the scene-side reference checks above are unchecked at load time today. (`src/source.rs`'s own module doc comment asserts composite nesting "is an error caught at validation time" — that claim is aspirational relative to the current `validate.rs`, not a description of present behavior; this doc defers to the source-of-truth code path, not the doc comment, per this migration's verification standard.)

### Violation Consequences

Where a rule IS enforced: `RenderSpec::load` returns `Err(LoadError::Validation(errors))`, and `LoadError`'s manual `Display` impl formats every collected violation on its own line rather than surfacing only the first — callers get a complete violation list from one load attempt.

Where a rule is NOT yet enforced: the malformed spec loads successfully (`Ok`), and the failure surfaces later, at a different layer, in a form that does not name the original schema mistake — e.g. an unresolved `TintRef` fails only if and when a draw call actually samples it, and a nested composite source's behavior at that point is unspecified rather than rejected. See `pitfall/001` for the concrete, disclosed trap this creates for spec authors.

### Formats

| File | Relationship |
|------|--------------|
| [format/001_scene_object_model.md](../format/001_scene_object_model.md) | Object id uniqueness, `default_state` existence, reserved-id exclusion |
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
| `src/error.rs` | `ValidationError`, `LoadError` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_test.rs` | Exercises `RenderSpec::load`'s `Validate` path, including the four enforced rules |

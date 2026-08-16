# Layer: L4 Scene Model

Declarative scene data: serializable, validate-able descriptions of *what
exists* — with no rendering code and no GPU dependency. An L4 model can be
loaded, inspected, diffed, and validated entirely off-GPU; only handing it
to an L3/L5 consumer produces pixels.

### Scope

- **Purpose**: Define the scene-model layer's role and record which stacks have one today.
- **Responsibility**: Name the model formats per stack and their no-rendering-code boundary.
- **In Scope**: `tilemap_scene`'s RON model (tile stack); glTF as the de facto d3 model.
- **Out of Scope**: Executing/compiling the model into frames (see [006_l5_scene_script_and_runners.md](006_l5_scene_script_and_runners.md)); the engines consuming the result (see [004_l3_stack_engine.md](004_l3_stack_engine.md)).

### Role

- **Declarative**: describes entities, not draw calls.
- **Serializable**: a file format (RON, glTF) is the canonical form, not an
  in-memory object graph.
- **Validate-able without a GPU**: model loading and validation must work
  headless — this is what makes L4 testable and toolable. This is the
  invariant `tilemap_scene` meets; the d3 occupant below does not (its
  loader requires a live GPU context just to parse).

### Occupants per Stack

| Stack | Model | State |
|-------|-------|-------|
| tile | `tilemap_scene`'s RON scene model (`RenderSpec`/`SceneSnapshot` — layers, palettes, variants; not the in-memory `Scene` runtime graph, which has no `Serialize`/`Deserialize` derive, see Sources below) | ✅ Dedicated crate; GPU-free by dependency surface once task 117 lands — currently `tiles_tools` pulls in `minwebgl` transitively through its default-on but unused `animation` feature ([`tilemap_scene` invariant/003](../../module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md)) |
| d2 (general) | None dedicated — content arrives as direct `tilemap_renderer` commands or via `scene_script` | 🔄 Gap accepted; no committed need yet |
| d3 | glTF, consumed through `renderer`'s loaders | 🔄 De facto: the format is standard, but there is no cgtools-owned model crate wrapping it; unlike `tilemap_scene`, its `load()` requires a live `WebGl2RenderingContext` to parse — not off-GPU-validatable |

`d3_scene` (`module/blank/d3_scene/`) reserves the slot for a d3-owned
scene model + script, gated on a committed scene-file requirement.

### Layers

| File | Relationship |
|------|--------------|
| [004_l3_stack_engine.md](004_l3_stack_engine.md) | The layer that renders what L4 describes |
| [006_l5_scene_script_and_runners.md](006_l5_scene_script_and_runners.md) | The layer that compiles/executes L4 models over time |

### Sources

| File | Relationship |
|------|--------------|
| `module/blank/d3_scene/` | Reserved d3 scene-layer slot |
| `module/helper/renderer/src/webgl/loaders/gltf.rs` | glTF ingestion — the de facto d3 model boundary |
| `module/helper/renderer/src/webgl/animation/loaders/gltf.rs` | Animation-specific glTF ingestion, alongside the main loader above |
| `module/helper/tilemap_scene/src/spec.rs` + `src/snapshot.rs` | The tile stack's declarative model ( `RenderSpec` / `SceneSnapshot`, RON-deserializable ) — not `scene.rs`, which is the runtime/retained-mode counterpart with no `Serialize`/`Deserialize` derive |

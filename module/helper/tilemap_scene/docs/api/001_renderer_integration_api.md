# API: Renderer Integration API

### Scope

- **Purpose**: Document the runtime contract a game uses to drive a loaded scene and render it — the programmatic surface, not the file format.
- **Responsibility**: Document `Scene`'s mutation/query methods, `Renderer::render`'s entry point, and `Camera` — as actually implemented, correcting three points where the original specification's contract differs from the shipped API.
- **In Scope**: Instance lifecycle (`spawn`/`despawn`), state/tint/sprite mutation, the per-frame `tick`/`render` cycle, `Camera`, error/fallback semantics for a stale or unknown handle.
- **Out of Scope**: The declarative schema these operations act on (see `format/`); the internal rendering algorithm `render()` triggers (see `algorithm/002`).

### Abstract

Game code drives a `Scene` — an in-memory, mutable instantiation of a loaded `RenderSpec`/`SceneSnapshot` pair (see `format/008`) — through a small set of methods: spawn and despawn instances, switch an instance's active state, override its tint or an `External` sprite slot, and advance time via `tick`. A separate `Renderer` consumes a `&Scene` plus a `&Camera` once per frame to produce draw commands; the renderer is stateless-with-cache (see `algorithm/002`) rather than owning scene state itself. The renderer treats instance state as opaque and game-owned — it never invents states or spawns objects on its own initiative.

### Operations

| Operation | Signature (conceptual) | Purpose |
|-----------|--------------------------|---------|
| `Scene::from_snapshot` / `Scene::new` | `(RenderSpec, SceneSnapshot) -> Scene` / `(RenderSpec) -> Scene` | Construct a scene from a loaded snapshot, or an empty one for runtime-only spawning. |
| `spawn` | `(object_id, placement) -> InstanceHandle` | Placement payload is anchor-specific (see `format/003`): one grid coord for `Hex`, a `(hex, direction)` pair for `Edge`, a pixel point for `FreePos`, nothing for `Viewport`. |
| `despawn` | `(instance)` | Removes the instance from the scene. |
| `object` / `state` | `(object_id) -> Option<ObjectHandle>` / `(object, name) -> Option<StateHandle>` | Resolve a declared object id / state name into a typed handle — the lookup step that precedes `set_state`. |
| `set_state` | `(instance, StateHandle)` | Switches the instance's active state. Takes a pre-resolved **typed** `StateHandle`, not a bare string — see Error Handling below for what happens on a mismatched handle. |
| `set_visible`, `set_tint`, `set_phase_offset`, `set_seed` | per-instance / per-scene overrides | Runtime overrides layered on top of the declarative defaults. |
| `set_external_sprite` | `(instance, slot, SpriteRef)` | Populates an `External` sprite-source slot (see `format/005`). |
| `set_global_tint` | `(TintRef \| None)` | Changes the pipeline-level tint at runtime (e.g. day/night cycle). |
| `move_to` | `(instance, placement)` | Repositions an existing instance. |
| `tick` / `tick_into` | `(dt) -> Vec<SceneEvent>` | Advances the shared clock; returns events including `OneShot` animation completions (see `algorithm/001`). |
| `instances`, `instances_at_hex`, `hex_instances`, `edge_instances`, `free_instances`, `viewport_instances`, `multihex_instances`, `len`, `is_empty` | queries | Read-only instance enumeration, whole-scene or filtered by anchor kind / position. |
| `Renderer::render` | `(&mut self, &Scene, &Camera) -> Result<&[RenderCommand], CompileError>` | Produces this frame's draw commands (see `algorithm/002`); cached and replayed on an unchanged `(scene_revision, clock, camera_signature)` triple. |
| `Camera::project` | `(world: (f32, f32)) -> (f32, f32)` | `screen = (world - camera.world_center) * camera.zoom + viewport_size / 2`. |

### Error Handling

**Divergences from the original specification's documented contract**, verified directly against `src/scene.rs`/`src/compile/camera.rs`/`src/compile/frame.rs` rather than transcribed from the (in these four respects, stale) spec text:

1. **No `set_camera` method exists on `Scene`.** The original specification described `set_camera(world_center, zoom)` as a `Scene` mutator used for culling/parallax. The actual design keeps `Camera` as an independent struct (`world_center`, `zoom`, `viewport_size`, with a `Default` impl at the origin, zoom `1.0`, `800×600`) constructed and passed per-call into `Renderer::render(&scene, &camera)` — camera state is renderer-side, not scene-side. A caller wanting to "move the camera" constructs a new `Camera` value for the next `render` call rather than mutating the scene.
2. **`set_state` takes a typed `StateHandle`, and does not perform the specification's documented warn-and-fall-back behavior on a stale handle.** The original specification's contract for `set_state(instance, name)` states: on an unrecognized `name`, log a warning, fall back to `default_state`, and continue rendering. The actual API splits this into two steps — `Scene::state(object, name) -> Option<StateHandle>` is where an unknown *name* fails (returns `None`; the "unknown name" case has moved to this lookup call, not `set_state` itself) — and `set_state(instance, handle)` itself takes an already-resolved `StateHandle`. If that handle's object doesn't match the instance's actual object (a stale handle from a different object), release builds silently no-op — no state change, no warning logged, no fallback to `default_state` — while debug builds hit a `debug_assert_eq!` panic. A caller relying on the specification's described warn-and-fallback safety net for this specific mismatch case will not get one in a release build.
3. **`RenderSpec.version` is not checked** against the loading implementation's supported range at any API entry point (`RenderSpec::load`, `Scene::from_snapshot`) — see `invariant/001`'s Version Compatibility discussion for the full disclosure; noted here because it means no API call in this surface can fail specifically due to a version mismatch, contrary to the specification's normative MUST.
4. **Missing-sprite / missing-asset failures at render time surface as `Err(CompileError::UnresolvedRef)` from `render()` itself, not as the specification's warn-and-placeholder path** — which is unimplemented (see `algorithm/002`'s Missing-sprite handling and `roadmap.md`'s `External` sprite-source item). The single exception is an unset `External` slot, which silently skips that layer's emit for the frame instead of erroring. `render()`'s `Result<_, CompileError>` therefore covers *both* structural failures (e.g. `CompileError::UnsupportedAnchor` for a `Square4`/`Square8` tiling spec that passed load-time validation unchecked, see `pitfall/001`) *and* per-sprite resolution failures.

### Compatibility Guarantees

The renderer treats instance state as opaque and game-owned by design — it never invents states or spawns objects, so a caller's own state machine is never second-guessed. Placement payload shape is anchor-specific and is expected to grow only additively as new anchor variants are added (see `format/003`'s Version Compatibility). Beyond the four divergences disclosed above, the operations in this doc are the actual, current runtime contract — not a forward-looking or aspirational one.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_animation_phase_and_frame_selection.md](../algorithm/001_animation_phase_and_frame_selection.md) | `tick`'s returned `SceneEvent`s include `OneShot` completion |
| [algorithm/002_scene_rendering_pass.md](../algorithm/002_scene_rendering_pass.md) | `Renderer::render` triggers this algorithm; `Camera::project` feeds its culling/screen-position step |

### Formats

| File | Relationship |
|------|--------------|
| [format/001_scene_object_model.md](../format/001_scene_object_model.md) | `object`/`state`/`set_state` operate on the object/state schema declared there |
| [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) | `spawn`'s `placement` payload shape |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | `set_external_sprite` populates an `External` source slot |
| [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) | `from_snapshot` consumes this structure |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_load_time_validation_partially_enforced.md](../pitfall/001_load_time_validation_partially_enforced.md) | `CompileError::UnsupportedAnchor` is where the Square-tiling gap actually surfaces to an API caller |

### Sources

| File | Relationship |
|------|--------------|
| `src/scene.rs` | `Scene` — full method surface |
| `src/renderer.rs` | `Renderer::render`, `cache_hits`, `cleanup`, `assets` |
| `src/compile/camera.rs` | `Camera`, `Camera::project` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_state_test.rs` | `spawn`/`despawn`/`set_state`/handle-resolution coverage |
| `tests/scene_events_test.rs` | `tick`/`SceneEvent` coverage |
| `tests/renderer_test.rs` | `Renderer::render` end-to-end coverage |
| `tests/catalog_test.rs` | `Scene::catalog()` coverage |

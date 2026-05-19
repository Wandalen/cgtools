# tilemap_scene

Compositional declarative scene format for 2D tile-based games.

`tilemap_scene` defines a serde-compatible representation of a *render spec*
(asset / object / pipeline declarations) and a retained-mode `Scene` (which
instances sit where on a grid, plus camera / seed / metadata) and a
`Renderer` that walks the scene each frame and emits a flat stream of
[`tilemap_renderer`]'s `RenderCommand`s. The format itself is normative in
`spec.md`; this crate is the reference implementation.

The runtime architecture is **Path A** — `Scene` owns the retained
render-world (mutated through typed `InstanceHandle`s), `Renderer` is a
stateless-with-cache algorithm that produces a `RenderCommand` stream
each frame. Multiple `Renderer`s can drive the same `Scene` into
different backends (e.g. WebGL + a headless test backend) without state
crosstalk.

## directory layout

| Path | Responsibility |
|------|----------------|
| `src/lib.rs` | Crate entry point — `mod_interface!` re-exports each top-level layer. No logic of its own. |
| `src/spec.rs` | `RenderSpec` — the top-level container for asset / tint / animation / effect / object / pipeline declarations. |
| `src/scene.rs` | `Scene` — the retained-mode render-world: per-anchor instance lists, spatial indexes, mutation / query API, monotonic `revision()` counter, `tick(dt) -> Vec<SceneEvent>`. |
| `src/instance.rs` | `InstanceHandle` / `ObjectHandle` / `StateHandle` — typed slotmap keys; `Placement` enum (`Hex` / `Edge` / `FreePos` / `Viewport` / `Multihex`); `Instance` runtime payload with per-instance overrides (tint, phase offset, visibility, external sprites). |
| `src/snapshot.rs` | `SceneSnapshot` — RON-deserialisable scene data (tiles / edges / multihex / free / viewport instances); bridged into a runtime `Scene` via `Scene::from_snapshot`. |
| `src/event.rs` | `SceneEvent` — observations emitted by `Scene::tick`; currently `AnimationCompleted { instance, state, layer_index, animation }`. |
| `src/renderer.rs` | `Renderer` — owns `CompiledAssets`, allocates GPU batches lazily, carries a per-frame cache that serves idle frames without re-walking the scene. |
| `src/object.rs` | `Object` — a renderable class with an `Anchor` and named state stacks. |
| `src/anchor.rs` | `Anchor` enum — hex / edge / vertex / multihex / world-pixel / viewport. Decides what "position" means for an object and what neighbour context its layers see. |
| `src/layer.rs` | `ObjectLayer` + `LayerBehaviour` — one textured strip with tint / blend / effects / parallax. |
| `src/source.rs` | `SpriteSource` — every way a layer can pick a sprite each frame: `Static`, `Variant`, `Animation`, `NeighborBitmask`, `NeighborCondition`, `VertexCorners`, `EdgeConnectedBitmask`, `ViewportTiled`, `External`. |
| `src/resource.rs` | Asset / tint / animation / effect resource declarations and `*Ref` wrappers. Re-exports `tilemap_renderer::types::BlendMode`. |
| `src/pipeline.rs` | `RenderPipeline` — pipeline buckets, sort modes, hex tiling configuration. |
| `src/coords.rs` | Public coordinate types and helpers used by the format (axial / cube / world-pixel pairs). |
| `src/hash.rs` | `hash_coord` / `hash_str` — normative hash primitives (SPEC §13) used for `HashCoord` variant selection and animation phase offsets. |
| `src/load.rs` | `RenderSpec::load` / `SceneSnapshot::load` and `from_ron_str` counterparts. |
| `src/validate.rs` | `Validate` trait + skeleton impls for `RenderSpec` and `SceneSnapshot`. SPEC §16 rules are not yet enforced — see the trait-level note. |
| `src/error.rs` | `LoadError`, `ValidationError`, `SnapshotLoadError`. |
| `src/compile/` | Internal lowering passes called by `Renderer`. See sub-table. |
| `tests/` | Integration tests — `scene_state_test`, `scene_events_test`, `renderer_test`, `renderer_cache_test`, `scene_model_compile_test`, `scene_model_test`. `tests/common/mod.rs` carries the shared `flatten_to_sprites` / `BatchFlattener` helpers used to project batch streams back to pre-batch `Sprite` commands for assertions. |
| `spec.md` | Normative format specification (v0.2.0). |
| `roadmap.md` | Open work and design sketches. |

### `src/compile/` sub-layer

| Path | Responsibility |
|------|----------------|
| `mod.rs` | Layer entry point; documents the two boundaries the compile layer owns (Y-down → Y-up flip, asset resolution). |
| `assets.rs` | `compile_assets` + `CompiledAssets` — turns asset declarations into `tilemap_renderer::assets::Assets` and pre-allocates every sprite reachable from any source / animation. Called once by `Renderer::new`. |
| `frame.rs` | `gather_frame_emits` — per-tick lowering that walks objects × layers × instances and returns structured per-bucket `Sprite` / screen-space lists; consumed by `Renderer::render` (which batches sprites for `SortMode::None` buckets and emits per-sprite for sorted buckets). `render_into` is a thin compatibility wrapper that flattens emits to the legacy per-sprite stream. |
| `camera.rs` | `Camera` — world-pixel → viewport-pixel projection used by the frame pass. |
| `coords.rs` | Hex-axial → world-pixel (Y-up) helpers; the single Y-axis flip from `tiles_tools` lives here. |
| `viewport.rs` | `viewport_transform` / `tiled_positions` — screen-space transforms for `ViewportTiled` sources, Y-up convention. |
| `edges.rs` | Edge-anchor canonicalisation, neighbour resolution, world-pixel placement, sprite rotation. |
| `vertex.rs` | Vertex-corner pattern resolution for `VertexCorners` sources. |
| `neighbors.rs` | Hex-anchor neighbour mask computation feeding `NeighborBitmask` / `NeighborCondition`. |
| `conditions.rs` | `NeighborCondition` rule evaluation. |
| `animation.rs` | `resolve_animation_frame` — deterministic per-tile frame pick given timing + phase offset. Also exposes `animation_duration_seconds` / `declared_phase_seconds` consumed by `Scene::tick`. |
| `ids.rs` | `IdMap` — deterministic allocator from string ids to `tilemap_renderer` `ResourceId`s. |
| `resolver.rs` | `AssetResolver` trait (the "where do bytes come from" boundary) and the default `PathResolver`. |
| `error.rs` | `CompileError` — every way the per-frame walk can fail. |

## usage

```toml
[dependencies]
tilemap_scene = { path = "../tilemap_scene" }
tilemap_renderer = { path = "../tilemap_renderer", features = [ "scene-model" ] }
```

### Spawn-driven scene construction

```rust,ignore
use std::sync::Arc;
use tilemap_scene::{ Camera, PathResolver, Placement, Renderer, RenderSpec, Scene };

let spec : RenderSpec = RenderSpec::load( "render_spec.ron" )?;
let mut renderer = Renderer::new( &spec, &PathResolver )?;
backend.load_assets( renderer.assets() );

let mut scene = Scene::new( Arc::new( spec ) );
let grass = scene.object( "grass" ).expect( "grass declared" );
scene.spawn( grass, Placement::Hex { q : 0, r : 0 } );
scene.spawn( grass, Placement::Hex { q : 1, r : 0 } );

// Per-frame:
let _events = scene.tick( frame_dt );          // advance clock, harvest OneShot completions
let cmds = renderer.render( &scene, &camera )?; // idle frames replay from cache
backend.submit( cmds );
```

### Loading from a RON `Scene` snapshot

```rust,ignore
use tilemap_scene::{ Scene, SceneSnapshot };

let snap : SceneSnapshot = SceneSnapshot::load( "scene.ron" )?;
let mut scene = Scene::from_snapshot( &snap, Arc::new( spec ) )?;
// Continue mutating via the runtime API.
```

### Per-instance overrides

```rust,ignore
let h = scene.spawn( knight, Placement::Hex { q : 4, r : 2 } );
scene.set_tint( h, Some( [ 0.8, 0.8, 1.0, 1.0 ] ) );      // moonlight tint
scene.set_phase_offset( h, Some( -scene.clock() ) );      // play OneShot from frame 0 now
scene.set_visible( h, false );                            // hide without de-spawn
scene.set_external_sprite( h, "body", body_sprite );      // populate External source slot
```

### Animation completion events

```rust,ignore
for event in scene.tick( dt )
{
  match event
  {
    tilemap_scene::SceneEvent::AnimationCompleted { instance, animation, .. } =>
    {
      // e.g. fire gameplay logic for "attack swing finished" or "spawn-in done".
    },
    _ => {},
  }
}
```

## related documents

- `spec.md` — normative format specification, including the SPEC §16 validation rule set.
- `roadmap.md` — open work, design sketches, and known gaps.
- repo-root `rulebook.md` — workspace-wide lint / style / test policy.
- `tilemap_renderer/readme.md` — the renderer this crate compiles into; co-evolves with this format.

# tilemap_scene

Compositional declarative scene format for 2D tile-based games.

`tilemap_scene` defines a serde-compatible representation of a *render spec*
(asset / object / pipeline declarations) and a *scene* (which objects sit
where on a grid, plus camera / seed / metadata) and a compile layer that
lowers the pair into a flat stream of [`tilemap_renderer`]'s
`RenderCommand`s. The format itself is normative in `spec.md`; this crate
is the reference implementation.

## directory layout

| Path | Responsibility |
|------|----------------|
| `src/lib.rs` | Crate entry point — `mod_interface!` re-exports each top-level layer. No logic of its own. |
| `src/spec.rs` | `RenderSpec` — the top-level container for asset / tint / animation / effect / object / pipeline declarations. |
| `src/scene.rs` | `Scene` — the runtime state (tile placements, entities, viewport instances, scene seed, bounds). |
| `src/object.rs` | `Object` — a renderable class with an `Anchor` and named state stacks. |
| `src/anchor.rs` | `Anchor` enum — hex / edge / vertex / multihex / world-pixel / viewport. Decides what "position" means for an object and what neighbour context its layers see. |
| `src/layer.rs` | `ObjectLayer` + `LayerBehaviour` — one textured strip with tint / blend / effects / parallax. |
| `src/source.rs` | `SpriteSource` — every way a layer can pick a sprite each frame: `Static`, `Variant`, `Animation`, `NeighborBitmask`, `NeighborCondition`, `VertexCorners`, `EdgeConnectedBitmask`, `ViewportTiled`, `External`. |
| `src/resource.rs` | Asset / tint / animation / effect resource declarations and `*Ref` wrappers. Re-exports `tilemap_renderer::types::BlendMode`. |
| `src/pipeline.rs` | `RenderPipeline` — pipeline buckets, sort modes, hex tiling configuration. |
| `src/coords.rs` | Public coordinate types and helpers used by the format (axial / cube / world-pixel pairs). |
| `src/hash.rs` | `hash_coord` / `hash_str` — normative hash primitives (SPEC §13) used for `HashCoord` variant selection and animation phase offsets. |
| `src/load.rs` | `RenderSpec::load` / `Scene::load` and `from_ron_str` counterparts. |
| `src/validate.rs` | `Validate` trait + skeleton impls for `RenderSpec` and `Scene`. SPEC §16 rules are not yet enforced — see the trait-level note. |
| `src/error.rs` | `LoadError` and `ValidationError`. |
| `src/compile/` | Phase-2 lowering from `(RenderSpec, Scene, Camera, time)` to a `Vec<RenderCommand>`. See sub-table. |
| `tests/` | Integration tests — `scene_model_test.rs` (parsing / serde round-trip / loader API) and `scene_model_compile_test.rs` (compile-pipeline behaviour by Slice). |
| `spec.md` | Normative format specification (v0.2.0). |
| `roadmap.md` | Open work and design sketches. |

### `src/compile/` sub-layer

| Path | Responsibility |
|------|----------------|
| `mod.rs` | Layer entry point; documents the two boundaries the compile layer owns (Y-down → Y-up flip, asset resolution). |
| `assets.rs` | `compile_assets` + `CompiledAssets` — turns asset declarations into `tilemap_renderer::assets::Assets` and pre-allocates every sprite reachable from any source / animation. |
| `frame.rs` | `compile_frame` — the per-tick lowering that walks objects × layers × instances and emits `RenderCommand`s in pipeline-bucket order. |
| `camera.rs` | `Camera` — world-pixel → viewport-pixel projection used by the frame pass. |
| `coords.rs` | Hex-axial → world-pixel (Y-up) helpers; the single Y-axis flip from `tiles_tools` lives here. |
| `viewport.rs` | `viewport_transform` / `tiled_positions` — screen-space transforms for `ViewportTiled` sources, Y-up convention. |
| `edges.rs` | Edge-anchor canonicalisation, neighbour resolution, world-pixel placement, sprite rotation. |
| `vertex.rs` | Vertex-corner pattern resolution for `VertexCorners` sources. |
| `neighbors.rs` | Hex-anchor neighbour mask computation feeding `NeighborBitmask` / `NeighborCondition`. |
| `conditions.rs` | `NeighborCondition` rule evaluation. |
| `animation.rs` | `resolve_animation_frame` — deterministic per-tile frame pick given timing + phase offset. |
| `ids.rs` | `IdMap` — deterministic allocator from string ids to `tilemap_renderer` `ResourceId`s. |
| `resolver.rs` | `AssetResolver` trait (the "where do bytes come from" boundary) and the default `PathResolver`. |
| `error.rs` | `CompileError` — every way compile can fail. |

## usage

```toml
[dependencies]
tilemap_scene = { path = "../tilemap_scene" }
tilemap_renderer = { path = "../tilemap_renderer", features = [ "scene-model" ] }
```

```rust,ignore
use tilemap_scene::
{
  compile_assets, compile_frame, Camera, PathResolver, RenderSpec, Scene,
};

let spec  : RenderSpec = RenderSpec::load( "render_spec.ron" )?;
let scene : Scene      = Scene::load( "scene.ron" )?;

let compiled = compile_assets( &spec, &PathResolver )?;
let commands = compile_frame( &spec, &scene, &compiled, &Camera::default(), 0.0 )?;

// Submit `commands` to any tilemap_renderer backend (WebGL2 / SVG / terminal).
```

## related documents

- `spec.md` — normative format specification, including the SPEC §16 validation rule set.
- `roadmap.md` — open work, design sketches, and known gaps.
- repo-root `rulebook.md` — workspace-wide lint / style / test policy.
- `tilemap_renderer/readme.md` — the renderer this crate compiles into; co-evolves with this format.

# tilemap_scene — development roadmap

- **Project:** Compositional declarative scene format for 2D tile-based games
- **Version:** 0.1.0
- **Status:** Active development — Slice 4 shipped, Slice 5 pending

## Current state

The compile-to-commands pipeline (`compile_assets` + `compile_frame`) is
stateless and covers the core vocabulary end-to-end. Used successfully by
the `examples/minwebgl/slay_map` WebGL demo.

### Shipped

**Primitives**

- `Object` with `Anchor::{ Hex, Edge, FreePos, Viewport }`
- `ObjectLayer` with `LayerBehaviour` (tint / blend / z-in-object / per-layer pipeline-bucket override)
- Named `states` map per object (replaces the earlier "animations" nomenclature — states can carry any `SpriteSource`, not just animations)

**Sprite sources**

- `Static(SpriteRef)` — one fixed frame
- `Variant { variants, selection }` — weighted list; `VariantSelection::{ HashCoord, Fixed, Random }`, Random seeded via `Scene.seed`
- `Animation(AnimationRef)` — `AnimationTiming::{ Regular, FromSheet, Irregular }`; `AnimationMode::{ Loop, PingPong, OneShot }`; `PhaseOffset::{ None, Fixed, HashCoord, Linear }`
- `NeighborBitmask` — 6-bit hex neighbour autotile; `ByMapping` (mask → leaf source with fallback) and `ByAtlas { layout: Bitmask6 }` (numeric frame lookup, 64 entries pre-allocated)
- `NeighborCondition` — per-side conditional emission; conditions: `NeighborIs`, `NoNeighbor`, `NeighborPriorityLower`, `AnyOf`, `AllOf`, `Not`; `{dir}` pattern substitution; handles skirts and Wesnoth-style edge blends
- `VertexCorners` — dual-mesh triangle blending; wildcard (`"*"`) matching; specificity → priority → declaration-order tiebreak per SPEC §9
- `EdgeConnectedBitmask` — 4-bit edge-endpoint autotile for rivers / edge roads; `ByMapping` + `ByAtlas { layout: EdgeHex }` (16 entries pre-allocated); edge canonicalisation so both-side declarations dedupe
- `ViewportTiled` — `Center`, `Stretch`, `Fit` (single `ScreenSpaceSprite`); `Repeat2D`, `RepeatX`, `RepeatY` (N sprites covering the viewport at camera-zoom scale)

**Asset kinds**

- `AssetKind::Atlas` — grid atlas with `tile_size` / `columns` / `origin` / `gap`, plus named `frames` and explicit per-frame `frame_rects` (pixel rect + optional anchor)
- `AssetKind::Single { size }` — whole-image-one-sprite

**Pipeline**

- Buckets with `SortMode::{ None, XAsc, XDesc, YAsc, YDesc, XAscYDesc, XAscYAsc, YDescXAsc, YAscXAsc }`
- Per-layer pipeline-bucket override via `ObjectLayer.pipeline_layer`
- `RenderPipeline.clear_color` (linear RGBA; `None` = transparent-black)
- `RenderPipeline.global_tint` (composition — lerp(white, color, strength) multiplied into every emitted sprite)

**Other infrastructure**

- `Camera` with translate + uniform zoom; `viewport_size` source precedence `pipeline.viewport_size` → `camera.viewport_size`
- `Scene.seed: Option<u64>` — folds to `u32` salt for `hash_coord`; deterministic across frames
- `FrameSpec::anchor` — per-frame pixel anchor, overrides `Object.pivot` when set; threaded via `CompiledAssets.sprite_anchors`
- RON + serde loader (`RenderSpec::load`, `Scene::load`) with validation hooks
- `ScreenSpaceSprite` command (implemented end-to-end in the WebGL adapter; SVG stubs)

**Test baseline:** 80 passing — 35 unit (hash, ids, camera, coords, animation, neighbors, conditions, vertex, edges, viewport), 35 integration compile (anchor × source coverage), 10 integration serde round-trip.

## Polish items — pick-by-need, no fixed slice

These are small-to-medium-size and independent. Implement when a real
game use-case demands one.

1. **`TintBehaviour::Flat` / `Masked` + `TeamColor` resolution.** Per-layer
   tint composition against `Scene.players[i].color` for team-coloured
   units. Medium. Touches `frame.rs` (`Sprite.tint` composition pass) and
   adds a small resolver helper.
2. **`Effects` (`VertexDisplace` / `AlphaPulse` / `ColorShift`).** Compile
   layer just passes effect references through; real work is adapter-side
   shader support. Largely blocked on backend. Consider dropping the variants
   entirely if no game asks — they're declared but not plumbed.
3. **`Validate` rule implementation.** `validate.rs` has TODO-comments for
   every SPEC §16 rule (unresolved refs, illegal source nesting, anchor ↔
   source compatibility, default_state existence, reserved ids, tiling
   whitelist, duplicate-id checks). Each rule is a ~10-line method.
   Tedious but high-value for catching bad specs at load time.
4. **`External` sprite source runtime plumbing.** Tied to the stateful
   renderer (Slice 5) — `set_sprite(instance, slot, sprite_ref)` populates
   an internal map that `compile_frame` looks up. Without Slice 5, the
   External source just emits nothing.
5. **`AssetKind::SpriteSheet` support.** Currently rejected at compile with
   `UnsupportedAssetKind`. Useful shorthand for horizontal / vertical /
   grid sprite sheets that an atlas already covers — optional.
6. **SVG adapter for `ScreenSpaceSprite`.** One-line dispatch (SVG already
   works in screen-space coordinates; compile pre-projects). Blocked only
   by style preference — kept as explicit stub during Slice 4 per user
   direction.
7. **`ViewportTiled::Repeat*` via `Mesh` command.** Current implementation
   emits N `ScreenSpaceSprite`s to cover the viewport. Efficient for small
   tile counts (≤ 16); for 256×256 textures on 4K viewports a single
   `Mesh` with `wrap=Repeat` UVs and a viewport-sized quad is ~100× fewer
   draws. Needs screen-space `Mesh` command (or flag on existing `Mesh`).
8. **World-anchored tiled background.** Current `ViewportTiled::Repeat2D`
   pins to the viewport (doesn't pan with camera). A game map that "floats
   on sea" wants the opposite — infinite tiled world under the hex grid.
   Needs a new anchor (e.g. `Anchor::WorldTiled { grid_step }`) or a
   `world_space: bool` flag on `ViewportTiled`. Design choice pending.
9. **`Anchor::Multihex`.** Declared but rejected. Needs: pixel position
   from anchor hex + configurable Y-sort source (`SortYSource::Anchor` vs
   `BottomOfShape`), culling check against the shape cells, and the
   restriction that the sprite source is `Static` / `Variant` / `Animation`
   (no neighbour-aware sources on multihex).
10. **Square tilings (`Square4` / `Square8`).** Enum values exist, rejected
    at load. Implementing means square-grid neighbour offsets (4 or 8),
    square-grid dual-mesh (4 corners per vertex), square pixel conversion.
    Scope inflation — only do if a square-grid game is actually planned.
11. **`HexConfig::from_hex_size` bounding-box helper.** `HexConfig::grid_stride`
    is the pixel spacing between adjacent-cell centres. For equilateral hex
    sprites this equals the sprite bounding box; for stylised sprites (e.g. the
    Slay atlas) the two diverge and callers must tune `grid_stride` empirically.
    A helper `HexConfig::from_hex_size(w, h)` that computes the equilateral-hex
    stride (`(w * 0.75, h)` for flat-top, `(w, h * 0.75)` for pointy-top) would
    remove the friction for authors who have a bounding box. Also consider an
    explicit **stride override** field for pixel-art hexes tuned away from exact
    `sqrt(3)/2` ratios. Low urgency.
12. ~~**🐛 BlendMode propagation in compile/frame.rs.**~~ *Fixed.* All 7 construction
    sites in `compile/frame.rs` now use `layer.behaviour.blend` instead of
    `BlendMode::default()`.

## Slice 5 — stateful `Renderer` + runtime mutation API

Goal: move from stateless `compile_frame` (rebuild-every-tick) to a
`Renderer` struct that tracks instances across frames. Unlocks runtime
API, sprite-batch optimisation, and animation completion callbacks.

### `Renderer` shape

```rust
pub struct Renderer {
    spec: RenderSpec,
    compiled: CompiledAssets,
    batches: HashMap<String, SpriteBatchHandle>,         // per pipeline bucket
    instances: IdGen<InstanceHandle>,                    // spawn-returned IDs
    instance_to_batch_index: HashMap<InstanceHandle, (BatchId, u32)>,
    clock: f32,
    camera: Camera,
    global_tint_override: Option<TintRef>,
    external_sprites: HashMap<(InstanceHandle, String), SpriteRef>,   // for External source
}
```

### Runtime API (per SPEC §14)

- `spawn(object_id, placement: Placement) -> InstanceHandle` — `Placement`
  is anchor-specific: `Hex(q, r)`, `Edge(hex, dir)`, `FreePos(x, y)`,
  `Viewport`, `Multihex(anchor)`.
- `despawn(instance)` — swap-remove from batches; the last-slot instance's
  index mapping also updates on swap.
- `set_state(instance, name)` — switches the active state of an instance.
- `set_sprite(instance, slot, SpriteRef)` — populates `External`.
- `set_global_tint(Option<TintRef>)` — runtime day/night.
- `set_camera(world_center, zoom)` — simple update.

### SpriteBatch migration

Replace per-tile `RenderCommand::Sprite` with one `CreateSpriteBatch` per
pipeline bucket + `AddSpriteInstance` per instance. First frame creates the
batches; subsequent frames only issue `SetSpriteInstance` for
moved / re-stated entities. One `DrawBatch` per bucket at the end of the
frame — usually 100× fewer commands than the stateless path.

Tracking: `instance_to_batch_index` maps `InstanceHandle → (batch_id, slot)`.
Swap-remove: when slot `k` in a batch is deleted, the last slot (`n-1`)
moves to `k`; we find its `InstanceHandle` and update its entry too.

### OneShot completion callbacks

OneShot animations currently render the last frame forever. A stateful
renderer can fire callbacks when OneShot finishes:

```rust
renderer.on_animation_complete(|instance, state_name| { /* ... */ });
```

Deferred within the slice — YAGNI until a game actually needs it for
gameplay (attack-finish triggers, etc.).

### Compile-layer refactor

Move `compile_frame`'s per-pass logic into methods on `Renderer`:

- First-frame path: `Renderer::new(spec, compiled) -> Renderer` builds the
  initial state without emitting commands.
- Each tick: `Renderer::render() -> Vec<RenderCommand>` emits only the
  deltas needed (batches already live on the GPU).
- Stateless `compile_frame` stays as a thin adapter for simple
  call-sites that don't need the runtime API.

## Format gaps that might surface

- **Edge autotile rotation details.** `EdgeConnectedBitmask.EdgeHex`
  layout's CCW/CW convention at vertex endpoints is currently interpreted
  one way (see `compile/edges.rs`); SPEC §5.9 is slightly hand-wavy. Pin
  during the first real river-autotile authoring pass.
- **Anchor validation.** The compile layer rejects nonsensical combos
  (`FreePos` with `NeighborBitmask` → `UnsupportedSource`), but errors
  surface late. Moving the checks into `validate.rs` would flag them at
  load time.

## Useful context for picking up

- The crate is `no-feature-flag` (all deps non-optional; simplest surface).
- House style: `mod private { ... }` + `mod_interface::mod_interface!` per
  file; `exposed use X;` for items that bubble up to the parent scope via
  `layer X;` in the parent.
- Workspace rulebook at repo-root `rulebook.md` applies — see *Test placement* and *Test file size* sections.
- The `slay_map` demo (untracked) in `examples/minwebgl/` exercises most
  shipped features — useful as a smoke test when iterating on the
  compile layer.

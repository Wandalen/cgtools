# tilemap_scene — development roadmap

- **Project:** Compositional declarative scene format for 2D tile-based games
- **Version:** 0.1.0
- **Status:** Active development — Path A (retained-mode `Scene` + `Renderer` with delta cache + sprite batching) shipped.

## Current state

The crate is now a retained-mode pipeline: consumers mutate a `Scene` via
typed `InstanceHandle`s; a separate `Renderer` walks the scene each frame
and emits a `RenderCommand` stream. Idle frames are served from a
per-renderer cache; mutating frames emit instanced `DrawBatch`es grouped
by `(bucket, sheet, blend, clip)` for `SortMode::None` buckets and
per-sprite `Sprite` commands for sorted buckets. The legacy stateless
`compile_frame` is gone — `Renderer::render(scene, camera)` is the sole
entry.

Used by `examples/minwebgl/slay_map`.

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

**Retained-mode runtime (Path A — Steps 1-4):**

- `Scene` — `SlotMap<InstanceHandle, Instance>` + per-anchor `Vec`s +
  `(q, r) → Vec<InstanceHandle>` spatial index. Mutators: `spawn` /
  `despawn` / `move_to` / `set_state` / `set_visible` / `set_tint` /
  `set_phase_offset` / `set_external_sprite` / `set_global_tint` /
  `set_seed`. Queries: `instance` / `instances` / `instances_at_hex` /
  per-anchor accessors / `spec` / `clock` / `revision`.
- `Scene::tick(dt) -> Vec<SceneEvent>` — advances the master clock and
  returns `SceneEvent::AnimationCompleted` for every leaf
  `SpriteSource::Animation` layer whose OneShot duration was crossed in
  this tick. Visibility-gated; deterministic across runs.
- `Scene::from_snapshot` — bridge that materialises a serde-loaded
  `SceneSnapshot` (`palette + map` ASCII form supported) by spawning
  every declared instance.
- `Scene::revision()` — monotonic mutation counter, bumped exactly once
  per successful state-changing call (`tick` does NOT bump). Underlies
  the renderer's idle-replay cache.
- `Renderer` — owns `CompiledAssets`, allocates batches on demand,
  carries a per-frame cache across calls.
  - **Idle-replay:** when `(scene.revision, scene.clock, camera
    fingerprint)` matches the snapshot from the previous call, returns
    the previously emitted command slice verbatim — no scene walk,
    no command rebuild. Exposed via `Renderer::cache_hits()` for
    consumer telemetry.
  - **Batch emission (SortMode::None buckets):** sprites grouped by
    `(bucket, sheet, blend, clip)` into instanced batches. First
    encounter emits `CreateSpriteBatch` + `BindBatch` + N×
    `AddSpriteInstance` + `UnbindBatch`; subsequent emissions reuse
    the same batch id with `Bind` + `Set` for the
    `0..min(old, new)` prefix + `RemoveInstance` to trim the tail +
    `AddSpriteInstance` to extend + `Unbind`. Unused keys flushed via
    `DeleteBatch`. One `DrawBatch` per live batch in walk order.
  - **Per-sprite emission (sorted buckets):** any bucket with a non-
    `None` `SortMode` continues to emit per-sprite `Sprite` commands
    to preserve visual ordering across multiple sheets.
- `RenderCommand::ScreenSpaceSprite` — viewport-anchored emits stay
  per-sprite (not batched in Step 4; deferred polish item).

**Test baseline:** 138 passing.
35 unit tests (hash, ids, camera, coords, animation, neighbors,
conditions, vertex, edges, viewport, snapshot expansion) +
17 `scene_state` (handle resolution, mutation API, revision bumps) +
13 `scene_events` (`tick` + OneShot completion semantics) +
6 `renderer_test` (per-instance overrides, snapshot bridge) +
14 `renderer_cache_test` (idle replay, mutation / tick / camera
invalidation, byte-equality on replay) +
39 `scene_model_compile_test` (anchor × source × pipeline coverage —
all migrated via `tests/common::flatten_to_sprites`) +
14 `scene_model_test` (RON + serde round-trip).

**Closed `TILEMAP_SCENE_FEEDBACK.md` items:**

- §1 Runtime spec mutation — gone. Per-instance phase override
  (`set_phase_offset`) replaces the duplicate-Object workaround.
- §2 `compile_assets` all-or-nothing — `Renderer::new` is the only
  compile path; scene mutations never trigger asset recompile.
- §3 Anchor per-`Object` — `Placement` is now per-instance
  (`Hex { q, r }` / `Edge { hex, dir }` / `FreePos { x, y }` /
  `Viewport` / `Multihex`).
- §4 String-keyed `Tile.objects` — `InstanceHandle` /
  `ObjectHandle` are interned `u32` ids; spawn-then-cache pattern.
- §5 `PhaseOffset` global — per-instance `set_phase_offset(Some(t))`.
- §6 No OneShot completion — `Scene::tick` returns
  `SceneEvent::AnimationCompleted` for every crossing.
- §7 Per-instance tinting — `set_tint(handle, Option<[f32; 4]>)`.
- §9 `compile_frame` rebuilds every frame — idle replay + batch
  reuse; mutations emit only the Set/Add/Remove diff for the
  affected batch.

**Open feedback items** (see *Polish items* + *Deferred from Step 4*
below): §8, §10, §11, §12, §13, §14.

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
4. ~~**`External` sprite source runtime plumbing.**~~ *Shipped.*
   `Scene::set_external_sprite( handle, slot, SpriteRef )` populates
   the per-instance slot map; the renderer resolves
   `SpriteSource::External { slot }` against it. Unset slots emit
   nothing (no error, no placeholder — see §12.2 of the spec for the
   pending magenta-checkerboard option).
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
13. ~~**🐛 LayerBehaviour.alpha not propagated in compile/frame.rs.**~~ *Fixed.*
    All 7 emit sites now apply `layer.behaviour.alpha` to the sprite's tint alpha
    channel via the `tinted()` helper. Also fixed: `LayerBehaviour::default()` now
    returns `alpha: 1.0` (was `0.0` via `f32::default()`, inconsistent with the
    serde default).

## Deferred from Step 4 — `Renderer` follow-ups

Pragmatic deferrals from the Path-A delivery. None block consumers; each
is a self-contained optimisation or scope-extension to revisit when a real
workload demands it.

1. **Fine-delta per-instance `SetSpriteInstance` for `SortMode::None`
   batches.** Current behaviour: any change to a `None`-sorted bucket
   forces a full repopulate of the affected batch (Bind →
   Set 0..min(old, new) → Remove trim → Add extend → Unbind). The plan
   originally called for **stable per-(instance, layer, emit) slot
   identity** so a single `move_to` on one hex emits exactly one
   `SetSpriteInstance` at the cached slot, independent of how many
   other hexes the batch contains. Needs: stable `EmitKey →
   (BatchKey, slot)` map in `Renderer`, walk-scene-vs-cache diff,
   selective Set with bit-equal payload check. Worth picking up when
   profiling shows the per-mutation Set storm on large maps.

2. **Sorted-bucket batching (multi-sheet safe path).** Sorted buckets
   (`YAsc` / `XAsc` / …) currently emit per-sprite `Sprite` commands so
   visual order is preserved across sheets. Two viable upgrades:
   (a) detect single-sheet sorted buckets at runtime and batch those
   (cheap, immediate win for typical unit layers);
   (b) encode the sort key into `Transform.depth` and rely on depth
   testing in the backend (requires backend-side blend / depth-test
   contract — bigger change). Pick (a) first.

3. **Viewport-pass batching.** `RenderCommand::ScreenSpaceSprite` is
   per-sprite. Typical viewport instance counts are small (HUD
   elements), so this is rarely a hotspot — revisit only if profiling
   flags it.

4. **Composite OneShot detection in `Scene::tick`.** Today's
   completion detection only inspects leaf `SpriteSource::Animation`
   layers. OneShot animations nested inside `Variant` /
   `NeighborBitmask` / `ViewportTiled` are silent. Lifting this
   requires resolving the active sub-source per-instance per-frame
   (duplicates renderer logic). Deferred until a game needs it.

5. **`Renderer::cleanup() -> Vec<RenderCommand>`.** The current
   contract is "commands emitted only from `render()`" so `Drop` does
   not emit `DeleteBatch`s. If a consumer cycles renderers within a
   single backend context they'll leak GPU batches. Add an explicit
   `cleanup()` that drains every live batch on demand.

6. **Per-instance dirty tracking.** `Scene::revision()` is coarse
   (any mutation = full diff scan). For very large scenes a per-handle
   dirty set in `Scene` would let `Renderer` walk only changed
   instances. Not needed below a few thousand instances.

7. **`Anchor::Multihex` rendering.** Still rejected at render time
   with `UnsupportedAnchor`. Independent of Step 4 — moves with the
   sprite-source coverage matrix, not the renderer architecture.

8. **`Sprite::Hash` / `Sprite::Eq` upstream.** Diffing in
   `apply_scene_diff` and the batch repopulate path falls back to
   "always emit Set" because `Sprite` has no `PartialEq` derive in
   `tilemap_renderer`. Adding it would let the renderer elide no-op
   Sets when only some instances actually changed.

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

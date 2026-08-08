# Format: Declared Resources

### Scope

- **Purpose**: Define `Asset`, `Tint`, `Animation`, and `Effect` — the reusable resources declared once and referenced by id.
- **Responsibility**: Document each resource's fields, its reference type (`*Ref`), and frame-lookup rules for atlas assets.
- **In Scope**: `Asset`/`AssetKind`, `Tint`/symbolic tints, `Animation`/`AnimationTiming`, `Effect`/`EffectKind`, `PhaseOffset`.
- **Out of Scope**: How a sprite source selects among these resources at render time (see `format/005`); how a layer applies a tint/effect during rendering (see `format/006`); how `phase_offset` is resolved into an actual time offset (see `algorithm/001`).

### Abstract

Four resource kinds are declared once at the top level of a `RenderSpec` (see `format/008`) and referenced by id from anywhere else in the spec: `Asset` (an image and how to slice it into sprite frames), `Tint` (a color transform), `Animation` (a sequence of frames with timing), and `Effect` (a shader-driven procedural modification with no per-frame sprite data of its own). Each has a corresponding `*Ref` newtype (`SpriteRef`, `TintRef`, `AnimationRef`, `EffectRef`) used everywhere a sprite source or layer behaviour needs to name one.

### Data Model

`Asset`: `id`, `path`, `kind: AssetKind`, `filter` (`Linear`/`Nearest`, default `Linear`), `mipmap` (`Off`/`Nearest`/`Linear`, default `Off`), `wrap` (`Clamp`/`Repeat`/`Mirror`, default `Clamp`). `AssetKind`:

| Variant | Fields | Use case |
|---------|--------|----------|
| `Single` | `size` | Whole image is one sprite |
| `Atlas` | `tile_size`, `columns`, `frames: HashMap<String,(u32,u32)>` (optional named-frame manifest), `origin`, `gap`, `image_size` | Grid-sliced sheets, optionally addressed by semantic frame name |
| `SpriteSheet` | `frame_count`, `layout: SheetLayout` (`Horizontal`/`Vertical`/`Grid{columns}`) | Animation-oriented sheets addressed purely by numeric index |

`SpriteRef` resolves an asset id plus a frame (name or numeric index) to a concrete pixel region. `Tint`: `id`, `color` (`"#rrggbb"`/`"#rrggbbaa"`), `strength` (`0.0..=1.0`, 0 = identity, 1 = full replacement), `mode: BlendMode` (default `Multiply`, type owned by `tilemap_renderer`). `Animation`: `id`, `timing: AnimationTiming`, `mode` (`Loop`/`PingPong`/`OneShot`), `phase_offset: PhaseOffset`. `AnimationTiming` is a genuine sum type — `Regular{frames, fps}` (uniform duration), `FromSheet{asset, start_frame, count, fps}` (pulls uniform-duration frames from a `SpriteSheet` asset), or `Irregular{frames: Vec<TimedFrame{sprite, duration_ms}>}` (per-frame duration, `fps` not applicable) — a spec author picks exactly one variant; there is no flat-struct "mixing" for validation to reject, unlike the original specification's phrasing (see Version Compatibility below). `Effect`: `id`, `kind: EffectKind` (`VertexDisplace{axis, amplitude, frequency}` / `AlphaPulse{min, max, frequency}` / `ColorShift{target, amplitude, frequency}`), `phase_offset`.

`PhaseOffset`: `None` (0 offset) | `HashCoord` (deterministic per-cell spread, see `algorithm/001`) | `Fixed(f32)` (constant offset in seconds) | `Linear{per_q, per_r}` (gradient across the grid) | **`Instance`** — deterministic per-*instance* spread keyed by a runtime-assigned seed rather than grid position, falling back to `0.0` when no seed is set. `Instance` is **not present in the original format specification's `phase_offset` enumeration** — a real, source-verified addition (`src/resource.rs`, confirmed also via `roadmap.md`'s consumer-feedback history) that this doc restores as fourth-and-a-half option alongside the three the original text listed.

### Encoding Structure

All four resource kinds are declared as flat top-level RON lists (`assets: [Asset, ...]`, not maps — consistent with `RenderSpec`'s other collections, see `format/008`) and referenced elsewhere by their `*Ref` newtype wrapping the declared `id` string. `SpriteRef` is authored in RON as a 2-tuple for brevity: `Static(("terrain_atlas", "grass_01"))` rather than a named-field struct literal. For an `Atlas` asset, frame lookup tries two paths in order: (1) the string is checked against `Atlas.frames`' named-frame manifest; (2) failing that, it is parsed as a non-negative integer and the grid layout computes `col = idx % columns, row = idx / columns`. If neither resolves, compilation fails with an explicit error naming the missing frame — there is no silent placeholder region at this stage (contrast the render-time missing-sprite handling in `algorithm/002`, which *does* substitute a placeholder). Symbolic tints `TeamColor` and `FogDependent` are resolved at render time rather than declared with a `color` field: `TeamColor` reads the instance's `owner` against `Scene.players[].color` (see `format/008`); `FogDependent` reads fog-of-war visibility state.

### Version Compatibility

New `AssetKind`/`EffectKind`/`PhaseOffset` variants are expected to be additive across minor versions. `PhaseOffset::Instance` is itself an example already-shipped ahead of a formal specification update. Sampler parameters (`filter`/`mipmap`/`wrap`) are asset-wide, not per-sprite — splitting one image into separate `Asset` declarations is the documented path for a future spec wanting per-sprite sampling, rather than a schema change.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_animation_phase_and_frame_selection.md](../algorithm/001_animation_phase_and_frame_selection.md) | Resolves `Animation.timing`/`mode`/`phase_offset` into a concrete frame at render time |

### Formats

| File | Relationship |
|------|--------------|
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | Leaf sources (`Static`, `Variant`, `Animation`, `External`) select frames from the assets/animations declared here |
| [format/006_layer_behaviour.md](../format/006_layer_behaviour.md) | `TintBehaviour`/`effects` reference `Tint`/`Effect` resources declared here by id |
| [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) | `RenderSpec.assets`/`tints`/`animations`/`effects` element types declared by this doc |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) | Id uniqueness within each resource collection and `*Ref` resolution |

### Sources

| File | Relationship |
|------|--------------|
| `src/resource.rs` | `Asset`, `AssetKind`, `SheetLayout`, `Tint`, `Animation`, `AnimationTiming`, `TimedFrame`, `AnimationMode`, `PhaseOffset`, `Effect`, `EffectKind` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/scene_model_test.rs` | RON round-trip coverage for resource declarations |

# Algorithm: Animation Phase & Frame Selection

### Scope

- **Purpose**: Compute which frame an animated sprite source shows at a given render time.
- **Responsibility**: Document the local-time formula, all `PhaseOffset` resolutions, and per-`AnimationMode` frame-index selection.
- **In Scope**: `t_local` computation, `PhaseOffset` variant resolution (including the `OneShot`-relative-origin refinement), `Loop`/`PingPong`/`OneShot` frame-index formulas, `Irregular` timing's cumulative-duration walk.
- **Out of Scope**: What `Animation`/`PhaseOffset` declare as data (see `format/004`); how the chosen frame is subsequently tinted/blended (see `format/006`).

### Abstract

Every animated sprite source (an `Animation` leaf source, or an `Animation`-driven slot inside a composite source — see `format/005`) needs one deterministic answer to "which frame, right now" — reproducible across runs and platforms so a screenshot test or a networked game stays consistent. The algorithm has two stages: first resolve a **local time** `t_local` for this specific layer instance from the shared master clock plus a per-layer phase offset, then map `t_local` onto a concrete frame index using the animation's own timing/mode declaration (see `format/004`). The phase-offset stage is also what `Variant::HashCoord`/`Random` selection (see `format/005`) and `VariantSelection`'s deterministic-hash approach share conceptually with animation phase — both lean on the same `coord_hash`/`str_hash` primitives for reproducible pseudo-randomness (see `invariant/001`'s referential-integrity neighbor, not duplicated here).

### Algorithm

**Stage 1 — local time.** The general formula is `t_local = t_global + phase_offset`, where `t_global` is a shared clock (by default the object instance's own clock, see Intra/Inter-Object Sync below) and `phase_offset` resolves per `PhaseOffset` variant:

| Variant | Resolution |
|---------|------------|
| `None` | `0.0`. |
| `Fixed(seconds)` | The constant, unchanged. |
| `HashCoord` | `(coord_hash(q, r, salt) as f32 / u32::MAX as f32) * animation_duration_seconds(animation)`, where `salt = str_hash(animation.id)` and `(q, r)` is the instance's grid position — requires a grid-anchored anchor. |
| `Linear{per_q, per_r}` | `q * per_q + r * per_r` seconds — a travelling-wave gradient across the grid (e.g. `per_q = 1.0 / fps` shifts the phase by exactly one frame per column). Requires a grid-anchored anchor. |
| `Instance` | Same hash-based approach as `HashCoord`, but keyed by a runtime-assigned per-instance seed (`coord_hash(seed as i32, 0, str_hash(animation.id))`) instead of grid position — falls back to `0.0` when the instance carries no seed. Spreads phase across instances that share one position or have no grid position at all (e.g. `FreePos`/`Viewport` anchors), which `HashCoord`/`Linear` cannot do since both require a grid coordinate. Not present in the specification text this doc replaces — see `format/004`. |

**`OneShot` uses a different time base, not a different formula.** For `AnimationMode::OneShot`, the "base" fed into the formula above is `t_global_master_clock - oneshot_origin` (the instance's most recent state-entry time) rather than the raw master clock directly — so a `OneShot` animation's local time restarts at zero every time the object re-enters the state that plays it, letting "completes and stops on its last frame" (Stage 2) mean something coherent across repeated re-entries rather than only ever firing once at the object's birth. `Loop`/`PingPong` use the raw shared clock directly, with no origin adjustment — see Intra/Inter-Object Sync below for what that clock actually is.

**Stage 2 — frame index**, given `t_local` and the animation's `timing`/`mode` (see `format/004`):

- **Regular / FromSheet timing**: `raw = floor(t_local * fps)`, then per `mode`:
  - `Loop` — `frame_index = raw mod frame_count`.
  - `PingPong` — `raw` reflects back and forth across `[0, frame_count - 1]` (a triangle-wave index sequence) instead of wrapping.
  - `OneShot` — `frame_index = min(raw, frame_count - 1)` — clamps on the last frame once `t_local` exceeds the play-through duration, rather than looping or reflecting.
- **Irregular timing**: walk `frames_timed` accumulating each entry's `duration_ms` until the running total reaches `t_local`; the frame whose interval contains `t_local` is current. `fps` does not apply to this timing kind.
- **Completion signal**: a `OneShot` animation that has reached its last frame is exposed to game code as a completion event (see `api/001`) — `algorithm/002`'s per-frame tick is what drives this detection, not this algorithm directly.

**Intra-object sync**: by default, every animated layer within one object instance shares the same `t_global` (the instance's own clock) — two layers with equal frame counts and no explicit `phase_offset` play in lock-step, which is how a unit body and its `Masked` team-color mask (see `format/006`) stay visually aligned without any explicit synchronization declaration. An explicit `phase_offset` on one layer deliberately decouples it from that shared clock (e.g. a blinking detail using `Fixed(0.5)`).

**Inter-object sync**: there is no implicit synchronization between different object instances. Instances at `phase_offset: None` all read the renderer's own master clock, so they *happen* to move together, but this is incidental, not guaranteed — an implementation is free to give an instance its own clock. `HashCoord`/`Instance` phase offsets exist specifically to break unwanted unison (torches, water, pulsing effects) by spreading neighbours or sibling instances across the phase space deterministically.

### APIs

| File | Relationship |
|------|--------------|
| [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) | `OneShot` completion is surfaced to game code as a scene event |

### Formats

| File | Relationship |
|------|--------------|
| [format/004_declared_resources.md](../format/004_declared_resources.md) | `Animation`/`AnimationTiming`/`AnimationMode`/`PhaseOffset` are declared there; this algorithm resolves them |
| [format/005_sprite_sources.md](../format/005_sprite_sources.md) | `Variant::HashCoord`/`Random` selection shares this algorithm's `coord_hash`/`str_hash` primitives |
| [format/006_layer_behaviour.md](../format/006_layer_behaviour.md) | Intra-object sync is what keeps a `Masked` mask's animation aligned with its body layer |

### Sources

| File | Relationship |
|------|--------------|
| `src/hash.rs` | `coord_hash`, `str_hash` — the deterministic primitives phase resolution is built on |
| `src/compile/animation.rs` | `resolve_animation_frame`, `declared_phase_seconds`, `pick_frame_index`, `animation_duration_seconds` |

### Tests

| File | Relationship |
|------|--------------|
| `src/compile/animation.rs` | Inline `#[cfg(test)]`: `regular_loop_wraps`, `one_shot_clamps`, `one_shot_origin_resets_local_time`, `pingpong_reflects`, `phase_offset_hashcoord_spreads_neighbours`, `phase_offset_instance_spreads_seeds`, `phase_offset_instance_falls_back_when_seed_missing`, `phase_offset_fixed_shifts_timeline`, `irregular_timing_honours_durations` |
| `tests/scene_events_test.rs` | Runtime `tick()` + `OneShot` completion-event semantics |

# Type: ECS Component Vocabulary

### Scope

- **Purpose**: Define the fixed set of `hecs` components `tiles_tools` ships, as the vocabulary game code composes onto entities.
- **Responsibility**: Document every component/enum's fields, construction, and any self-enforced invariant (e.g. `Health` clamping). Group by the four categories the source itself declares (Spatial, Gameplay, Visual, Behavioral).
- **In Scope**: `Position<C>`, `Movable`, `Size` (Spatial); `Health`, `Stats`, `Team` (Gameplay); `Sprite`, `Animation` (Visual); `PlayerControlled`, `AI`/`AIState`, `Trigger`/`TriggerType` (Behavioral).
- **Out of Scope**: The systems that read/write these components at runtime (see `api/001`); which of those systems are functional vs. stub (see `pitfall/002`).

### Definition

All 13 items below live in `src/ecs/components.rs` and are re-exported via `ecs::*` (`src/ecs/mod.rs:54`). Every component is a plain data struct with no behavior beyond simple accessors/builders — the module's own doc comment states the design intent directly: *"Components are pure data structures that describe entity properties and capabilities"* (`src/ecs/components.rs:4-5`).

**Spatial:**

| Component | Fields | Notes |
|-----------|--------|-------|
| `Position<C>` | `coord: C` | Generic over any coordinate type in `type/001`'s table; gains `distance_to`/`neighbors`/`is_adjacent_to` when `C: Distance`/`C: Neighbors`. |
| `Movable` | `range: u32`, `diagonal_movement: bool`, `can_pass_through_entities: bool`, `can_pass_through_obstacles: bool` | Builder methods (`with_diagonal`, `with_entity_passthrough`, `with_obstacle_passthrough`) set the three bools; data only — nothing in `api/001`'s `World` currently reads these flags to constrain movement (see `pitfall/002`). |
| `Size` | `width: u32`, `height: u32` | `area()` helper; `single()`/`square(n)` constructors. |

**Gameplay:**

| Component | Fields | Notes |
|-----------|--------|-------|
| `Health` | `current: u32`, `maximum: u32` | See Validation below — the one component with a self-enforced numeric invariant. |
| `Stats` | `attack: u32`, `defense: u32`, `speed: u32`, `level: u32` | `damage_calculate(target_defense)` = `(attack - target_defense/2).max(1)` (saturating) — a pure helper method, not itself wired into `CombatSystem` (see `pitfall/002`). |
| `Team` | `id: u32`, `default_hostile: bool` | `is_allied_with`/`is_hostile_to` — same `id` is never hostile regardless of `default_hostile`. |

**Visual:**

| Component | Fields | Notes |
|-----------|--------|-------|
| `Sprite` | `texture_id: String`, `tint: [f32; 4]`, `scale: f32`, `rotation: f32`, `visible: bool` | Pure display data; `tiles_tools` has no renderer of its own that reads it. |
| `Animation` | `current_frame: u32`, `frame_count: u32`, `frame_duration: f32`, `frame_timer: f32`, `looping: bool`, `playing: bool` | `update(dt)` is a real, self-contained frame-advance implementation (accumulate `frame_timer`, roll to next frame at `frame_duration`, wrap or stop at `frame_count` depending on `looping`) — driven by `AnimationSystem` (see `api/001`). |

**Behavioral:**

| Component | Fields | Notes |
|-----------|--------|-------|
| `PlayerControlled` | `player_id: u32` | Marker-with-data; identifies player-owned entities. |
| `AI` | `state: AIState`, `target: Option<hecs::Entity>`, `decision_timer: f32`, `decision_interval: f32` | `AIState` enum: `Idle`, `Patrolling`, `Pursuing`, `Attacking`, `Fleeing`, `Guarding`. `update(dt)` accumulates `decision_timer`; `should_make_decision()` compares it to `decision_interval`. The timer bookkeeping is real — what happens *at* a decision point is not (see `pitfall/002`). |
| `Trigger` | `trigger_type: TriggerType`, `repeatable: bool`, `activated: bool`, `cooldown: f32`, `cooldown_timer: f32` | `TriggerType` enum: `OnEnter`, `OnExit`, `OnProximity`, `OnInteract`, `OnTimer(u32)`. `can_activate()`/`activate()` implement real repeatable/cooldown gating logic. |

### Validation

**`Health` is the one component enforcing a numeric bound on itself**, and it does so by clamping rather than by rejecting: `damage(amount)` uses `current.saturating_sub(amount)` (floors at `0`, never underflows/panics — `src/ecs/components.rs:226-229`); `heal(amount)` uses `(current + amount).min(maximum)` (ceilings at `maximum` — `src/ecs/components.rs:232-235`). `current <= maximum` therefore holds by construction through every mutator the type exposes; there is no public path that sets `current` directly without going through one of these clamped methods (the field is `pub`, so a caller constructing `Health { current: 999, maximum: 10 }` via struct-literal syntax bypasses the clamp entirely — the invariant is enforced by the *methods*, not by the field's visibility).

**No other component in this table has a field-value invariant.** `Stats`, `Size`, `Movable`, `Sprite`, `Animation`'s frame fields, `Trigger`'s cooldown fields — all accept any value their `pub` fields allow; nothing rejects, say, a `Size { width: 0, height: 0 }` or an `Animation` with `frame_count: 0`.

### APIs

| File | Relationship |
|------|--------------|
| [api/001_ecs_world_runtime_api.md](../api/001_ecs_world_runtime_api.md) | `World::update`'s per-system pass reads/writes these components; `spawn`/`EntityBuilder` attach them |

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | `Position<C>`'s `C` is any coordinate type from that doc's table; `C`'s `Serialize`/`Deserialize` availability propagates into `Position<C>`'s own derive |

### Pitfalls

| File | Relationship |
|------|--------------|

### Sources

| File | Relationship |
|------|--------------|
| `src/ecs/components.rs` | All 13 items in the tables above |
| `src/ecs/mod.rs` | `pub use components::*` re-export |

### Tests

No dedicated unit tests exist for individual component methods (`Health::damage`/`heal`, `Trigger::can_activate`, `Team::is_hostile_to`, etc.) within `src/ecs/components.rs` itself; coverage is indirect, through whichever integration tests happen to exercise a `World` that uses these components.

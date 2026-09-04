# API: ECS World Runtime

### Scope

- **Purpose**: Document `ecs::World`'s public operation surface — what is genuinely functional versus the one confirmed no-op — and its direct exposure of `hecs` types.
- **Responsibility**: Document every public `World`/`EntityBuilder` operation, hecs error propagation, and the `hecs`-version compatibility surface this API leaks.
- **In Scope**: `ecs::World` (all public methods), `ecs::EntityBuilder`, `ecs::GameEvent`, the systems `World::update` dispatches to.
- **Out of Scope**: `MovementSystem::movement_process` and the other `ecs::systems` types callable directly but not wired into `World::update`'s automatic dispatch; individual component definitions (see `type/002`).

### Abstract

`ecs::World` is a real, working central container wrapping `hecs::World` directly (`pub hecs_world: hecs::World`, `src/ecs/world.rs:54`) — not an opaque abstraction over it. `spawn`/`despawn`/`query`/`query_mut`/`get`/`get_mut` all delegate straight to the underlying `hecs::World` method of the same name. `update(dt)` genuinely dispatches to 4 real systems each call — `AnimationSystem`, `AISystem`, `CombatSystem`, `CleanupSystem` — translating their results into a per-frame `GameEvent` list, and additionally applies queued movement requests. `movement_request` queues a typed target coordinate (boxed apply closure, latest request per entity wins) that the next `update` writes into the entity's `Position<C>` component, emitting `GameEvent::EntityMoved` (implemented by task 063; formerly a no-op tracked as `pitfall/002`). `EntityBuilder` provides 6 archetype helpers (`unit`/`player`/`enemy`/`obstacle`/`trigger`/`decoration`) composing real component bundles.

### Operations

| Operation | Behavior |
|-----------|----------|
| `World::new()` | Constructs an empty `hecs::World` plus empty request/event tracking. |
| `World::spawn(components)` | Delegates directly to `hecs::World::spawn`; returns `hecs::Entity`. |
| `World::despawn(entity)` | Delegates directly to `hecs::World::despawn`; returns `Result<(), hecs::NoSuchEntity>`. |
| `World::query::<Q>()` / `query_mut::<Q>()` | Delegate directly to `hecs::World::query`/`query_mut`. |
| `World::get::<T>(entity)` / `get_mut::<T>(entity)` | Delegate to `hecs::World::get::<&T>`/`get::<&mut T>`; return `Result<hecs::Ref<T>, hecs::ComponentError>` / `Result<hecs::RefMut<T>, hecs::ComponentError>`. |
| `World::update(dt)` | Real dispatch, in order: `AnimationSystem::animations_update`, `AISystem::ai_update`, `movement_requests_process` (applies queued requests to `Position` components, emitting `GameEvent::EntityMoved` per applied request), `CombatSystem::combat_process` (translated into `GameEvent::Damage`/`EntityDefeated`), `CleanupSystem::defeated_entities_cleanup` (translated into `GameEvent::EntityDestroyed`). |
| `World::movement_request(entity, target)` | Queues `target` as a typed, boxed apply closure — latest request per entity wins; applied by the next `update`. A request whose entity is gone, or whose coordinate type does not match the entity's `Position<C>`, is discarded without effect. |
| `World::events()` / `events_clear()` | Return/clear the `Vec<GameEvent>` accumulated by the most recent `update` call. |
| `World::elapsed_time()` | Returns cumulative `dt` summed across all `update` calls. |
| `World::entities_in_range_find(center, range)` / `nearest_entity_find(center)` | Real spatial queries over `Position<C>` components, generic over `C: Distance`. |
| `EntityBuilder::unit/player/enemy/obstacle/trigger/decoration` | Each returns a real `impl hecs::DynamicBundle` composing `Position` + a fixed set of gameplay components (e.g. `player` adds `Movable::new(3).with_diagonal()`, `Team::new(0)`, `PlayerControlled::new(player_id)`). |

### Error Handling

No custom error type wraps ECS failures — `despawn` propagates `hecs::NoSuchEntity` directly; `get`/`get_mut` propagate `hecs::ComponentError` directly. A caller matching on these error types is matching `hecs`'s own error enums, not a `tiles_tools`-owned type.

### Compatibility Guarantees

**None at the `hecs` boundary.** `World::hecs_world` is a `pub` field of type `hecs::World` (`src/ecs/world.rs:54`), and every operation above returns or accepts `hecs`-native types (`hecs::Entity`, `hecs::Ref`/`hecs::RefMut`, `hecs::ComponentError`, `hecs::NoSuchEntity`, `hecs::DynamicBundle`, `hecs::Query`) directly rather than through `tiles_tools`-owned wrapper types. A caller that touches `hecs_world` directly, or matches on any of these error/return types by name, is coupled to `hecs`'s own API surface — a `hecs` major-version upgrade is a breaking-change surface for such callers, not something this API insulates against. This is a direct divergence from the architecture originally planned in `architectural_evaluation/001` (`pub struct Entity(hecs::Entity)`, `pub struct World(hecs::World)` as private-field newtypes) — the shipped `World`/`Entity` usage is unwrapped throughout.

### Architectural Evaluations

| File | Relationship |
|------|--------------|
| [architectural_evaluation/001_ecs_library_selection.md](../architectural_evaluation/001_ecs_library_selection.md) | The ADR that selected `hecs`; this API's direct exposure of `hecs` types diverges from that ADR's own sketched abstraction-layer plan |

### Types

| File | Relationship |
|------|--------------|
| [type/002_ecs_component_vocabulary.md](../type/002_ecs_component_vocabulary.md) | The 13 components `World`'s queries and `EntityBuilder`'s archetypes operate on |

### Sources

| File | Relationship |
|------|--------------|
| `src/ecs/world.rs` | `World`, `EntityBuilder`, `GameEvent` |
| `src/ecs/systems.rs` | `AnimationSystem`, `AISystem`, `CombatSystem`, `CleanupSystem`, all dispatched from `World::update` |
| `src/ecs/mod.rs` | Module re-exports (`components::*`, `systems::*`, `world::*`) |

### Tests

Inline doc-tests exist on both `ecs/mod.rs`'s and `ecs/world.rs`'s module-level doc comments (spawn/query examples) — both exercise `spawn`/`query`/`update` against real component types, and `ecs/world.rs`'s additionally exercises `movement_request` + `update` end-to-end, asserting the `Position` component actually changed. `tests/integration/ecs_tests.rs` pins the same path plus the discard cases (despawned entity, mismatched coordinate type) and `EntityMoved` event emission.

# Pitfall: `World::request_movement` Never Moves Anything

### Scope

- **Purpose**: Warn that `World::request_movement` accepts and silently discards its target coordinate, so an entity's `Position` never changes as a result of calling it.
- **Responsibility**: Document the exact discard path and the per-frame clear that erases even the discarded request.
- **In Scope**: `ecs::World::request_movement`, `ecs::World::update`'s call to `process_movement_requests`.
- **Out of Scope**: `MovementSystem::process_movement` (`src/ecs/systems.rs:41+`), a separate, directly-callable system not wired into `World::update`'s automatic dispatch — not evaluated as part of this migration.

### Trap

Calling `world.request_movement(entity, target)` then `world.update(dt)`, expecting `entity`'s `Position` component to move toward `target` on that or a subsequent update — the method's name and parameter both suggest exactly this.

### Failure

`request_movement`'s `target` parameter is prefixed `_target` and is never read (`src/ecs/world.rs:145-150`):

```rust
pub fn request_movement< C >( &mut self, entity : hecs::Entity, _target : C )
where C : 'static + Clone,
{
  self.movement_requests.insert( entity, "movement_requested".to_string() );
}
```

The only thing stored is a fixed literal string, `"movement_requested"`, keyed by `entity` — the actual target coordinate is dropped on the floor. `World::update` calls `process_movement_requests` every frame (`src/ecs/world.rs:129`), whose entire body is:

```rust
fn process_movement_requests( &mut self )
{
  // TODO: Implement proper type-safe movement request processing
  self.movement_requests.clear();
}
```

No `Position` component is read or written anywhere in this path. The net effect of any number of `request_movement` calls, across any number of `update` calls, is zero change to any entity's position.

### Mitigation

Mutate the `Position` component directly — `world.get_mut::<Position<C>>(entity)` — or write a custom system, instead of `request_movement`, until this path is implemented.

### APIs

| File | Relationship |
|------|--------------|
| [api/001_ecs_world_runtime_api.md](../api/001_ecs_world_runtime_api.md) | `request_movement` is documented there as part of the full `World` operation set, with this pitfall cross-referenced from its Operations entry |

### Sources

| File | Relationship |
|------|--------------|
| `src/ecs/world.rs` | `World::request_movement`, `World::process_movement_requests`, `World::update` |

### Tests

No test currently asserts a `Position` component changes after `request_movement` + `update` — such a test would fail today, which is itself confirmation of the no-op status.

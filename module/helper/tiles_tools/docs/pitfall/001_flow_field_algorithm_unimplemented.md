# Pitfall: Flow Field Algorithm Is Entirely Unimplemented

### Scope

- **Purpose**: Warn that `FlowField`/`IntegrationField` compile, run, and return values, but every query answers with a fixed constant regardless of input.
- **Responsibility**: Document each stub method's fixed return value and the two core algorithm functions' empty bodies.
- **In Scope**: `flowfield::FlowField<System, Orientation>`, `flowfield::IntegrationField<System, Orientation>`, `FlowDirection`.
- **Out of Scope**: Per-entity A* pathfinding, which is genuinely implemented (see `algorithm/002`).

### Trap

Calling `FlowField::new(width, height)` then `.flow_calculate(&goal, is_passable, get_cost)`, expecting a populated flow field usable for RTS-style multi-unit movement — the module's own doc comment describes exactly this use case ("particularly useful for RTS games where many units need to move toward the same destination").

### Failure

Every method in `src/flowfield.rs` is a stub returning a fixed constant, independent of its arguments:

| Method | Returns | Evidence |
|--------|---------|----------|
| `IntegrationField::new` | `max_cost: u32::MAX` always | `// Simplified stub implementation for testing` (`src/flowfield.rs:89`) |
| `IntegrationField::cost_get` | `0` always | `// Simplified stub implementation - would access Grid2D` (`src/flowfield.rs:104-105`) |
| `IntegrationField::cost_set` | no-op (empty body) | `// Simplified stub implementation - would modify Grid2D` (`src/flowfield.rs:114`) |
| `IntegrationField::in_bounds` | `true` always | `// Simple bounds checking - in a full implementation this would use coordinate bounds` (`src/flowfield.rs:123-124`) |
| `FlowField::flow_direction_get` | `None` always | `// Simplified stub implementation - would access Grid2D` (`src/flowfield.rs:171-172`) |
| `FlowField::integration_field_calculate` (private, called by `flow_calculate`) | nothing — **empty function body**, only comments describing what a real Dijkstra pass "would" do | `src/flowfield.rs:193-203` |
| `FlowField::flow_directions_generate` (private, called by `flow_calculate`) | nothing — **empty function body**, real logic exists only as a commented-out code block | `src/flowfield.rs:213-229+` |

`flow_calculate` (the public entry point) calls exactly these two empty private functions and nothing else (`src/flowfield.rs:158-163`) — so calling it does not raise an error, but leaves the `FlowField` in the same all-default state `new()` produced. Every subsequent `flow_direction_get` call returns `None` for every coordinate, indistinguishable from a coordinate that was never part of the grid at all.

### Mitigation

Use `pathfind::astar` per-entity (see `algorithm/002`) instead of `FlowField` for any current multi-unit movement need — it is a genuine, working implementation. Treat `FlowField`/`IntegrationField` as a designed-but-not-yet-built API surface; do not build gameplay logic against its return values.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/002_generic_astar_pathfinding.md](../algorithm/002_generic_astar_pathfinding.md) | The working alternative for movement/pathing until flow fields are implemented |

### Sources

| File | Relationship |
|------|--------------|
| `src/flowfield.rs` | `FlowField`, `IntegrationField`, `FlowDirection`, every stub method listed above |

### Tests

No test currently exercises `FlowField::flow_calculate` end-to-end and asserts a non-default `flow_direction_get` result — such a test would fail today, which is itself confirmation of the stub status.

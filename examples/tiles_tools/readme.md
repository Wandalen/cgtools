# 🔬 Tiles Tools Examples

Native demos of the `tiles_tools` crate — tile-grid game systems: pathfinding, ECS, collision, events, field of view, and complete small games built from them.

## 🚀 How to Run

Each example is a native binary — no wasm target or trunk needed:

```bash
cd <example>
cargo run --release --all-features
```

Or, from any directory, by partial unique match against the example path:

```bash
action/run beginner_tutorial
```

## 📂 Examples

| Example | Responsibility |
|---------|----------------|
| `advanced_pathfinding_demo/` | The A* family side by side — multi-goal, edge costs, and advanced variants |
| `beginner_tutorial/` | Step-by-step introduction via a small grid dungeon-explorer |
| `debug_demo/` | Debug tooling: grid visualization styles, coordinate systems, pathfinding traces |
| `ecs_collision_demo/` | ECS collision detection and resolution between entities |
| `event_system_demo/` | Publish/subscribe events with priorities and consumption |
| `field_of_view_demo/` | Field-of-view algorithms compared: shadowcasting, ray casting, Bresenham |
| `game_of_life/` | Conway's Game of Life on the `tiles_tools` ECS with grid-aware neighbors |
| `game_systems_demo/` | Higher-level systems integrated: turn management, initiative, actions |
| `serialization_demo/` | Game-state save/load across JSON and binary formats |
| `simple_collision_demo/` | Minimal collision companion: a few entities on a square grid |
| `stealth_game/` | Stealth game on the field-of-view system — guards, line-of-sight, detection |
| `tactical_rpg/` | Turn-based tactical combat on a hexagonal grid with AI opponents |

# Algorithm: Generic A* Pathfinding

### Scope

- **Purpose**: Document `astar`/`astar_with_edge_costs`/`astar_advanced`, the coordinate-generic A* pathfinding entry points, and the one config field (`allow_diagonal`) that is stored but never consulted.
- **Responsibility**: Document the wrapping relationship to the external `pathfinding` crate, the heuristic/expansion functions supplied, and `PathfindingConfig`'s consumed-vs-unconsumed fields.
- **In Scope**: `astar`, `astar_with_edge_costs`, `astar_advanced`, `PathfindingConfig<C>`.
- **Out of Scope**: The `Distance`/`Neighbors` formulas these functions are generic over (see `algorithm/001`); flow-field-based mass movement, which is unimplemented (see `pitfall/001`).

### Abstract

`tiles_tools::pathfind` provides three A* entry points, all thin, genuinely-functional wrappers over the external `pathfinding` crate's own `astar` — not a from-scratch reimplementation. Each is generic over any `C: Distance + Neighbors + Eq + Clone + Hash` (see `algorithm/001`), so the identical pathfinding logic works unmodified across hexagonal, square, triangular, and isometric coordinates. `astar_advanced` additionally accepts a `PathfindingConfig<C>` bundling several optional constraints — of which `max_distance` and `obstacles` are read by the algorithm, and `allow_diagonal` is not.

### Algorithm

**`astar(start, goal, is_accessible, cost)`** (`src/pathfind.rs:176-205`) delegates directly to `pathfinding::prelude::astar` with three closures:
- **Expansion**: `coord.neighbors().iter().filter(|c| is_accessible(c)).map(|c| (c.clone(), cost(c)))` — every neighbor the coordinate system's own `Neighbors` impl produces, filtered by the caller's accessibility predicate, each surviving neighbor weighted by the caller's cost function.
- **Heuristic**: `goal.distance(coord)` — the coordinate system's own `Distance` formula (see `algorithm/001`), reused directly as the A* heuristic rather than a separately-tuned one.
- **Success**: `*p == *goal`.

**`astar_with_edge_costs`** (`src/pathfind.rs:238+`) is the same shape with one difference: the cost closure receives both endpoints (`Fn(&C, &C) -> u32`) instead of only the destination, enabling costs that depend on the *transition* (e.g. diagonal vs. orthogonal movement costing differently — see the type's own doc-test) rather than only the destination cell.

**`astar_advanced(start, goal, config: &PathfindingConfig<C>)`** (`src/pathfind.rs:402+`) adds a pre-check and per-neighbor filtering, both reading fields directly off `config`:
- `config.max_distance`: if `Some(max_dist)` and `start.distance(goal) > max_dist`, returns `None` immediately without searching (`src/pathfind.rs:413-418`).
- `config.obstacles`: each candidate neighbor is rejected if `config.obstacles.contains(neighbor)` (`src/pathfind.rs:432-434`).
- `config.allow_diagonal`: **declared, defaulted to `true`, and settable via a builder method — but never read.** A file-wide search of `src/pathfind.rs` for the identifier `allow_diagonal` returns exactly three lines: the field declaration (`PathfindingConfig<C>`, `src/pathfind.rs:288`), its `Default`-style initialization to `true` (`src/pathfind.rs:304`), and one builder-style setter that flips it to `false` (`src/pathfind.rs:366`). No occurrence exists inside `astar_advanced`'s own body, or anywhere else that filters or weights neighbor candidates. Unlike `max_distance`/`obstacles` (both genuinely consulted, verified above), `allow_diagonal` has no effect on pathfinding output regardless of its value — a caller who sets it to `false` expecting diagonal moves to be excluded gets the same result set as leaving it at its `true` default, because whether a given neighbor *is* diagonal is a property of the coordinate system's `Neighbors` impl (see `algorithm/001`), which this flag does not gate.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_coordinate_distance_and_neighbor_formulas.md](../algorithm/001_coordinate_distance_and_neighbor_formulas.md) | Supplies both the expansion (`neighbors()`) and heuristic (`distance()`) functions this algorithm is built from |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_flow_field_algorithm_unimplemented.md](../pitfall/001_flow_field_algorithm_unimplemented.md) | The mass-movement alternative to per-entity A*, entirely unimplemented rather than partially-consumed like `allow_diagonal` here |

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | `C: Distance + Neighbors` bound is satisfied by any type in that doc's table |

### Sources

| File | Relationship |
|------|--------------|
| `src/pathfind.rs` | `astar`, `astar_with_edge_costs`, `astar_advanced`, `astar_multi_goal`, `PathfindingConfig<C>` |

### Tests

Inline doc-tests exist for `astar_with_edge_costs` (diagonal-vs-orthogonal cost example, `src/pathfind.rs:219-236`) and `astar_advanced` (obstacle/terrain-cost example). No test currently exercises `allow_diagonal` at all — its dead-field status is a direct reading of the source, not something an existing (passing or failing) test surfaces.

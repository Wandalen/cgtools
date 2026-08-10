# tiles_tools — test suite

Two layers:

- **Top-level `*_test.rs` files** — per-module behavior tests driving the public
  surface, gated only by the same feature that gates the source module in
  `src/lib.rs` (`enabled` for most, `serialization` for the serialization file).
  They run on a plain `cargo test -p tiles_tools`. Established by task 072 from
  the former inline `#[cfg(test)]` modules; 5 tests pinning private state remain
  inline in `src/` as documented exceptions (see the exception comments in
  `src/debug.rs`, `src/field_of_view.rs`, `src/flowfield.rs`).
- **`integration/` suite** — cross-cutting scenario tests behind the opt-in
  `integration` feature, entered through `integration_tests.rs`. Runs under
  `cargo test -p tiles_tools --all-features` (the workspace's canonical
  verification). See [integration/readme.md](integration/readme.md).

## Responsibility Table

| File | Responsibility |
|---|---|
| `integration_tests.rs` | Entry point compiling the feature-gated integration suite |
| `integration/` | Cross-cutting integration scenarios (opt-in `integration` feature) |
| `debug_test.rs` | Debugger, inspector, profiler, and formatting utilities behavior |
| `events_test.rs` | Event bus lifecycle, priorities, consumption, statistics |
| `field_of_view_test.rs` | Direct `VisibilityMap` construction and mutation API |
| `flowfield_test.rs` | Flowfield public construction surface (live coverage; see task 078) |
| `game_systems_test.rs` | Turn management, state machine, resources, quests, effects |
| `serialization_test.rs` | Save/config round-trips, compression, managers, error variants |
| `spatial_test.rs` | Bounds arithmetic, spatial entities, quadtree operations |

## Adding new tests

1. Public-surface tests for one `src/` module go in that module's `*_test.rs`
   file; cross-module scenarios go under `integration/`.
2. Gate a new file with the same `#![ cfg( feature = "..." ) ]` the source module
   carries in `src/lib.rs`.
3. Tests needing private access stay inline in `src/` with an exception comment
   naming the task and the rejected alternatives.

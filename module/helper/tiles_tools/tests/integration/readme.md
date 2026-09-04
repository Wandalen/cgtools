# Integration Tests

Feature-gated integration test suite for `tiles_tools` — every module here compiles
only with the `integration` feature (enabled by `--all-features`), via the gate in
`mod.rs`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| mod.rs | Feature gate, lint policy, test module registration |
| conversion_tests.rs | Cross-system coordinate conversion contracts |
| coordinates_tests.rs | Coordinate creation, distance, and neighbor operations |
| ecs_tests.rs | ECS world, components, systems, movement requests |
| field_of_view_tests.rs | Field-of-view calculation contracts |
| flowfield_tests.rs | Flow-field calculation, batch, multi-goal, and ECS contracts (hex-only) |
| geometry_tests.rs | Hexagon geometry generator contracts |
| isometric_coords_tests.rs | Isometric coordinate system behavior |
| square_coords_tests.rs | Square coordinate system behavior |
| triangular_coords_tests.rs | Triangular coordinate system behavior |

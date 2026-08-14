# tilemap_scene tests

Integration and unit test suite for the `tilemap_scene` crate. All tests live here — `src/` carries
no inline test modules (task 073 relocated the last 38; every tested item is reachable through the
crate's `mod_interface` root re-exports).

Two levels coexist:

- **Unit level** — `compile_units_test.rs` and `hash_test.rs` call individual exposed functions
  directly (`condition_evaluate`, `canonical_edge`, `animation_frame_resolve`, `coord_hash`, …).
- **Integration level** — the remaining files drive whole subsystems (`assets_compile` /
  `compile_frame`, `Scene::tick`, `Renderer`, `Catalog`) and assert on emitted command streams or
  event streams. `edge_rotation`, for example, is covered at BOTH levels on purpose: the unit table
  pins the rotation formula, while `scene_model_compile_test.rs` pins that compiled edge sprites
  carry it through to render commands.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| catalog_test.rs | Catalog and its builder API surface |
| common/ | Shared fixture helpers for integration tests |
| compile_units_test.rs | Unit contracts of compile-layer primitives (ids, conditions, camera, edges, vertex, viewport, animation, coords) |
| hash_test.rs | Known-answer determinism pins for normative SPEC §13 hashes |
| hex_config_test.rs | HexConfig::from_hex_size grid-stride arithmetic |
| renderer_cache_test.rs | Renderer per-frame idle-replay cache acceptance |
| renderer_test.rs | Renderer asset-compile-once and per-instance override contract |
| scene_events_test.rs | Scene::tick event-stream semantics |
| scene_model_compile_test.rs | Compile pipeline integration (assets_compile + compile_frame) |
| scene_model_test.rs | scene-model parsing, serde round-trip, loader API |
| scene_state_test.rs | Retained-mode Scene mutation API |
| sorted_batching_test.rs | Sorted-bucket DrawBatch collapsing |

## Adding tests

1. Pick the level: a new exposed pure function gets its section in `compile_units_test.rs` (or a
   new sibling unit file if it is not compile-layer); pipeline-observable behaviour goes in the
   matching integration file.
2. One home per behaviour at each level — check this table's owner before creating a file, and add
   a row here when you do.
3. Fixture builders shared by more than one integration file belong in `common/` (see its
   `mod.rs` header for the import pattern).
4. Verify with `cargo test -p tilemap_scene --all-features` (run detached via
   `longrun .launch` from the workspace root; check the log for the per-suite breakdown).

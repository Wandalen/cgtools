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
| common/ | Shared fixture helpers for integration tests (`compile.rs`: spec / scene / render fixtures for the `*_compile_test.rs` files) |
| compile_units_test.rs | Unit contracts of compile-layer primitives (ids, conditions, camera, edges, vertex, viewport, animation, coords) |
| hash_test.rs | Known-answer determinism pins for normative SPEC §13 hashes |
| hex_config_test.rs | HexConfig::from_hex_size grid-stride arithmetic |
| layer_tint_compile_test.rs | LayerBehaviour.tint (None pass-through, Flat composition, Masked and non-Multiply tint rejection) through every sprite-emitting compile pass |
| opaque_pass_test.rs | Opaque/transparent bucket split, per-bucket depth pinning, single coverage across batches and under opaque layers, the single painter's pass without a depth buffer (`Renderer::capabilities_set`, also on the real SVG backend), a batch whose `occlude_overlap` changes is replaced instead of drawing under its old params, and an opaque bucket with a viewport sprite rides the transparent pass for that frame only, reusing its batch |
| renderer_cache_test.rs | Renderer per-frame idle-replay and camera-pan cache-hit acceptance; a replay submitted to one backend draws the frame again without repeating its batch edits (BUG-541); `batch_id_base_set` asserts in debug while batches are live |
| renderer_test.rs | Renderer asset-compile-once and per-instance override contract |
| ron_syntax_error_test.rs | Invalid RON reported as `LoadError::Ron`, kept distinct from validation failures |
| scene_events_test.rs | Scene::tick event-stream semantics |
| scene_model_compile_test.rs | Compile pipeline integration (assets_compile + compile_frame) |
| scene_model_test.rs | scene-model parsing, serde round-trip, loader API |
| scene_state_test.rs | Retained-mode Scene mutation API; a move that keeps the placement kind and anchor cell keeps the index order |
| sorted_batching_test.rs | Sorted-bucket DrawBatch collapsing |
| vertex_corners_compile_test.rs | VertexCorners through the compile pipeline: pattern matching, orient_to_grid frame selection, corner_source channels, offset |
| vertex_resolve_cache_test.rs | Revision-keyed VertexCorners resolve cache (`Scene::tiles_revision`, re-resolve only on hex-cell changes, a same-cell move keeps the cell's stack order (BUG-540), re-enabling a disabled layer, a renderer reused for another scene with equal counters resolves it afresh), the `FadeGate` effect (easing, idle-replay invalidation, an undeclared id reported and ignored) and `AlphaPulse.restart_on_spawn` (wave restarts at a spawn, free-running one does not, a spawn that leaves the layer's vertex content alone carries the wave on, even when it changes another layer of the same bucket) |

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

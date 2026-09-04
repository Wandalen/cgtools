# Practical Development Roadmap: Tiles Tools Engine

**Version:** 2.0 (Ground-Truth Rewrite)
**Date:** 2026-08-10
**Status:** Phases 1, 2, 3, 6 complete · Phase 4 partially complete (1 known gap, 1 sub-area not started) · Phase 5 not started
**Next Priority:** see [Next Priority Actions](#next-priority-actions)

> **Revision note:** this file previously contained two contradictory status
> claims for Phase 2 (a "🎉 COMPLETED!" dashboard alongside a "0/4 milestones
> complete, Milestone 08 starting today" table) and described Phase 3 as
> "just started" and Phases 4/6 as pure future work. All of that was stale.
> This version was rewritten from the crate's actual source, tests, and
> `docs/` (the authoritative implementation contract in this repo — see
> `docs/definition/readme.md`), not from the roadmap's own prior claims. See
> [Revision History](#revision-history).

## Quick Start for Contributors

**Current Priority:** see [Next Priority Actions](#next-priority-actions) — the documented functional gap (Flow Fields) or Phase 5 (Procedural Generation), depending on interest.

**Ready to Code:**
1. Clone repo and run `cargo nextest run -p tiles_tools --all-features` to verify the current state
2. Check [Development Environment](#development-environment) setup
3. Read [Known Gaps](#known-gaps) before touching Flow Fields — it already has a documented pitfall explaining the current stub behavior
4. Follow [Definition of Done](#definition-of-done) criteria

## Development Environment

**Prerequisites:**
- Rust 1.70+ with stable toolchain
- Git configured for the cgtools workspace
- `cargo clippy` installed (this workspace uses a custom codestyle — see the repo's rulebooks; do not run `cargo fmt`)

**Setup Commands:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools
cargo nextest run --all-features   # Verify all tests pass
cargo clippy --all-targets --all-features -- -D warnings  # Ensure no warnings
cargo doc --open                   # Review current API
```

**Files to Understand First:**
1. `src/coordinates/hexagonal.rs` - Reference implementation pattern for a coordinate system
2. `src/coordinates.rs` - Distance/Neighbors traits
3. `tests/integration/coordinates_tests.rs` - Testing patterns
4. `docs/definition/readme.md` - Authoritative index of documented types, algorithms, data structures, and pitfalls

## Implementation Status Dashboard

| Component | Status | Files | Docs |
|-----------|--------|-------|------|
| Hexagonal Coords | ✅ Complete | `coordinates/hexagonal.rs` | ✅ `docs/type/001`, `docs/algorithm/001` |
| Pixel Conversion | ✅ Complete | `coordinates/pixel.rs` | ✅ `docs/algorithm/001` |
| Square Coords | ✅ Complete | `coordinates/square.rs` | ✅ `docs/type/001` |
| Triangular Coords | ✅ Complete | `coordinates/triangular.rs` | ✅ `docs/invariant/001` |
| Isometric Coords | ✅ Complete | `coordinates/isometric.rs` | ✅ `docs/type/001` |
| Coordinate Conversion | ✅ Complete | `coordinates/conversion.rs` | ✅ `docs/algorithm/005` |
| Grid Storage (`Grid2D`) | ✅ Complete | `collection.rs` | ✅ `docs/data_structure/001` |
| Rectangular Layout over Hex Grids | ✅ Complete | `layout.rs` | ⚠️ no `docs/` instance yet |
| Spatial Partitioning (`Quadtree`) | ✅ Complete | `spatial.rs` | ✅ `docs/data_structure/002` |
| Hex Geometry / Mesh Gen | ✅ Complete | `geometry.rs` | ✅ `docs/algorithm/004` |
| A* Pathfinding | ✅ Complete | `pathfind.rs` | ✅ `docs/algorithm/002` |
| ECS Integration (`World`, components, systems) | ✅ Complete | `ecs/` | ✅ `docs/api/001`, `docs/type/002` |
| Field of View | ✅ Complete | `field_of_view.rs` | ✅ `docs/algorithm/003` |
| Flow Fields | ⚠️ Scaffolded only — every query returns a fixed stub value | `flowfield.rs` | ⚠️ `docs/pitfall/001` |
| Region analysis / flood fill / multi-grid pathfinding | ⏳ Not started | - | - |
| Serialization / save-load | ⚠️ Complete except compression | `serialization.rs` | ✅ `docs/persistence/001` · ⚠️ `docs/pitfall/003` |
| Event System | ✅ Complete | `events.rs` | - |
| Debug Visualization | ✅ Complete | `debug.rs` | - |
| Game Systems (turn order, resources, quests) | ✅ Complete | `game_systems.rs` | - |
| Procedural Generation (WFC, tileset, noise) | ⏳ Not started | - | - |
| Benchmark Suite | ✅ Complete | `benches/` (2 suites) | - |
| Examples | ✅ Complete | `examples/tiles_tools/` (12 example crates) | ✅ `readme.md` example table |

**Current, reproducible test count:** 237 tests passing (`cargo nextest run -p tiles_tools --all-features`, verified 2026-08-10). Re-run the command yourself for the live number — do not trust a number written into this file, it will drift.

## Known Gaps

Three functional gaps are known and already documented in `docs/pitfall/` — this section links to them rather than duplicating their content, per `docs/pitfall/readme.md`:

| Pitfall | Impact | Blocking? |
|---------|--------|-----------|
| [`pitfall/001`](docs/pitfall/001_flow_field_algorithm_unimplemented.md) — Flow Field algorithm unimplemented | `FlowField`/`IntegrationField` compile and run but every query returns a fixed constant | Blocks real RTS-style multi-unit movement |
| [`pitfall/003`](docs/pitfall/003_savefile_compression_is_a_fake_wrapper.md) — Save-file compression is a fake wrapper | `data_compress` adds a 7-byte header without shrinking anything | Non-blocking — save/load still round-trips correctly, just without real compression |
| [`pitfall/004`](docs/pitfall/004_hexagonal_axial_distance_method_ambiguity.md) — Hexagonal axial distance method ambiguity | Two same-named `distance` methods resolve differently by argument shape | Non-blocking — surprising, not incorrect, when used carefully |

## Definition of Done

**Code Quality:**
- [ ] All code follows existing patterns from `hexagonal.rs`
- [ ] Zero compiler warnings with `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] All public functions have rustdoc comments with examples
- [ ] Code formatted per this workspace's custom codestyle (not `cargo fmt`)

**Testing:**
- [ ] Unit tests for all public functions
- [ ] Integration tests for cross-module behavior (pathfinding, ECS, etc.)
- [ ] `cargo nextest run --all-features` passes
- [ ] Test coverage matches the depth of the hexagonal implementation

**Documentation:**
- [ ] All public APIs documented
- [ ] Code examples in rustdoc compile and run
- [ ] New Domain Types/algorithms get a `docs/` doc instance (see `docs/definition/readme.md`)

**Integration:**
- [ ] Works with existing `Grid2D` storage
- [ ] Compatible with the A* pathfinding algorithm
- [ ] No breaking changes to existing APIs without a changelog entry

---

## Phase 1: Hexagonal Foundation

### Milestone 01: ECS Research ✅ (2 days completed)
**Outcome:** Selected HECS over Bevy ECS  
**Deliverable:** [`docs/architectural_evaluation/001_ecs_library_selection.md`](docs/architectural_evaluation/001_ecs_library_selection.md) with analysis  
**Impact:** Foundation for Phase 3 ECS integration

### Milestone 02: Hexagonal Coords ✅ (3 days completed)
**Outcome:** Full hexagonal coordinate system with type safety  
**Key Features:** Axial/Offset systems, both orientations, math ops  
**Files:** `src/coordinates/hexagonal.rs` (450 LOC)

### Milestone 03: `pixel_conversion_implement` ✅
*   **Description:** Accurate hex-to-pixel coordinate transformations
*   **Status:** **COMPLETED** - Precise transformations with both orientations
*   **Deliverables:**
    - Pixel coordinate struct with vector operations
    - Hex-to-pixel conversion for both Pointy and Flat orientations
    - Pixel-to-hex conversion with proper rounding
    - Integration with ndarray_cg for vector math
*   **Files:** `src/coordinates/pixel.rs`

### Milestone 04: `grid_storage_implement` ✅
*   **Description:** Generic 2D grid storage for coordinate-indexed data
*   **Status:** **COMPLETED** - Full Grid2D implementation with indexing
*   **Deliverables:**
    - Generic `Grid2D<System, Orientation, T>` container
    - Coordinate-based indexing with bounds checking
    - Iterator support (iter, indexed_iter)
    - Insert/remove/get operations for Option<T> grids
    - Default and custom initialization functions
*   **Files:** `src/collection.rs`

### Milestone 05: `pathfinding_generic_implement` ✅
*   **Description:** Generic A* pathfinding algorithm for any coordinate system
*   **Status:** **COMPLETED** - Trait-based generic implementation
*   **Deliverables:**
    - Generic A* function working with Distance/Neighbors traits
    - Configurable accessibility and cost functions
    - Integration with pathfinding crate for optimization
    - Type-safe pathfinding for any coordinate system
*   **Files:** `src/pathfind.rs`

### Milestone 06: `geometry_hexagonal_implement` ✅
*   **Description:** Hexagonal mesh generation and geometric utilities
*   **Status:** **COMPLETED** - Full mesh generation capabilities
*   **Deliverables:**
    - Hexagonal triangle mesh generation
    - Vertex data creation for rendering
    - Transformation matrix support
    - Line generation utilities
    - Unit hexagon primitives
*   **Files:** `src/geometry.rs`

### Milestone 07: `testing_infrastructure_establish` ✅
*   **Description:** Comprehensive testing framework with Test Matrix methodology
*   **Status:** **COMPLETED** - Full test coverage with integration tests
*   **Deliverables:**
    - Integration test structure in `tests/` directory
    - Test Matrix documentation for systematic coverage
    - Feature-gated integration tests
    - Comprehensive coordinate system testing
    - Pathfinding algorithm validation
*   **Files:** `tests/integration/`

---

## Phase 2: Grid System Extensions ✅ Complete

Universal grid system library with seamless interoperability across four coordinate systems.

### Milestone 08: Square Coordinates ✅
**Outcome:** 4-connected (Manhattan) and 8-connected (Chebyshev) square grids, phantom-typed like the hexagonal pattern  
**Files:** `src/coordinates/square.rs`, `tests/integration/square_coords_tests.rs`

### Milestone 09: Triangular Coordinates ✅
**Outcome:** 12-neighbor tessellation (3 edge + 9 vertex adjacent) with dual up/down triangle types  
**Files:** `src/coordinates/triangular.rs`, `tests/integration/triangular_coords_tests.rs`, [`docs/invariant/001`](docs/invariant/001_triangular_coordinate_sum_constraint.md) documents the coordinate-sum correctness constraint

### Milestone 10: Isometric Coordinates ✅
**Outcome:** Pseudo-3D diamond-projection coordinates with screen-to-world/world-to-screen transforms, built on the square grid's underlying logic  
**Files:** `src/coordinates/isometric.rs`, `tests/integration/isometric_coords_tests.rs`

### Milestone 11: Conversion Utilities ✅
**Outcome:** Universal coordinate system interoperability — exact conversions where geometrically possible (Square ↔ Isometric, perfect roundtrip), approximate conversions elsewhere (Hexagonal, Triangular ↔ all others), plus batch conversion utilities  
**Files:** `src/coordinates/conversion.rs`, `tests/integration/conversion_tests.rs`, [`docs/algorithm/005`](docs/algorithm/005_coordinate_system_conversion.md)

---

## Phase 3: ECS Integration ✅ Complete

**Goal:** HECS-based ECS with grid-aware components and systems.

**Delivered:** a full `hecs`-backed `World` with `Position`, `Movable`, `Health`, `Stats`, `Team`, `AI`, `Animation`, `PlayerControlled` components and Movement, Combat, AI, Animation systems, plus an equipment/abilities system and initiative-based turn management. See [`docs/api/001`](docs/api/001_ecs_world_runtime_api.md) (runtime API) and [`docs/type/002`](docs/type/002_ecs_component_vocabulary.md) (component vocabulary). Exercised by the `ecs_collision_demo` and `tactical_rpg` examples.

**Movement requests:** `World::movement_request` queues a typed target coordinate and the next `update` applies it to the entity's `Position` component, emitting `GameEvent::EntityMoved` (implemented by task 063; formerly the layer's one no-op call, tracked as `pitfall/002` until retired).

---

## Phase 4: Advanced Analysis 🚧 Partially Complete

**Field of View** ✅ Complete — shadowcasting and raycasting algorithms, light source management, visibility state tracking. See [`docs/algorithm/003`](docs/algorithm/003_field_of_view_calculation.md); exercised by `field_of_view_demo` and `stealth_game`.

**Flow Fields** ⚠️ Scaffolded only — the `FlowField`/`IntegrationField` API surface exists and compiles, but every method body is a stub returning a fixed value; no real integration-field or flow-direction computation happens. See [`docs/pitfall/001`](docs/pitfall/001_flow_field_algorithm_unimplemented.md).

**Region analysis, flood fill, multi-grid pathfinding utilities** ⏳ Not started — no corresponding module, doc instance, or test exists anywhere in the crate.

---

## Phase 5: Procedural Generation ⏳ Not Started

No Wave Function Collapse, tileset-definition, noise-generation, or configuration-driven generation code exists anywhere in `src/` — confirmed by a repo-wide search for these terms. This phase has not been started at all, unlike Phases 3/4 which have partial implementations.

**Scope (unchanged from original plan):**
- Wave Function Collapse implementation
- Tileset definition system with constraints
- Noise generation for terrain
- Configuration-driven generation

---

## Phase 6: Performance & Examples ✅ Substantially Complete

**Examples** ✅ Complete — 12 standalone example crates under `examples/tiles_tools/` (`beginner_tutorial`, `tactical_rpg`, `stealth_game`, `serialization_demo`, `advanced_pathfinding_demo`, `debug_demo`, `ecs_collision_demo`, `event_system_demo`, `field_of_view_demo`, `game_of_life`, `game_systems_demo`, `simple_collision_demo`), covering every planned deliverable (Game of Life, Tactical RPG) and considerably more. See `readme.md`'s example table and `examples/how_to_run.md`.

**Benchmarking suite** ✅ Complete — `benches/coordinate_benchmarks.rs` and `benches/pathfinding_benchmarks.rs` (criterion-based, feature-gated).

**Performance optimization pass** — no dedicated tracked activity found; not claimed as either done or outstanding here. If a specific performance regression or bottleneck is found, file it as a bug rather than reopening this phase generically.

---

## Next Priority Actions

In rough priority order, grounded in [Known Gaps](#known-gaps) and the phase statuses above:

1. **Close `docs/pitfall/001`** — implement a real Flow Field / Integration Field algorithm (replace the stub bodies in `flowfield.rs`). Unblocks RTS-style multi-unit movement, the stated purpose of the module.
2. **Phase 5: Procedural Generation** — genuinely unstarted; Wave Function Collapse is the largest single piece of scope left in the whole roadmap.
3. **Region analysis / flood fill / multi-grid pathfinding** (remainder of Phase 4) — no code exists yet.
4. **Lower priority, non-blocking:** `docs/pitfall/003` (real compression) and `docs/pitfall/004` (rename or merge the two ambiguous `distance` methods).

---

## Success Metrics & Risk Management

### Definition of Success
- **API Stability:** No breaking changes between minor versions without a changelog entry
- **Performance:** All operations under specified time limits (see `docs/algorithm/` for per-algorithm complexity notes)
- **Documentation:** rustdoc coverage with working examples for all public APIs
- **Testing:** integration tests covering every coordinate system and cross-module interaction (pathfinding, ECS, FOV)
- **Usability:** new contributors productive within 2 hours using this roadmap + `docs/definition/readme.md`

### Risk Mitigation
- **Scope Creep:** phase boundaries stay meaningful even after most phases are complete — new work goes in the phase it actually belongs to, not bolted onto whichever phase is currently open
- **Stale Documentation:** this roadmap previously drifted far enough from reality to contradict itself (see Revision History) — re-verify against `src/`, `docs/`, and a live test run before trusting any status claim here, including this one, more than a few months old
- **Performance Issues:** benchmarking suite exists (`benches/`) — extend it before optimizing, not after
- **API Design Flaws:** 12 real examples exist and exercise most of the public API — run them when changing a public signature
- **Testing Gaps:** Test Matrix methodology (see `tests/integration/`) documents intended coverage per module

---

## Implementation Notes

### Architectural Principles Applied

- **Dependency Inversion**: All algorithms work with abstract traits (Distance, Neighbors)
- **Type Safety**: Generic coordinate systems prevent mixing incompatible types
- **Modularity**: Each coordinate system is self-contained and interchangeable
- **Performance First**: Cache-friendly data layouts and optimized algorithms
- **Extensibility**: New grid types can be added without modifying existing code

### Current Architecture Strengths

1. **Generic Design**: Pathfinding works with any coordinate system implementing the traits
2. **Type Safety**: Phantom types prevent coordinate system mixing errors
3. **Mathematical Correctness**: All coordinate transformations are mathematically accurate
4. **Performance Optimized**: Efficient data structures and algorithms throughout
5. **Comprehensive Testing**: Test Matrix methodology ensures complete coverage

---

## Revision History

- **2026-08-10** — Rewritten from ground truth (`src/`, `tests/`, `docs/`, `changelog.md`, `examples/tiles_tools/`, a live `cargo nextest` run) to resolve a self-contradictory Phase 2 status (a "COMPLETED!" dashboard next to a "0/4 milestones complete" table) and outdated "Phase 3 just started, Phases 4-6 pure future work" framing. Actual state at rewrite time: Phases 1, 2, and 6 complete; Phase 3 complete except one documented no-op (`docs/pitfall/002`); Phase 4 complete for Field of View but Flow Fields is a stub (`docs/pitfall/001`) and region analysis/flood fill was never started; Phase 5 (Procedural Generation) genuinely not started. Milestone numbering and phase structure preserved; the Milestone 08 hour-by-hour task template was removed as no-longer-actionable execution detail now that the milestone is long complete, condensed into the same historical-summary style already used for Milestones 01-07.
- **2025-08-08** — Prior version (1.3), the one this rewrite replaced.

### Updated Project Status

**Current State:** Phases 1, 2, 3, and 6 complete; Phase 4 substantially complete with documented gaps; Phase 5 not started.
**Next Priority:** see [Next Priority Actions](#next-priority-actions) above.
**Target:** Complete multi-grid-system library with functional flow fields, functional ECS movement, and procedural generation.

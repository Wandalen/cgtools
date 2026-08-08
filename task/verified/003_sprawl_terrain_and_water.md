# SPRAWL Milestone 2: Terrain and Water

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** 002

## Goal

Implement noise-based elevation/moisture terrain generation and marching-squares shoreline vectorization in the `sprawl` crate, wiring the `step_terrain()` and `step_hydrology()` stub exports declared by task 002 with real bodies — proven by producing visually distinct biomes from an elevation+moisture grid, closed vector polygons from binary terrain data, and 60 FPS canvas rendering sustained with pan/zoom at 2048x2048 of real generated terrain. Motivated by terrain and water being the visual/spatial foundation every later milestone (hub placement, traffic, parcels) depends on for coastline-aware placement. Testable: noise-based generation yields visually distinct biomes; marching squares yields closed polygon rings; render sustains 60 FPS at 2048x2048 with real terrain.

**Related Tasks:** Split from `001` (`task/unverified/001_sprawl_procedural_city_dashboard.md` — pending `Q-01` supersession authorization) per `Q-01` in `task/decisions.md`, Option A. Second of 5 sibling tasks; `blocked_by` 002 (Wasm Bridge and Canvas). Task 004 is `blocked_by` this one.

## In Scope

- New workspace dependencies registered in root `Cargo.toml` `[workspace.dependencies]`: `noise` (terrain elevation/moisture generation), `geo` (shoreline/water/land polygon operations)
- `terrain` module: layered Simplex/Perlin noise (`noise` crate) → elevation `ndarray::Array2<f64>` + moisture `ndarray::Array2<f64>`; biome classification via thresholds on (elevation, moisture) → enum { Water, Sand, Grass, Rock, Snow }; parallel row-wise noise evaluation via `rayon`
- `shoreline` module: marching squares vectorization of a binary (elevation < sea_level → water) grid → `Vec<Vec<(f64, f64)>>` closed polygon rings, converted to `geo::Polygon`; contour smoothing via Catmull-Rom/Chaikin subdivision (`primitive_generation` curve utilities)
- `wasm_bridge::step_terrain()` and `wasm_bridge::step_hydrology()` implemented against the stub signatures declared in task 002 (`step_hydrology()` here wires shoreline-vectorization output only — see Out of Scope)
- Rendering: terrain biome colors, water polygon fill, integrated into the `minwebgl` render loop established in task 002
- 60 FPS canvas rendering with pan/zoom sustained at 2048x2048 with real generated terrain — supersedes task 002's placeholder-content proof with real content
- Geometry: `geo` crate boolean operations for shoreline/water/land polygons; point-in-polygon via `geo::Contains`

## Out of Scope

- Hydraulic erosion simulation (particle-based raindrop/gradient-descent river carving, lake pooling) — explicitly marked "optional stretch" under Milestone 2 in the original specification; `step_hydrology()` in this task wires shoreline vectorization only, not erosion
- Hub placement, traffic simulation, road generation, bridges — task 004
- Parcel subdivision, building footprints, label relaxation — task 005
- AI integrations, segmentation mask export — task 006

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `verb/test` passes with zero failures and zero warnings
-   No function exceeds 50 lines; no duplication; public items have `///` doc comments
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`
-   All Rust code uses 2-space indentation, `mod_interface` pattern, `error_tools` exclusively
-   All tests in `tests/` directories, no mocking, no `cargo fmt`
-   `sprawl` crate follows workspace conventions: `mod_interface`, `former` builders, feature-gated modules
-   Maximize reuse of existing workspace crates (`rayon`, `primitive_generation`); no reimplementation of existing functionality
-   New workspace dependencies (`noise`, `geo`) added to root `Cargo.toml` `[workspace.dependencies]`
-   All rendering goes through `minwebgl`

## Test Matrix

*(Required for tasks that produce tests. Write before writing any test code.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Generate terrain from elevation+moisture noise grid | default seed | Produces visually distinct biomes (Water/Sand/Grass/Rock/Snow) |
| T02 | Run marching squares on binary terrain grid (sea_level threshold) | default terrain | Produces closed vector polygon rings |
| T03 | Render 2048x2048 real generated terrain with pan/zoom active | Canvas/WebGL pipeline | Sustains 60 FPS |
| T04 | Workspace dependency registration | root `Cargo.toml` `[workspace.dependencies]` | `noise`, `geo` present and resolve |

## Acceptance Criteria

- Terrain generation produces visually distinct biomes from noise (elevation + moisture grid)
- Marching squares produces closed vector polygons from binary terrain data
- Terrain renders with biome colors; water polygons render with fill
- 60 FPS canvas rendering sustained with pan/zoom for a 2048x2048 real generated terrain
- `noise` and `geo` are registered in root `Cargo.toml` `[workspace.dependencies]`
- Hydraulic erosion is absent (deferred, optional stretch)
- All existing workspace tests continue to pass (Level 3: nextest + doctests + clippy)
- No new clippy warnings introduced
- Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Terrain & shoreline**
- [ ] C1 — Does terrain generation produce visually distinct biomes from the elevation+moisture grid?
- [ ] C2 — Does marching squares produce closed vector polygons from binary terrain data?

**Rendering**
- [ ] C3 — Does terrain render with biome colors and do water polygons render with fill?
- [ ] C4 — Is 60 FPS sustained at 2048x2048 with real generated terrain and pan/zoom active?

**Out of Scope confirmation**
- [ ] C5 — Is hydraulic erosion simulation absent from `sprawl::hydrology` (deferred, optional stretch)?
- [ ] C6 — Is hub/traffic/road/parcel/AI generation logic absent?

### Measurements

- [ ] M1 — Render frame rate: `2048x2048 real terrain, pan/zoom active` → sustains 60 FPS (was: 60 FPS with placeholder content only, task 002)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures (Level 3: nextest + doctests + clippy)
- [ ] I2 — compiler clean: `cargo clippy --all-targets --all-features -- -D warnings` → 0 new warnings

### Anti-faking checks

- [ ] AF1 — real generation coverage: `grep -rn "#\[test\]" module/helper/sprawl/tests/` → at least one test each for `terrain` and `shoreline` modules, not a single monolithic smoke test

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

**Aggregate verdict:** PASS — all 8 dimensions 🟢 on both passes.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-08 12:55:06]** `FILED` — Filed by splitting task 001 (`task/cancelled/001_sprawl_procedural_city_dashboard.md`) per `Q-01` in `task/decisions.md`, Option A (`tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`, adapted from crate-boundary to milestone-boundary partitioning). Covers Development Milestone 2 only. Goal: real terrain/water generation and rendering wired into the wasm_bridge scaffold from task 002.
- **[2026-08-08 12:55:06]** `VERIFY_PASS` — Readiness Verification Gate (Tier 2 Dual-Role Self-Check) run: all 8 dimensions PASS on first pass, no issues found. State → 🎯 Verified; file moved to `task/verified/`.

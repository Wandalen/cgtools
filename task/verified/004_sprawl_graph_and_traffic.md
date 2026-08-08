# SPRAWL Milestone 3: Graph and Traffic

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
- **blocked_by:** 003

## Goal

Implement Poisson-disk hub placement, A*-based agent traffic simulation, and road-hierarchy classification/rendering in the `sprawl` crate, wiring the `step_hubs()` and `step_traffic(agent_count)` stub exports declared by task 002 with real bodies — proven by evenly-spaced coastline-biased hub placement, a 1,000+-agent A* simulation completing in under 2 seconds, and roads rendering with width matching their classification. Motivated by hub/traffic/road infrastructure being the load-bearing skeleton that parcel subdivision (task 005) partitions against. Testable: Poisson disk sampling yields evenly-spaced, suitability-biased hubs; A* simulation with 1,000+ agents completes in <2s; heatmap thresholds correctly classify highway/arterial/local roads.

**Related Tasks:** Split from `001` (`task/unverified/001_sprawl_procedural_city_dashboard.md` — pending `Q-01` supersession authorization) per `Q-01` in `task/decisions.md`, Option A. Third of 5 sibling tasks; `blocked_by` 003 (Terrain and Water). Task 005 is `blocked_by` this one.

**Cross-reference:** Bridge detection/geometry is described in the original Technical Specification's Phase 3 ("Urban Planning and Infrastructure") alongside roads, but the original Development Milestones section places "Implement bridge detection and rendering" under Milestone 4, not Milestone 3. This task follows the Development Milestones' own placement (the authoritative split boundary) — bridges are delivered by task 005, not here.

## In Scope

- New workspace dependency registered in root `Cargo.toml` `[workspace.dependencies]`: `petgraph` (road network graph representation) — `geo` was already registered by task 003
- `hubs` module: Poisson disk sampling (Bridson's algorithm, O(n)) for infrastructure hub placement; suitability heuristic (flat terrain — low elevation gradient — near water/coastline); `tiles_tools::spatial` quadtree for neighbor rejection during sampling; output `Vec<Hub>` with position and type (port, industrial, residential, commercial)
- `traffic` module: navigation graph built from the terrain grid (grid nodes, terrain-cost edges — steep = expensive, water = impassable); `pathfinding::directed::astar` for individual agent routing between hub pairs; `petgraph::Graph` for explicit road-network graph representation; spawn 1,000–10,000 agents, pathfind between random hub pairs, accumulate path segments into an `ndarray::Array2<u32>` heatmap; `rayon` for parallel agent batches; classify highway/arterial/local via heatmap thresholds
- `roads` module: convert heatmap to vector road segments (trace connected components above threshold); Douglas-Peucker simplification via `geo`; assign road width by classification; render via `line_tools` (caps, joins, variable width)
- `wasm_bridge::step_hubs()` and `wasm_bridge::step_traffic(agent_count: u32)` implemented against the stub signatures declared in task 002
- Traffic heatmap rendering

## Out of Scope

- Bridge detection/geometry (road-water intersection, perpendicular crossing, span + ramps) — placed under Milestone 4 per the original Development Milestones ordering; delivered by task 005
- Parcel subdivision, building footprints — task 005
- Label relaxation, AI integrations, segmentation mask export — task 006
- Terrain/shoreline generation — delivered by task 003 (consumed here as input)

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
-   Maximize reuse of existing workspace crates (`tiles_tools`, `pathfinding`, `geo`, `line_tools`, `rayon`); no reimplementation of existing functionality
-   New workspace dependency (`petgraph`) added to root `Cargo.toml` `[workspace.dependencies]`
-   All rendering goes through `minwebgl`/`line_tools`

## Test Matrix

*(Required for tasks that produce tests. Write before writing any test code.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Run Poisson disk sampling (Bridson's algorithm) on terrain | coastline-suitability bias enabled | Evenly-spaced hubs biased toward flat, coastal terrain |
| T02 | Run A* traffic simulation with 1,000+ agents | wasm release build | Completes in <2 seconds |
| T03 | Classify traffic heatmap cells against thresholds | threshold_high / threshold_med | Produces highway/arterial/local road hierarchy |
| T04 | Render classified road network | `line_tools`, width by classification | Roads render with correct width per classification tier |
| T05 | Workspace dependency registration | root `Cargo.toml` `[workspace.dependencies]` | `petgraph` present and resolves |

## Acceptance Criteria

- Poisson disk sampling generates evenly-spaced hubs with terrain suitability bias
- A* traffic simulation with 1,000+ agents completes in <2 seconds (wasm, release mode)
- Road hierarchy (highway/arterial/local) derived from traffic heatmap thresholds
- Roads render via `line_tools` with width matching classification
- `petgraph` is registered in root `Cargo.toml` `[workspace.dependencies]`
- Bridge geometry is absent (delivered by task 005)
- All existing workspace tests continue to pass (Level 3: nextest + doctests + clippy)
- No new clippy warnings introduced
- Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Hubs & traffic**
- [ ] C1 — Does Poisson disk sampling generate evenly-spaced hubs with terrain-suitability bias?
- [ ] C2 — Does the A* traffic simulation with 1,000+ agents complete in under 2 seconds (wasm release)?

**Roads**
- [ ] C3 — Is road hierarchy (highway/arterial/local) correctly derived from traffic heatmap thresholds?
- [ ] C4 — Do roads render with width matching their classification?

**Out of Scope confirmation**
- [ ] C5 — Is bridge detection/geometry logic absent?
- [ ] C6 — Is parcel/label/AI generation logic absent?

### Measurements

- [ ] M1 — Traffic simulation throughput: `1,000+ agent A* run, wasm release build` → completes in <2s (was: not yet implemented)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures (Level 3: nextest + doctests + clippy)
- [ ] I2 — compiler clean: `cargo clippy --all-targets --all-features -- -D warnings` → 0 new warnings

### Anti-faking checks

- [ ] AF1 — real generation coverage: `grep -rn "#\[test\]" module/helper/sprawl/tests/` → at least one test each for `hubs`, `traffic`, and `roads` modules, not a single monolithic smoke test

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

- **[2026-08-08 12:55:06]** `FILED` — Filed by splitting task 001 (`task/cancelled/001_sprawl_procedural_city_dashboard.md`) per `Q-01` in `task/decisions.md`, Option A (`tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`, adapted from crate-boundary to milestone-boundary partitioning). Covers Development Milestone 3 only. Goal: hub placement, A* traffic simulation, and classified road rendering wired into the wasm_bridge scaffold from task 002. Bridge detection/geometry deliberately excluded despite Technical Specification Phase 3 grouping — see Cross-reference note above; attributed to task 005 per Development Milestones' own ordering.
- **[2026-08-08 12:55:06]** `VERIFY_PASS` — Readiness Verification Gate (Tier 2 Dual-Role Self-Check) run: all 8 dimensions PASS on first pass, no issues found. State → 🎯 Verified; file moved to `task/verified/`.

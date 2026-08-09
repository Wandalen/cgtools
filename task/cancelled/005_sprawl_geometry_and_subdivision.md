# SPRAWL Milestone 4: Geometry and Subdivision

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🚫 (Cancelled)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** 004

## Goal

Implement recursive polygon subdivision for city-block parcels, building-footprint instanced rendering, and road-water bridge detection/geometry in the `sprawl` crate, wiring the `step_parcels()` stub export declared by task 002 with a real body — proven by non-overlapping city-block parcels subdividing the road network's polygons, correctly-generated bridge spans at road-water crossings, and instanced building-footprint rendering. Motivated by parcels and bridges being the last structural-geometry layer before the AI/polish pass (task 006) can operate on a complete map. Testable: OBB subdivision yields non-overlapping parcels; bridge geometry generates correctly at detected crossings; footprints render instanced without overlap.

**Related Tasks:** Split from `001` (`task/unverified/001_sprawl_procedural_city_dashboard.md` — pending `Q-01` supersession authorization) per `Q-01` in `task/decisions.md`, Option A. Fourth of 5 sibling tasks; `blocked_by` 004 (Graph and Traffic). Task 006 is `blocked_by` this one.

**Cross-reference:** Bridge detection/geometry is delivered here per the original Development Milestones' own placement ("Implement bridge detection and rendering" under Milestone 4), even though the Technical Specification's Phase 3 describes it alongside roads — see task 004's History for the corresponding note. Label relaxation is described in the Technical Specification's Phase 4 alongside parcels ("City Blocks and Labeling"), but the original Development Milestones place "Implement label relaxation algorithm" under Milestone 5, not Milestone 4 — this task follows Development Milestones' own placement (the authoritative split boundary); labels are delivered by task 006, not here.

## In Scope

- New workspace dependency registered in root `Cargo.toml` `[workspace.dependencies]`: `rstar` (R-tree spatial index for "which buildings are near this road segment" bounding-box overlap queries — Technical Specification Phase 1's "Spatial Indexing")
- `parcels` module: recursive polygon subdivision via OBB (Oriented Bounding Box) subdivision — compute OBB, split along the longest axis, recurse until area < min_parcel_size; alternative Voronoi relaxation via `csgrs` (Delaunay feature) → dual Voronoi; `wfc` for procedural district type assignment (commercial, residential, park, industrial); output `Vec<Parcel>` (polygon, type, area); `rstar` R-tree index for building-to-road adjacency queries
- Building-footprint generation within parcels; instanced-geometry rendering (reuse `primitive_generation` for footprint tessellation)
- `bridges` module: road-water intersection detection; shortest perpendicular crossing via `geo::line_intersection`; bridge geometry generation (straight span with on/off ramps) — see Cross-reference above
- `wasm_bridge::step_parcels()` implemented against the stub signature declared in task 002

## Out of Scope

- Label relaxation — placed under Milestone 5 per the original Development Milestones ordering despite the Technical Specification's Phase 4 pairing it with parcels; delivered by task 006 (see Cross-reference above)
- AI integrations, segmentation mask export, dashboard UI wiring, dark theme styling — task 006
- Hub placement, traffic simulation, road generation — delivered by task 004 (consumed here as input)

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
-   Maximize reuse of existing workspace crates (`csgrs`, `wfc`, `geo`, `primitive_generation`); no reimplementation of existing functionality
-   New workspace dependency (`rstar`) added to root `Cargo.toml` `[workspace.dependencies]`
-   All rendering goes through `minwebgl`

## Test Matrix

*(Required for tasks that produce tests. Write before writing any test code.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Subdivide polygons formed by the road network | OBB subdivision | Produces non-overlapping city-block parcels |
| T02 | Detect road-water intersections and generate bridge geometry | `geo::line_intersection` perpendicular crossing | Produces a bridge span with on/off ramps at each crossing |
| T03 | Render building footprints as instanced geometry | parcels with assigned district type | Footprints render without overlap, correctly instanced |
| T04 | Workspace dependency registration | root `Cargo.toml` `[workspace.dependencies]` | `rstar` present and resolves |

## Acceptance Criteria

- Polygon subdivision generates non-overlapping city blocks between road segments
- Bridge geometry generates correctly at road-water intersections (span + on/off ramps)
- Building footprints render as instanced geometry within parcels
- District type assignment (commercial/residential/park/industrial) is present on generated parcels
- `rstar` is registered in root `Cargo.toml` `[workspace.dependencies]`
- Label relaxation is absent (delivered by task 006)
- All existing workspace tests continue to pass (Level 3: nextest + doctests + clippy)
- No new clippy warnings introduced
- Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Parcels & footprints**
- [ ] C1 — Does polygon subdivision generate non-overlapping city blocks between road segments?
- [ ] C2 — Do building footprints render as instanced geometry within parcels, without overlap?
- [ ] C3 — Is district type assignment present on generated parcels?

**Bridges**
- [ ] C4 — Does bridge geometry generate correctly at road-water intersections (span + on/off ramps)?

**Out of Scope confirmation**
- [ ] C5 — Is label relaxation logic absent?
- [ ] C6 — Is AI/segmentation/UI logic absent?

### Measurements

*(none — this task has no throughput/latency targets distinct from the Invariants below)*

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures (Level 3: nextest + doctests + clippy)
- [ ] I2 — compiler clean: `cargo clippy --all-targets --all-features -- -D warnings` → 0 new warnings

### Anti-faking checks

- [ ] AF1 — real generation coverage: `grep -rn "#\[test\]" module/helper/sprawl/tests/` → at least one test each for `parcels` and `bridges` modules, not a single monolithic smoke test

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | Non-blocking: parcels/footprints and bridges are structurally independent (bridges depend only on roads+water from tasks 003/004, not on parcels) — bundled here because the source's own Milestone 4 header groups them. `Q-01`/Option A authorized a per-milestone split, not per-independently-testable-cluster; recorded as a possible future seam, not split further without separate authorization | — (non-blocking; no fix required) |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 non-blocking | — |

**Aggregate verdict:** PASS — all 8 dimensions 🟢 on both passes (D2 carries one recorded Non-Blocking Finding; `governance/maav.rulebook.md § MAAV : Severity-Tiered Convergence`).

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-08 12:55:06]** `FILED` — Filed by splitting task 001 (`task/cancelled/001_sprawl_procedural_city_dashboard.md`) per `Q-01` in `task/decisions.md`, Option A (`tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`, adapted from crate-boundary to milestone-boundary partitioning). Covers Development Milestone 4 only. Goal: parcel subdivision, building footprints, and bridge geometry wired into the wasm_bridge scaffold from task 002. Label relaxation deliberately excluded despite Technical Specification Phase 4 grouping — see Cross-reference note above; attributed to task 006 per Development Milestones' own ordering.
- **[2026-08-08 12:55:06]** `VERIFY_PASS` — Readiness Verification Gate (Tier 2 Dual-Role Self-Check) run: all 8 dimensions PASS. D2's adversarial pass recorded a Non-Blocking observation (parcels/bridges are independently deliverable; see Verification Record) that does not fail the dimension. State → 🎯 Verified; file moved to `task/verified/`.
- **[2026-08-09]** `CANCELLED` — Reason: Filer (i4@wbox.pro) cancelled the entire SPRAWL initiative (parent task 001 and this milestone split, `Q-01` in `task/decisions.md`) as exploratory/idea-stage work, not committed for implementation.

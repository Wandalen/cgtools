# SPRAWL Milestone 1: Wasm Bridge and Canvas

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
- **blocked_by:** null

## Goal

Set up the `sprawl` crate (`module/helper/sprawl`) with a working `wasm_bridge` scaffold — `init(seed, width, height)`, `get_render_buffer() -> *const u8`, `set_parameter(name, value)`, `get_stats_json() -> JsValue` — proven end-to-end by drawing colored rectangles from Rust memory onto an HTML5 canvas via `minwebgl`, with `browser_input`-driven pan/zoom and a `minwebgl::exec_loop` render loop sustaining 60 FPS. Motivated by this being the foundation every other SPRAWL milestone builds on: no generation phase can render anything until the Rust→Wasm→Canvas pipeline and workspace dependency set exist. Testable: `sprawl` compiles to `wasm32-unknown-unknown`; the render loop sustains 60 FPS with placeholder content; pan/zoom responds to mouse drag and wheel input.

**Related Tasks:** Split from `001` (`task/unverified/001_sprawl_procedural_city_dashboard.md` — pending `Q-01` supersession authorization) per `Q-01` in `task/decisions.md`, Option A. First of 5 sibling tasks (002-006); tasks 003-006 are `blocked_by` this one in sequence.

## In Scope

- New crate `sprawl` at `module/helper/sprawl`: `mod_interface` pattern, `error_tools` exclusively, 2-space indentation, `tests/` directory
- `wasm_bridge` module foundational exports: `init(seed: u32, width: u32, height: u32)`, `get_render_buffer() -> *const u8`, `set_parameter(name: &str, value: f64)`, `get_stats_json() -> JsValue`
- `wasm_bridge` phase-step exports declared as stubs with fixed signatures (`step_terrain()`, `step_hydrology()`, `step_hubs()`, `step_traffic(agent_count: u32)`, `step_parcels()`) — bodies implemented by tasks 003-005 without changing this public API surface
- Rust → Wasm → Canvas proof: draw colored rectangles from Rust memory into the zero-copy render buffer
- `browser_input` wiring for pan/zoom (mouse drag, wheel zoom, pinch)
- `minwebgl::exec_loop` render loop
- Core math dependencies wired: `ndarray_cg` (Vec2/Vec3/Mat3x3/Mat4x4), `mdmath_core` (float arithmetic traits) — Technical Specification Phase 1's "2D Vector and Math Library"; both already available as existing workspace crates, no new `Cargo.toml` registration needed

## Out of Scope

- Actual terrain/hydrology/hub/traffic/parcel generation algorithms — `step_*` bodies are stubs only here; real implementations land in tasks 003 (terrain/hydrology), 004 (hubs/traffic), 005 (parcels)
- Frontend application code (React/Svelte) — lives outside the Rust workspace, same boundary as the parent task
- WebGPU rendering backend (`minwebgpu`), 3D satellite view (`renderer` crate) — deferred stretch goals, same as parent task
- Segmentation mask export, toponymy JSON export, label relaxation, dashboard UI wiring — task 006

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
-   Maximize reuse of existing workspace crates (`minwebgl`, `browser_input`, `mingl`, `ndarray_cg`, `mdmath_core`); no reimplementation of existing functionality
-   All rendering goes through `minwebgl`

## Test Matrix

*(Required for tasks that produce tests. Write before writing any test code.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Compile `sprawl` crate | target `wasm32-unknown-unknown` | Compiles without errors |
| T02 | Render loop with placeholder colored rectangles | Canvas/WebGL pipeline via `minwebgl::exec_loop` | Sustains 60 FPS |
| T03 | Pan/zoom interaction | `browser_input` mouse drag + wheel events | Canvas view translates/scales correctly |
| T04 | `wasm_bridge` scaffold call sequence | `init` → `get_render_buffer` → `set_parameter` → `get_stats_json` | All four exports callable from JS without panicking |

## Acceptance Criteria

- `sprawl` crate compiles to wasm32-unknown-unknown without errors
- Render loop sustains 60 FPS with placeholder (colored-rectangle) content
- Pan/zoom responds correctly to mouse drag and wheel/pinch input
- `wasm_bridge` exposes `init`/`get_render_buffer`/`set_parameter`/`get_stats_json`, all callable from JS
- `step_terrain`/`step_hydrology`/`step_hubs`/`step_traffic`/`step_parcels` are declared with the exact signatures tasks 003-005 will implement against
- All existing workspace tests continue to pass (Level 3: nextest + doctests + clippy)
- No new clippy warnings introduced
- Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Crate scaffold & bridge**
- [ ] C1 — Does `sprawl` compile to `wasm32-unknown-unknown` without errors?
- [ ] C2 — Does `wasm_bridge` expose `init`, `get_render_buffer`, `set_parameter`, `get_stats_json`, all callable from JS?
- [ ] C3 — Are `step_terrain`, `step_hydrology`, `step_hubs`, `step_traffic`, `step_parcels` declared with fixed signatures (stub bodies)?

**Rendering & input**
- [ ] C4 — Does the render loop draw colored rectangles from Rust memory via the zero-copy buffer?
- [ ] C5 — Does pan/zoom respond to mouse drag, wheel, and pinch input?

**Out of Scope confirmation**
- [ ] C6 — Is real terrain/hydrology/hub/traffic/parcel generation logic absent (stubs only)?
- [ ] C7 — Is the React/Svelte frontend application code absent from this crate?

### Measurements

- [ ] M1 — Render frame rate: `placeholder content, exec_loop active` → sustains 60 FPS (was: not yet implemented)

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures (Level 3: nextest + doctests + clippy)
- [ ] I2 — compiler clean: `cargo clippy --all-targets --all-features -- -D warnings` → 0 new warnings

### Anti-faking checks

- [ ] AF1 — scaffold exports are real, not no-ops: `grep -n "todo!\|unimplemented!" module/helper/sprawl/src/wasm_bridge.rs` → zero matches on the four foundational exports (`init`, `get_render_buffer`, `set_parameter`, `get_stats_json`); the five `step_*` stubs are the only permitted placeholders

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟡 | 🟢 | Adversarial pass found this task registering all 4 new workspace deps (`noise`/`geo`/`petgraph`/`rstar`) in root `Cargo.toml` while consuming none of them itself (only pre-existing `ndarray_cg`/`mdmath_core`) | Removed workspace-dependency registration from this task; redistributed by actual first consumer: `noise`+`geo`→003, `petgraph`→004, `rstar`→005 |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 1 fixed | 1/1 |

**Aggregate verdict:** PASS — all 8 dimensions 🟢 on both passes.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-08 12:47:17]** `FILED` — Filed by splitting task 001 (`task/cancelled/001_sprawl_procedural_city_dashboard.md`) per `Q-01` in `task/decisions.md`, Option A (`tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`, adapted from crate-boundary to milestone-boundary partitioning since D2/MOST-Goal-Scoped has no dedicated split procedure of its own). Covers Development Milestone 1 only. Goal: working Rust→Wasm→Canvas pipeline with pan/zoom at 60 FPS.
- **[2026-08-08 12:55:06]** `VERIFY_PASS` — Readiness Verification Gate (Tier 2 Dual-Role Self-Check) run: all 8 dimensions PASS. D6's adversarial pass caught an overbroad dependency-registration bullet (fixed in place — see Verification Record); all other dimensions clean on first pass. State → 🎯 Verified; file moved to `task/verified/`.

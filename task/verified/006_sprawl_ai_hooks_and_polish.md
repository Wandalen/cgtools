# SPRAWL Milestone 5: AI Hooks and Polish

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
- **blocked_by:** 005

## Goal

Implement segmentation-mask export, the `get_features_json()` toponymy hook, and label relaxation in the `sprawl` crate, completing the `wasm_bridge` API surface the (out-of-scope) frontend consumes for AI integrations — proven by correct color-coded segmentation output, a valid base64-encoded PNG payload, non-overlapping label placement, and valid JSON feature data callable from JS. Motivated by this being the final data-production layer: every payload an external AI call would consume (feature JSON for naming, segmentation PNG for satellite imagery) must exist before the frontend can orchestrate those calls. Testable: segmentation mask renders correct per-feature colors; mask encodes to valid base64 PNG; label relaxation converges to non-overlapping positions; `get_features_json()` returns valid JSON.

**Related Tasks:** Split from `001` (`task/unverified/001_sprawl_procedural_city_dashboard.md` — pending `Q-01` supersession authorization) per `Q-01` in `task/decisions.md`, Option A. Fifth and last of 5 sibling tasks; `blocked_by` 005 (Geometry and Subdivision). No downstream task is blocked by this one — it is the terminal milestone of the split.

**Cross-reference:** Label relaxation is delivered here per the original Development Milestones' own placement ("Implement label relaxation algorithm" under Milestone 5), even though the Technical Specification's Phase 4 describes it alongside parcels — see task 005's History for the corresponding note.

## In Scope

- `wasm_bridge::get_features_json() -> JsValue` — exposes feature coordinates/types for the (out-of-scope) frontend to send to Gemini/OpenAI for toponymy naming
- `segmentation` module: render a color-coded mask to an offscreen `ndarray::Array2<[u8; 4]>` RGBA buffer (Water `#0000FF`, Roads `#808080`, Buildings `#FFFFFF`, Parks `#00FF00`); `image` crate PNG encoding; `base64` crate encoding for API payload
- `labels` module: force-directed label placement — each label a physical body with repulsive force against other labels and road intersections; iterate N steps of spring-force simulation, converge to non-overlapping positions; constraint: labels remain within their feature's bounding box; reuse `tiles_tools::spatial` quadtree for neighbor lookup during force calculation — see Cross-reference above

## Out of Scope

- Actual calls to external AI APIs (Gemini/OpenAI, Stable Diffusion/ControlNet) — orchestrated by the (out-of-scope) frontend; this task only produces the JSON/PNG payloads they would consume
- The frontend application itself (React/Svelte, TypeScript dashboard shell), including dashboard UI wiring (left panel sliders, bottom timeline, PiP overlay) and dark theme styling (`#0B131E` background, neon cyan/magenta overlays) — lives outside the Rust workspace; same frontend boundary as the parent task's own Out of Scope, applied consistently to this milestone's UI-flavored bullets
- Terrain/hub/traffic/parcel generation — delivered by tasks 003-005 (consumed here as input)

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
-   Maximize reuse of existing workspace crates (`image`, `base64`, `serde_json`, `tiles_tools`); no reimplementation of existing functionality
-   Frontend code (React/Svelte) lives outside the Rust workspace, interfacing only via `wasm-bindgen` exports

## Test Matrix

*(Required for tasks that produce tests. Write before writing any test code.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | Render segmentation mask for all feature types | color-coded RGBA buffer | Correct color codes per feature type (water/roads/buildings/parks) |
| T02 | Encode segmentation mask | `image` crate PNG + `base64` | Produces a valid base64-encoded PNG payload |
| T03 | Run label relaxation on overlapping labels | force-directed iteration | Converges to non-overlapping label positions within feature bounding boxes |
| T04 | Call `get_features_json` from JS | `wasm_bridge` export | Returns valid JSON with feature coordinates and types |

## Acceptance Criteria

- Segmentation mask renders correct color codes for all feature types
- Segmentation mask encodes to a valid base64 PNG payload
- Label relaxation converges to non-overlapping positions within feature bounding boxes
- `get_features_json()` returns valid JSON feature data, callable from JS
- Actual external AI API calls are absent (frontend-orchestrated, out of scope)
- Frontend application code (dashboard UI, dark theme) is absent from this crate
- All existing workspace tests continue to pass (Level 3: nextest + doctests + clippy)
- No new clippy warnings introduced
- Every Test Matrix row has a corresponding passing test

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Segmentation & toponymy**
- [ ] C1 — Does the segmentation mask render correct color codes for all feature types?
- [ ] C2 — Does the mask encode to a valid base64 PNG payload?
- [ ] C3 — Does `get_features_json()` return valid JSON with feature coordinates and types?

**Labels**
- [ ] C4 — Does label relaxation converge to non-overlapping positions within feature bounding boxes?

**Out of Scope confirmation**
- [ ] C5 — Are actual external AI API calls absent (Gemini/OpenAI/ControlNet)?
- [ ] C6 — Is frontend application code (dashboard UI wiring, dark theme styling) absent from this crate?

### Measurements

*(none — this task has no throughput/latency targets distinct from the Invariants below)*

### Invariants

- [ ] I1 — test suite: `verb/test` → 0 failures (Level 3: nextest + doctests + clippy)
- [ ] I2 — compiler clean: `cargo clippy --all-targets --all-features -- -D warnings` → 0 new warnings

### Anti-faking checks

- [ ] AF1 — real generation coverage: `grep -rn "#\[test\]" module/helper/sprawl/tests/` → at least one test each for `segmentation` and `labels` modules, not a single monolithic smoke test

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | Non-blocking: segmentation export, the `get_features_json` toponymy hook, and label relaxation are structurally independent of each other (each consumes only tasks 003-005's output, never one another) — bundled here because the source's own Milestone 5 header groups them. `Q-01`/Option A authorized a per-milestone split, not per-independently-testable-cluster; recorded as a possible future seam, not split further without separate authorization | — (non-blocking; no fix required) |
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

- **[2026-08-08 12:55:06]** `FILED` — Filed by splitting task 001 (`task/cancelled/001_sprawl_procedural_city_dashboard.md`) per `Q-01` in `task/decisions.md`, Option A (`tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`, adapted from crate-boundary to milestone-boundary partitioning). Covers Development Milestone 5 only, minus dashboard UI wiring and dark-theme styling (excluded as frontend work, consistent with the parent task's own Out-of-Scope frontend boundary). Goal: segmentation export, toponymy JSON hook, and label relaxation completing the wasm_bridge API surface. Terminal task of the split — no sibling is blocked by this one.
- **[2026-08-08 12:55:06]** `VERIFY_PASS` — Readiness Verification Gate (Tier 2 Dual-Role Self-Check) run: all 8 dimensions PASS. D2's adversarial pass recorded a Non-Blocking observation (segmentation/labels/toponymy are independently deliverable; see Verification Record) that does not fail the dimension. State → 🎯 Verified; file moved to `task/verified/`.

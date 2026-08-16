# primitive_generation ladder classification

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/primitive_generation
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **verification_date:** 2026-08-16 11:39:51
- **blocked_by:** null
- **in_motion:** false
- **accepting_at:** 2026-08-16 11:28:14
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **priority:** 0
- **completed_at:** 2026-08-16 11:39:51
- **completed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

`rulebook.md`'s rendering-layer placement table names every crate on or beside the
L0-L5 ladder except `primitive_generation` — `docs/layer/001_l0_drivers.md` already
flags this explicitly as "an open classification gap, not a resolved one," noting the
crate's math-only feature gate (`future`/`math`/`diagnostics`, same shape as
`animation`'s beside-the-ladder gate) is contradicted by direct GL-context imports
in `src/primitive_data.rs` (`WebGl2RenderingContext`, `renderer::webgl::*`) and a
`dep:renderer` dependency on the full L3 engine crate. Resolve this by investigating
the actual depth of that GL coupling, recording a classification decision as a new
`Q-NN` entry in `task/decisions.md` (matching the Options/Recommendation/Assumed
shape of Q-01-Q-03), and updating `rulebook.md` and `docs/layer/001_l0_drivers.md`
to reflect it — a documentation-and-decision task with no source-code changes.
Success is testable by `grep -c "primitive_generation" rulebook.md` returning ≥1
(currently 0) and the open-gap sentence no longer appearing in `docs/layer/001_l0_drivers.md`.

## In Scope

- Read `module/helper/primitive_generation/src/primitive_data.rs` and
  `module/helper/primitive_generation/src/lib.rs` in full to characterize the GL
  coupling: is `primitives_data_to_gltf( gl : &WebGl2RenderingContext, ... )` (which
  constructs a `PbrMaterial`, creates GL buffers, and assembles a renderable `GLTF`
  scene) the crate's core value proposition, or a separable capability alongside a
  GPU-free generation core?
- Decide the classification per `rulebook.md`'s existing categories:
  - **(a) Beside-the-ladder**, like `animation`, if the GL-upload path is separable
    from pure geometry generation without a structural rewrite; or
  - **(b) An explicitly GL-coupled placement**, distinct from `animation`'s shape,
    if the coupling (including the `dep:renderer` dependency on the full L3 engine)
    is load-bearing and not worth separating.
- Record the decision as a new `Q-NN` entry in `task/decisions.md`, in the same
  Options -> Recommendation -> Assumed/Verification-mechanism/Contingency shape as
  Q-01/Q-02/Q-03, citing the specific evidence found (function name, file, line).
- Update `rulebook.md`'s rendering-layer placement section (the ladder table or the
  "Beside the ladder" list, per the decision) to add `primitive_generation`.
- Update `docs/layer/001_l0_drivers.md`'s `primitive_generation` paragraph to state
  the resolved classification instead of "an open classification gap, not a
  resolved one."

## Out of Scope

- Any change to `primitive_generation`'s `Cargo.toml` dependencies or
  `src/primitive_data.rs` code — this task documents/classifies the CURRENT state.
  If the decision recommends splitting GL-upload out of the crate, that refactor is
  a separate follow-up task, filed only if/when the decision calls for it.
- Reclassifying any other crate on or beside the ladder.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Non-code task: test-related items omitted.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `rulebook.md`, `docs/layer/001_l0_drivers.md`, and `task/decisions.md` updated
    and mutually consistent (same classification stated in all three)
-   No file under `module/helper/primitive_generation/src/` modified
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

*(Non-code documentation/decision task — rows are text-consistency checks, not `cargo test` cases.)*

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -c "primitive_generation" rulebook.md` | Updated placement section | ≥1 (was: 0) |
| T02 | Read `docs/layer/001_l0_drivers.md`'s `primitive_generation` paragraph | Updated paragraph | Does not contain "open classification gap, not a resolved one"; states a resolved position |
| T03 | Read `task/decisions.md` | New `Q-NN` entry | Present; follows Options/Recommendation/Assumed format matching Q-01-Q-03; cites `primitive_data.rs` evidence by name/line |
| T04 | `git diff --stat -- module/helper/primitive_generation/src/` | Source tree | Empty — zero source changes |

## Acceptance Criteria

-   `rulebook.md` names `primitive_generation` in its placement table or
    beside-the-ladder list
-   `docs/layer/001_l0_drivers.md`'s paragraph reflects the same resolved
    classification as `rulebook.md`
-   `task/decisions.md` has a new `Q-NN` entry recording the decision with cited
    evidence
-   Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Documentation consistency**
- [ ] C1 — Does `rulebook.md`'s placement section name `primitive_generation`?
- [ ] C2 — Does `docs/layer/001_l0_drivers.md`'s paragraph state the same classification as `rulebook.md`, with the "open classification gap" sentence removed?
- [ ] C3 — Does `task/decisions.md` contain a new `Q-NN` entry in the Options/Recommendation/Assumed shape?

**Out of Scope confirmation**
- [ ] C4 — Is `module/helper/primitive_generation/src/` untouched (zero diff)?

### Measurements

- [ ] M1 — placement mention: `grep -c "primitive_generation" rulebook.md` → ≥1 (was: 0)
- [ ] M2 — open-gap sentence removed: `grep -c "open classification gap, not a resolved one" docs/layer/001_l0_drivers.md` → 0 (was: 1)

### Invariants

- [ ] I1 — source tree unaffected: `git diff --stat -- module/helper/primitive_generation/src/` → empty
- [ ] I2 — workspace still builds: `cargo check --workspace` → 0 errors (doc-only change, unaffected)

### Anti-faking checks

- [ ] AF1 — decision cites real evidence: the `Q-NN` entry's Recommendation names the
  specific function (`primitives_data_to_gltf`) and file (`src/primitive_data.rs`)
  driving the classification, not a generic restatement of `animation`'s rationale —
  guards against rubber-stamping "beside-the-ladder, done" without engaging the
  crate's actual `dep:renderer` + GLTF-assembly coupling.

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | Live-verified `task/decisions.md` Q-01/Q-02/Q-03 precedent (`## Q-NN —` headings, Options→Recommendation→Assumed shape confirmed by direct read) | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | — | — |

## Outcomes

### Acceptance Results

- **Verified by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ (independent acceptance verifier — did not author the rulebook.md/docs/layer/decisions.md edits under review; fresh read of all artifacts this session)
- **Date:** 2026-08-16
- **Verdict:** PASS

#### Checklist

- [x] C1 — Does `rulebook.md`'s placement section name `primitive_generation`? — YES: `rulebook.md:70`, L4 row of the ladder table: "`tilemap_scene` (RON model); glTF via `renderer` loaders; procedural glTF assembly via `primitive_generation`". Direct read of the full § Rendering layer placement section (`rulebook.md:56-95`) confirms `primitive_generation` is on the L4 rung of the table and is absent from the "Beside the ladder" list (`rulebook.md:77-90`), consistent with an on-ladder (not beside-the-ladder) classification.
- [x] C2 — Does `docs/layer/001_l0_drivers.md`'s paragraph state the same classification as `rulebook.md`, with the "open classification gap" sentence removed? — YES: `docs/layer/001_l0_drivers.md:86-98` states `primitive_generation` "is **not** a beside-the-ladder consumer... It is now named in [rulebook.md]'s L4 (scene model) row as a second, procedural producer of that same artifact type... not listed beside the ladder" — matches `rulebook.md`'s L4 placement exactly (same classification, same rung). The old "open classification gap, not a resolved one" sentence is gone (confirmed by M2 below, actual count 0).
- [x] C3 — Does `task/decisions.md` contain a new `Q-NN` entry in the Options/Recommendation/Assumed shape? — YES, with a documented observation. `task/decisions.md:85-93` contains `## Q-04 — \`primitive_generation\`'s L0-L5 ladder placement`, state `✅ Decided`, owner `user1@w002`, date `2026_08_16`, listed in the Index table (`task/decisions.md:22`). It cites specific, independently-verified evidence: function name `primitives_data_to_gltf`, file `src/primitive_data.rs:141-289` (I directly confirmed the function spans exactly those lines), `dep:renderer` with `features = ["full"]` (confirmed via `Cargo.toml` read), and 8 line citations into `renderer/src/webgl/loaders/gltf.rs` (518/610/657/744/867/965/1042/1262 — all 8 independently spot-checked via `sed -n`, all read `gl : &gl::WebGl2RenderingContext,`). **Observation (non-blocking):** `task/decisions.md`'s own file-level Format rule (line 9) states the Options/Recommendation/Assumed shape applies to **🔍 Unverified** entries specifically; **✅ Decided** entries (Q-04's actual state) instead use "a single collapsed statement with rationale" — which is exactly the shape Q-04 uses, and exactly the shape used by its two true state-comparable precedents, Q-02 and Q-03 (both also ✅ Decided — confirmed by direct read of `task/decisions.md:61-82`, neither uses separate Options/Recommendation/Assumed headers). Only Q-01 uses the full Options/Recommendation/Assumed shape, but solely because it is `➖ Cancelled` (which preserves original pre-decision analysis per the same Format rule) — not a state-comparable precedent for a directly-Decided entry. Judged YES on substance: Q-04 is fully consistent with `decisions.md`'s own governing format convention and with its real ✅-Decided precedents; the task's Goal/Checklist text describing this as "Options/Recommendation/Assumed shape... matching Q-01-Q-03" overgeneralizes Q-01's Cancelled-preserved shape onto Q-02/Q-03, which is a task-authoring imprecision, not an executor defect.
- [x] C4 — Is `module/helper/primitive_generation/src/` untouched (zero diff)? — YES: `git status --porcelain -- module/helper/primitive_generation/src/` → empty output.

#### Measurements

- [x] M1 — placement mention: `grep -c "primitive_generation" rulebook.md` → `1` — MET (expected ≥1, was 0).
- [x] M2 — open-gap sentence removed: `grep -c "open classification gap, not a resolved one" docs/layer/001_l0_drivers.md` → `0` — MET (expected 0, was 1).

#### Invariants

- [x] I1 — source tree unaffected: `git diff --stat -- module/helper/primitive_generation/src/` → empty output — HOLD.
- [x] I2 — workspace still builds: `cargo check --workspace` → HOLD. Launched detached via `longrun .launch dir::/home/user1/pro/lib/yrd_gamedev/cgtools -- cargo check --workspace` (Durable Log `./-0112_longrun.log`, pid 621509). Completed naturally (not a timeout-kill): `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 57.64s`, `──── exit 0 · pid 621509 · 2026-08-16 · 11:34:26 · elapsed 57s ────`. `grep -c "^error" -- ./-0112_longrun.log` → `0`. 0 errors confirmed both by exit code and by direct text search of the log, not exit code alone.

#### Anti-faking checks

- [x] AF1 — decision cites real evidence: the `Q-NN` entry's Recommendation names the specific function (`primitives_data_to_gltf`) and file (`src/primitive_data.rs`) driving the classification, not a generic restatement of `animation`'s rationale — PASS. I independently read `primitive_data.rs` (302 lines) and `primitive.rs` (458 lines) myself rather than taking the task file's or the decision's characterization on faith, and separately verified every specific technical claim the Q-04 Recommendation makes:
  - `primitive_data.rs` imports `WebGl2RenderingContext` and `renderer::webgl::{AttributeInfo, Geometry, IndexInfo, Material, Mesh, Node, Object3D, Primitive, Scene, loaders::gltf::GLTF, material::PbrMaterial}` unconditionally (`primitive_data.rs:17-35`) — confirmed by direct read; no `#[cfg(feature = ...)]` gate appears anywhere in the file (confirmed by reading the full 302 lines).
  - `primitives_data_to_gltf` (`primitive_data.rs:141-289`, confirmed exact span by direct read) takes `gl : &WebGl2RenderingContext`, calls `gl.create_buffer()` twice (lines 157, 177), `gl::buffer::upload`/`gl::index::upload` (lines 274-275), and returns `GLTF` (imported as `renderer::webgl::loaders::gltf::GLTF`) — real GL-buffer creation and upload, not a math-only usage. It is publicly re-exported via `mod_interface!`'s `orphan use` block (`primitive_data.rs:292-302`).
  - `Cargo.toml`: `default = ["enabled"]`; `enabled` requires `dep:renderer` and `dep:minwebgl` together with `dep:mod_interface`; `renderer = { workspace = true, features = ["full"], optional = true }` — matches the decision's citation exactly.
  - Cross-check against `animation` (the crate the "beside-the-ladder" alternative would liken it to): `animation/Cargo.toml` uses the identical `minwebgl` feature gate (`future`/`math`/`diagnostics`), confirming the surface-similarity claim is real, not fabricated. `grep -rn "WebGl2RenderingContext\|WebGlBuffer\|create_buffer\|renderer::" module/helper/animation/src/` → zero matches (exit 1) — independently confirms `animation` is genuinely GL-context-free where `primitive_generation` is not.
  - Cross-check against `renderer`'s own glTF loaders (the claim that requiring a live GL context doesn't disqualify L4 membership): spot-checked all 8 cited line numbers in `renderer/src/webgl/loaders/gltf.rs` (518, 610, 657, 744, 867, 965, 1042, 1262) — every one reads `gl : &gl::WebGl2RenderingContext,` (or without trailing comma at the last parameter position), confirming the claimed shape match is real, not asserted.
  - **Falsification attempts (adversarial pass) — actively tried to make the opposite (beside-the-ladder) classification hold:** (1) Checked whether `primitive.rs` (the crate's other, larger, 458-line module) is genuinely GL/renderer-free the way `animation` is, which would support "GPU-free generation core" — `grep -n "WebGl2RenderingContext\|WebGlBuffer\|create_buffer\|renderer::" module/helper/primitive_generation/src/primitive.rs` → zero matches (exit 1); confirmed true, this module alone would support the beside-the-ladder framing if it were self-contained. (2) Tested whether the GL/renderer coupling is actually severable today by attempting a real build without it: `cargo check -p primitive_generation --no-default-features` → **fails immediately** with `error[E0433]: cannot find \`mod_interface\` in the crate root` at `lib.rs:11` — the crate cannot compile AT ALL without the default `enabled` feature bundle (which pulls in `dep:renderer`/`dep:minwebgl` together), because `lib.rs`'s `layer primitive;`/`layer primitive_data;` declarations are unconditional (no `#[cfg(feature=...)]`), unlike `text`/`font-processing`'s properly-gated symbols. This directly falsifies the strongest available counter-argument ("split the GL-upload path out, like `animation`'s shape") — there is no severable build configuration today; splitting would require a structural Cargo-feature rewrite, exactly as the decision's "Reject beside-the-ladder classification" paragraph argues. (3) Verified the `#q-04--primitive_generations-l0-l5-ladder-placement` anchor in `docs/layer/001_l0_drivers.md:89` actually resolves to the real Q-04 heading by independently computing the GitHub-slug transform of the heading text and cross-checking it against an existing same-repo precedent (`Q-03`'s heading slug, used at `task/completed/105_shader_chunks_params_new_crate.md:31` and `module/shader/shader_chunks_params_core/docs/algorithm/001_range_inference_heuristic.md:6`) — both follow the identical em-dash-collapses-to-double-hyphen pattern, confirming the Q-04 cross-reference is not a broken/guessed link. No evidence found supporting the opposite classification; every falsification attempt corroborated the L4 (on-ladder) classification instead.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-16 11:28:09 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | EXEC_COMPLETE | execution complete |
| 2026-08-16 11:28:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-16 11:39:51 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | ACCEPTANCE_PASS | acceptance passed |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-16]** `FILED` — Task filed via `/doc_tsk`-methodology gap-closure round (docs/layer follow-up): resolve `primitive_generation`'s undocumented ladder position.

## Related Documentation

- `rulebook.md` — the rendering-layer placement table and beside-the-ladder list this task updates
- `docs/layer/001_l0_drivers.md` — the doc instance carrying the open-gap paragraph this task resolves
- `task/decisions.md` — Q-01/Q-02/Q-03 precedent for this task's new `Q-NN` entry
- `module/helper/primitive_generation/src/primitive_data.rs` — the evidence source (GL-context imports, `dep:renderer` usage)

# obj_load: adopt existing mingl/minwebgl obj-loading helpers, removing 3 markers

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 2
- **state:** 🎯 (Verified)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/obj_load
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **verification_date:** 2026-08-12
- **blocked_by:** null
- **priority:** 2
- **executing_at:** 2026-08-13 02:22:14
- **executing_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **in_motion:** false
- **accepting_at:** 2026-08-14 03:33:54
- **accepting_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **verified_at:** 2026-08-14 03:41:35

## Goal

`examples/minwebgl/obj_load/src/main.rs` hand-rolls OBJ loading via a raw `tobj::load_obj_buf` call with a materials callback that unconditionally fails (`Err( tobj::LoadError::OpenFileFailed )`), and carries 3 live `qqq` markers asking for functionality that already exists elsewhere in the workspace:

- `:27` — asks for a diagnostic Report helper with verbosity control. `mingl::model::obj::ReportObjModel` + `make_reports()` (re-exported as `mingl::web::model::obj::make_reports`, and from there via `minwebgl::model::obj::make_reports` since `minwebgl/src/model/obj.rs` is a `reuse ::mingl::web::model::obj;`) already provide a `Display`-able report with bounding box, bounding sphere, and arity-set diagnostics per model. No verbosity knob exists, but the marker's core ask — "Report having all the diagnostic information inside" — is satisfied.
- `:29` — asks for a load-from-byte-slice helper. `mingl::web::model::obj::load_model_from_slice( obj_buffer: &[u8], material_folder: &str, load_options: &tobj::LoadOptions ) -> tobj::LoadResult` (re-exported through `minwebgl::model::obj`) already does exactly this, including async `.mtl` fetching via `web::file::load`.
- `:36` — asks "why error?" next to the example's own stub callback. The real answer: `load_model_from_slice`'s own callback actually attempts `web::file::load( &format!("{material_folder}/{p}") )` and only returns `OpenFileFailed` if that fetch genuinely fails (logging the error first) — the example's hand-rolled stub is an incomplete placeholder that never tries.

This task replaces the example's hand-rolled loading logic with calls to the existing `gl::model::obj::load_model_from_slice` and `gl::model::obj::make_reports`, deleting all 3 markers because the code they flagged no longer exists in its old form. No new library code — everything needed already ships in `mingl`/`minwebgl`.

## In Scope

- `examples/minwebgl/obj_load/src/main.rs`:
  - Replace the `Cursor`/`BufReader` + raw `tobj::load_obj_buf(...)` block (current lines ~22-39) with a single call to `gl::model::obj::load_model_from_slice( &obj_buffer, "static", &tobj::GPU_LOAD_OPTIONS ).await` (material folder `"static"` matches the model's own load path, `"static/suzanne.obj"`)
  - Remove the now-unused `use std::io::{ BufReader, Cursor };` import
  - Delete all 3 `qqq` markers (`:27`, `:29`, `:36`) — the code they flagged is gone
  - Replace the bare `gl::log::info!( "# of models : {}", models.len() )` diagnostic with a real report dump via `gl::model::obj::make_reports( &models, &materials )`, logging each report (handle the `materials` half of `tobj::LoadResult` — a `Result<Vec<Material>, LoadError>` — explicitly rather than leaving it commented out as the current code does)

## Out of Scope

- Marker `:41` (obj_viewer new example proposal) — filed separately as task 098 (Draft, deferred per YAGNI); not this task's concern
- Adding a "verbosity" parameter to `ReportObjModel`/`make_reports` — no concrete consumer need beyond the original marker's wish; if diagnostic output review after this task lands shows a real need, that becomes its own future task
- Any change to `mingl` or `minwebgl` library code — both helpers already exist, are public, and are re-exported through `minwebgl::model::obj`; this task only changes the example's call site
- Any change to the example's shaders, buffers, VAO setup, camera, or render loop — only the model-loading block changes

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cargo check -p minwebgl_obj_load --target wasm32-unknown-unknown` (or the workspace's standard wasm32 check invocation) passes with zero errors
-   `cargo clippy -p minwebgl_obj_load --target wasm32-unknown-unknown -- -D warnings` passes with zero warnings (no dead imports left behind)
-   All 3 markers (`:27`, `:29`, `:36`) absent from the file
-   Manual browser smoke-test: the example still loads and renders `suzanne.obj` correctly (this crate's WebGL rendering path has no automated test harness in this workspace — matches the established precedent for other minwebgl examples)
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution`
-   Task state updated to ✅ on verification pass; file moved to `task/completed/`

## Test Matrix

*(Browser-only WebGL example — no automated test harness exists for this crate's rendering path in this workspace, matching the precedent for other minwebgl examples. Verification is mechanical (check/clippy) plus manual visual confirmation, captured under Verification below rather than as automated Test Matrix rows.)*

## Acceptance Criteria

-   `examples/minwebgl/obj_load/src/main.rs` calls `gl::model::obj::load_model_from_slice` instead of raw `tobj::load_obj_buf`
-   `use std::io::{ BufReader, Cursor };` is absent (no longer needed)
-   Markers `:27`, `:29`, `:36` are absent from the file
-   `gl::model::obj::make_reports` output is logged in place of the bare model count
-   `cargo check -p minwebgl_obj_load` and `cargo clippy -p minwebgl_obj_load -- -D warnings` both exit 0
-   Manual browser run shows Suzanne still renders (no visual regression)

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**main.rs**
- [x] C1 — Does the model-loading block call `gl::model::obj::load_model_from_slice` instead of `tobj::load_obj_buf`?
- [x] C2 — Is `use std::io::{ BufReader, Cursor };` absent?
- [x] C3 — Are markers `:27`, `:29`, `:36` all absent from the file?
- [x] C4 — Does the diagnostic logging use `gl::model::obj::make_reports` instead of the bare `models.len()` call?

**Out of Scope confirmation**
- [x] C5 — Is marker `:41` still present (untouched — deferred to task 098, not deleted here)?
- [x] C6 — Are the shader/buffer/VAO/camera/render-loop sections byte-for-byte unchanged from the pre-edit file?

### Measurements

- [x] M1 — grep count: `grep -cE "for Yevgen" examples/minwebgl/obj_load/src/main.rs` → 0 (was: 3)
- [x] M2 — grep count: `grep -c "load_model_from_slice" examples/minwebgl/obj_load/src/main.rs` → ≥1 (was: 0)

### Invariants

- [x] I1 — `cargo check -p minwebgl_obj_load` (wasm32 target) → 0 errors
- [x] I2 — `cargo clippy -p minwebgl_obj_load --target wasm32-unknown-unknown -- -D warnings` → 0 warnings

### Anti-faking checks

- [x] AF1 — the replaced block genuinely calls the library function (not a re-implementation of the same logic under a different name): `grep -n "tobj::load_obj_buf\b" examples/minwebgl/obj_load/src/main.rs` → no match (only `load_obj_buf_async` inside mingl itself, not duplicated here)
- [ ] AF2 — manual browser load of the example confirms Suzanne renders — a passing `cargo check` alone does not prove the runtime behavior is unchanged

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| G1 | Scope Coherence | — | 🟢 | In Scope names exact functions/imports to change; Out of Scope explicitly excludes :41, verbosity extension, and any mingl/minwebgl library edit. | — |
| G2 | MOST Goal Quality | — | 🟢 | Motivated (3 markers pointing at code that duplicates existing library functions — read `load_model_from_slice`'s full implementation directly, confirmed it already answers `:36`), Observable (markers gone, function calls present), Scoped (one file's loading block), Testable (check/clippy + manual render). | — |
| G3 | Value/YAGNI | — | 🟢 | Null Hypothesis: "if this isn't done, does anything break?" — no, but the example keeps hand-rolling logic that duplicates existing, better-tested library code (violates this project's own no-duplication rule) and keeps 3 stale markers alive; concrete committed decision from 065's triage, not speculative. Verbosity extension explicitly deferred (no consumer need stated). | — |
| G4 | Implementation Readiness | — | 🟢 | Exact function signature confirmed by reading `mingl/src/web/model/obj.rs:205-243` directly; exact call site (`obj_load/src/main.rs:22-44`) read in full; the one open judgment call (materials `Result` handling) is named explicitly in In Scope rather than hidden. | — |
| G5 | Execution Scope | — | 🟢 | `examples/minwebgl/obj_load/src/main.rs` resolves inside this repository. | — |
| G6 | Crate Scope Unity | — | 🟢 | Sole deliverable path is inside `examples/minwebgl/obj_load` — one crate; the helpers it calls already exist and are explicitly Out of Scope to modify, so this stays single-crate despite consuming a cross-crate API. | — |
| G7 | Crate Locality | — | 🟢 | Targets the leaf example crate that owns the call site, not mingl/minwebgl (which already have the code). | — |
| G8 | Crate Single Responsibility | — | 🟢 | `obj_load`'s responsibility ("demonstrate loading and rendering an OBJ model") is unchanged — this task makes the demo use the workspace's own intended API, it doesn't add a second responsibility. | — |
| **Total** | | — | 🟢 | — | — |

Adversarial pass: the strongest challenge here is "are you sure `load_model_from_slice` and `make_reports` are actually already correct/complete, or would adopting them just move the bug?" — checked by reading `load_model_from_slice`'s full body (not just its signature): it already handles the async `.mtl` fetch-and-log-and-fail path the example's stub was missing, and is exported via `mod_interface!`'s `orphan use` block, confirming it's genuinely part of the public API, not a private helper this task would be reaching past. Checked `materials` field type (`Result<Vec<Material>, LoadError>`) is correctly called out in In Scope as a named open point rather than glossed over. No blocking finding surfaced.

## Outcomes

### Acceptance Results

- **Verified by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ (acceptance walk per tsk_verify Part B / PROC16; session distinct from the executor's)
- **Date:** 2026-08-14
- **Verdict:** FAIL (1 issue)

**Separation-of-concerns disclosure (tsk_verify B1):** verifier and executor share the coarse
`user1@w002` user@host identity (executor `.../cgtools/task/`, verifier `.../cgtools/`); the
verifying session did not author the implementation. Disclosed, not a walk blocker; the FAIL
verdict routes through `.acceptance_fail`, which carries no same-session guard.

#### Checklist

- C1 🟢 (by intent) — main.rs:22 calls
  `gl::model::obj::model_load_from_slice( &obj_buffer, "static", &tobj::GPU_LOAD_OPTIONS ).await` —
  the task text's `load_model_from_slice` drifted noun-first per the executor's EXEC_COMPLETE
  disclosure; argument list matches In Scope exactly; no raw `tobj::load_obj_buf` remains.
- C2 🟢 — `use std::io::{ BufReader, Cursor };` absent (removed in 6390aeb4's first hunk).
- C3 🟢 — all three deleted markers spelled `for Yevgen`; parent of 6390aeb4 carries exactly 3
  such hits, current file 0.
- C4 🟢 (by intent) — bare `models.len()` log replaced by a per-report loop over
  `gl::diagnostics::obj::reports_make( &models, &materials )` (main.rs:27-30). Double drift
  disclosed here: name `make_reports`→`reports_make` (executor-disclosed) AND module path
  `gl::model::obj`→`gl::diagnostics::obj` (not in the executor's disclosure; the green wasm32
  compile gate proves it is the real current API path). The materials half is handled explicitly
  (`materials.expect(...)`, main.rs:23) and the old commented-out materials line is deleted, per
  In Scope.
- C5 🟢 — the `:41` obj_viewer marker survives (spelled `for Yevhen`, drifted to line 25; parent
  had 1 such hit, current has 1) — untouched, deferred to task 098 as required.
- C6 🟢 — 6390aeb4's diff for the file is exactly two hunks (`@@ -1,7 +1,5 @@` imports,
  `@@ -21,28 +19,16 @@` loading/logging block); shaders/buffers/VAO/camera/render loop
  byte-identical.

#### Measurements

- M1 🟢 — `grep -cE "for Yevgen"` → 0 (was 3, per parent-revision count).
- M2 🟢 (by intent) — the item's literal `grep -c "load_model_from_slice"` → 0 solely because of
  the disclosed noun-first rename; the drift-corrected `grep -c "model_load_from_slice"` → 1
  (was 0 pre-edit). The item's intent — the library slice-loader is adopted — holds.

#### Invariants

- I1 🟢 — `cargo check -p minwebgl_obj_load --target wasm32-unknown-unknown` → exit 0 (detached
  run, Completion Marker `exit 0 · pid 3953999`, log `-0003_longrun.log` in session scratchpad,
  `T097_I1_CHECK=0`).
- I2 🟢 — `cargo clippy -p minwebgl_obj_load --target wasm32-unknown-unknown -- -D warnings` →
  exit 0 (`T097_I2_CLIPPY=0`, same log).

#### Anti-faking checks

- AF1 🟢 — `grep "tobj::load_obj_buf"` → no match; both helpers are genuine mingl library code
  (`module/min/mingl/src/web/model/obj.rs`), with zero local re-implementations in the example
  (`fn model_load_from_slice|fn reports_make` under the example → 0 hits).
- AF2 🔴 — NOT PERFORMED, and not performable in this environment: the item requires a manual
  browser load confirming Suzanne still renders. The executor disclosed "no browser available";
  this machine's known verification ceiling is console-tier (no pixel-level browser confirmation
  possible), and a green compile gate cannot substantiate a runtime-render claim. Blocking —
  round 2's sole remaining work is this human-gated browser smoke-test.

**Adversarial pass:** challenged whether AF2 could be down-judged non-blocking given every
mechanical gate is green — rejected as written, but the sharpest runtime-regression vector was
probed and retired: the new code `.expect()`s the materials `Result` where the old stub ignored
it, which would panic at startup if `suzanne.obj` referenced an unfetchable `.mtl`; direct
inspection shows `assets/obj/suzanne.obj` is a self-authored placeholder whose geometry contains
no `mtllib`/`usemtl` statements (its header comment states no material library is emitted), and
`index.html`'s trunk `copy-file` link is what materializes it under `static/` at build time — so
the materials expect cannot fire for the shipped asset. What remains genuinely unverified is only
the visual render itself. Also probed the undisclosed `gl::diagnostics::obj` module-path drift for
stray second consumers — none; compile gates green.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-13 02:22:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | CLAIM_EXEC | execution claimed |
| 2026-08-13 02:22:16 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | EXEC_COMPLETE | execution complete — API drift from spec: helpers are named `model_load_from_slice`/`reports_make` (noun-first), not the task text's `load_model_from_slice`/`make_reports`; re-derived call site against current `mingl/src/web/model/obj.rs` and the already-working `obj_viewer` example precedent. wasm32 check + clippy (`-D warnings`) both exit 0; AF2 (manual browser render) not performed — no browser available in this environment. |
| 2026-08-14 03:33:54 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_ACCEPT | acceptance claimed |
| 2026-08-14 03:41:35 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | ACCEPTANCE_FAIL | acceptance failed |

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-08-12** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: adopt existing `mingl`/`minwebgl` obj-loading helpers in obj_load's example, closing markers :27/:29/:36 with zero new library code.

# obj_load: adopt existing mingl/minwebgl obj-loading helpers, removing 3 markers

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🎯 (Verified)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/obj_load
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-12
- **blocked_by:** null
- **priority:** 2

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
- [ ] C1 — Does the model-loading block call `gl::model::obj::load_model_from_slice` instead of `tobj::load_obj_buf`?
- [ ] C2 — Is `use std::io::{ BufReader, Cursor };` absent?
- [ ] C3 — Are markers `:27`, `:29`, `:36` all absent from the file?
- [ ] C4 — Does the diagnostic logging use `gl::model::obj::make_reports` instead of the bare `models.len()` call?

**Out of Scope confirmation**
- [ ] C5 — Is marker `:41` still present (untouched — deferred to task 098, not deleted here)?
- [ ] C6 — Are the shader/buffer/VAO/camera/render-loop sections byte-for-byte unchanged from the pre-edit file?

### Measurements

- [ ] M1 — grep count: `grep -cE "for Yevgen" examples/minwebgl/obj_load/src/main.rs` → 0 (was: 3)
- [ ] M2 — grep count: `grep -c "load_model_from_slice" examples/minwebgl/obj_load/src/main.rs` → ≥1 (was: 0)

### Invariants

- [ ] I1 — `cargo check -p minwebgl_obj_load` (wasm32 target) → 0 errors
- [ ] I2 — `cargo clippy -p minwebgl_obj_load --target wasm32-unknown-unknown -- -D warnings` → 0 warnings

### Anti-faking checks

- [ ] AF1 — the replaced block genuinely calls the library function (not a re-implementation of the same logic under a different name): `grep -n "tobj::load_obj_buf\b" examples/minwebgl/obj_load/src/main.rs` → no match (only `load_obj_buf_async` inside mingl itself, not duplicated here)
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

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **2026-08-12** `FILED` — Task filed by user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/. Goal: adopt existing `mingl`/`minwebgl` obj-loading helpers in obj_load's example, closing markers :27/:29/:36 with zero new library code.

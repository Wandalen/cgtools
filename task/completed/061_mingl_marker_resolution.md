# Resolve mingl's 7 task markers (decomposed from task 038)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** crate
- **unit:** module/min/mingl
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Resolve the 7 live task markers in `module/min/mingl` (census 2026-08-10, task 038 — re-derive at
pickup; all 7 were independently confirmed still-live by task 034's issues.md reconciliation, which
routed them here). Grouped by kind:

**Data-type descriptor questions (`src/data_type.rs`):**
- `:50` and `:70` — `// xxx : usize?` ×2 (should the descriptor arithmetic use `usize` instead of
  the current `i32`? Decide once, apply to both sites consistently — they're the same question).
- `:84` — `// xxx : qqq : verify` (unverified descriptor invariant — write the verifying test, then
  delete the marker).
- `src/data_type/f32.rs:29` — `// qqq : xxx : implement similar for other primitive types`
  (IntoVectorDataType nested-array impls exist for f32 only — task 030's readme rewrite documents
  this f32-only limitation as current behavior; extending it makes that limitation note obsolete,
  so update `readme.md`'s Data Conversion Support table in the same change).

**Dependency decisions (cross-cutting pair with minwebgl — coordinate with task 062):**
- `Cargo.toml:68` — `# bytemuck = { workspace = true, ... } # xxx : replace` (commented-out
  bytemuck dep with a replace instruction; mingl now re-exports `asbytes` (Pod-based) instead —
  decide whether the marker is satisfied by the asbytes adoption and delete it, or whether some
  bytemuck-specific need remains).
- `src/derive.rs:12` — `exposed use ::former; // xxx : make it unncecessary` (former re-export the
  crate wants to drop — audit in-workspace users of `mingl::derive::former` first).

**Error handling:**
- `src/web/file.rs:71` — `// qqq : implement typed errors` (web file loading returns untyped errors).

Per-marker outcomes follow task 038's triage contract: fix in code, or file evidence why the marker
stays, or delete if investigation proves it stale. Verify with
`cargo test -p mingl --all-features` (via `longrun .launch`); readme claims must stay aligned with
task 030's verified-claims rewrite.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 038's workspace marker census (80 lines →
  per-crate tasks per Crate Scope Unity). This cluster is the surviving half of root issues.md's
  still-live items (task 034 routed them to 038; 038 routed them here).
- **[2026-08-10]** `IMPLEMENTED` — All 7 markers resolved; census grep over mingl now returns
  zero hits. Per-marker outcomes:
  - `data_type.rs:50/:70` `xxx : usize?` ×2 — decided: stays `i32`, deliberately. Consumer
    evidence: descriptors feed WebGL `GLint` parameter slots (`vertex_attrib_pointer` family);
    minwebgl buffer/geometry expose public `natoms : i32` API and the renderer gltf loader does
    `i32` arithmetic on the fields directly — `usize` would force a cast at every GL boundary.
    Markers deleted; rationale comment added above `VectorDataType` so the question doesn't
    get re-asked.
  - `data_type.rs:84` `xxx : qqq : verify` — verifying tests written: `tests/tests/data_type_test.rs`
    (4 test fns: scalar descriptor, flat arrays, nested arrays across f32/u8/i32, `DataType`
    byte widths). Marker deleted; adjacent commented-out `nelements()` variant block deleted too.
  - `data_type/f32.rs:29` `qqq : xxx : implement similar for other primitive types` — proven
    STALE: all 6 sibling files (i8/i16/i32/u8/u16/u32) already carry all 3 impls including
    nested arrays (per-file grep evidence). Marker deleted. Investigation exposed a wrong
    readme claim (nested-array support documented as f32-only) — `readme.md` Primitive
    Coverage line corrected to all-supported-scalars, and `data_type_test.rs` exercises u8/i32
    nested arrays to pin the corrected claim.
  - `Cargo.toml:68` `# bytemuck ... # xxx : replace` — satisfied by the asbytes adoption
    (`mem.rs` reuses `::asbytes`). Deleted the 3 commented-out dep lines (bytemuck, anyhow,
    slice-of-array) and reduced `mem.rs` from 143 lines of commented-out bytemuck-era AsBytes
    code to the 9-line asbytes reuse skeleton.
  - `derive.rs:12` `exposed use ::former; // xxx : make it unncecessary` — investigated: the
    exposure is load-bearing and cannot be dropped from this repo. The `Former` derive expands
    to `former::`-prefixed (not `::former::`) paths, so downstream derive sites resolving
    through mingl's exposed namespace (minwebgl `shader.rs` `ShaderSource`) need the crate
    name itself; `reuse ::former;` propagates items but not the module path. Marker deleted,
    exposure kept with rationale comment — removable only if upstream `former` switches to
    absolute paths.
  - `web/file.rs:71` `qqq : implement typed errors` — implemented typed `Error` enum via
    `error::typed::Error` derive: `DataUrl( &'static str )` + `Js( JsValue )`, with
    `From< JsValue > for Error` and `From< Error > for JsValue` (Js variant returns the
    original browser error object, so wasm callers returning `Result< _, JsValue >` keep
    using `?`). `load` now returns `Result< Vec< u8 >, Error >`; # Returns/# Errors docs
    updated. Workspace check proves every caller shape (minwebgl, renderer, primitive_generation,
    examples) still compiles.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Dual-Role Self-Check ran with three genuine
  in-loop adversarial catches: (1) B1 — Write tool used on pre-existing `mem.rs` instead of Edit;
  announced per Violation Response Protocol, full same-turn read verified no concurrent
  modification was clobbered, content correct. (2) B5 — first workspace check (`-0022`, exit 101)
  caught the `former`-exposure regression: the "zero consumers" audit behind dropping
  `exposed use ::former;` had only grepped textual `former::` references and missed
  derive-expansion consumers — 17 errors in minwebgl (root E0433). Exposure restored with
  rationale; relaunch (`-0023`) exit 0. Earlier, file.rs's typed-error derive failed E0433/E0599
  (`-0020`) — targeted import insufficient, thiserror paths resolve only via `use crate::*;`
  through error_tools' re-export chain; fixed, suite green. (3) D4 — the draft's premise for the
  f32 marker ("impls exist for f32 only") was falsified by sibling-file inspection, flipping the
  planned implement into a stale-delete plus a readme-claim correction. Final evidence: mingl
  suite on post-fix code `-0024` (13 inline + 38 integration, 0 failed), workspace check `-0023`
  exit 0, census grep exit 1 (zero markers).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Draft premise "f32-only nested impls" falsified — all 6 siblings already implemented | Marker deleted as stale; readme claim corrected; non-f32 nested arrays pinned by test |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟡 | 🟢 | Write used on pre-existing mem.rs instead of Edit | Announced per Violation Response Protocol; same-turn full read verified content, no clobber |
| B2 | Test-First | 🟢 | 🟢 | — | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | — | — |
| B5 | Fix Verification | 🟡 | 🟢 | former-exposure drop broke minwebgl (17 errors, `-0022`); file.rs derive E0433/E0599 (`-0020`) | Exposure restored with rationale (`-0023` exit 0); `use crate::*;` preamble fix; final suite `-0024` 51 tests 0 failed |
| B6 | Knowledge Preservation | 🟢 | 🟢 | — | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 3 findings resolved in-loop | 15/15 |

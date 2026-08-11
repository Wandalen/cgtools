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

## Verification

### Checklist

- [x] C1 — Are all 7 originally-live task markers (`xxx`/`qqq`) gone from `module/min/mingl`? `git grep -c -E "xxx *:|qqq *:" -- module/min/mingl/src module/min/mingl/Cargo.toml` → no output (0 hits).
- [x] C2 — Does `data_type.rs` keep `i32` (not `usize`) for `VectorDataType.natoms`/`.nelements`, with the decision rationale documented in place of the deleted marker? Confirmed: `pub natoms : i32` / `pub nelements : i32` at `src/data_type.rs:58,60`; rationale comment at lines 47-50 citing the WebGL `GLint`-boundary argument from the History.
- [x] C3 — Does `tests/tests/data_type_test.rs` exist with the 4 claimed verifying tests? Confirmed 4 `#[ test ]` fns — `scalar_descriptor_is_flat_single_atom`, `flat_array_descriptor_has_nelements_one`, `nested_array_descriptor_has_row_length_nelements`, `byte_size_matches_scalar_width` — and the file's own module doc explicitly states it replaces "a `verify` marker".
- [x] C4 — Was the `data_type/f32.rs:29` marker's premise ("nested-array impls exist for f32 only") genuinely false, i.e. do all 6 sibling files also carry a nested-array `IntoVectorDataType` impl? `grep -c "impl< const N : usize, const N2 : usize > IntoVectorDataType for \[ \["` across `src/data_type/{f32,i8,i16,i32,u8,u16,u32}.rs` → `1` in every one of the 7 files.
- [x] C5 — Is `Cargo.toml` free of the commented-out `bytemuck`/`anyhow`/`slice-of-array` dependency lines? `grep -inE "bytemuck|slice-of-array|anyhow" module/min/mingl/Cargo.toml` → 0 hits.
- [x] C6 — Does `mem.rs` carry the reduced `asbytes`-reuse skeleton (not the 143-line commented-out bytemuck-era block)? Confirmed: current file is exactly 9 lines; body is `reuse ::asbytes;`.
- [x] C7 — Is `derive.rs`'s `exposed use ::former;` still present with the rationale comment added after the in-loop minwebgl-breakage catch? Confirmed present, with the "`Former` derive expands to `former::`-prefixed paths..." comment directly above it.
- [x] C8 — Does `web/file.rs` implement the typed `Error` enum (`DataUrl`/`Js` variants) with `load` returning `Result<Vec<u8>, Error>`, replacing the untyped-error marker? Confirmed: `#[ derive( Debug, error::typed::Error ) ] pub enum Error { DataUrl( &'static str ), Js( JsValue ) }`; `From< JsValue > for Error` and `From< Error > for JsValue` both present; `pub async fn load( .. ) -> Result< Vec< u8 >, Error >`.

### Measurements

- [x] M1 — Live task-marker count in `module/min/mingl` (`src/` + `Cargo.toml`, `xxx`/`qqq`): current `0` (was: `7` — `git grep -c -E "xxx *:|qqq *:" 25ceae76 -- module/min/mingl/src module/min/mingl/Cargo.toml` → `Cargo.toml:1`, `data_type.rs:3`, `data_type/f32.rs:1`, `derive.rs:1`, `web/file.rs:1`).
- [x] M2 — `mem.rs` line count: current `9` (was: `143`, `git show 25ceae76:module/min/mingl/src/mem.rs | wc -l`).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p mingl --all-features` → exit 0, 51/51 passed.
- [ ] I2 — Compiler/lints clean (crate-scoped): `cargo clippy -p mingl --all-targets --all-features -- -D warnings` → exit 101, NOT clean. Root cause fully isolated to a different, workspace-local crate: `module/helper/browser_log/src/panic.rs:82`'s `#[ allow( clippy::exhaustive_structs ) ]` lacks a `reason = ".."`, tripping the workspace's `allow_attributes_without_reason = "warn"` lint (escalated to a hard error by `-D warnings`). `browser_log` is pulled in only transitively, via mingl's optional `web_log` feature; the build aborts there before mingl's own source is ever clippy-checked. `git log -1 --format="%h %ad %s" --date=iso -- module/helper/browser_log/src/panic.rs` → commit `5f33be66`, dated 2026-08-11 (today) — lands after this task's 2026-08-10 completion and touches none of the 7 markers' files, so this is pre-existing drift unrelated to this task, not a regression it introduced. (Independently corroborated: a concurrent sibling verification of the unrelated `primitive_generation` crate hit the identical `browser_log:82` failure in the same session.)

### Anti-faking checks

- [x] AF1 — Guards against a marker being deleted without genuine resolution rather than fixed/justified: every one of the 7 sites (C2-C8) has a corresponding source-level change or in-place rationale comment, not a bare deletion; re-running M1's `git grep` after any future edit must still return 0 hits, and any newly-added marker must carry the same fix-or-justify discipline before deletion.
- [x] AF2 — Guards against `exposed use ::former;` (C7) being dropped again via the same reasoning that broke minwebgl in-loop (History: a "zero consumers" audit that only grepped textual `former::` references missed derive-expansion consumers — the `-0022` workspace check caught 17 resulting E0433 errors in minwebgl before completion; the fix was verified by the `-0023` relaunch, exit 0): the rationale comment directly above the line documents why it's load-bearing; any future removal attempt must be checked with a workspace-wide build, not a `-p mingl`-scoped one, before being considered safe.

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

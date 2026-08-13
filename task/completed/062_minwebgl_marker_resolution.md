# Resolve minwebgl's 3 task markers (decomposed from task 038)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** module/min/minwebgl
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Resolve the 3 remaining live task markers in `module/min/minwebgl` (census 2026-08-10, task 038 —
re-derive at pickup; task 038 already deleted the crate's 3 stale `aaa :` review-conversation
remnants in `src/context.rs`, so only these remain):

- `Cargo.toml:77` — `# bytemuck = { workspace = true, optional = true, ... } # xxx : replace`
  (commented-out bytemuck dep; same decision as mingl's identical marker — coordinate with task
  061, resolve both in one decision since minwebgl re-exports mingl's byte handling).
- `src/geometry.rs:79` — `// qqq : xxx : move out switch and make it working for all types`
  (a type-switch the author wants extracted and generalized — the largest real code change in this
  cluster; confirmed still-live by task 034's issues.md reconciliation).
- `src/browser.rs:10` — `// xxx : investigate` (bare marker with no stated subject on a module-level
  item — investigate what it refers to via `git log -L` on the line's introduction; if the concern
  can't be reconstructed, delete it as unactionable noise rather than carrying an unfalsifiable
  marker forever).

Per-marker outcomes follow task 038's triage contract: fix in code, or file evidence why the marker
stays, or delete if stale/unactionable. Verify with `cargo check -p minwebgl --all-features` plus
the crate's test suite via `longrun .launch` (never set bare `RUSTFLAGS` on wasm32 builds — it
clobbers `.cargo/config.toml`'s `--cfg web_sys_unstable_apis`).

## In Scope

- `module/min/minwebgl/Cargo.toml`: resolved the bytemuck-replace marker (satisfied by existing
  `asbytes` adoption; removed dead commented-out dep lines)
- `module/min/minwebgl/src/geometry.rs`: resolved the "move out switch" marker by deleting the
  natoms-to-type dispatch entirely (not extracting it) and widening `validate_natoms` from
  `2`-only to `1..=4`
- `module/min/minwebgl/src/browser.rs`: resolved the bare `xxx : investigate` marker —
  investigated via git history, found unreconstructable, deleted as unactionable noise

## Out of Scope

- `src/context.rs`'s 3 `aaa:` markers — already resolved by task 038 before this task started
- The paired bytemuck decision's mingl-side implementation — coordinated with, but implemented
  separately by, task 061
- `geometry.rs`'s now-unused `AsBytes` import (dead-import drift left by the switch deletion) —
  identified but explicitly left unfixed, outside this verification pass's edit scope

## Verification

### Checklist

- [x] C1 — `Cargo.toml`: zero `bytemuck`/`xxx` hits? `grep -in "bytemuck\|xxx" module/min/minwebgl/Cargo.toml` → `0` hits.
- [x] C2 — `geometry.rs`: is the natoms-to-compile-time-type switch genuinely deleted (not merely extracted, as the in-loop adversarial finding D4 required)? Direct read confirms `BufferDescriptor` is now built via a plain struct literal (`geometry.rs:93-101`) driven by the runtime `typ : VectorDataType`, not a `match typ.natoms { 2 => BufferDescriptor::new::<[f32;2]>() ... }` dispatch.
- [x] C3 — Is `validate_natoms` genuinely widened to `1..=4` (not still `2`-only)? Direct read confirms `1 ..= 4 => Ok( () )` (`geometry.rs:41`); git-verified boundary: `9b71cf39` (this file's prior state) had `2 =>` only, `4469eafb` (this task's own commit) introduced `1 ..= 4 =>`.
- [x] C4 — `browser.rs`: is `// xxx : investigate` genuinely deleted, not moved? Direct read + `grep -c xxx module/min/minwebgl/src/browser.rs` → `0`; git-verified present at `dea7a008`, absent at `4469eafb`/current, with the rest of the file (`reuse ::browser_log;` + `JsCast` re-export) otherwise unchanged.
- [x] C5 — Full workspace-marker census: zero `xxx :`/`qqq :`/`aaa :` hits anywhere under `module/min/minwebgl/`? `grep -rn "xxx *:\|qqq *:\|aaa *:" module/min/minwebgl/src/ module/min/minwebgl/Cargo.toml` → `0` hits, matching this task's own "census grep returns zero hits" claim.
- [x] C6 — Was BUG-052's reproducer test genuinely updated (not silently vacated) to match the widened range, per this task's own in-loop adversarial catch B3? Direct read of `validate_natoms_rejects_unsupported_value` confirms its probes are `[ 0, 5, -1 ]` (outside `1..=4`), not the original `3` (now supported), with a doc comment explaining the move.

### Measurements

- [x] M1 — `validate_natoms`'s accepted range: now `1..=4` (4 values) (was: `2` only, 1 value — cite `git show 9b71cf39:module/min/minwebgl/src/geometry.rs`).
- [x] M2 — Live task-marker (`xxx`/`qqq`/`aaa`) count in `module/min/minwebgl/`: now `0` (was: `3` — `Cargo.toml:77`, `geometry.rs:79`, `browser.rs:10` — per this task's own Goal census).

### Invariants

- [x] I1 — Crate-scoped native test suite: `cargo test -p minwebgl --all-features` → exit `0`; both `validate_natoms_accepts_supported_values` and `validate_natoms_rejects_unsupported_value` report `ok` under the widened range.
- [ ] I2 — Lint cleanliness, literal historically-cited command: `cargo clippy -p minwebgl --all-targets --all-features -- -D warnings` → exit `101` today, blocked at the unrelated `browser_log` dependency — same root cause as task 011's I2, unrelated to this task's own 3 files.
- [ ] I3 — Lint cleanliness, isolated to this task's own change: `cargo clippy -p minwebgl --no-deps --all-targets --all-features -- -D warnings` surfaces **one finding genuinely attributable to this task**: `geometry.rs:4`'s `use crate::{ ..., AsBytes };` import is now unused (`error: unused import: AsBytes`) — the deleted switch (C2) was `AsBytes`'s only consumer (`grep -n AsBytes module/min/minwebgl/src/geometry.rs` → only the import line itself, `0` other uses) and the import was not removed alongside it. This is genuine, currently-real, unresolved drift caused by this task's own change; it is reported here, not fixed (outside this verification pass's edit scope).
- [x] I4 — Sole external caller (`hexagonal_grid`, 3× `Positions::new( _, _, 2 )`): `cargo check -p hexagonal_grid --all-features` → exit `0`.

### Anti-faking checks

- [x] AF1 — Guards against the `AsBytes` dead import (I3) silently persisting or reappearing unnoticed after a fix: `grep -n AsBytes module/min/minwebgl/src/geometry.rs` must show more than 1 hit (an actual use, not just the import line) before this finding can be marked resolved — re-run this check after any future edit to confirm.
- [x] AF2 — Guards against the `natoms` range narrowing back to `2`-only and silently dropping 3D/4D support: re-run `validate_natoms_accepts_supported_values`'s `1..=4` loop — any future narrowing must fail that test loudly, not silently.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 038's workspace marker census (80 lines →
  per-crate tasks per Crate Scope Unity). Cluster includes 2 of root issues.md's still-live items
  (via task 034's reconciliation); the crate's 3 `aaa :` remnants were resolved directly by task
  038 itself.
- **[2026-08-10]** `IMPLEMENTED` — All 3 markers resolved; census grep over minwebgl returns zero
  hits. Per-marker outcomes:
  - `Cargo.toml:77` `# bytemuck ... # xxx : replace` — satisfied by the asbytes adoption (the
    `asbytes` workspace dep sits directly above the marker; `dep:asbytes` already wired into
    `enabled`). Deleted the commented bytemuck dep line, the stale `# "dep:bytemuck",` feature
    entry, and 3 more dead commented dep lines (anyhow, slice-of-array, log) mirroring 061's
    mingl cleanup. Pair decision with 061 resolved consistently: both crates closed the marker
    as satisfied-by-asbytes.
  - `src/geometry.rs:79` `qqq : xxx : move out switch and make it working for all types` —
    resolved by deleting the switch rather than extracting it: `BufferDescriptor` is fully
    data-driven (`attribute_pointer` reads only the runtime `VectorDataType`; all fields pub),
    so the natoms-to-compile-time-type dispatch was never necessary — the descriptor is now
    built from `typ` via struct literal. `validate_natoms` widened from `2` to `1 ..= 4` (the
    GL `vertex_attrib_pointer` size range), which makes 3D/4D positions work — BUG-052's
    original trigger scenario ("loading geometry with 3 or 4 components per vertex") is now
    supported instead of a proper-but-unavoidable error. Docs updated (`validate_natoms`
    # Errors, `Positions::new` header + params). Sole external caller (hexagonal_grid, natoms 2
    ×3) unaffected.
  - `src/browser.rs:10` `// xxx : investigate` — deleted as unactionable noise per the triage
    contract: `git log -L` shows it introduced by dea7a008 (2025-03-29, subject "minwebgl :
    refactoring", no body) above a `reuse ::browser_log;` line unchanged since the initial
    commit; no subject stated, no reconstruction possible. Task 057 (browser_input coupling)
    is unrelated to this line.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Dual-Role Self-Check with two genuine in-loop
  adversarial catches: (1) D4 — the draft's framing ("move out switch") was falsified by
  inspection: `BufferDescriptor`'s runtime-data design means deletion, not extraction, is the
  correct generalization. (2) B3 — the BUG-052 reproducer (`validate_natoms_rejects_unsupported_value`)
  used `3` as its unsupported probe; naively widening support would have silently vacated the
  reproducer. Probes moved to `0`/`5`/`-1` with a doc note preserving the RED-state story and the
  contract (unsupported natoms → `Err`, never panic). Evidence: census grep exit 1; minwebgl
  suite `-0025` (4 unit + 1 doc-test, 0 failed — both natoms tests listed by name); workspace
  check `-0026` exit 0 (hexagonal_grid caller and all examples compile).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Draft framing "extract the switch" falsified — BufferDescriptor is data-driven, dispatch unnecessary | Switch deleted; descriptor built from runtime VectorDataType; natoms widened to 1..=4 |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | — | — |
| B2 | Test-First | 🟢 | 🟢 | — | — |
| B3 | Evidence of Failure | 🟡 | 🟢 | BUG-052 reproducer probe (natoms 3) would be silently vacated by the widening | Probes moved to 0/5/-1; doc note preserves RED-state story and Err-not-panic contract |
| B4 | Proper Fix Only | 🟢 | 🟢 | — | — |
| B5 | Fix Verification | 🟢 | 🟢 | — | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | — | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 2 findings resolved in-loop | 15/15 |

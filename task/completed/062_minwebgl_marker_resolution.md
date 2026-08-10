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
- **unit_type:** crate
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

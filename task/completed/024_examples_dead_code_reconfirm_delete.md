# Re-confirm and delete non-functional example crates

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

The audit flagged at least one non-functional example (referencing a `derive_tools_issue`-style stale
workaround) and one example duplicating logic that belongs in a shared crate, as delete candidates (P3,
Delete-candidate bucket). **Exact example directories were not preserved precisely through this session's
context compaction — re-derive at pickup** by scanning `examples/{math,minwebgl,minwebgpu,minwgpu}/*` for
crates that fail to build standalone or that reimplement logic already available from a workspace helper
crate, before deleting anything. Cross-check against task 031 (broken example "How to run" links) — a
non-functional example is a stronger candidate for deletion than merely having a broken doc link, so
resolve which examples are being deleted before that task rewrites their links.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3
  (carried-forward, not re-verified) tier, Delete-candidate bucket.
- **[2026-08-10]** `IMPLEMENTED` — Re-derived per the Goal's own criteria, then deleted exactly one crate.
  **Build criterion:** every example crate in the 4 audited dirs was standalone-checked on its declared
  target — 53 `runtime:browser` crates on `wasm32-unknown-unknown` + 6 native/offscreen crates on host —
  **zero build failures** (`-0034_longrun.log`, exit 0; earlier `-0033` failure was my sweep script's own
  CR-contaminated name extraction — `minimize_wasm/Cargo.toml` is CRLF-encoded — not a crate defect).
  **Deleted: `examples/minwebgl/derive_tools_issue/`** — non-functional by its own claimed purpose:
  (a) readme + module doc call it "a minimal WebGL application demonstrating the issue of using
  `EnumCount` with `derive_tools`", but `main()` is empty, zero WebGL/browser code exists, and its own
  metadata tags it `runtime:native` while sitting in `minwebgl/`; (b) the actual reproducer line
  `use derive_tools::EnumCount` is commented out, replaced by `use strum::EnumCount` — the surviving test
  asserts strum's derive yields `Test::COUNT == 3`, demonstrating nothing about `derive_tools`;
  (c) upstream-issue reproducers are ephemeral artifacts belonging with the upstream issue, not a
  permanent examples catalog; (d) `demo_completeness.md` listed it not-completed. References cleaned:
  `examples/index.md` row, `examples/demo_completeness.md` row. Workspace deps kept — `derive_tools`
  (ndarray_cg, mingl, minwebgpu) and `strum` (hexagonal_map, browser_input) have other consumers.
  `locales.md:52`'s stale row left untouched: generated file ("Do not edit manually", maintained by
  `.locale.doc.generate`) already stale in unrelated ways (still lists pre-019 `mdmath_ia`).
  **Duplicated-logic criterion → NO further deletion:** hex/tile examples consume `tiles_tools` properly;
  `mapgen_tiles_rendering` uses `ndarray_cg`; `filter` (108-line cursor-radius emboss-kernel demo, hosts
  the Fix(BUG-053) doc site) vs `filters` (41-file GPU filter suite) are distinct raw-GL demos, neither
  reimplements a workspace helper; `text_rendering`'s local mesh fn was deliberately designated sole owner
  by task 021 (the dead shared-crate copy was what got deleted); `outline`/`narrow_outline`/
  `renderer_with_outlines` near-duplication is task 022's consolidation subject — deferred there rather
  than pre-empted by deletion here. **031 cross-check satisfied:** the deletion set (exactly 1) is
  resolved before 031's link sweep. Verification: `cargo metadata` workspace OK; post-deletion sweep
  green — 53 wasm32 + 5 native (`-0035_longrun.log`, exit 0); repo-wide reference grep clean outside
  task records.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — 15-dimension Tier 2 gate passed (see Verification
  Record). One finding resolved in-loop: the first sweep attempt produced a spurious "invalid character
  in package name" failure that pattern-matched to a broken crate — adversarial reading of the error
  traced it to CRLF line endings in `minimize_wasm/Cargo.toml` contaminating my extraction pipeline
  (`tr -d '\r'` fix), preventing a false delete-candidate. Moved draft/ → completed/.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | Considered deleting `filter` as subsumed by `filters`; rejected — fails the task's own criteria (not helper-crate duplication), and it hosts the BUG-053 fix-documentation site | — |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | Pre-existing working-tree mods in examples/ (verb/run standardization, hello_triangle_quickstart rows) inspected via git diff before editing shared files; my edits confined to additive row removals | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | — | — |
| B2 | Test-First | 🟢 | 🟢 | Delete decision gated on a full standalone build sweep run BEFORE deleting, re-run after | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | 4 independent evidence lines for the deletion (empty main, commented-out reproducer, native tag in minwebgl/, not-completed status) | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Deletion, not archival; no backup copies; references cleaned same session | — |
| B5 | Fix Verification | 🟡 | 🟢 | First sweep failed spuriously (CRLF-contaminated name extraction, `-0033`) — could have been misread as a crate build failure; adversarial error-reading found my tooling defect | `tr -d '\r'`; clean rerun `-0034`; post-deletion rerun `-0035` all green |
| B6 | Knowledge Preservation | 🟢 | 🟢 | Duplication re-derivation outcome (021 ownership decision, 022 deferral) recorded here for 031/022 pickup | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Sweep script + logs hyphen-prefixed; no orphaned references (locales.md exception documented — generated file) | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved | 1/1 |

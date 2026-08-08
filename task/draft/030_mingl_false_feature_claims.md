# Fix mingl's false feature claims in docs

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/mingl
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`mingl`'s documentation claims a feature or capability the audit found not actually implemented in the
crate's real source (P5 — remaining doc drift, Fix-in-place). **Exact claim and file were not preserved
precisely through this session's context compaction — re-derive by diffing the crate's readme/doc claims
against `src/` at pickup** (note: `module/min/mingl/src/web/exec_loop.rs` is the file task 012 confirms
minwebgl should be reusing — check that file's own doc comments for accuracy while in this crate, since
it's directly relevant). Kept as a separate task from task 029 (browser_log's own false claims) per Crate
Scope Unity even though both were found in the same audit pass.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P5 (doc drift)
  tier, Fix-in-place bucket. Flagged: citation detail needs re-derivation at pickup.

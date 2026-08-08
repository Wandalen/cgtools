# Fix minwebgl panic-on-recoverable-failure bugs

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
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgl
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix `minwebgl` sites that panic on conditions that are recoverable/expected (e.g. resource-acquisition or
WebGL-context failures a caller should be able to handle), rather than surfacing them via `Result` (P1 —
soundness bucket, Fix-in-place). **Carried forward from the audit triage plan — exact file/line citations
are not re-verified in this filing pass; re-confirm against current `module/min/minwebgl/src/` before
touching.** Scope is distinct from task 012 (exec_loop.rs duplication, a dead-code/hygiene concern in the
same crate, not a soundness one) — keep these two efforts separate even though they share a crate.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P1 (soundness bugs)
  tier, Fix-in-place bucket.

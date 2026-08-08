# Workspace-wide sweep: clear the xxx/qqq/aaa/TODO marker backlog

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** workspace
- **unit:** lib/yrd_gamedev/cgtools
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

The audit counted roughly 86 `xxx:`/`qqq:`/`aaa:`/`TODO:` task markers scattered across the workspace's
source (P8 — mechanical hygiene tier). **Re-derive the current count at pickup**
(`grep -rn "xxx:\|qqq:\|aaa:\|TODO:" module/ examples/` or equivalent) since it has likely drifted since
the audit. For each marker: resolve it directly if trivial, file it as its own properly-scoped task if
it represents real, non-trivial work (per Crate Scope Unity — one task per crate's markers, not one giant
cross-workspace task), or delete it if it's stale/already-addressed. This overlaps with task 034 (root
`issues.md` retirement) — the 8 items already catalogued there are a subset of this marker backlog;
reconcile the two rather than double-filing the same markers as separate tasks from each.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P8 (mechanical
  hygiene) tier, Fix-in-place bucket (cross-cutting).

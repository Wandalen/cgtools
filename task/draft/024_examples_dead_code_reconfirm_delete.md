# Re-confirm and delete non-functional example crates

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

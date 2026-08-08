# Fix behaviour_tree hang/livelock risk

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/behaviour_tree
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix a condition in `behaviour_tree` identified during the workspace audit where a specific node
configuration or evaluation cycle can hang or livelock rather than terminate (P2 — remaining logic bugs,
Fix-in-place). **Carried forward from the audit triage plan — exact file/line and the precise triggering
condition are not re-verified in this filing pass; re-confirm against current
`module/helper/behaviour_tree/src/` before touching.** A regression test needs a bounded-time assertion
(e.g. a timeout-wrapped evaluation) to actually catch a hang rather than blocking the test suite itself.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, Fix-in-place bucket.

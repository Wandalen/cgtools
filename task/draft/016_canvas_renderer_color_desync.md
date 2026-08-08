# Fix canvas_renderer silent color-desync bug

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/canvas_renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix a bug in `canvas_renderer` where rendered color state can silently desynchronize from the logical
state it's meant to track (no panic, no error — just visually wrong output), identified during the
workspace audit (P2 — remaining logic bugs, Fix-in-place). **Carried forward from the audit triage plan —
exact file/line is not re-verified in this filing pass; re-confirm against current
`module/helper/canvas_renderer/src/` before touching.** Write a test that asserts color-state consistency
across the specific operation sequence that triggers the desync before fixing.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, Fix-in-place bucket.

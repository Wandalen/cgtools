# Fix minwebgpu's 3 documented-invariant-violating panics

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
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgpu
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix 3 sites in `minwebgpu` where code panics on conditions its own doc comments document as
recoverable/expected, rather than returning `Result`/`Option` per the documented contract (P1 —
soundness bucket, Fix-in-place). **Carried forward from the audit triage plan — exact file/line citations
for the 3 sites were in the delivered plan but are not re-verified in this filing pass; re-confirm each
against current `module/min/minwebgpu/src/` before touching any of them.** Each site needs its own failing
test demonstrating the panic on documented-recoverable input before the fix lands.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P1 (soundness bugs)
  tier, Fix-in-place bucket.

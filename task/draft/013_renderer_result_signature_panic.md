# Fix renderer panic that violates its own Result-returning signature

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix a site in `renderer` where a function whose own signature returns `Result<_, _>` panics instead of
returning `Err` on a failure condition its signature already advertises as handleable (P1 — soundness
bucket, Fix-in-place). **Carried forward from the audit triage plan — exact file/line is not re-verified
in this filing pass; re-confirm against current `module/helper/renderer/src/` before touching.** Distinct
from task 020 (renderer's Composer/raw.rs dead-code and Quick Start doc drift, a hygiene concern in the
same crate) — keep separate even though both live in `renderer`.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P1 (soundness bugs)
  tier, Fix-in-place bucket.

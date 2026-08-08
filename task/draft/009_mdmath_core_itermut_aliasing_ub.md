# Fix mdmath_core IterMut aliasing/UB soundness bug

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
- **unit:** lib/yrd_gamedev/cgtools/module/math/mdmath_core
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix a soundness bug in `mdmath_core`'s mutable-iterator code identified during the workspace-wide audit
(P1 — soundness bucket, Fix-in-place): an `IterMut`-style construct produces aliased mutable
references/undefined behavior under certain access patterns. **Carried forward from the audit triage
plan — exact file/line was cited in the delivered plan but is not re-verified in this filing pass;
re-confirm the precise citation against current `module/math/mdmath_core/src/` before making any change**,
per the plan's own ground rule that findings must be re-confirmed immediately before the file they cite is
touched. Write a failing test demonstrating the aliasing (e.g. via `miri` or a targeted borrow-check-evading
pattern) before fixing.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P1 (soundness bugs)
  tier, Fix-in-place bucket.

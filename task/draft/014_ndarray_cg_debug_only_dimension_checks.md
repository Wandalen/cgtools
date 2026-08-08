# Fix ndarray_cg debug-only dimension checks (silently unchecked in release)

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
- **unit:** lib/yrd_gamedev/cgtools/module/math/ndarray_cg
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`ndarray_cg` has dimension/bounds checks gated to debug builds only (e.g. `debug_assert!`) where the
consequence of a mismatch in release is silent wrong-data output rather than a loud failure, violating
the workspace's "loud failures, never silent" testing principle (P2 — remaining logic bugs, Fix-in-place).
**Carried forward from the audit triage plan — exact file/line is not re-verified in this filing pass;
re-confirm against current `module/math/ndarray_cg/src/` before touching**, and decide case-by-case
whether each site should become a real runtime check (`Result`/panic) or is genuinely
performance-critical-enough to justify staying debug-only with an explicit doc comment explaining why.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, Fix-in-place bucket.

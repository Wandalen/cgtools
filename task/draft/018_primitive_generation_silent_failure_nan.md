# Fix primitive_generation doc-contradicting silent failure and NaN-producing precondition gap

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/primitive_generation
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix two related `primitive_generation` issues found during the workspace audit (P2 — remaining logic
bugs, Fix-in-place): (1) a function whose doc comment promises an error/validation result instead fails
silently on invalid input; (2) a separate precondition gap that lets degenerate input reach geometry math
and produce `NaN` output rather than being rejected upfront. **Carried forward from the audit triage
plan — exact file/line citations are not re-verified in this filing pass; re-confirm against current
`module/helper/primitive_generation/src/` before touching.** Distinct from task 021 (this crate's
`ufo.rs` dead-code and doc-drift cleanup, a hygiene concern) and from BUG-007/task 008 (this crate's
`csgrs`/`core2` dependency issue) — three separate concerns sharing one crate, keep them separate.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, Fix-in-place bucket.

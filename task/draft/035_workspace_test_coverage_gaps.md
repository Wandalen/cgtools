# Close workspace-wide test coverage gaps (needs decomposition at pickup)

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

The audit identified three distinct test-coverage problems spanning many crates (P7 — test coverage
tier): (1) crates with zero test coverage at all; (2) crates whose tests violate the "all tests in
`tests/` directories" convention by testing public API inline within `src/`; (3) crates whose docs claim a
level of test coverage the actual test suite doesn't support. **This is explicitly a tracking/umbrella
Draft, not directly actionable as filed** — this session's context compaction lost the specific per-crate
breakdown originally produced by the audit subagents. At pickup: re-run a workspace-wide sweep (per-crate
`find tests/`, `grep -rn "#\[test\]" src/`, and a diff of each crate's coverage claims against its actual
tests) to rebuild the per-crate list, then decompose into one task per affected crate per Crate Scope
Unity (`tsk.rulebook.md § Core Procedures : Procedure - Decompose by Crate`, PROC17) rather than trying to
force this single Draft through full `File Task` as one multi-crate task.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P7 (test coverage)
  tier, Fix-in-place bucket. Flagged as needing decomposition — do not promote to full task as-is.

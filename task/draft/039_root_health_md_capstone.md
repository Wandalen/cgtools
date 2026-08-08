# Populate root health.md as a living workspace health dashboard (capstone — do last)

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

Root `health.md` is confirmed (direct read this session) to be a literal empty stub — just the title
`# cgtools health`, no body. P-capstone (do last, after the rest of this triage plan's tasks land):
populate it as a living per-crate health dashboard (build status, test coverage, doc-entity adoption,
open marker count, known deprecated/delete-candidate crates) that summarizes the outcome of tasks
008-038. Deliberately sequenced last — writing this before the other fixes land would just document a
known-broken state and need immediate revision; **do not start this until at least the P0 (task 008) and
P1 (soundness bug) tier tasks have landed**, so the dashboard reflects real post-fix state rather than
being stale on arrival.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, capstone tier (do
  last), Fix-in-place bucket (root file, not crate-scoped).

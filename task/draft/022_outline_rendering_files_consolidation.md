# Consolidate near-identical outline-rendering files (crate TBD — needs re-scoping)

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

The workspace audit flagged a pair (or set) of near-identical outline-rendering source files as
consolidation candidates (P3, dead-code/hygiene bucket, Fix-in-place) — likely somewhere under
`module/helper/line_tools` or `module/helper/renderer` given those crates own outline/line-rendering
responsibility, but **the exact files and crate were not preserved precisely through this session's
context compaction and must be re-derived from scratch at pickup** (re-run a workspace-wide search for
near-duplicate rendering/outline source files, e.g. by comparing file sizes and diffing candidates, rather
than trusting this citation). Note: `module/helper/line_tools/src/d2/line.rs` and `d3/line.rs` show
uncommitted local modifications as of this session's start (per initial `git status`) — check whether
those in-flight edits already address this finding before starting new work here.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code /
  consolidation) tier, Fix-in-place bucket. Flagged low-confidence: citation not preserved through
  compaction, needs full re-derivation at pickup.

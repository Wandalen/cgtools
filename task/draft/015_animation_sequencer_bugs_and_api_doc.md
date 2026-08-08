# Fix animation crate's Sequencer/Tween bugs and wrong API doc table

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/animation
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix 3 logic bugs identified in `animation`'s `Sequencer`/`Tween` code during the workspace audit (P2 —
remaining logic bugs, Fix-in-place), and separately correct the crate's readme/doc API table, which was
found to describe an API shape that doesn't match the real one. **Carried forward from the audit triage
plan — exact file/line citations for the 3 bugs and the specific wrong table entries are not re-verified
in this filing pass; re-confirm against current `module/helper/animation/src/` and its readme before
touching anything.** Bundled as one task since both concerns are small and confined to the same crate;
split into separate tasks at pickup if either turns out to be larger than expected.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, merged with a P5 (doc drift) item for the same crate, Fix-in-place bucket.

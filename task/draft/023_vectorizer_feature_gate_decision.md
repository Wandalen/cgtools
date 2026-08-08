# Decide vectorizer's fate: fix feature-gate blocker and re-enable, or delete

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/vectorizer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`module/helper/vectorizer` is commented out of root `Cargo.toml`'s workspace `members` list
(`# "module/helper/vectorizer", # TODO: Fix feature gate issues`), and the workspace-dependencies entry
for it points at the wrong path (`path = "module/vectorizer"` instead of the real
`module/helper/vectorizer`) — both confirmed by direct read this session, and both newly-discovered
findings not present in the original audit. The crate has 14 source files but zero test files and zero
cross-references from any other crate (confirmed via workspace-wide grep this session). This is a
decision task (P3 bucket): investigate what "feature gate issues" actually blocks compilation, then
either (a) fix the feature-gate problem, correct the dependency path, and re-add to `members`, with tests
added since none exist today, or (b) if the crate is genuinely unmaintained/superseded, delete it
entirely (Delete-candidate). Whichever direction, fix the wrong dependency path as part of the same
change.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (decision point)
  tier, Delete-candidate/Fix-in-place decision bucket.

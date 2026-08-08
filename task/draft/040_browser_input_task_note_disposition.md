# Decide disposition of browser_input's orphaned task/ note

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/browser_input
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`module/helper/browser_input/task/001_dependency_cleanup.md` is a pre-existing, informal task note
(no `readme.md`, none of this system's canonical Execution State/History structure) proposing to
replace browser_input's `minwebgl` dependency with `ndarray_cg` for two math types (`I32x2`, `F64x3`)
to remove WebGL coupling. Its mere presence in a directory literally named `task/` causes
`tsk.rulebook.md § Hierarchical Systems : Structure Detection`'s (TA124) `TASK_DIR_COUNT` check to
detect this workspace as hierarchical, even though no genuine `type: local` system exists there.
Per `§ Hierarchical Systems : Consistency Check` (TA125), this workspace currently matches its own
"Aggregated Index Missing Entirely" CRITICAL VIOLATION condition (hierarchical detected by
TASK_DIR_COUNT=2, no Aggregated Index/Global ID Registry exists) — this is real and unresolved
regardless of whether `task/readme.md` declares `type: root`, since TA124's Root System Detection
falls back to "shallowest task/readme.md" independent of that field. Decide and execute one of: (a)
adopt this note as a proper `type: local` Task System (readme.md + canonical template) and build out
this root's Aggregated Index + Global ID Registry per TA062-TA064/TA123 for real — the "do it
properly" path, committing to ongoing dual-table maintenance for a workspace that otherwise has
nothing to aggregate; (b) migrate its one idea into this root system as a normal Draft task and
retire the note; or (c) leave it as an unrelated pre-existing artifact and rename/relocate it out of
a `task/`-named directory — the cheapest fix, since it removes the mechanical trigger entirely rather
than building infrastructure to satisfy it. Requires edit access to `module/helper/browser_input/`,
outside this session's task/+docs/-only scope — needs its own authorization.

**Concrete evidence this isn't just theoretical:** its filename ID (`001`) collides with this
system's own `task/unverified/001_sprawl_procedural_city_dashboard.md` — coincidental today (two
independent, ungoverned numbering sequences), but a real conflict TA063's global ID-uniqueness
requirement would need resolved (renumber one side) the moment option (a) or (b) above is chosen.

## History

- **[2026-08-08]** `FILED` — Filed during task-backlog normalization; discovered while investigating
  why `task/readme.md`'s `type: root` metadata had no corresponding real hierarchy. Root's metadata
  was flattened (`type: root` removed) rather than building out full hierarchical machinery for a
  single unadopted note — see `task/readme.md` history/commit context. This task tracks the
  browser_input-side half of that finding.

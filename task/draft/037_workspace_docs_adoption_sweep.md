# Workspace-wide sweep: adopt docs/ entity structure in remaining crates

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

The audit found only 5 of 27 workspace crates have adopted the `docs/` doc-entity structure
(`docs/feature/`, `docs/invariant/`, `docs/api/`, etc.) that recent commits (`refactor: migrate ...
documentation to docs/ entity structure`, visible in this repo's own git log) are actively rolling out
elsewhere (P8 — mechanical hygiene tier). **Re-derive the current 5/27 count at pickup** — this repo's
docs migration is actively in progress per its own recent commit history, so the true count has likely
already moved since this finding was made. For each remaining crate, migrate its existing scattered docs
(readme.md sections, standalone `.md` files) into the appropriate `docs/` doc-entity subdirectories,
following whatever pattern the 5 already-migrated crates establish. Likely worth decomposing per-crate at
pickup, same as tasks 035/036.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P8 (mechanical
  hygiene) tier, Fix-in-place bucket (cross-cutting).

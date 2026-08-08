# Fix broken "How to run" links across example crates

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

The audit found roughly 47 of ~52 example crates under `examples/{math,minwebgl,minwebgpu,minwgpu}/*`
have broken "How to run" links/instructions in their readmes, against the 5-point structure documented in
the workspace's `conventions.md` (P5 — remaining doc drift, Fix-in-place). This is a mechanical,
templated sweep — fix location is "every example readme's How-to-run section," a systematic pattern
rather than one artisanal fix per crate, similar in kind to how BUG-007's own fix was scoped by fix
pattern rather than by every affected consumer crate. **Re-derive the exact broken-link pattern and exact
count at pickup** (grep all `examples/*/*/readme.md` for the How-to-run section and validate each link/
command against the crate's real structure) rather than trusting the carried-forward count. Coordinate
with task 024 (non-functional example deletion) — resolve which examples are being deleted first, so
their links aren't fixed only to be deleted right after.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P5 (doc drift)
  tier, Fix-in-place bucket (mechanical/cross-cutting sweep).

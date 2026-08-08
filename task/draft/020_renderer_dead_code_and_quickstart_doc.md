# Resolve renderer's Composer/raw.rs dead code and fix non-compiling Quick Start doc

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Three renderer hygiene items bundled together (P3, dead-code/hygiene bucket, Fix-in-place — decide
wire-in-vs-delete for the code items, then fix the doc): (1)
`module/helper/renderer/src/webgl/post_processing/composer.rs` — `Composer` has 5 in-file references
(struct, impl, doc comments, export at line 226) but zero references anywhere else in the workspace
(confirmed via workspace-wide grep this session) — decide whether to wire it into the actual render
pipeline or delete it; (2) `module/helper/renderer/src/webgl/material/raw.rs` — confirmed 0 bytes this
session — delete after confirming no `mod raw;` declaration still references it; (3) the crate's readme
Quick Start example doesn't compile against the current API — carried forward from the audit triage plan,
re-confirm the exact mismatch against current `module/helper/renderer/src/` before rewriting.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code)
  tier merged with a P5 (doc drift) item for the same crate, Fix-in-place / Delete-candidate bucket.

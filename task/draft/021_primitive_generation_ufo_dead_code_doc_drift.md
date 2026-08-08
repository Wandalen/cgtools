# Delete primitive_generation's dead contours_to_mesh and fix capability-understating doc

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

Two low-stakes `primitive_generation` cleanup items bundled together (P3, dead-code/hygiene bucket): (1)
`src/text/ufo.rs`'s `contours_to_mesh` function (lines 382-545, confirmed by direct read this session) is
marked `#[cfg(feature = "font-processing")] #[allow(dead_code)]` and is NOT included in the crate's own
`mod_interface!` export block (lines 755-765, which only exports `load_fonts, Glyph, Font, text_to_mesh,
text_to_countour_mesh`) — confirmed 100% unreachable from outside the crate; delete it, or wire it into
the public API if it turns out to be intended future surface (check git history/commit messages for intent
before deleting outright); (2) separately, the crate's docs understate its actual capabilities relative to
what the code supports — carried forward from the audit triage plan, re-confirm the specific
under-description against current docs before rewriting.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P3 (dead code) tier
  merged with a P5 (doc drift) item for the same crate, Delete-candidate / Fix-in-place bucket.

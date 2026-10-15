# 316: Track tilemap_renderer adapter-terminal stub (implementation deferred)

## Execution State

- **id:** 316
- **title:** Track tilemap_renderer adapter-terminal stub (implementation deferred)
- **state:** 🚫 (Cancelled)
- **open:** false
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-18 13:44:37
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_renderer
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **cancelled_at:** 2026-08-18 13:52:58
- **cancelled_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

**This is explicitly a tracking placeholder, not active work — do not claim.**
`tilemap_renderer`'s `adapter-terminal` feature gate exists (`src/adapters/terminal.rs`), but the
crate's own `roadmap.md` lists it under `### deferred to follow-up PRs`: no `Backend` implementation
or type exists yet — everything is pending, starting with the basics (`Backend` impl with path/text
output via ASCII/Unicode cells, sprite/mesh/batch support, effect support, gradient approximation; see
`roadmap.md § remaining work : terminal adapter gaps`). This is a deliberate scope decision already
recorded in `roadmap.md`, not an oversight — the crate's five other adapters (SVG, WebGL2, WebGPU,
native, none) are all implemented and documented (`docs/feature/001` through `006`); building a sixth,
substantially different (text-cell-based, not pixel-based) backend now would be pure speculation with
no concrete consumer or test surface to validate against — a direct YAGNI violation. This task exists
solely so the gap has a Unified ID and doesn't silently fall out of the task system's view; no
implementation should begin until a real, concrete consumer need for terminal output exists.

## In Scope

- **If and only if a real consumer emerges:** design and implement `TerminalBackend` — the `Backend`
  trait, path/text output via ASCII/Unicode cells, sprite/mesh/batch support, effect support, and
  gradient approximation — following the same adapter-parity discipline already established for the
  five implemented adapters (compile-and-construct-level tests at minimum, matching
  `docs/feature/003_terminal_backend_adapter.md`'s eventual scope once such a doc exists).

## Out of Scope

- Any speculative implementation now, absent a concrete consumer — this task exists to keep the gap
  visible in the task system, not to schedule work.
- Partial/half-implemented `Backend` coverage (e.g. path output only, no sprite/batch support).
- Editing `roadmap.md`'s existing "deferred to follow-up PRs" / "terminal adapter gaps" sections — they
  already accurately describe this gap; this task file is the tracking artifact, not a doc rewrite.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized without a fresh, concrete
  trigger. If revisited, Delivery Requirements should be re-derived at that time against whichever real
  consumer's actual needs, not written speculatively now.

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a watch-item task by design (mirrors task 056's, 098's, and
  291's pattern; see `task/draft/056_vectorizer_revival_watch_item.md`,
  `task/draft/098_obj_viewer_example_proposal_watch_item.md`, and
  `task/draft/291_reconsider_gpu_hal_mipmapmsaacompute_support_if_a_real_consumer_emerges.md`). Not
  intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state unless a real consumer
  need first materializes.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 13:44:37 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 13:52:30 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CANCELLED | Premise falsified within ~9 minutes of filing: a concurrent actor implemented a full `TerminalBackend` (`impl Backend for TerminalBackend`, ~810 new lines in `src/adapters/terminal.rs`, wired into `mod_interface`) while this watch-item was being filed. "No `Backend` implementation exists yet; do not implement absent a consumer" is no longer true, so the task's own premise is void. Not converted to a completion-tracking task because the actor's own work (once stabilized) is the natural place to register that completion — filing a second, redundant task for it here would itself be duplication. `roadmap.md`'s "terminal adapter gaps" section still describes it as a stub as of this cancellation; that staleness is a separate doc-sync gap for whoever finishes that work to close, not something this watch-item should speak for. |
| 2026-08-18 13:52:58 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CANCEL | task cancelled |

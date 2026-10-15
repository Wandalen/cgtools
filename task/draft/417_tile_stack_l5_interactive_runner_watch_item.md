# 417: Reconsider tile-stack L5 interactive runner if a real consumer emerges

## Execution State

- **id:** 417
- **title:** Reconsider tile-stack L5 interactive runner if a real consumer emerges
- **state:** 📝 (Draft)
- **open:** true
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-19 23:58:00
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_scene
- **actor:** null
- **started_at:** null
- **expires_at:** null

## MOST Goal

**This is explicitly a tracking placeholder, not active work — do not claim.**
`docs/layer/006_l5_scene_script_and_runners.md`'s Contract names two runner modes as part of the L5 layer's
formal contract — interactive (browser/window loop) and off-screen (headless compile → snapshot) — but the
tile stack, the only fully-fledged L5 occupant today, has realized only the off-screen half:
`tilemap_scene`'s `compile/frame.rs` + `renderer.rs` (~24 tests) proves determinism, and that is the whole
of its current runner surface. Round-7's docs/layer gap audit (this session) corrected an overstatement in
both `docs/layer/006` and `docs/render_stack/002_tile.md` that had previously read "L5 is fully realized" /
"fully realizes" — the accurate text now reads "off-screen compile path only; no interactive runner or
example consumer exists yet." No example, tool, or downstream consumer in this workspace currently drives a
tile-stack scene through a live, continuously-updating interactive loop (contrast with `scene_script`'s own
`examples/orrery/webgpu`, which does exactly this for the d2/data path via WebGPU). Building an interactive
runner now — deciding its API shape, its relationship to `tilemap_scene::Renderer`, which backend(s) it
targets, and which example would exercise it — would be speculative architecture with no concrete consumer
to validate against, a YAGNI violation matching this repo's own precedent for exactly this shape of gap
(see `task/draft/291_reconsider_gpu_hal_mipmapmsaacompute_support_if_a_real_consumer_emerges.md`). No
implementation or design work should begin until a real, concrete consumer need exists.

## In Scope

- **If and only if a real consumer emerges:** design and implement an interactive runner for the tile
  stack's L5 layer — a live, continuously-updating loop analogous to `scene_script`'s own interactive
  realization in `examples/orrery/webgpu`, but compiling/executing `tilemap_scene` RON scenes instead of
  Rhai scripts — following the same off-screen-mode-first discipline and determinism guarantees already
  proven for the existing compile/runner path.

## Out of Scope

- Any speculative design or implementation now, absent a concrete consumer — this task exists to keep the
  door open, not to schedule work.
- Wiring `scene_script`'s lints into a production load path — a separate, already-actionable, tightly
  scoped gap from the same round-7 audit, filed as a full implementation task instead (see Related
  Documentation).

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized without a fresh, concrete
  trigger. If revisited, Delivery Requirements should be re-derived at that time against whichever real
  consumer's actual needs, not written speculatively now.

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a watch-item task by design (mirrors task 291's, task 056's, and
  task 098's pattern; see `task/draft/291_reconsider_gpu_hal_mipmapmsaacompute_support_if_a_real_consumer_emerges.md`).
  Not intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state unless a real consumer
  need first materializes.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 23:58:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/ | FILED | Filed via `/doc_tsk` after round-7's docs/layer gap audit corrected an overstated "L5 fully realized" claim, revealing the tile stack has no interactive runner and no consumer requiring one — watch-item only, per this repo's own task-291 precedent for the same shape of gap. |

## Related Documentation

- `docs/layer/006_l5_scene_script_and_runners.md` — the layer doc naming both runner modes in its Contract
  section, and whose round-7 fix corrected the overstated "fully realized" claim
- `docs/render_stack/002_tile.md` — the tile stack's own identity card, likewise corrected in round-7
- `task/draft/291_reconsider_gpu_hal_mipmapmsaacompute_support_if_a_real_consumer_emerges.md` — the
  precedent watch-item this task's structure and rationale directly mirror
- `task/unverified/416_scene_script_production_lint_enforcement.md` — the sibling gap from the same
  round-7 audit that *was* tightly-scoped enough to file as an actionable implementation task

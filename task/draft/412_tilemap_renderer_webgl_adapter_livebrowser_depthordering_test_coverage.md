# 412: tilemap_renderer webgl adapter: live-browser depth-ordering test coverage

## Execution State

- **id:** 412
- **title:** tilemap_renderer webgl adapter: live-browser depth-ordering test coverage
- **state:** 📝 (Draft)
- **open:** true
- **in_motion:** false
- **round:** 1
- **filed:** 2026-08-19 23:05:52
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tilemap_renderer
- **actor:** null
- **started_at:** null
- **expires_at:** null

## MOST Goal

Close a real, currently-undocumented test-coverage gap found while adding the SVG-side
cross-backend depth-ordering test (`svg_ignores_depth_preserves_submission_order` in
`tests/svg_backend_test.rs`, closing part of `docs/invariant/003_z_layer_draw_ordering.md`'s
Tests table): `tests/webgl_backend_test.rs` has zero coverage of `Transform::depth`, draw order,
or the depth buffer — its own doc comment states plainly *"No `WebGl2RenderingContext`/`web_sys`
call anywhere in this file"*. `roadmap.md` documents WebGL2's depth handling as implemented
(`DEPTH_TEST`/`LEQUAL`, `[-max_depth, max_depth]` range, "reliable for fully opaque draws") but
no test proves it. Distinct from task 246's Out of Scope note (which defers *native/offscreen*
WebGL2 context testing as a workspace-wide infrastructure gap) — this task targets a real
**browser** context via `browsee`, the same mechanism already proven out in tasks 191/218/251/342.

## In Scope

- A `browsee`-driven live-browser test submitting multiple sprites/meshes at non-monotonic
  `Transform::depth` values to a real `WebGlBackend`-rendered canvas, then pixel-reading the
  result to confirm depth-order (not submission-order) compositing — the WebGL-side mirror of
  `svg_ignores_depth_preserves_submission_order`'s proof, but for actual rendered pixels instead
  of DOM element order.
- Following this crate's existing browsee-based verification precedent and directory conventions
  (see `gpu_hal/tests/manual/readme.md`'s procedure, tasks 191/218/251).

## Out of Scope

- Native/offscreen `WebGl2RenderingContext` testing — already deferred workspace-wide per task
  246's Out of Scope note; not reopened here.
- Any other adapter's depth-ordering coverage — SVG's is done (see above); WebGPU/native/terminal
  are separate, untouched by this task.
- Any change to `WebGlBackend`'s actual depth-handling implementation — this is a test-coverage
  gap, not a known behavioral defect.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- N/A while this task remains 📝 Draft — no implementation is authorized until this is fleshed
  out into a full Quality-Gate task (Test Matrix, Acceptance Criteria, Delivery Requirements
  re-derived against the actual scoped browsee test approach at that time).

## Acceptance Criteria

- N/A while this task remains 📝 Draft — a tracking placeholder, not yet scoped for execution.
  Not intended to progress through SUBMIT/VERIFY toward 🎯 Verified/claimable state until
  fleshed out.

## Verification

- N/A while this task remains 📝 Draft — same rationale as Acceptance Criteria above.

## Related Documentation

- `module/helper/tilemap_renderer/docs/invariant/003_z_layer_draw_ordering.md` — Tests table,
  the SVG-side row this task's WebGL-side counterpart would complete
- `task/accepting/246_tilemap_renderer_webgl_adapter_test_coverage.md` — Out of Scope note this
  task's browser-based (not native/offscreen) approach does not reopen
- `task/unverified/251_tilemap_renderer_adapterwebgpu_and_adapterwebgl_browser_pixel_verification.md`
  — sibling browsee-based pixel-verification precedent for this same adapter

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 23:05:52 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-19 23:30:22 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/ | NOTE | Verified, not submitted — flagging an internal inconsistency instead. Unlike siblings 399-409, this task's MOST Goal and In Scope are fully concrete and independently confirmed accurate: `svg_ignores_depth_preserves_submission_order` genuinely exists at `tests/svg_backend_test.rs:521`; `tests/webgl_backend_test.rs:5` genuinely reads "No `WebGl2RenderingContext`/`web_sys` call anywhere in this file"; cited tasks 191/218/251/246 all confirmed to exist — **but 342 does not** (`tsk .get 342` returns nothing), the same citation-defect pattern found in 404/405/406. Separately, and more significantly: Delivery Requirements/Acceptance Criteria/Verification are the identical boilerplate placeholder text used by the genuinely-unscoped siblings ("not yet scoped for execution... re-derived... at that time") — but this task's own MOST Goal/In Scope already describe a fully-scoped, mechanically-mirrorable piece of work (explicitly "the WebGL-side mirror of `svg_ignores_depth_preserves_submission_order`'s proof"). This reads as a copy-paste artifact from batch-filing the 399-409/412 set, not a genuine YAGNI-scoped placeholder. Deriving concrete AC/Verification from the existing In Scope text would be straightforward — deliberately not done here, since authoring new task scope is outside this verification pass's remit. Left in 📝 Draft; flagging for whoever owns this task to either flesh out AC/Verification to match the already-concrete goal, or explain why it should stay parked. |

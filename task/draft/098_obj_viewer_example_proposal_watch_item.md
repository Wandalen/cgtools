# Consider a new obj_viewer example if a real consumer emerges

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
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/obj_viewer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`examples/minwebgl/obj_load/src/main.rs:41` carried `# qqq : for Yevhen : implement a example obj_viewer, which allow upload any 3d model and see very detailed and full diagnostics information`, from task 038's original census (2026-08-10). Task 065's triage (2026-08-12) reviewed this ask: a new example crate accepting an arbitrary user-uploaded 3D model (not just the fixed `suzanne.obj`) and displaying full diagnostic information via the reporting machinery adopted in task 097 (`mingl::model::obj::make_reports`/`ReportObjModel`).

**This is explicitly a tracking placeholder, not active work.** No implementation should begin speculatively — there is no concrete, committed consumer need for a general-purpose model-upload viewer beyond the original marker's own wish. If a real need emerges (e.g. a debugging workflow that requires inspecting arbitrary user-supplied models, or a documentation/demo requirement calling for one), revisit building it as its own example crate, reusing the diagnostic-report machinery already adopted by task 097 rather than re-implementing it. Until that trigger condition exists, no further action is needed on this task.

**Related Tasks:** `065` (`task/completed/065_examples_marker_triage.md` once closed) — source of this marker's triage decision. `097` (`task/completed/097_obj_load_adopt_existing_helpers.md`) — the reporting machinery this proposal would reuse if ever built.

## History

- **[2026-08-12]** `FILED` — Filed via lightweight Draft capture (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) as part of task 065's marker-triage disposition. Classified as a YAGNI deferral: the marker's ask (a new example crate) has no concrete consumer beyond its own wish, so it is tracked as a watch-item rather than filed as active work, mirroring task 056's pattern for the vectorizer revival question.

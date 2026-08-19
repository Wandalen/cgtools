# Consider a new obj_viewer example if a real consumer emerges

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 🚫 (Cancelled)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/obj_viewer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null
- **cancelled_at:** 2026-08-19 12:11:48
- **cancelled_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## Goal

`examples/minwebgl/obj_load/src/main.rs:41` carried `# qqq : for Yevhen : implement a example obj_viewer, which allow upload any 3d model and see very detailed and full diagnostics information`, from task 038's original census (2026-08-10). Task 065's triage (2026-08-12) reviewed this ask: a new example crate accepting an arbitrary user-uploaded 3D model (not just the fixed `suzanne.obj`) and displaying full diagnostic information via the reporting machinery adopted in task 097 (`mingl::model::obj::make_reports`/`ReportObjModel`).

**This is explicitly a tracking placeholder, not active work.** No implementation should begin speculatively — there is no concrete, committed consumer need for a general-purpose model-upload viewer beyond the original marker's own wish. If a real need emerges (e.g. a debugging workflow that requires inspecting arbitrary user-supplied models, or a documentation/demo requirement calling for one), revisit building it as its own example crate, reusing the diagnostic-report machinery already adopted by task 097 rather than re-implementing it. Until that trigger condition exists, no further action is needed on this task.

**Related Tasks:** `065` (`task/completed/065_examples_marker_triage.md` once closed) — source of this marker's triage decision. `097` (`task/completed/097_obj_load_adopt_existing_helpers.md`) — the reporting machinery this proposal would reuse if ever built.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-19 12:11:48 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CANCEL | task cancelled |

## History

- **[2026-08-12]** `FILED` — Filed via lightweight Draft capture (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) as part of task 065's marker-triage disposition. Classified as a YAGNI deferral: the marker's ask (a new example crate) has no concrete consumer beyond its own wish, so it is tracked as a watch-item rather than filed as active work, mirroring task 056's pattern for the vectorizer revival question.
- **[2026-08-19]** `SUPERSEDED (partial)` — Discovered, while attempting to act on this task's own `unit:` path, that `examples/minwebgl/obj_viewer` already exists at HEAD and has since commit `2be3d2cc` (2026-08-10, two days *before* this task was even filed): a complete interactive OBJ viewer with orbit-camera controls, full PBR texture/material pipeline (`src/material.rs`, `src/mesh.rs`), a bundled "lost-empire" demo scene, a `showcase.webp`, and its own already-`completed` bug fix (`task/bug/completed/340_...md`, which explicitly names `obj_viewer` alongside `obj_load`/`make_cube_map`). Authored by `Avramenko Yevhenii <yevhenii.av@obox.systems>` per its `Cargo.toml` — the same "Yevhen" the original `qqq:` marker was addressed to. It already reuses task 097's reporting machinery, logging full diagnostics to the browser console (`gl::log::info!("{report}")` per model). **Not covered by the existing crate:** the marker's other half — accepting an *arbitrary user-uploaded* model via a file picker (the existing crate only ever loads one fixed, bundled "lost-empire" scene) and displaying the diagnostics in-page rather than console-only. Cancelling this task as-is because its own `unit:` path is a genuine collision with unrelated, already-shipped work — building "at this path" would mean overwriting that work, which is not wanted — not because the marker's full ask is satisfied. The upload-picker gap is real; whether it is still worth a dedicated (differently-pathed) task is deferred to the filer. (Caution for future readers: an in-session attempt to "implement" this from scratch briefly overwrote 6 of the existing crate's tracked files before being caught via `git status`/`git show` and fully reverted — always check whether a task's `unit:` path already exists before creating it.)

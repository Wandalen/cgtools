# Triage the examples tree's 13 task markers — most need human decisions (decomposed from task 038)

## Execution State

- **Executor Type:** human
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** workspace
- **unit:** examples
- **verified_by:** self (Tier 2 Dual-Role Self-Check)
- **verification_date:** 2026-08-12
- **blocked_by:** null
- **priority:** 0

## Goal

Triage the 13 live task markers across 8 example crates (census 2026-08-10, task 038 — re-derive at
pickup). Deliberate Crate Scope Unity deviation, stated up front: this is ONE task spanning 8 crates
because 6 of the 13 markers are addressed to named people (Yevhen/Yevgen) and need their (or the
owner's) decision, not code work — filing 8 skeleton tasks around pending human decisions would be
premature. Once decisions land, any resulting code work gets its own per-crate task.

**Crate-deletion decisions (owner call, blocks everything else about those crates):**
- `diamond/Cargo.toml:15` — `# qqq : for Yevhen : rid of this crate`.
- `make_cube_map/Cargo.toml:15` — `# qqq : for Yevhen : rid of this crate`.
  Decide: delete these two example crates (and their index/demo_completeness rows) or keep and
  remove the markers.

**Feature requests addressed to a named person (`obj_load/src/main.rs`, ×4):**
- `:27` — diagnostic Report helper with verbosity control; `:29` — load-from-byte-slice helper;
- `:36` — `why error?` (unexplained error path); `:41` — obj_viewer example proposal.
  Decide which become real tasks (the helpers belong in mingl/minwebgl scope, the viewer is a new
  example) and which are dropped.

**Documentation/quality asks (executable without decisions, lowest risk):**
- `raycaster/src/main.rs:52,53` — explain shader roles, more docs overall.
- `attributes_matrix/src/main.rs:11` — "make usecase more impressive changing code minimally".
- `uniforms_ubo/src/main.rs:44,52` — `does it give any benefit?` ×2 (measure or explain the UBO
  benefit claim, then delete the marker).
- `pbr_lighting/src/gui_setup.rs:449,483` — `// TODO: add later` ×2 (unimplemented GUI branches —
  implement or delete the placeholder arms).

Per-marker outcomes follow task 038's triage contract: decide, then fix / file follow-up / delete.
Verification: after any code change, the touched example must still build
(`cargo check -p <example> --target wasm32-unknown-unknown` via `longrun .launch`, no bare
RUSTFLAGS); after any crate deletion, `examples/index.md` and `demo_completeness.md` rows must be
removed in the same change and the workspace must still resolve (`cargo metadata` exit 0).

## Outcomes

Re-derivation at pickup (2026-08-12, per this task's own "re-derive at pickup" caveat) found the
2026-08-10 census stale: only 7 of the original 13 markers were still live. 6 were already resolved
by other work since the census: `raycaster/src/main.rs:52,53` (an explanatory shader-role comment
already present), `uniforms_ubo/src/main.rs:44,52` (both UBO-benefit markers gone), and
`pbr_lighting/src/gui_setup.rs:449,483` (both `// TODO: add later` markers gone). `attributes_matrix
/src/main.rs`'s marker survived but had drifted (line 11→118) and lost its original payload text,
becoming a bare, contentless `// xxx`.

**Decisions made (2026-08-12):**
- **diamond / make_cube_map crate-deletion markers:** KEEP both crates — each is a complete,
  documented, fully-registered demo (`examples/index.md`, `demo_completeness.md` both
  "yes/yes/yes/yes"); delete the stale markers only. Filed as tasks 094, 095.
- **obj_load `:27`/`:29`/`:36`:** ACCEPT, bundled as one task — re-derivation found
  `mingl::web::model::obj::load_model_from_slice` and `make_reports`/`ReportObjModel` already
  implement exactly what `:27` and `:29` ask for (re-exported via `minwebgl::model::obj`);
  `load_model_from_slice`'s real materials-fetch logic is the direct answer to `:36`. No new library
  code needed — only adopting the existing helpers in the example. Filed as task 097.
- **obj_load `:41` (obj_viewer proposal):** DEFER per YAGNI — no concrete consumer beyond the
  marker's own wish. Filed as Draft watch-item task 098, mirroring task 056's pattern.
- **attributes_matrix's drifted `// xxx`:** DELETE — contentless, original payload text
  unrecoverable, adjacent code already adequately commented. Filed as task 096.
- **raycaster, uniforms_ubo, pbr_lighting markers:** No action needed — already resolved by other
  work; confirmed absent via direct grep re-derivation.

All resulting code work is filed as separate per-crate tasks (094-098) per this task's own stated
plan ("Once decisions land, any resulting code work gets its own per-crate task") — this task's own
scope (getting the human decisions made) is complete.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 038's workspace marker census (80 lines →
  per-crate tasks; this tranche deliberately kept whole because 6/13 markers are human-addressed
  decisions, recorded as an explicit D6 deviation). Executor Type set to human for the decision
  rounds; code follow-ups will be filed per-crate once decided.
- **[2026-08-12]** `COMPLETE` — Decisions made in-conversation with the task's filer; re-derivation
  found 7/13 markers still live, not 13. Disposition filed as tasks 094 (diamond), 095
  (make_cube_map), 096 (attributes_matrix), 097 (obj_load :27/:29/:36 bundled), 098 (obj_viewer :41,
  deferred Draft). This task's own scope (human decisions) fulfilled; code work handed off per-crate
  as planned.

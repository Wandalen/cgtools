# Triage the examples tree's 13 task markers — most need human decisions (decomposed from task 038)

## Execution State

- **Executor Type:** human
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** workspace
- **unit:** examples
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

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

## History

- **[2026-08-10]** `FILED` — Decomposed from task 038's workspace marker census (80 lines →
  per-crate tasks; this tranche deliberately kept whole because 6/13 markers are human-addressed
  decisions, recorded as an explicit D6 deviation). Executor Type set to human for the decision
  rounds; code follow-ups will be filed per-crate once decided.

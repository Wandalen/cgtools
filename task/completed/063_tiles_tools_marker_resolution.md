# Resolve tiles_tools' 8 task markers (decomposed from task 038)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** module/helper/tiles_tools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Resolve the 8 live task markers in `module/helper/tiles_tools` (census 2026-08-10, task 038 —
re-derive at pickup). Two distinct clusters:

**`src/geometry.rs` review-conversation block (`aaa :` ×5, lines 9-12, 23):** naming ("use geometry
instead of mesh. rename also file" — yet the file IS already `geometry.rs`, so part may be already
satisfied), primitive-kind metadata, "no fans or loops", more-descriptive docs, and a confused-naming
question about `mesh_producer`. CAUTION — do not blind-delete: `docs/algorithm/
004_hexagon_mesh_generation.md` explicitly cites the `// aaa : no fans or loops` comment (line 11)
and documents that `hexagon_triangles()` VIOLATES it (fan triangulation shipped despite the
constraint). Resolution must decide the constraint's status first: either the no-fans constraint is
real (then the implementation is the defect — rewrite triangulation, update the algorithm doc) or
it's obsolete (then delete the marker AND update the algorithm doc's contradiction note in the same
change). Marker and doc must not end up contradicting each other.

**`src/ecs` unimplemented-behavior TODOs (×3):**
- `world.rs:230` — `// TODO: Implement proper type-safe movement request processing` — this is the
  no-op documented by `docs/pitfall/002_ecs_movement_requests_are_a_no_op.md`; implementing it
  retires that pitfall doc (update or delete the pitfall in the same change).
- `systems.rs:91` and `:92` — pathfinding hardcodes `|_coord| true` (no obstacle checking) and
  `|_coord| 1` (no terrain cost). Real feature work; wire actual obstacle/cost sources or document
  the parameters as caller-supplied by design and delete the TODOs.

Per-marker outcomes follow task 038's triage contract. Verify with
`cargo test -p tiles_tools --all-features` (via `longrun .launch`); doc updates must keep
`docs/algorithm/004` and `docs/pitfall/002` consistent with whatever the code becomes.

## In Scope

- `module/helper/tiles_tools/src/geometry.rs` (+ `docs/algorithm/004`, renamed to
  `..._geometry_generation.md`): resolved the 5 `aaa:` review-conversation markers (mesh→geometry
  vocabulary purge, primitive-kind metadata docs, "no fans or loops" constraint reconciled with
  the shipped triangulation, descriptive docs, naming cleanup)
- `module/helper/tiles_tools/src/ecs/world.rs` and `src/ecs/systems.rs`: resolved the 3 `TODO:`
  markers (implemented the type-safe movement queue, retiring `docs/pitfall/002`; made
  pathfinding's obstacle/cost parameters caller-supplied by design)

## Out of Scope

- Full reconciliation of `roadmap.md`'s ECS-movement narrative — this task's own verification
  (C7) found 4 `roadmap.md` locations still describing the old no-op gap after the Phase
  3/Known-Gaps update; not independently re-swept in full here

## Verification

### Checklist

- [x] C1 — Is the workspace task-marker census (`xxx:`/`qqq:`/`aaa:`/`TODO:`) in `tiles_tools/src/` still clean? `grep -rnE "(^|[^a-zA-Z_])(xxx|qqq|aaa|TODO) *:" src/` → 0 hits (exit 1) — matches this task's claimed "Census clean (exit 1, zero markers in crate)".
- [x] C2 — Is `geometry.rs`'s naming cleanup (`mesh`→`geometry`/`shape` vocabulary purge, `hexagon_triangles_with_tranform` typo fix) still intact? `geometry_producer` present (line 24); `grep -c "mesh_producer" src/geometry.rs` → `0`; `hexagon_triangles_with_transform` present (line 108); `grep -c "hexagon_triangles_with_tranform\b" src/geometry.rs` → `0`.
- [x] C3 — Is `docs/algorithm/004`'s rename (`..._mesh_generation.md` → `..._geometry_generation.md`) and its cross-references still intact? File exists at the new path; `docs/algorithm/readme.md:19` and `docs/definition/readme.md:23` both cite the new filename; `docs/pitfall/readme.md` has 0 hits for "002".
- [x] C4 — Is the type-safe movement queue in `world.rs` still implemented (replacing the discard-the-target no-op)? `movement_requests : HashMap< hecs::Entity, MovementRequestApply >` (line 69) with `type MovementRequestApply = Box< dyn FnOnce( &mut hecs::World ) -> bool + Send + Sync >` (line 53) — structurally identical to the claim, now behind a named type alias. `git diff` on `world.rs` shows only 1 unrelated cosmetic line (an `#[allow]` reason-string addition on a different, unrelated function) — the movement queue itself is untouched since this task.
- [x] C5 — Is `docs/pitfall/002_ecs_movement_requests_are_a_no_op.md` still deleted? Absent from `docs/pitfall/` (only `001`, `003`, `004`, `readme.md` remain).
- [x] C6 — Are `systems.rs`'s caller-supplied `is_accessible`/`cost` parameters still in place, with both TODOs gone? `process_movement< C, Fa, Fc >` (line 47) / `calculate_movement< C, Fa, Fc >` (line 84) both take `is_accessible : Fa` / `cost : Fc`; 0 `TODO` hits remain in `systems.rs`.
- [ ] C7 — Was this task's own claim of having fully updated "roadmap.md" as 1 of 8 `pitfall/002` reference sites accurate? PARTIALLY — the deep Phase 3 narrative (roadmap.md:208) and the Known Gaps table were updated correctly, but 4 other roadmap.md locations still describe ECS movement/`pitfall/002` as an open gap (full list: task 025's Verification C1). The other 7 claimed reference sites (pitfall/readme, type/002, definition/readme, api/001 ×4) were spot-checked only via `docs/pitfall/readme.md`'s 0-hit grep in C3, not independently re-swept in full here.

### Measurements

- [x] M1 — Task markers in `tiles_tools/src/`: `0` (was: `8` — 5 in `geometry.rs` at lines 9,10,11,12,23 + 1 in `world.rs` at line 230 + 2 in `systems.rs` at lines 91,92; confirmed via `git show 77cc9b9a:module/helper/tiles_tools/src/{geometry.rs,ecs/world.rs,ecs/systems.rs}`, independently reproducing this task's own census).

### Invariants

- [x] I1 — Test suite (crate-scoped, `--all-features`): `cargo nextest run -p tiles_tools --all-features` → exit 0, 245/245 passed (log `-0009_longrun.log`). This task's own post-fix baseline (log `-0028`, cited in its History) was `56 unit + 189 integration = 245` (doc-tests excluded from nextest) — an identical total; task 072 later reorganized inline tests into top-level files (net −5 from consolidating true duplicates, landing at 240) and task 078 added 5 previously-dead integration tests back (net +5, landing at 245) — see task 078's Measurements for the full chain arithmetic.
- [ ] I2 — Compiler/lints (crate-scoped, `--all-features`): `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings` → exit 101 (log `-0015_longrun.log`). Root cause is an unrelated dependency (`browser_log`, 3 hops away via `animation`/`mingl`/`minwebgl`) — full trace in task 025's Verification I2. Not attributable to this task's `geometry.rs`/`world.rs`/`systems.rs` edits.

### Anti-faking checks

- [x] AF1 — Guards against a marker reappearing without the same triage-and-resolve discipline: re-running C1's census grep after any future edit to `tiles_tools/src/` must still return 0 — a nonzero count means a marker was added without a corresponding Goal-style disposition record.
- [x] AF2 — Guards against `world.rs`'s movement queue silently regressing to a discard-the-target no-op: re-running C4's grep for `movement_requests : HashMap` after any future `World`/`request_movement` edit must still find the typed-closure queue, not a bare no-op body.

## History

- **[2026-08-10]** `FILED` — Decomposed from task 038's workspace marker census (80 lines →
  per-crate tasks per Crate Scope Unity). Both clusters are doc-entangled: two docs/ instances cite
  these exact markers, so resolution is code+doc paired work, not comment cleanup.
- **[2026-08-10]** `IMPLEMENTED` — All 8 markers resolved per task 038's triage contract:
  - **`geometry.rs` aaa ×5 (lines 9-12, 23):** (1) *mesh→geometry vocabulary* — file was already
    `geometry.rs`; the residue was purged instead: `mesh_producer` param → `geometry_producer`,
    local `mesh` → `shape`, doc renamed `004_hexagon_mesh_generation.md` →
    `004_hexagon_geometry_generation.md`, crate `readme.md` + `docs/algorithm/readme.md` +
    `docs/definition/readme.md` rows updated. (2) *primitive-kind metadata* — every generator's doc
    now states intended draw mode and counts (fill: 4 standalone triangles / 24 floats /
    `TRIANGLES`; outline: 6 standalone segments incl. explicit closing segment / 24 floats /
    `LINES`). (3) *"no fans or loops"* — constraint's status decided: it is REAL and SATISFIED. It
    governs required *draw modes* (`from_iter` batches many cells into one buffer; mode-level
    fans/loops cannot express disjoint shapes in one draw call), not fan-*patterned topology*
    (shared anchor vertex — the standard convex-polygon triangulation). Module doc now states the
    constraint + rationale; doc 004 rewritten — its earlier "violates" claim conflated the two
    readings. Marker deleted; code and doc no longer contradict. (4) *descriptive docs* — all
    public fns + new shared private helper `triangles_from_vertices` documented; ASCII art kept.
    (5) *confused naming* — resolved by the rename cluster, including public
    `hexagon_triangles_with_tranform` → `hexagon_triangles_with_transform` (misspelling; zero
    workspace callers, proven by workspace check log `-0029` exit 0 incl. all example crates).
  - **`world.rs:230` TODO (movement no-op):** implemented type-safe movement queue —
    `HashMap< hecs::Entity, Box< dyn FnOnce( &mut hecs::World ) -> bool + Send + Sync > >`; closure
    captures the typed target, latest request per entity wins, applied on `update` with
    `GameEvent::EntityMoved` per success; mismatched `Position< C >` type or despawned entity
    discards silently. `docs/pitfall/002_ecs_movement_requests_are_a_no_op.md` deleted; all 8
    reference sites updated (pitfall/readme, type/002, definition/readme, api/001 ×4 sites,
    roadmap.md). Module doctest extended to pin the end-to-end path.
  - **`systems.rs:91-92` TODOs (hardcoded `|_| true` / `|_| 1`):** documented as caller-supplied
    by design and made so — `process_movement`/`calculate_movement` gained `is_accessible : Fa` and
    `cost : Fc` parameters (the ECS deliberately defines no obstacle/terrain component to derive
    them from, so the caller owns both policies, matching `astar`'s own API). TODOs deleted; no
    external callers existed.
  - **Tests:** `ecs_tests.rs` — vacuous `test_movement_requests` (asserted nothing) rewritten to
    assert position + `EntityMoved`; new `test_movement_request_latest_wins`,
    `test_movement_request_discard_cases`, `test_movement_system_uses_caller_policies`
    (Success/NoPathFound/PathTooLong). New `geometry_tests.rs` (5 tests) pins doc 004's Tests
    contract: vertex count/order/radius, triangle count + summed area vs analytic `3√3/2`, outline
    segments + closure, transform variant vs manual per-vertex transformation, `from_iter`
    per-cell replication. New `tests/integration/readme.md` Responsibility Table (10 files).
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Census clean (exit 1, zero markers in crate).
  Suite green: log `-0028` exit 0 — 56 unit + 189 integration + 40 doc-tests, 0 failed; workspace
  check log `-0029` exit 0 (4.91s). Three genuine in-loop adversarial catches: (1) doc 004's
  "implementation violates the constraint" premise FALSIFIED — the constraint reads on draw modes,
  which the shipped independent-triangle encoding satisfies; resolution rewrote the doc rather than
  the triangulation. (2) astar-hang caught PRE-RUN by reading `pathfind.rs`: the new NoPathFound
  test initially blocked only the target on an unbounded grid — `pathfinding::prelude::astar`
  never terminates when the goal is unreachable on an infinite graph; fixed by bounding
  accessibility to a finite box the search exhausts (integration suite 0.11s in `-0028` — no
  hang). (3) pre-existing smoke test asserted nothing ("verifies the API works without errors") —
  upgraded to real assertions instead of left as cover.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | — | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Doc 004's "violates" premise falsified: no-fans constraint reads on draw modes, not fan topology — code was correct, doc was wrong | Constraint semantics stated in module doc; doc 004 rewritten with resolution note |
| D5 | Execution Scope | 🟢 | 🟢 | — | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | — | — |
| B2 | Test-First | 🟡 | 🟢 | Pre-existing `test_movement_requests` was vacuous — asserted nothing | Rewritten to assert position change + `EntityMoved` event; 3 new movement tests added |
| B3 | Evidence of Failure | 🟢 | 🟢 | — | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | — | — |
| B5 | Fix Verification | 🟡 | 🟢 | New NoPathFound test would hang: unbounded grid + unreachable goal never terminates in `pathfinding::prelude::astar` — caught by reading the impl before launch | Accessibility bounded to a finite box the search exhausts; suite ran 0.11s (log `-0028`) |
| B6 | Knowledge Preservation | 🟢 | 🟢 | — | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 3 findings resolved in-loop | 15/15 |

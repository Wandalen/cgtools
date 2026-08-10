# Re-enable or retire the disabled flowfield integration tests in tiles_tools

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** crate
- **unit:** module/helper/tiles_tools
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`tests/integration/flowfield_tests.rs` (479 lines, ~21 tests) is dead coverage:
`tests/integration/mod.rs:36` comments it out — "Temporarily disabled until flowfield
generic constraints are resolved" — so it never compiles or runs. Found by task 072.

Two independent defect classes keep it broken:

1. **Private-field reads** — e.g. `flow_field.width` / `flow_field.height`
   (`FlowField`'s dimensions are private, `#[ allow( dead_code ) ]`). Task 072
   relocated 3 live inline tests to `tests/flowfield_test.rs` that overlap the
   disabled file's creation/variant tests (`test_integration_field_creation`,
   `test_flow_direction_variants`, `test_multi_goal_flow_field_creation`) — dedup
   against that file when reviving.
2. **Generic-constraint mismatches** — tests instantiate `FlowField::< (), () >` and
   then call coordinate-bound methods (`calculate_flow` with
   `SquareCoord< FourConnected >` goals); the stub-era API and the tests disagree.

Resolution procedure:
1. Decide per test: repair against the current public API, or retire if it asserts
   stub-era behavior the implementation no longer promises. Never keep a test
   disabled — "No Disabled Tests: fix them or remove them."
2. Dedup repaired tests against `tests/flowfield_test.rs` (one home per behavior).
3. Uncomment `mod flowfield_tests;` in `tests/integration/mod.rs`; also decide the
   fate of the placeholder comments for never-created `grid_tests` /
   `pathfinding_tests` / `generation_tests` modules there.
4. Verify with `longrun .launch dir::<workspace root> -- cargo test -p tiles_tools --all-features` —
   integration suite green with the module enabled.

## History

- **[2026-08-10]** `FILED` — Found during task 072's tests/ survey: the module is the
  only existing-but-disabled test file in the crate, and its disablement predates the
  072 census (both the private-field reads and the generic-constraint drift).
- **[2026-08-10]** `IMPLEMENTED` — Census at pickup: 21 tests (not ~21), and the file
  never compiled even at authoring time (`test_rts_scenario_simulation` uses
  `Movable` without importing it). The draft's two defect classes bottom out in one
  root cause deeper than recorded: **`Grid2D` — the flowfield backing store — is
  hexagonal-only** (`collection.rs:5` imports `hexagonal::Coordinate`; its
  `Index`/`IndexMut` require `C : Into< hexagonal::Coordinate >`, and no other
  coordinate system converts in), so every square-coordinate flowfield test is
  structurally unsatisfiable, not merely drifted. Worse: `hexagonal::Coordinate` had
  manual `Clone`/`Eq`/`PartialEq`/`Hash` but **no `Ord`**, while `calculate_flow` /
  `add_goal` demand `C : Ord` — so those public methods were uncallable by ANY
  coordinate type in the crate (dead API with zero possible call sites). Fixed
  properly, not for tests' sake: manual `PartialOrd`/`Ord` on
  `Coordinate< System, Orientation >` (lexicographic `( q, r )`, consistent with
  `PartialEq`; manual like its sibling impls so `System`/`Orientation` stay
  unbounded — a derive would wrongly bound them). Per-test dispositions (21 = 5
  kept + 16 retired): KEPT repaired to `FlowField::< Axial, Pointy >`-family
  instantiation — `test_hex_grid_with_water_obstacles` (the only calculate_flow
  call-site coverage in the crate), `test_batch_flow_direction_queries` +
  `test_group_movement_flow_application` (length-preservation contracts, real
  implemented behavior), `test_multi_goal_capture_points` (add_goal → one field
  per goal, absorbing the retired resource-gathering variant's len assertion),
  `test_flow_field_ecs_integration` (ECS×flowfield batch, query repaired to the
  live `let mut query` idiom). RETIRED: 4 duplicates of live coverage
  (integration-field creation, flow-field creation [also private-field reads],
  flow-direction variants [novel None≠Move assertion absorbed into
  tests/flowfield_test.rs's test_flow_direction_enum], multi-goal creation);
  5 square-coordinate structurally-unsatisfiable and assertion-free tests (basic,
  obstacles, terrain-costs, unreachable-goal, single-cell); 2 assertion-free
  redundancies (hexagonal_grid_flow_field — no distinct behavior beyond the
  water-obstacles survivor since the stub never calls the closures;
  flow_field_with_no_goal — get_flow_direction covered transitively by batch);
  1 private-field duplicate (zero-dimension); 2 wall-clock timing asserts against
  a stub (large-grid <5s, many-units <100ms — Fragile Test class, benches/ home
  if ever wanted); 1 composite duplicate that never compiled (rts_scenario =
  ecs_integration + group_flow + Team, each covered elsewhere). File: 480 → 178
  lines via boundary-asserted rewrite script; header matrix rewritten to the 5
  survivors + a "Why hexagonal-only" section. `mod.rs`: `mod flowfield_tests;`
  enabled, the disablement comment AND the three never-created placeholder
  comments (`grid_tests`/`pathfinding_tests`/`generation_tests`) deleted — dead
  commented code; future suites are tasks, not comments. Stale cross-references
  updated: tests/readme.md row, integration/readme.md row,
  tests/flowfield_test.rs header.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Log `-0052`
  (`cargo test -p tiles_tools --all-features`) exit 0, 15s: integration suite
  **189 → 194** (exactly the 5 revived tests, each visible by name in the log),
  unit 5/5 (inline exceptions intact), flowfield_test 3/3 (absorbed pin included),
  all sibling suites + 40 doc-tests green — the additive `Ord` impl regressed
  nothing across the full crate. Health metrics regenerated: tests fns 257 → 241
  (−21 dead +5 live — the old grep was counting tests that never ran), files 18,
  inline 5, allows 460 (the rewritten file keeps the suite's standard allow
  block). Dependent-crate safety: an additive trait impl on an owned type cannot
  break dependents (it only enables previously-uncompilable code; orphan rules
  preclude conflicts).

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Scope Coherence | 🟢 | 🟢 | All four draft procedure steps executed (per-test decide, dedup, mod.rs enable + placeholder fate, --all-features verify); src edit confined to the one impl the revival required | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | Census re-derived at pickup: 21 exact (draft said ~21); both dedup overlaps the draft named confirmed and handled | — |
| D3 | Value/YAGNI | 🟢 | 🟢 | No new test infrastructure; retired 16 tests rather than scaffolding stubs to keep them; timing tests retired instead of relocated to benches/ nobody asked for | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Confirming pass planned "repair square tests to hex + fix instantiations" wholesale; adversarial pass against the actual bounds found the deeper truth: Grid2D is hexagonal-only (square tests unsatisfiable — retire, don't rewrite their story) AND no coordinate type satisfied `C : Ord`, so `calculate_flow`/`add_goal` were dead public API for every type in the crate | Manual `PartialOrd`/`Ord` on `hexagonal::Coordinate` (lexicographic, `PartialEq`-consistent, `System`/`Orientation` unbounded like the sibling manual impls); survivors instantiate `< Axial, Pointy >` |
| D5 | Execution Scope | 🟢 | 🟢 | 7 files touched: hexagonal.rs, flowfield_tests.rs, mod.rs, flowfield_test.rs, 2 readmes, this record | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Single-crate task; Ord addition is additive (cannot break dependents — only enables previously-impossible code) | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟢 | 🟢 | "No Disabled Tests: fix them or remove them" satisfied — zero disabled tests remain; no mocks; kept bodies moved with named transforms only (type-param substitution, Square→Hex coordinate substitution, live query idiom, stale-stub-comment removal, absorbed assertions); commented-out placeholder modules deleted | — |
| B2 | Test-First | 🟢 | 🟢 | The revived tests ARE the coverage proving the Ord fix works — first-ever call sites of calculate_flow/add_goal | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | No failing runs this task — single launch green first try; the pre-existing failure evidence is the disablement itself plus the never-compiled Movable import documented in IMPLEMENTED | — |
| B4 | Proper Fix Only | 🟢 | 🟢 | Ord motivated by the crate's own public bound (priority-queue ordering for the future Dijkstra), not test convenience; square tests retired rather than semantically rewritten into hex duplicates; no bound relaxed to dodge the problem | — |
| B5 | Fix Verification | 🟢 | 🟢 | Log `-0052` exit 0: integration 189→194 with all 5 revived tests named in the log; unit 5/5, flowfield_test 3/3, doc 40/40; full-crate suite is the Ord regression net | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | "Why hexagonal-only" recorded in the file header; Ord impl doc names the dead-API condition it fixes; full 21-test disposition accounting in IMPLEMENTED; stale cross-references in 4 files updated | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | 480→178 lines; dead placeholder comments deleted; no backup files; rewrite script boundary-asserted (21 markers, known first line, stub instantiations present) | — |
| **Total** | | 🔴 | 🟢 | 1 finding resolved in-loop | 15/15 |

# Re-enable or retire the disabled flowfield integration tests in tiles_tools

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** crate
- **unit:** module/helper/tiles_tools
- **verified_by:** null
- **verification_date:** null
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

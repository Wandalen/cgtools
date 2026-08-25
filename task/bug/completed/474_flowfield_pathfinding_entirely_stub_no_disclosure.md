# BUG-474: `tiles_tools::flowfield`'s entire pathfinding API is stub code with no disclosure -- every query silently returns a fixed, input-independent answer

- **Severity:** High (no crash, no panic -- but every public entry point silently returns a
  fixed, input-independent answer, indistinguishable at the type level from a working
  implementation; a caller has no signal short of reading source that this module does
  nothing)
- **state:** Completed
- **Affects:** Any consumer of `tiles_tools::flowfield` expecting real multi-unit pathfinding
  (`FlowField::flow_calculate`/`flow_apply`/`flow_directions_batch_get`/`group_flow_calculate`,
  `MultiGoalFlowField::goal_add`/`optimal_direction_get`, `DynamicFlowField::incremental_update`,
  `FlowFieldAnalyzer::flow_analyze`/`flow_optimize`, `IntegrationField::cost_get`/`cost_set`/
  `in_bounds`). Confirmed zero call sites outside this crate's own tests via
  `grep -rn` across the workspace, so no downstream crate is silently broken by the disclosure
  added here -- but any future consumer would have been.
- **Component:** module/helper/tiles_tools (`src/flowfield.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** None known.

## Symptom

```rust
// pre-fix -- src/flowfield.rs
pub fn flow_direction_get< C >( &self, _coord : &C ) -> Option< FlowDirection >
where /* ... */
{
  // Simplified stub implementation - would access Grid2D
  None
}
```

Every method on `IntegrationField`/`FlowField`/`MultiGoalFlowField`/`DynamicFlowField`/
`FlowFieldAnalyzer` either does nothing (`cost_set`, `flow_optimize`, `integration_field_calculate`,
`flow_directions_generate`, `combined_field_recalculate`) or returns a fixed value regardless of
input (`cost_get` always `0`, `in_bounds` always `true`, `flow_direction_get`/
`optimal_direction_get` always `None`, `flow_analyze` always the same zeroed
`FlowFieldAnalysis`). `flow_apply`/`flow_directions_batch_get`/`group_flow_calculate` cascade
from `flow_direction_get`'s stub and are therefore themselves always-`None`/all-`None`. None of
this was disclosed on the public items themselves -- the only hint was a buried aside in the
module doc's *example* section ("In a complete implementation, FlowField would: ...").

## Impact

**Who is affected:** Any caller of this module's public API expecting real pathfinding output.
Because every stub method type-checks and returns a plausible-looking value (`Option`, `Vec`,
a populated-looking struct) rather than panicking or returning an error, a caller has no
runtime signal that nothing was computed -- `flow_calculate(&goal, is_passable, get_cost)`
silently ignores all three arguments.

**What breaks:** Any RTS-style multi-unit movement built on this module would silently never
move any unit (every `flow_apply` call returns `None`, meaning "already at goal or blocked" to
a naive caller, when in fact no computation ever happened).

**Consumer audit:** Zero call sites outside `tiles_tools` itself
(`grep -rln 'flowfield::' --include="*.rs" .` from the repo root, excluding this crate) --
confirmed via direct audit, not assumed. The existing consumers are entirely this crate's own
`tests/flowfield_test.rs` and `tests/integration/flowfield_tests.rs`.

**Magnitude:** 13 public items across the module; see Fix Location.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading every `src/` file end to end. `flowfield.rs`'s module doc
comment's own example section contained the tell: "In a complete implementation, FlowField
would: 1. Calculate integration field ... 2. Generate flow directions ... 3. Provide movement
guidance" -- phrased as future work, while every public method it describes already exists and
returns a value.

## Minimum Reproducible Example

```rust
// module/helper/tiles_tools/tests/integration/flowfield_tests.rs
let mut flow_field = FlowField::<Axial, Pointy>::new(10, 10);
let goal = HexCoord::<Axial, Pointy>::new(4, -2);
flow_field.flow_calculate(&goal, |_| true, |_| 1);
assert_eq!(flow_field.flow_direction_get(&goal), None);
// pre-fix: passes silently, with zero indication `flow_calculate` computed nothing at all
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(integration_tests) and test(flowfield_tests)'
```

## Root Cause

`flowfield.rs` was authored as a scaffold: every struct/method signature matching the module's
intended final API, but every body left as a stub ("Simplified stub implementation for
testing", "In a full implementation, this would use Dijkstra's algorithm..."). The scaffold
was never wired to an actual `Grid2D`-backed cost/direction store -- `IntegrationField` and
`FlowField` don't even hold a `Grid2D` field internally, despite their trait bounds
(`Grid2D<System, Orientation, u32>: Index<C, Output = u32>`) implying they should. The gap
between "API scaffold exists" and "API is disclosed as non-functional" was never closed.

## Why Not Caught

`tests/integration/flowfield_tests.rs`'s five tests (`test_hex_grid_with_water_obstacles`,
`test_batch_flow_direction_queries`, `test_group_movement_flow_application`,
`test_multi_goal_capture_points`, `test_flow_field_ecs_integration`) each call into the stub
API but assert only that the call completes, and that output length/count matches input
length/count (`assert_eq!(directions.len(), test_coordinates.len())`) -- never that the
*values* returned are meaningful. `test_multi_goal_capture_points` went as far as computing
`optimal_direction_get`'s result into a variable prefixed `_direction`, explicitly discarding
it. One test's own trailing comment, `// Units should path around water`, was aspirational
prose, never an assertion. A fully non-functional module can pass every test in its own suite
indefinitely under this pattern.

## Fix Location

`module/helper/tiles_tools/src/flowfield.rs`:

- Judgment call (see Prevention/Pitfall): implementing a genuine Dijkstra-based flow-field
  algorithm is a substantial, novel feature addition -- `IntegrationField`/`FlowField` would
  need an actual `Grid2D` field added to their structs (a structural change, not a bug fix),
  plus a real priority-queue-based cost propagation and steepest-descent direction pass, each
  requiring untested design decisions (exact cost semantics per coordinate system, grid-bounds
  handling) nobody has specified. That is out of scope for a bug-fix sweep and was not
  attempted here.
- Applied instead: **disclosure**, not behavior change. Every affected public item
  (`IntegrationField::cost_get`/`cost_set`/`in_bounds`; `FlowField::flow_calculate`/
  `flow_direction_get`/`flow_directions_batch_get`/`flow_apply`/`group_flow_calculate`;
  `MultiGoalFlowField::goal_add`/`optimal_direction_get`; `DynamicFlowField::incremental_update`;
  `FlowFieldAnalyzer::flow_analyze`/`flow_optimize`) is now marked
  `#[deprecated(note = "...")]` with a note stating exactly what it always returns and why,
  referencing this bug. Every internal call from one now-deprecated item to another is wrapped
  in `#[expect(deprecated, reason = "...")]` so the crate's own compilation stays warning-free.
  The module's top-level doc comment gained a prominent "Stub Status (BUG-474)" section stating
  the same facts in prose, ahead of the algorithm description.

## Prevention

`tests/integration/flowfield_tests.rs`'s five existing tests were strengthened (not replaced)
with explicit assertions on the stub's current all-`None`/always-fixed output, so the module
stops silently passing tests that give it credit for doing nothing -- any future real
implementation must consciously update these assertions, converting silent-pass into a genuine
signal.

## Pitfall

An API scaffold with type-correct, plausible-looking stub returns (`Option`, `Vec`, a populated
struct) is strictly more dangerous than one that panics or returns `Err` -- it composes
silently into a caller's own logic with no error to propagate or catch. A module doc comment's
example section hinting "In a complete implementation, X would ..." is not equivalent to a
disclosure on the actual public items a caller's IDE/rustdoc will show them -- the hint has to
be load-bearing (a compiler warning, e.g. `#[deprecated]`) to actually reach a caller who never
reads the module's prose.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, reading `src/flowfield.rs` end to end. |
| 2026-08-20 | fixed | Disclosure-only fix: `#[deprecated]` on all 13 affected public items, `#[expect(deprecated)]` at 4 internal cascading call sites, module doc "Stub Status" section, strengthened assertions in the 5 pre-existing integration tests. No runtime behavior changed. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Disclosure completeness | — | 🟢 | Adversarial pass: grepped every `pub fn`/`pub struct` in `flowfield.rs` against the `#[deprecated]` list -- confirmed all 13 non-functional public items covered; `new()`/`width()`/`height()`/`mark_dirty()`/`is_dirty()` correctly left undeprecated since they perform real, correct work (storing/returning constructor arguments, set insertion/lookup). | — |
| D2 | Compilation cleanliness | — | 🟢 | `cargo check -p tiles_tools --all-features --tests` and `cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings` both clean (0 warnings) after adding `#![expect(deprecated)]` to the two test files that call the now-deprecated API (`tests/integration/flowfield_tests.rs`). | — |
| D3 | Test strengthening validity | — | 🟢 | Confirmed each of the 5 strengthened assertions (`assert_eq!`/`assert!` on `None`/all-`None`) actually exercises live stub output, not a tautology -- each reads a real return value from a real call, not a hardcoded literal. | — |

**Reproduced:** N/A (documentation/disclosure fix, no runtime behavior change to reproduce a
failure/pass delta for) -- see Prevention for how the fix's correctness was instead verified:
the strengthened tests assert the stub's exact current output, and `cargo nextest run -p
tiles_tools --all-features` confirms all pass. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/flowfield.rs` | Added a "Stub Status (BUG-474)" module doc section; marked 13 public items `#[deprecated(note = "...")]`; wrapped 4 internal cascading calls in `#[expect(deprecated, reason = "...")]`. No behavior changed. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/integration/flowfield_tests.rs` | Added `#![expect(deprecated)]` module attribute and a BUG-474 doc section; strengthened all 5 existing tests with explicit assertions on the stub's current all-`None`/always-fixed output instead of only length/count checks. |

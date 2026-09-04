# BUG-343: `MovementSystem::movement_calculate` rejects reachable targets because its raw-distance pre-check uses a different metric than the weighted-cost check that actually gates `movable.range`

- **Severity:** Medium (no crash or data corruption, but a documented, caller-facing contract --
  "pass your own `cost` policy and it decides reachability" -- silently fails whenever that
  policy's real weighted cost diverges from raw grid distance, which is exactly the case any
  non-uniform-cost terrain policy produces)
- **state:** Verified
- **Affects:** `tiles_tools::ecs::systems::MovementSystem::{movement_process, movement_calculate}`
  (`src/ecs/systems.rs`) -- any call whose caller-supplied `cost` closure makes a target's true
  weighted path cost cheaper than `current.distance(target)` (e.g. any `cost` returning values
  below 1 per step, or any policy where some steps are free/discounted) while that raw distance
  exceeds `movable.range`
- **Component:** `module/helper/tiles_tools` (`src/ecs/systems.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **Fix Task:** [378](../../verifying/378_register_tiles_tools_movement_rangecheck_costmetric_fix_closes_bug343.md)

## Symptom

```bash
# Actual (pre-fix): entity at (0,0), Movable::new(2) (range 2), target (10,0) (raw Manhattan
# distance 10), caller's cost policy returns 0 for every step (a fully valid "free road" terrain
# policy per the `Fc: FnMut(&C) -> u32` signature -- real weighted path cost is 0, well within range).
$ cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_movement_uses_cost_not_raw_distance_for_range_check
thread 'integration::ecs_tests::test_movement_uses_cost_not_raw_distance_for_range_check' panicked at module/helper/tiles_tools/tests/integration/ecs_tests.rs:661:3:
a target with raw distance > range but real weighted cost <= range must succeed, got [OutOfRange { requested_distance: 10, maximum_range: 2 }]
test result: FAILED. 1 failed

# Expected (fixed): reachability is decided solely by the pathfinder's own weighted cost.
$ cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_movement_uses_cost_not_raw_distance_for_range_check
test integration::ecs_tests::test_movement_uses_cost_not_raw_distance_for_range_check ... ok
test result: ok. 1 passed
```

## Impact

**Who is affected:** any caller of `MovementSystem::movement_process`/`movement_calculate`
whose `cost` policy is not uniformly `>= 1` per step -- e.g. a terrain type that is free or
discounted (roads, rivers with a current, a "dash" ability), or any policy the caller
intentionally scales down relative to raw grid distance. The function's own doc comment states
`is_accessible`/`cost` are "the caller's obstacle and terrain policies... the caller owns both,"
implying the caller's `cost` is authoritative for reachability -- this bug breaks that contract
for exactly the policies that make `cost` diverge from `distance`.

**What breaks:** a target that the caller's own cost policy says is easily reachable (real
weighted path cost well within `movable.range`) is rejected with `MovementResult::OutOfRange`
before the pathfinder (`astar`) is ever invoked -- no path is computed, no `Success` is possible,
regardless of how cheap the actual route is. The failure is silent from the caller's perspective:
`OutOfRange` is a normal, documented variant, not a panic or error, so nothing signals that the
rejection used the wrong metric.

**Entity Scope:** `None` -- source-level pathfinding-gate defect, not entity directory instances.

## How Discovered

During a systematic bug-hunt pass across `tiles_tools`'s ECS module, comparing every place
`movable.range` is consulted inside `movement_calculate` showed two different metrics gating the
same budget: a pre-pathfind check comparing `current.distance(target)` (raw grid distance) against
`range`, and a post-pathfind check comparing `astar`'s returned path `cost` (weighted, caller
policy-dependent) against the same `range`. `astar` itself (`src/pathfind.rs:176-205`) never
receives `range` at all -- it is not passed as a search budget -- so the pre-pathfind check is the
only place raw distance is used, and it runs before the authoritative cost-based check ever gets
a chance to evaluate the target. Direct execution of `movement_calculate` with a `cost` policy
cheaper than raw distance confirmed the pre-check rejects targets the cost-based check would have
accepted.

## Minimum Reproducible Example

**Verify Command** (run from repo root; ≤3 lines):
```bash
cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests::test_movement_uses_cost_not_raw_distance_for_range_check
```
**What:** violates the function's own documented contract that the caller's `cost` policy (not
raw grid distance) decides reachability against `movable.range`.

**Expected** (fixed): 1 passed -- entity moves to `(10, 0)`, `MovementResult::Success`.

**Actual** (pre-fix, directly observed via temporary revert-and-rerun of this fix): 1 failed --
`a target with raw distance > range but real weighted cost <= range must succeed, got
[OutOfRange { requested_distance: 10, maximum_range: 2 }]`.

**Known MRE limitation (check 203/205):** none -- `MovementSystem::movement_calculate` is pure,
dependency-free ECS pathfinding logic operating on this crate's own `astar`/`Distance` types;
reproducing it requires the `tiles_tools` crate itself (no `/tmp`-based synthetic fixture can
exercise it without vendoring the crate), so the Verify Command runs as an ordinary
`cargo test -p tiles_tools` against the real crate directly, consistent with this repo's existing
precedent for in-workspace-only ECS logic bugs (e.g. BUG-132).

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `movement_calculate` rejects some targets using raw grid distance instead of weighted path cost | ✅ Verified | `src/ecs/systems.rs:98-107` (pre-fix) computed `current.distance(target)` and compared it directly to `movable.range`, before any pathfinding ran | E1, E2 |
| H2 | The pre-check and the post-pathfind check use different, potentially-divergent metrics for the same `range` budget | ✅ Root Cause | Pre-check used `Distance::distance` (raw grid distance); post-check (`src/ecs/systems.rs:116`, pre-fix line numbering) used `astar`'s returned weighted `cost` -- a caller-supplied `cost` policy of `\|_\| 0` makes the two metrics diverge maximally (10 vs. 0) | E1, E2, E3 |
| H3 | `astar` does not receive `range` as a search budget, so the pre-check is not acting as a legitimate optimization over the pathfinder's own range-awareness | ✅ Verified | `src/pathfind.rs:176-188` -- `astar`'s signature takes only `start, goal, is_accessible, cost`, no range/budget parameter | E4 |
| H4 | No existing test exercises a `cost` policy that diverges from raw distance while raw distance exceeds `range` | ✅ Verified | `test_movement_system_uses_caller_policies` (`tests/integration/ecs_tests.rs`, pre-existing) uses targets with raw distance <= range in every case -- the two metrics never disagree in that test | E5 |
| H5 | Removing the pre-check does not change `MovementResult` for any existing test, since none of them ever reach the `OutOfRange` branch | ✅ Verified | Full scoped `ecs_tests` module re-run post-fix: 24/24 pass, including `test_movement_system_uses_caller_policies` unchanged | E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/ecs/systems.rs:98-107` (pre-fix) | `let distance = current.distance(target); if distance > movable.range { return MovementResult::OutOfRange {...} }` runs unconditionally before pathfinding | H1 ✅, H2 ✅ |
| E2 | Terminal output (this report, MRE section) | Direct test execution with `cost = \|_\| 0`, raw distance 10, range 2 confirms `OutOfRange { requested_distance: 10, maximum_range: 2 }` is returned instead of `Success` | H1 ✅, H2 ✅ |
| E3 | `src/ecs/systems.rs:116` (pre-fix; now `:124` post-fix) | `if cost <= movable.range { Success } else { PathTooLong }` -- the *only* other place `range` is consulted, using the pathfinder's returned weighted `cost`, not raw distance | H2 ✅ |
| E4 | `src/pathfind.rs:176-188` | `pub fn astar<C, Fa, Fc>(start: &C, goal: &C, mut is_accessible: Fa, mut cost: Fc) -> Option<(Vec<C>, u32)>` -- no `range`/budget parameter in the signature | H3 ✅ |
| E5 | `tests/integration/ecs_tests.rs`, `test_movement_system_uses_caller_policies` (pre-existing, unmodified) | All 3 sub-cases use targets with raw Manhattan distance <= `movable.range` (2, 4, 4 against range 5) | H4 ✅ |
| E6 | Terminal output (`cargo test -p tiles_tools --all-features --test integration_tests -- integration::ecs_tests`, post-fix) | `test result: ok. 24 passed; 0 failed` | H5 ✅ |

## Root Cause

```
movement_calculate( current, target, movable, is_accessible, cost )
{
  let distance = current.distance( target );      // <- metric A: raw grid distance
  if distance > movable.range { return OutOfRange }  // <- gates on metric A

  let path_result = astar( current, target, is_accessible, cost );
  match path_result {
    Some( ( path, cost ) ) =>
      if cost <= movable.range { Success } else { PathTooLong }
                //  ^-- metric B: weighted path cost, from the CALLER's own `cost` closure
    None => NoPathFound,
  }
}

  Two different metrics (A: Distance::distance, B: caller-defined weighted cost) both compared
  against the same `movable.range` budget. `astar` never receives `range`, so metric A is not a
  legitimate pre-filter derived from the pathfinder's own logic -- it is an independent,
  disagreeing gate that runs first and can reject what metric B would have accepted.
```
The two metrics coincide only when the caller's `cost` policy is uniformly `>= 1` per step (so
weighted cost >= raw distance always) -- any cheaper-than-uniform policy breaks the coincidence,
and the raw-distance pre-check (which runs first) wins the disagreement by returning early.

## Why Not Caught

The only pre-existing test exercising `movement_calculate`'s range behavior,
`test_movement_system_uses_caller_policies`, uses cost policies of `\|_\| 1` and `\|_\| 4` --
both uniformly `>= 1` per step -- against targets whose raw distance never exceeds
`movable.range`. Under a uniform `>= 1` cost policy, weighted cost is always `>= ` raw distance,
so the raw-distance pre-check can only ever be *more* permissive than or equal to the cost-based
check, never less -- the two metrics never disagreed in any scenario that test covered, so the
bug had no historical trigger.

## Fix Location

**`src/ecs/systems.rs:97-118`** (before/after):

```rust
// Before:
// Check if target is within movement range
let distance = current.distance( target );
if distance > movable.range
{
  return MovementResult::OutOfRange
  {
    requested_distance : distance,
    maximum_range : movable.range,
  };
}

// Use pathfinding to find valid path
let path_result = astar( current, target, is_accessible, cost );

// After:
// Fix(BUG-343): removed the raw-grid-distance pre-check that used to run
// before pathfinding -- it rejected purely on `current.distance(target)`
// exceeding `movable.range`, a completely different metric from the
// weighted path `cost` this function actually gates reachability on
// below (`cost <= movable.range`). [...full comment at src/ecs/systems.rs:98-116...]
// Use pathfinding to find valid path
let path_result = astar( current, target, is_accessible, cost );
```
Source comment (`Fix(BUG-343)`/`Root cause`/`Pitfall`) added in place of the removed pre-check,
immediately above the unchanged `astar` call. `MovementResult::OutOfRange` remains defined (public
enum variant, part of the crate's API surface) but is no longer constructed anywhere in this
crate -- reachability is now decided solely by the existing post-pathfind `cost <= movable.range`
check, which was already correct and unchanged.

## Prevention

Detection command for a range/budget value being compared against two different metrics within
the same function (heuristic-only, requires human review of each match):
```bash
grep -n "movable.range\|\.range\b" src/ecs/systems.rs
```
Run against the fixed file, this finds exactly one remaining comparison (`cost <= movable.range`,
the correct one) plus the parameter/field declarations -- confirming the divergent second gate is
gone. This is a starting point for review, not a general-purpose detector: it is specific to this
function's own field name and would need adaptation for any other `range`-gated pathfinding call
in this crate.

**Pitfall:** when a budget value (`range`) is checked against a pathfinder's result *and* against
some pre-pathfind heuristic, confirm both checks use the same metric the pathfinder itself will
use to compute cost -- a `Distance`-based heuristic (grid distance) and a caller-supplied `cost`
closure (weighted, potentially non-uniform) are not interchangeable, and a heuristic pre-filter
that is cheaper to compute is only a valid optimization when it is *provably never stricter* than
the authoritative check it precedes.

## Generalized Version

**Broken assumption:** a cheap-to-compute heuristic (`Distance::distance`) pre-filtering a target
before an expensive pathfind is a safe optimization, because it "obviously" agrees with the
pathfinder's own eventual weighted-cost result.

Fails whenever:
1. A pre-pathfind check compares one metric (e.g. raw grid distance) against a budget, AND
2. The pathfinder's own authoritative check inside/after the search compares a *different* metric
   (e.g. caller-supplied weighted cost) against the same budget, AND
3. The caller's cost policy can make the second metric diverge from the first (e.g. any per-step
   cost below 1, or non-uniform terrain costs)

**Detection invariant:**
```
for every function gating a budget B with two checks C1 (pre-computation) and C2 (post-computation):
  C1 and C2 must use the same metric, OR
  C1 must be provably never stricter than C2 for any input the caller can supply
```
Single confirmed instance in this workspace (grep swept `tiles_tools/src/**/*.rs` for other
`.range`/budget comparisons paired with a separate `Distance::distance` pre-check in the same
function; only `movement_calculate` matched this shape). Not a duplicate of any prior bug in this
repo's `task/bug/` history (dedup search:
`grep -rli "movement_calculate\|OutOfRange\|raw.distance" task/bug/` found two unrelated hits --
`task/bug/completed/305_tactical_rpg_false_los_aoe_attack_claim.md` (a false line-of-sight claim
in an unrelated demo) and `task/bug/completed/159_drawbuffers_raw_oob_index_panic.md` (an
unrelated WebGL buffer index panic) -- neither addresses this function or this metric-mismatch
pattern).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found during a systematic bug-hunt pass over `tiles_tools`'s ECS module; root-caused by comparing every `movable.range` consultation site inside `movement_calculate` |
| 2026-08-18 | fix_applied | Removed the raw-distance pre-check at `src/ecs/systems.rs:97-107` (pre-fix); reachability now decided solely by the existing `cost <= movable.range` post-pathfind check. Reproducer test confirmed FAIL pre-fix (`OutOfRange` returned) and PASS post-fix (`Success` returned); full `ecs_tests` module (24 tests) and scoped clippy (`cargo clippy -p tiles_tools --all-targets --all-features -- -D warnings`) both clean |
| 2026-08-18 | verified | VERIFY Gate (Tier 2 Dual-Role Self-Check): fixed MRE portability (check 203/205 — hardcoded per-user path) and Evidence Table state-symbol gaps (check 304); `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`) 272/272 passed, including `test_movement_uses_cost_not_raw_distance_for_range_check`. See `## Verification Record`. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present, correct order, no `_Investigation ongoing._` placeholders; header carries Severity/state/Affects/Component/Filed/repo_identity/filed_by. | — |
| D2 | MRE Validity & Reproducibility | 🔴 | 🟢 | Confirming pass accepted the Verify Command at face value; adversarial pass found it hardcoded `cd /home/user1/pro/lib/yrd_gamedev/cgtools` — an absolute, per-user path violating check 205 ("free of project-specific paths"), inconsistent with sibling BUG-341/346 (same filing batch) which use repo-relative/no-`cd` forms, and missing the `**Known MRE limitation**` disclosure this repo's own precedent (BUG-132) uses for pure in-workspace ECS logic bugs that cannot be reproduced via a synthetic `/tmp` fixture. | Removed the hardcoded path; added the `**Known MRE limitation (check 203/205):**` disclosure matching BUG-132's precedent; directly executed `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`) — 272/272 passed, including `test_movement_uses_cost_not_raw_distance_for_range_check`. |
| D3 | Cross-Reference Integrity | 🔴 | 🟢 | ≥3 Hypothesis rows (5), 1 `✅ Root Cause` (H2), bidirectional H↔E links, and `Refs:` backreferences (`Fix(BUG-343)` at `src/ecs/systems.rs:98`, `test_kind: bug_reproducer(BUG-343)` at `tests/integration/ecs_tests.rs:618`) all confirmed; but Evidence Table's Hypothesis column cited bare H-IDs (`H1, H2`) without the state symbols check 304 requires and BUG-132's precedent uses (`H1 ✅, H2 ❌`). | Backfilled each Evidence Table row's Hypothesis cell with its Hypothesis Table state symbol (all ✅ here — no disproven hypotheses in this bug). |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root Cause prose traceable to the sole `✅ Root Cause` row (H2); Fix Location (`src/ecs/systems.rs:97-118`) confirmed accurate against live source; Generalized Version states broken assumption + detection invariant. Adversarially checked whether removing the pre-check entirely (vs. reimplementing it in the cost metric) could regress anything — full 272-test suite post-fix shows no regression. | — |
| D5 | Execution Scope | 🟢 | 🟢 | `repo_identity: self`; Fix Location resolves inside this repo/crate; no cross-boundary or foreign work. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | `**Component:**` (`module/helper/tiles_tools`) matches the crate `## Fix Location`'s `src/ecs/systems.rs:97-118` resolves to. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix lives directly in `tiles_tools`'s own `MovementSystem::movement_calculate` — the crate that structurally owns pathfinding/movement logic — not a pushed-up aggregator. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface added; `tiles_tools`'s existing pathfinding/ECS-movement responsibility is unchanged, only its correctness. | — |

**Reproduced:** YES — `cargo nextest run -p tiles_tools --all-features` (detached via `longrun`, exit 0): 272/272 tests passed, including `test_movement_uses_cost_not_raw_distance_for_range_check`; independently confirmed the live source at `src/ecs/systems.rs:98-118` matches the documented fix (raw-distance pre-check removed, reachability decided solely by `cost <= movable.range`), 2026-08-18.

## Refs: src/

- `src/ecs/systems.rs` — removed the raw-grid-distance pre-check in `MovementSystem::movement_calculate`; reachability is now decided solely by the pathfinder's returned weighted cost compared against `movable.range`

## Refs: tests/

- `tests/integration/ecs_tests.rs` — new reproducer test `test_movement_uses_cost_not_raw_distance_for_range_check`: a target with raw distance 10 > range 2 but a `cost` policy of `\|_\| 0` (real weighted cost 0) must succeed

# BUG-346: `bresenham_line_trace`'s greedy directional walk makes Bresenham `line_of_sight` asymmetric around walls

- **Severity:** High
- **state:** Verified
- **Affects:** Every `FieldOfView::with_algorithm(FOVAlgorithm::Bresenham)` query (`line_of_sight`,
  and any `fov_calculate`/`visibility_calculate` call that dispatches to `bresenham_fov_calculate`)
  between two coordinates whose greedy neighbor-walk paths diverge around an obstacle cluster
- **Component:** `module/helper/tiles_tools` (`src/field_of_view.rs`, `bresenham_line_trace`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/ (self)
- **verification_date:** 2026-08-18
- **Fix Task:** [381](../../verifying/381_register_tiles_tools_bresenham_line_trace_symmetry_fix_closes_bug346.md)

## Symptom

`FieldOfView::line_of_sight(&self, from, to, blocks_sight)` must be symmetric — whether a wall
blocks sight between two points cannot depend on which point the caller labels `from` and which
`to`. For the `Bresenham` algorithm it is not:

```
# Square/EightConnected grid, walls at (2,1), (2,2), (3,2):
# A = (0,0), B = (5,3)

fov.line_of_sight(&A, &B, blocks)  ->  true   # wrong: a wall cluster sits between them
fov.line_of_sight(&B, &A, blocks)  ->  false  # correct-looking, but now direction-dependent
```

Same two points, same wall set, opposite answers depending on call direction.

## Impact

**Who is affected:** any consumer of `FieldOfView::with_algorithm(FOVAlgorithm::Bresenham)` —
either directly via `line_of_sight`, or indirectly via `fov_calculate`/`visibility_calculate`,
whose `bresenham_fov_calculate` always calls `bresenham_line_check(viewer, target, ...)` with the
viewer fixed as `from` for every target in range.

**What breaks:** a unit can see an enemy that, symmetrically, should not be able to see it back
(or vice versa) whenever a wall cluster sits such that the two directions' greedy walks diverge
around it. This is a silent logic error — no panic, no error, just a wrong boolean that only
shows up as inconsistent gameplay behavior (stealth/AI-detection asymmetry, ranged-attack
line-of-sight exploits).

**Magnitude:** 1 function (`bresenham_line_trace`), the sole line-tracing primitive for the
`Bresenham` FOV algorithm; every caller of that algorithm is affected. The other 3 algorithms
(`Shadowcasting`, `RayCasting`, `FloodFill`) do not call this function and are not affected.

**Entity Scope:** `None` — a code-level algorithm defect, not entity directory instances.

## How Discovered

```bash
$ cargo test -p tiles_tools --all-features --test integration_tests \
    integration::field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall -- --exact

thread 'integration::field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall'
panicked at module/helper/tiles_tools/tests/./integration/field_of_view_tests.rs:689:3:
assertion `left == right` failed: line_of_sight must not depend on call direction: A->B = true, B->A = false
  left: true
 right: false
test result: FAILED. 200 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

A prior investigation pass identified the asymmetry by direct reading of `bresenham_line_trace`'s
greedy-walk loop (§ Hypothesis Table below); this report re-confirms it with the permanent
reproducer test above, run against the pre-fix source.

## Minimum Reproducible Example

**Verify Command** (run from repo root; ≤3 lines):
```bash
cargo test -p tiles_tools --all-features --test integration_tests \
  integration::field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall -- --exact
```
**What:** `line_of_sight(A, B, blocks)` must equal `line_of_sight(B, A, blocks)` for the same
wall set — the algorithm's answer must not depend on which endpoint is labeled `from`.

**Expected** (fixed): test passes — `test ... test_bresenham_line_of_sight_is_symmetric_around_wall ... ok`.

**Actual** (pre-fix, directly confirmed by running the same test against the current, unfixed
source before applying the fix below):
```
thread 'integration::field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall'
panicked at module/helper/tiles_tools/tests/./integration/field_of_view_tests.rs:689:3:
assertion `left == right` failed: line_of_sight must not depend on call direction: A->B = true, B->A = false
  left: true
 right: false
test result: FAILED. 200 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `bresenham_line_trace` (field_of_view.rs:762-810, pre-fix) is a greedy "step to whichever neighbor is closest to the fixed target" walk seeded at `from` — this walk is not required to be path-reversible, so tracing `from->to` can visit different intermediate cells than `to->from` | ✅ Root Cause | Direct read: `current = from.clone()` (767), loop compares only `neighbor.distance(to)` (780) with no symmetry-restoring step anywhere in the walk | E1 |
| H2 | `bresenham_fov_calculate` (field_of_view.rs:679-730) always calls `bresenham_line_check(viewer, target, blocks_sight)` with the *viewer* fixed as `from` for every target in range, so `line_of_sight(A,B)` and `line_of_sight(B,A)` each drive a *differently-seeded* call into the asymmetric walk of H1 | ✅ Verified | Direct read: line 715, `Self::bresenham_line_check(viewer, &target, blocks_sight)` — `viewer` is always the first argument regardless of which of A/B the caller passed as `from` to the outer `line_of_sight` | E2 |
| H3 | `bresenham_line_check` (field_of_view.rs:733-756) forwards `from`/`to` straight into `bresenham_line_trace` with no canonicalization of its own, so nothing upstream of H1's walk corrects the asymmetry | ✅ Verified | Direct read: line 739, `Self::bresenham_line_trace(from, to)` — no reordering, no symmetry check | E3 |
| H4 | For the concrete repro (walls at (2,1),(2,2),(3,2), A=(0,0), B=(5,3) on a Square/EightConnected grid), the greedy walk A->B slips around the wall cluster on one side while the greedy walk B->A steps through a blocked cell, because the two walks independently minimize distance to opposite fixed targets and are not mirror images of each other | ✅ Verified | Terminal evidence (E4): the reproducer test's own assertion output shows `A->B = true, B->A = false` for exactly this configuration, matching the mechanism H1-H3 predict | E4 |
| H5 | The other 3 `FOVAlgorithm` variants (`Shadowcasting`, `RayCasting`, `FloodFill`) do not call `bresenham_line_trace` and so do not share this defect | ✅ Verified | `grep -n "bresenham_line_trace" src/field_of_view.rs` shows the only caller is `bresenham_line_check`, itself only called from `bresenham_fov_calculate`, itself only reached via the `FOVAlgorithm::Bresenham` match arm (line 312) | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tiles_tools/src/field_of_view.rs:762-810` (`bresenham_line_trace`, pre-fix, direct read via `git show HEAD:...`) | Walk starts at `current = from.clone()` and greedily steps to whichever neighbor minimizes `neighbor.distance(to)`, with no step that restores direction-independence | H1 |
| E2 | `module/helper/tiles_tools/src/field_of_view.rs:715` (`bresenham_fov_calculate`, direct read) | `Self::bresenham_line_check(viewer, &target, blocks_sight)` — viewer is always the fixed first argument | H2 |
| E3 | `module/helper/tiles_tools/src/field_of_view.rs:739` (`bresenham_line_check`, direct read) | `Self::bresenham_line_trace(from, to)` — direct pass-through, no canonicalization | H3 |
| E4 | Terminal output (this report, MRE section; also captured in `-0001_longrun.log:329-333`, pre-fix combined test run) | Reproducer assertion fails with the exact predicted values: `A->B = true, B->A = false` for the concrete wall configuration | H4 |
| E5 | `grep -n "bresenham_line_trace\|FOVAlgorithm::Bresenham" src/field_of_view.rs` (direct read) | Only call chain into `bresenham_line_trace` is `Bresenham` match arm (312) → `bresenham_fov_calculate` (679) → `bresenham_line_check` (733) → `bresenham_line_trace` (762) | H5 |

## Root Cause

```
line_of_sight(A, B)                            line_of_sight(B, A)
  -> fov_calculate(viewer=A, ...)                 -> fov_calculate(viewer=B, ...)
  -> bresenham_fov_calculate(viewer=A, ...)        -> bresenham_fov_calculate(viewer=B, ...)
  -> bresenham_line_check(A, B, blocks)            -> bresenham_line_check(B, A, blocks)
  -> bresenham_line_trace(A, B)                    -> bresenham_line_trace(B, A)
       current = A; greedily step toward               current = B; greedily step toward
       whichever neighbor minimizes                    whichever neighbor minimizes
       distance-to-B at each step                      distance-to-A at each step
       => visits path P_AB                             => visits path P_BA
          (specific cell sequence)                         (not guaranteed == reverse(P_AB))
```
`bresenham_line_trace` computes a path by greedily minimizing distance to a *fixed* target at
each step; this is a local, direction-dependent search, not a symmetric line algorithm. Nothing
in the call chain (`line_of_sight` → `fov_calculate` → `bresenham_fov_calculate` →
`bresenham_line_check` → `bresenham_line_trace`) canonicalizes the two endpoints before walking,
so `P_AB` and `P_BA` are two independently-computed paths that happen to share both endpoints but
are not required to be — and for the repro's wall cluster, are not — reverses of each other. One
path can route around the wall on one side while the other runs straight through a blocked cell.

## Why Not Caught

Every existing FOV test (`test_ray_casting_fov`, `test_shadowcasting_fov_square_grid`,
`test_hexagonal_shadowcasting_fov`, `test_penetrating_light`, `test_light_with_obstacles`, etc.)
calls `line_of_sight` or a FOV query in only one direction per scenario, or uses obstacle layouts
symmetric enough that directional asymmetry would not surface even if present. No existing test
called `line_of_sight` in both directions between the same pair of endpoints and asserted the
results match — the exact assertion this bug's own asymmetry violates.

## Fix Location

**`module/helper/tiles_tools/src/field_of_view.rs:762-810`** (`bresenham_line_trace`, pre-fix
full body; fix inserted at line 765, immediately after the opening brace):

```rust
// Before:
fn bresenham_line_trace<C>(from: &C, to: &C) -> Vec<C>
where
  C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
{
  let mut line_positions = Vec::new();
  let mut current = from.clone();
  line_positions.push(current.clone());

  // Simple neighbor-based line tracing
  while current != *to
  {
    let neighbors = current.neighbors();
    let mut best_neighbor = None;
    let mut best_distance = u32::MAX;

    // Find neighbor that gets us closest to the target
    for neighbor in neighbors
    {
      let distance_to_target = neighbor.distance(to);
      if distance_to_target < best_distance
      {
        best_distance = distance_to_target;
        best_neighbor = Some(neighbor);
      }
    }

    if let Some(next) = best_neighbor
    {
      if next == current
      {
        break; // Prevent infinite loop
      }
      current = next;
      line_positions.push(current.clone());

      // Prevent infinite loops in complex coordinate systems
      if line_positions.len() > 1000
      {
        break;
      }
    }
    else
    {
      break; // No valid path found
    }
  }

  line_positions
}

// After:
fn bresenham_line_trace<C>(from: &C, to: &C) -> Vec<C>
where
  C: Distance + Neighbors + Clone + std::hash::Hash + Eq,
{
  // Fix(BUG-346): canonicalize the walk direction (always walk from the
  // hash-smaller endpoint toward the hash-larger one, then reverse the
  // result if the caller's `from`/`to` were the other way round) so the
  // set of intermediate cells visited no longer depends on which endpoint
  // the caller labeled `from` vs `to`.
  // Root cause: the walk below is a greedy "step to whichever neighbor is
  // closest to the fixed target" search, which is not path-reversible --
  // tracing A->B and B->A could visit different intermediate cells, so one
  // direction could route around a wall the other ran straight through.
  // Pitfall: canonicalizing via a coordinate-specific ordering would need
  // an `Ord` bound that ripples out to every coordinate system usable with
  // `FieldOfView` (all 4 algorithms share this function's generic bounds);
  // comparing `Hash` output instead needs no new bound and is still
  // deterministic across both call directions, since the two hash values
  // being compared are identical regardless of which endpoint is passed
  // as `from` vs `to`.
  use std::hash::Hasher;
  let hash_of = | c : &C |
  {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    c.hash( &mut hasher );
    hasher.finish()
  };

  if from == to
  {
    return vec![ from.clone() ];
  }

  let swapped = hash_of( from ) > hash_of( to );
  let ( start, end ) = if swapped { ( to, from ) } else { ( from, to ) };

  let mut line_positions = Vec::new();
  let mut current = start.clone();
  line_positions.push(current.clone());

  // Simple neighbor-based line tracing
  while current != *end
  {
    let neighbors = current.neighbors();
    let mut best_neighbor = None;
    let mut best_distance = u32::MAX;

    // Find neighbor that gets us closest to the target
    for neighbor in neighbors
    {
      let distance_to_target = neighbor.distance(end);
      if distance_to_target < best_distance
      {
        best_distance = distance_to_target;
        best_neighbor = Some(neighbor);
      }
    }

    if let Some(next) = best_neighbor
    {
      if next == current
      {
        break; // Prevent infinite loop
      }
      current = next;
      line_positions.push(current.clone());

      // Prevent infinite loops in complex coordinate systems
      if line_positions.len() > 1000
      {
        break;
      }
    }
    else
    {
      break; // No valid path found
    }
  }

  if swapped
  {
    line_positions.reverse();
  }

  line_positions
}
```
This preserves `bresenham_line_check`'s existing contract unchanged: `line_positions[0]` is still
always `from` (post-fix, via the `reverse()` when `swapped`), so its `skip(1)`/break-on-`to` logic
needs no changes.

## Prevention

Detection command for this general pattern (a directional/greedy line-tracing function called
from both endpoint orders without a symmetry test):
```bash
grep -n "fn line_of_sight\|fn.*line_trace\|fn.*line_check" module/helper/tiles_tools/src/field_of_view.rs
```
This is a starting point for review, not a precise check — confirming actual symmetry requires a
property-style test (call both directions, assert equal), which is exactly what the new
reproducer test in `field_of_view_tests.rs` adds. Any future line-tracing/pathing primitive
callable in both directions between two points should carry the same bidirectional-equality test.

**Pitfall:** a "trace from A toward B" algorithm that greedily minimizes distance to a *fixed*
target at each step is not automatically symmetric — path-reversibility must be either proven
(e.g., a true integer Bresenham/DDA algorithm) or engineered in (canonicalize direction, walk
once, reverse if needed) — it must never be assumed just because the function takes `from`/`to`
in a seemingly-interchangeable order.

## Generalized Version

**Broken assumption:** "a function named/shaped like `trace_line(from, to)` that greedily walks
toward a fixed target is inherently symmetric in its two arguments, the way a true line/segment
between two points is."

Fails for any greedy or locally-optimizing search whenever:
1. The search minimizes distance/cost toward a target that is fixed for the whole walk, AND
2. More than one neighbor can tie or nearly tie for "closest to target" at some step, so the
   specific choice made depends on which endpoint is the fixed target, AND
3. An obstacle or cost feature sits such that the two possible walks (one per direction) diverge
   around it rather than being forced onto the same cells.

**Detection invariant:**
```
for every bidirectional query built on a directional greedy walk trace(from, to):
  trace_based_query(A, B, world) == trace_based_query(B, A, world)  for all A, B, world
```
Confirmed as a single instance in this crate (`bresenham_line_trace` is the only "trace toward a
fixed target" walk in `field_of_view.rs`; the other 3 FOV algorithms use symmetric
BFS/shadowcasting expansion outward from the viewer, not a directional trace — confirmed via H5/E5
above). Dedup search: `grep -rli "bresenham\|line_of_sight" task/bug/` finds only
`task/bug/completed/135_octant_shadows_cast_index_desync.md` (a different function,
octant-based shadowcasting index math, not `bresenham_line_trace`) and
`task/bug/completed/302_field_of_view_demo_readme_omits_flood_fill_algorithm.md` (a docs-only
bug) — neither targets this function or this defect.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Re-confirmed via direct source reading and a new permanent bidirectional reproducer test, following up a prior investigation pass's finding |
| 2026-08-18 | note | SUBMIT: state Draft -> Unverified; reproducer confirmed FAIL pre-fix and PASS post-fix, fix applied, full scoped suite (`cargo test -p tiles_tools --all-features`) green |
| 2026-08-18 | VERIFY Gate | Reproducer test `integration::field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall` confirmed passing (`cargo test -p tiles_tools --all-features --test integration_tests -- integration::field_of_view_tests::test_bresenham_line_of_sight_is_symmetric_around_wall --exact`: 1 passed; 0 failed) against current source; fix in `module/helper/tiles_tools/src/field_of_view.rs` confirmed present in `bresenham_line_trace` (line 762 onward) -- hash-based canonicalization (`hash_of`/`swapped` at lines 784-798) and result reversal (`line_positions.reverse()` at lines 843-846) match the report's claimed After block. state: Unverified -> Verified |
| 2026-08-18 | note | VERIFY Gate two-pass re-check (Tier 2 Dual-Role Self-Check, `governance/maav.rulebook.md`): adversarial pass found neither `src/field_of_view.rs` nor `tests/integration/field_of_view_tests.rs` carried the canonical FI027 backreference (only `Fix(BUG-346)`/`test_kind:` markers existed, matching the same gap BUG-298's own VERIFY Gate previously found and fixed in this repo); added `// BUG-346 task/bug/346_....md -- ...` backreference comment adjacent to each marker, re-verified via `grep -rn 'BUG-346' src/ tests/`; full `tiles_tools` scoped suite re-run (`cargo nextest run -p tiles_tools --all-features`: 272 passed / 0 failed, including this bug's reproducer); `## Verification Record` appended below |

## Refs: src/

- `module/helper/tiles_tools/src/field_of_view.rs` — `bresenham_line_trace` now canonicalizes walk direction via hash comparison before walking, reversing the result if the caller's `from`/`to` were the higher-hash endpoint first

## Refs: tests/

- `module/helper/tiles_tools/tests/integration/field_of_view_tests.rs` — new reproducer: asserts `line_of_sight(A, B)` equals `line_of_sight(B, A)` for a wall configuration that previously produced opposite answers

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE uses an in-repo `cargo test` command, not literal `/tmp/mreNNN/` paths -- deliberate, precedented local adaptation for a crate-internal algorithm defect (matches BUG-298/BUG-300's own already-verified shape in this repo), not an oversight | — |
| D3 | Cross-Reference Integrity | 🟡 | 🟢 | State was already flipped to Verified (with a History row) by a prior pass that left no `## Verification Record`, and neither `src/field_of_view.rs` nor `tests/integration/field_of_view_tests.rs` carried the canonical FI027 backreference (only `Fix(BUG-346)`/`test_kind:` markers existed) | Added canonical backreference comment adjacent to each existing marker in both files; re-verified via `grep -rn 'BUG-346' src/ tests/` |
| D4 | Root Cause Quality | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 0 open | 1/1 |

**Reproduced:** YES -- exit 0 (`test_bresenham_line_of_sight_is_symmetric_around_wall` ... ok), 2026-08-18. Full `tiles_tools` scoped suite (`cargo nextest run -p tiles_tools --all-features`, 272 passed / 0 failed) re-confirmed post-fix.

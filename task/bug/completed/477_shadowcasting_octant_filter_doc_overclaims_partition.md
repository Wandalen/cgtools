# BUG-477: `octant_shadows_cast`'s direction filter admits 5-of-6 (hex) or 5-of-8 (square) directions per call, but its doc claimed a clean non-overlapping partition

- **Severity:** Medium (no incorrect visibility output -- empirically verified byte-for-byte
  identical to an independent reference algorithm -- but the doc's claimed "systematic" per-octant
  partition was false, masking a real, silent performance cost from redundant recomputation)
- **state:** Completed
- **Affects:** Any consumer of `FieldOfView::with_algorithm(FOVAlgorithm::Shadowcasting)` reading
  the module's doc comments to reason about the algorithm's cost or correctness properties.
- **Component:** module/helper/tiles_tools (`src/field_of_view.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-20
- **Related Bugs:** None known.

## Symptom

```rust
// src/field_of_view.rs, octant_shadows_cast's per-ring direction filter
if ( i + total_directions - octant ) % total_directions < 3
  || ( i + total_directions - octant ) % total_directions > total_directions - 3
```

For a 6-direction (hex) coordinate system this admits every direction *except* the one exactly
opposite `octant` (5 of 6); for an 8-direction (square, 8-connected) system it excludes only the
3 directions centered on the opposite side (5 of 8). `shadowcasting_fov_calculate`'s doc comment
claimed this "processes octants systematically", implying a clean, non-overlapping per-direction
partition -- the real, much broader filter makes every one of the `total_directions` calls
redundantly recompute nearly the entire reachable area.

## Impact

**Who is affected:** No consumer sees incorrect *visibility* results -- see Root Cause / How
Discovered for the empirical verification that output is correct. Anyone reading the doc
comment to reason about the algorithm's per-call cost or its relationship to `FloodFill`'s
simpler approach was misled into believing it does less redundant work than it does.

**What breaks:** Nothing user-visible -- see Symptom. This is a documentation-accuracy defect,
not a correctness defect.

**Consumer audit:** N/A -- doc-only fix, no behavioral surface to audit.

**Magnitude:** Two doc comments (`shadowcasting_fov_calculate`, `octant_shadows_cast`) plus one
inline comment at the filter expression itself; see Fix Location.

**Entity Scope:** None -- a code-level/documentation defect.

## How Discovered

Found during a repo-wide "find and fix all bugs and UX/DX defects" sweep of
`module/helper/tiles_tools`, reading `src/field_of_view.rs` end to end and hand-tracing the
octant filter's modulo arithmetic against its doc comment's claim.

## Minimum Reproducible Example

```rust
// module/helper/tiles_tools/tests/field_of_view_test.rs
let hex_shadow: HashSet<_> = FieldOfView::with_algorithm(FOVAlgorithm::Shadowcasting)
  .fov_calculate(&hex_viewer, 4, |_| false).visible_positions().collect();
let hex_flood: HashSet<_> = FieldOfView::with_algorithm(FOVAlgorithm::FloodFill)
  .fov_calculate(&hex_viewer, 4, |_| false).visible_positions().collect();
assert_eq!(hex_shadow, hex_flood); // passes both before and after -- see Prevention
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tiles_tools && cargo nextest run -E 'binary(field_of_view_test) and test(shadowcasting_matches_flood_fill)'
```

## Root Cause

The filter's modulo-distance formula was authored to approximate "directions near `octant`" but
was never checked against its own doc comment's stronger claim of a clean per-direction
partition. Hand-tracing the formula for `total_directions == 6` shows it admits every `i` except
`i == octant + 3 (mod 6)` (the exact opposite direction) -- a threshold of `< 3` /
`> total_directions - 3` out of a `0..total_directions` range is inherently a majority-admitting
band, not a single-direction selector, for any `total_directions` this module supports (6 or 8).

## Why Not Caught

No existing test compared `Shadowcasting`'s visible-position *set* against another algorithm's
-- `tests/integration/field_of_view_tests.rs` only checks aggregate counts/booleans, which
cannot distinguish "correct but redundant" from "correct and efficient". The doc comment's
"systematic" claim was therefore never cross-checked against either the filter's own formula or
an independent reference implementation.

## Fix Location

`module/helper/tiles_tools/src/field_of_view.rs`: **documentation fix, not a behavior fix** --
see Prevention for the judgment call. `shadowcasting_fov_calculate` and
`octant_shadows_cast`'s doc comments, plus an inline comment at the filter expression itself,
now describe the actual overlapping-band behavior (5-of-6 / 5-of-8 directions admitted per
call) and its redundant-recomputation performance cost, instead of claiming a systematic
non-overlapping partition.

## Prevention

Judgment call: **not** narrowing the filter to a literal single-direction selector
(`i == octant` only), despite that looking like the "obviously correct" fix for a doc claiming
a clean partition. Hand-tracing that alternative shows it degenerates into a single straight ray
per call (this coordinate system's direction vectors are fixed/translation-invariant and compose
additively ring-over-ring), which would *under*-cover the true visible area and regress real
cells to invisible -- a strictly worse bug than today's redundancy. Instead: two new permanent
tests in `tests/field_of_view_test.rs`
(`test_shadowcasting_matches_flood_fill_open_field_all_coordinate_systems`,
`test_shadowcasting_matches_flood_fill_with_obstacle_hex`) assert `Shadowcasting` and
`FloodFill` produce byte-for-byte identical visible-position sets, including correct occlusion
behind a wall, across hex/square-4/square-8 coordinate systems -- providing the concrete
verification this finding required before choosing doc-only over a behavior change, and guarding
against a future well-intentioned-but-wrong narrowing of this filter.

## Pitfall

A filter that looks obviously wrong from its formula alone (5 of 6, or 5 of 8, directions
admitted per "single-direction" call) is not automatically an incorrectness bug -- when every
write into a shared result is idempotent (same computation regardless of which caller performs
it, as `visibility_map.visibility_set` is here), redundant over-inclusion only costs
performance, never correctness. Conversely, a narrower filter that looks "obviously more
correct" can silently under-cover instead -- verify empirically against a reference
implementation before changing filter math, never from formula inspection alone.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/helper/tiles_tools`, hand-tracing `octant_shadows_cast`'s filter against its own doc claim. |
| 2026-08-20 | fixed | Documentation-only fix: doc comments corrected to describe the actual overlapping-band behavior; behavior itself left unchanged after empirical verification ruled out a narrower filter as a regression risk. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Empirical correctness verification | — | 🟢 | Adversarial pass: before committing to doc-only, wrote and ran a temporary scratch comparison test (Shadowcasting vs FloodFill visible-position-set equality) across 4 scenarios (hex/square4/square8 open field, hex with obstacle) -- confirmed byte-for-byte identical results in all 4, including matching occlusion behind a wall, before converting the validated logic into the permanent tests listed under Prevention. | — |
| D2 | Regression-guard adequacy | — | 🟢 | Confirmed the permanent tests would catch a future narrowing to `i == octant`-only: hand-traced that alternative's degenerate single-ray behavior and documented it directly in the test's own doc comment as a guard against that specific future mistake. | — |

**Reproduced:** N/A (documentation-only fix, no runtime behavior changed -- see Fix Location).
The two new permanent tests instead provide the correctness evidence the doc-only decision
required: both pass against the unchanged filter, confirming "redundant but not incorrect" as
an empirical fact rather than a formula-inspection guess. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/src/field_of_view.rs` | Corrected `shadowcasting_fov_calculate` and `octant_shadows_cast`'s doc comments, plus an inline comment at the filter itself, to describe the actual overlapping-band behavior instead of claiming a systematic non-overlapping partition. No behavior changed. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tiles_tools/tests/field_of_view_test.rs` | Added `test_shadowcasting_matches_flood_fill_open_field_all_coordinate_systems` and `test_shadowcasting_matches_flood_fill_with_obstacle_hex`, asserting exact visible-position-set equality between Shadowcasting and FloodFill across hex/square4/square8 coordinate systems, with and without obstacles. |

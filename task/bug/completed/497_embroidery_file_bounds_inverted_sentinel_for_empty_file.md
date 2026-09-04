# BUG-497: `EmbroideryFile::bounds()` returns an inverted-sentinel tuple for an empty file instead of signaling absence

- **Severity:** High (silently manufactures a fake "bounds" value with no error -- any caller
  computing width/height via `max_x - min_x` on the untouched sentinels underflows: `i32::MIN -
  i32::MAX` panics in debug and wraps in release)
- **state:** Completed
- **Affects:** Any caller of `EmbroideryFile::bounds()` on a file with zero stitches. Confirmed
  concretely for this crate's own `pec::writer::content_write` and `pes::writer::version1_write`/
  `version6_write`, all three of which called `bounds()` unconditionally regardless of whether the
  file had any stitches.
- **Component:** `module/helper/embroidery_tools` (`src/embroidery_file.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Found in the same sweep as BUG-498 (same crate, both in the PEC/PES writer
  path) but a different mechanism (inverted min/max sentinel vs. raw byte-length UTF-8
  truncation) -- filed separately, no shared root cause.

## Symptom

```rust
// pre-fix -- src/embroidery_file.rs
pub fn bounds( &self ) -> ( i32, i32, i32, i32 )
{
  let mut max_x = i32::MIN;
  let mut min_x = i32::MAX;
  let mut max_y = i32::MIN;
  let mut min_y = i32::MAX;

  for stitch in self.stitches()
  {
    max_x = max_x.max( stitch.x );
    min_x = min_x.min( stitch.x );
    max_y = max_y.max( stitch.y );
    min_y = min_y.min( stitch.y );
  }

  ( min_x, min_y, max_x, max_y ) // unchanged sentinels when stitches() is empty
}
```

For a fresh `EmbroideryFile::new()` (zero stitches), the loop body never executes, so the
function returns `(i32::MAX, i32::MAX, i32::MIN, i32::MIN)` -- a tuple that type-checks and
looks like a legitimate bounds value, but has `min > max` on both axes.

## Impact

**Who is affected:** Any caller treating the returned tuple as genuine bounds, most concretely
any caller computing a width/height via `max_x - min_x` / `max_y - min_y` -- `i32::MIN -
i32::MAX` overflows `i32`'s range (`(-2147483648) - 2147483647`), panicking in a debug build and
silently wrapping in release.

**What breaks:** Both in-crate writer call sites (`pec::writer::content_write`,
`pes::writer::version1_write`, `pes::writer::version6_write`) called `bounds()` unconditionally
and separately checked `emb.stitches().is_empty()` for branching -- so the inverted sentinel was
computed but (for `pes::writer`) its actual header/bounds fields were never populated from it in
the empty branch, and for `pec::writer` it was passed straight into `pec_block_write`, which
itself separately re-checks `is_empty()` before ever reading the tuple. No live panic was
reachable through today's 2 in-crate callers, but the API contract itself was unsafe for any
future or external caller performing the natural `max - min` computation.

**Consumer audit:** Grepped the workspace for `\.bounds\(\)` callers -- confined to
`embroidery_tools`'s own 2 writer files (3 call sites total, listed above). No external crate
calls this method.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of `module/helper/embroidery_tools`.

## Minimum Reproducible Example

```rust
// module/helper/embroidery_tools/tests/embroidery_file_test.rs
let emb = EmbroideryFile::new(); // zero stitches
assert_eq!( emb.bounds(), None ); // pre-fix: Some((2147483647, 2147483647, -2147483648, -2147483648))
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/embroidery_tools && cargo nextest run -E 'test(bounds_returns_none_for_empty_file)'
```

## Root Cause

`min_x`/`min_y` were seeded at `i32::MAX` and `max_x`/`max_y` at `i32::MIN` -- a standard
min/max-reduction seeding pattern, correct for the non-empty case -- but the function returned
these seeds completely unchanged whenever `self.stitches()` was empty, with no check that the
loop ever ran.

## Why Not Caught

The pre-existing test (`bounds_returns_min_and_max_stitch_coordinates`) only ever constructed a
file with real stitches -- nothing exercised the empty-file case, the one input where the
seed-reduction pattern's assumption (the loop runs at least once) fails.

## Fix Location

`module/helper/embroidery_tools/src/embroidery_file.rs`: changed `bounds()`'s return type from
`( i32, i32, i32, i32 )` to `Option< ( i32, i32, i32, i32 ) >`, returning `None` via `?` on an
empty stitch iterator and seeding min/max from the first real stitch instead of sentinel
constants (eliminating the sentinel-seed pattern entirely, not just guarding it). Updated the 2
call sites: `pec::writer::content_write` now uses `emb.bounds().unwrap_or( ( 0, 0, 0, 0 ) )` (a
value that is provably never observed, since `pec_block_write` independently re-checks
`is_empty()` before reading it); `pes::writer::version1_write`/`version6_write` were restructured
from "call `bounds()` unconditionally + separately branch on `is_empty()`" (a redundant double
check, since the two conditions are always equivalent) to a single
`if let Some( extends ) = emb.bounds() { ... } else { ... }`, which also incidentally fixes a
dead-computation-in-the-empty-branch code smell in the same edit.

## Prevention

New test `bounds_returns_none_for_empty_file` asserts `EmbroideryFile::new().bounds() == None`.
Existing test `bounds_returns_min_and_max_stitch_coordinates` updated to assert
`Some( ( -5, -20, 30, 40 ) )` for the non-empty case, locking in the `Option`-wrapped contract on
both sides.

## Pitfall

A min/max-reduction over a possibly-empty collection has no legitimate value to return for the
empty case -- returning the untouched sentinel seeds silently manufactures a fake "empty design
spans everywhere" result instead of surfacing the absence of data. `Option` makes the empty case
a compile-time-checked branch at every call site instead of a runtime landmine.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `module/helper/embroidery_tools`. |
| 2026-08-20 | fixed | `bounds()` changed to return `Option<...>`; both call sites updated. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted `bounds()` to a version that still wraps the old inverted-sentinel computation in `Some(...)` (simulating the pre-fix shape through the new signature) and confirmed `bounds_returns_none_for_empty_file` fails with `Some((2147483647, 2147483647, -2147483648, -2147483648))`; restored the fix and confirmed 20/20 crate tests pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-497)`/`Root cause`/`Pitfall` 3-field comment applied at `bounds()`'s definition. | — |
| D3 | Scope containment | — | 🟢 | Changes confined to `embroidery_file.rs` (signature) and the 2 writer call sites that consume it; no unrelated files touched. | — |

**Reproduced:** YES -- temporarily reverted `bounds()` to compute the old inverted-sentinel
tuple (wrapped in `Some` to keep the crate compiling against the new `Option` signature);
`bounds_returns_none_for_empty_file` failed with `left: Some((2147483647, 2147483647,
-2147483648, -2147483648))`, `right: None`. Restored the fix; full crate suite (20/20) passes
with 0 warnings. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/src/embroidery_file.rs` | `bounds()` now returns `Option< ( i32, i32, i32, i32 ) >`; seeds min/max from the first real stitch instead of sentinel constants. |
| `module/helper/embroidery_tools/src/format/pec/writer.rs` | `content_write` uses `emb.bounds().unwrap_or( ( 0, 0, 0, 0 ) )`. |
| `module/helper/embroidery_tools/src/format/pes/writer.rs` | `version1_write`/`version6_write` restructured to `if let Some( extends ) = emb.bounds() { ... } else { ... }`, removing the redundant separate `is_empty()` branch. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/embroidery_tools/tests/embroidery_file_test.rs` | Added `bounds_returns_none_for_empty_file`; updated `bounds_returns_min_and_max_stitch_coordinates` to assert `Some(...)`. |

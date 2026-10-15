# BUG-491: `line_tools::joins`' `Join::Round/Miter/Bevel` with a `0` precision component silently return empty geometry, not NaN

- **Severity:** High (no crash -- the returned `Vec`s are simply empty, which most callers pass
  straight to a GPU buffer upload with no length check, so a `0`-precision join silently vanishes
  from the rendered line with no error anywhere)
- **state:** Completed
- **Affects:** Any consumer of `Join::Round`/`Join::Miter`/`Join::Bevel`'s `.geometry()` (via
  `line_tools::d2::Line`/`d3::Line`'s `join_set`) that ever constructs a `Join` with a `0`
  `column_precision`. Confirmed via workspace-wide audit: the only real (non-test) construction
  sites, `examples/minwebgl/2d_line/src/main.rs:36-38,105`, all use fixed non-zero literals
  (`7, 7` / `16, 8`) -- so no *currently shipped* caller triggers this today. It is nonetheless a
  latent defect in a public, `usize`-typed API with no documented minimum, reachable by any future
  caller (e.g. a UI-driven precision slider defaulting to or allowing `0`).
- **Component:** `module/helper/line_tools` (`src/joins.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same missing-floor defect *shape* as BUG-236 (`caps.rs::round_cap_geometry`)
  and BUG-237 (`helpers.rs::circle_geometry`) -- both already fixed via an identical `.max( 1 )`
  guard on their own `segments`/precision parameter. This bug applies the same guard to the last
  unguarded functions of that shape in this crate. Filed and fixed together as one bug (all three
  affected functions share one root cause and one fix), not split into three, since `caps.rs`/
  `helpers.rs` already went through this exact process under BUG-236/BUG-237 and this bug's own
  discovery explicitly named all three `joins.rs` functions up front, unlike those two which were
  each found and fixed independently before the pattern was recognized as recurring.

## Symptom

```rust
// pre-fix -- src/joins.rs
pub fn round_geometry( row_precision : usize, column_precision : usize ) -> ( Vec< gl::F32x2 >, Vec< f32 > )
{
  // no floor on row_precision / column_precision
  ...
  for j in 0..column_precision  // exclusive range, empty when column_precision == 0
  {
    verticies.push( ... );  // never executed for column_precision == 0
    uvs.push( ... );
  }
  ...
}
```

`Join::Round( 8, 0 ).geometry()` (and the `Miter`/`Bevel` equivalents) returns completely empty
`vertices`/`uvs` `Vec`s and a `len` of `0` -- silently, with no error, warning, or panic.

## Impact

**Who is affected:** Any caller that constructs `Join::Round`/`Miter`/`Bevel` with a `0`
`column_precision` (directly, or indirectly via a computed/configurable value -- e.g. a
slider-driven precision UI defaulting to `0` before the user first adjusts it). The resulting
empty geometry is typically uploaded straight to a GPU vertex buffer with no length check
downstream, so the join segment of the line simply fails to render, with nothing in the render
path signaling why.

**What breaks:** Visual only for the affected join -- no panic, no corrupted memory, no effect on
any other part of the line's geometry (points/caps/other joins are computed independently).

**Consumer audit:** `grep -rn "Join::Round(\|Join::Miter(\|Join::Bevel("` across the workspace
(excluding `line_tools`'s own `src`/`tests`) finds exactly one real call site,
`examples/minwebgl/2d_line/src/main.rs` (4 constructions, lines 36-38 and 105), all using fixed
non-zero literals (`Join::Miter( 7, 7 )`, `Join::Bevel( 7, 7 )`, `Join::Round( 16, 8 )`) -- none of
which trigger this defect today. `tilemap_renderer` references the `Join` type (`src/types.rs`,
`tests/command_consistency_test.rs`) but does not construct it with a literal `0` either.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Assigned as part of a repo-wide bug/UX sweep of `line_tools`, `canvas_renderer`, and
`browser_input`, explicitly naming this as the same missing-`.max( 1 )`-guard defect class already
fixed via BUG-236/BUG-237 in this same crate, and asking for the identical guard to be applied to
`round_geometry`/`bevel_geometry`/`miter_geometry`'s `row_precision`/`column_precision`.

**The originating description of this bug claimed `column_precision == 0` produces NaN vertex
data directly (matching BUG-236/BUG-237's own symptom) -- this was empirically checked before
being taken at face value, and found to be incorrect for the *returned* geometry.** A throwaway
probe test (`cargo nextest run -E 'test(probe_zero_precision_combinations)'`, run against the
unmodified pre-fix source, since removed and replaced by the real regression tests below) printed
`len`/`vertices.len()`/"any NaN present" for all 9 combinations of
`{Round, Miter, Bevel} x {zero row, zero column, both zero}`. Result: **`any_nan = false` for
every single combination**, including every `column_precision == 0` case -- disproving the
NaN hypothesis for the actual output. See Root Cause below for why: a genuine NaN *is* computed
internally, but a loop-bound coincidence prevents it from ever reaching the returned `Vec`s.

## Minimum Reproducible Example

```rust
// module/helper/line_tools/tests/webgl/joins.rs
let ( vertices, _indices, uvs, len ) = Join::Round( 8, 0 ).geometry();
// pre-fix: len == 0, vertices.is_empty() == true, uvs.is_empty() == true
// (not NaN -- completely empty)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/line_tools && cargo nextest run -E 'test(bug_491)'
```

## Root Cause

All three functions (`round_geometry`, `bevel_geometry`, `miter_geometry`) compute row/column
values via expressions of the shape `k as f32 / column_precision as f32` (and the equivalent for
`row_precision`) with no floor on either parameter -- for `column_precision == 0` this is a
genuine `0.0 / 0.0`, which is NaN under IEEE 754 (division never panics on a zero float divisor).
This NaN is written into an internal scratch structure (`vertex_row_list`/`column_list`).

**The critical detail the original bug description missed:** every loop that reads
`vertex_row_list`/`column_list` back out into the *returned* `verticies`/`uvs` buffers is bounded
by the exclusive range `0..column_precision` -- which is empty when `column_precision == 0`. So
the internally-computed NaN is written into a buffer that the read loop then never actually reads
from, and the function returns completely empty `Vec`s instead of NaN-populated ones.
`row_precision == 0` alone does not produce NaN either, for a different reason: the relevant
intermediate (`rm`, from `1.0 - ( i as f32 / row_precision as f32 )`) is already rescued by a
pre-existing `.max( center_offset )` call -- `f32::max` returns the non-NaN argument whenever one
side is NaN -- leaving a valid, maximally-thin single-row shape rather than empty output.

## Why Not Caught

No existing test constructed any `Join` variant with a `0` precision component. The `Join`
variants' own doc comments describe the two `usize` fields only as "level of triangualtion in the
horizontal and vertical directions," stating no minimum. The masking behavior itself is also
fragile, not intentional: nothing documents or tests that the exclusive read-range is relied on to
suppress the internally-computed NaN, so a superficially reasonable future change (e.g. widening a
`0..column_precision` read loop to `0..=column_precision` to "include the last segment," without
realizing it currently doubles as an accidental NaN guard) would silently reintroduce genuine NaN
into the returned geometry.

## Fix Location

`module/helper/line_tools/src/joins.rs`: `row_precision`/`column_precision` are now floored via
`.max( 1 )` (through variable shadowing, at the top of each function) in all three functions --
`round_geometry`, `bevel_geometry`, `miter_geometry` -- mirroring `caps.rs::round_cap_geometry`
(BUG-236) and `helpers.rs::circle_geometry` (BUG-237)'s established convention for this exact
parameter shape. This removes reliance on the accidental exclusive-range masking and makes a
`0`-precision join degenerate to a valid minimum (1-segment) shape instead of silently vanishing.

## Prevention

New file `module/helper/line_tools/tests/webgl/joins.rs`, five tests: one per join kind asserting
`Join::{Round,Miter,Bevel}( 8, 0 ).geometry()` is non-empty and fully finite
(`*_does_not_produce_empty_geometry_bug_491`), one covering `row_precision == 0` alone and the
fully-degenerate `(0, 0)` case across all three kinds, and one confirming the `.max( 1 )` floor
does not perturb ordinary non-zero precision values. All are constructed through the crate's real
public entry point (`Join::geometry`), not by calling the private generator functions directly.

## Pitfall

An exclusive read-range that happens to prevent a downstream NaN from reaching callers is not a
substitute for flooring the value at its source: it silently changes the failure mode from "NaN"
to "empty output" rather than fixing anything, and depends on every future edit to every read-loop
in the function never touching that exact range shape. More generally: **a bug report's own
literal description of the symptom is a hypothesis, not a fact -- when the actual mechanism is
cheap to verify directly (here, a throwaway probe test against the real unfixed code), verify it
before writing the permanent Root Cause narrative**, since a plausible-but-wrong symptom
description (NaN) would have produced tests asserting the wrong thing (a NaN check that could
never actually fail, since the real defect was empty output) and left the true failure mode
uncovered.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Assigned as part of a repo-wide bug/UX sweep, naming the same missing-`.max( 1 )`-guard defect class as BUG-236/BUG-237. The assignment's literal "produces NaN" claim was checked via a throwaway probe test before being written into this report, and found inaccurate for the returned geometry -- see How Discovered / Root Cause for the corrected mechanism (silently empty output, not NaN). |
| 2026-08-20 | fixed | `.max( 1 )` floor applied to `row_precision`/`column_precision` in `round_geometry`, `bevel_geometry`, `miter_geometry`, matching BUG-236/BUG-237's established convention. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: ran the 5 new tests against the unmodified pre-fix source first (RED) -- 4 of 5 failed exactly as predicted from the probe data (`join_geometry_with_ordinary_precision_is_unaffected_by_the_floor` passed both before and after, as expected, since it never exercises `0`). Restored fix, all 5 pass (GREEN). Full crate + scoped 3-crate suite: `cargo nextest run -p line_tools -p canvas_renderer -p browser_input --all-features` -- 139/139 pass; `cargo clippy -p line_tools -p canvas_renderer -p browser_input --all-targets --all-features -- -D warnings` clean; `cargo test --doc` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-491)`/`Root cause`/`Pitfall` 3-field format applied to `round_geometry` (full) with abbreviated reference comments on `bevel_geometry`/`miter_geometry`, mirroring this crate's own `lib.rs` Fix(BUG-238) precedent for a fix repeated across sibling functions. | — |
| D3 | Scope containment | — | 🟢 | Confirmed via `git diff` that only `src/joins.rs` (fix) and `tests/webgl/joins.rs`+`tests/webgl/mod.rs`+`tests/webgl/readme.md` (tests/registration) were touched for this bug -- no edits outside the assigned `line_tools`/`canvas_renderer`/`browser_input`/`task/` scope. | — |

**Reproduced:** YES -- the 5 new tests, run against the unmodified pre-fix source, failed 4/5
(the 5th, which only exercises ordinary non-zero precision, was unaffected either way, as
expected). Restoring the `.max( 1 )` fix passes all 5. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/line_tools/src/joins.rs` | `.max( 1 )` floor on `row_precision`/`column_precision` in `round_geometry`, `bevel_geometry`, `miter_geometry`; `Fix(BUG-491)`/`Root cause`/`Pitfall` comment (full on `round_geometry`, abbreviated reference on the other two). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/line_tools/tests/webgl/joins.rs` | New file: 5 tests covering zero/degenerate precision for all 3 join kinds plus an ordinary-precision non-regression check. |
| `module/helper/line_tools/tests/webgl/mod.rs` | Added `mod joins;`. |
| `module/helper/line_tools/tests/webgl/readme.md` | Registered `joins.rs` (and, closing a pre-existing gap found while editing this table, the already-present but unregistered `caps.rs`/`helpers.rs`/`colors_desync.rs`) in the Responsibility Table. |

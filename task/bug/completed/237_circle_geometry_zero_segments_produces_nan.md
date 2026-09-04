# BUG-237: `circle_geometry( 0 )` divides by zero, producing a `NaN` vertex

- **Severity:** Low (no crash, no panic -- `f32` division never panics -- and unlike BUG-236,
  this function has zero in-tree callers today, so the blast radius is confined to the crate's
  exported public API surface rather than any currently-executing code path)
- **state:** Completed
- **Affects:** Any `circle_geometry( segments )` call with `segments == 0`.
- **Component:** `module/helper/line_tools` (`src/helpers.rs`, `circle_geometry`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** [BUG-236](./236_round_cap_geometry_zero_segments_produces_nan.md) -- the
  identical defect shape (unrescued division by a `segments`/`wedge`-count parameter inside an
  inclusive `0..=segments` range loop that still executes once at `segments == 0`), found in
  `caps::round_cap_geometry` in the same scouting pass; same relationship as BUG-181/BUG-193's
  "defect class duplicated in a second file" precedent. Also mirrors the already-fixed
  BUG-142/BUG-233 (`.max( .. )`-floor-a-divisor-parameter convention).

## Symptom

```rust
// pre-fix
pub fn circle_geometry( segments : usize ) -> Vec< [ f32; 2 ] >
{
  let mut positions = Vec::with_capacity( segments );
  for wedge in 0..=segments                          // segments=0: still runs once (wedge=0)
  {
    let theta = 2.0 * std::f32::consts::PI * wedge as f32 / segments as f32;
    // wedge=0, segments=0: wedge as f32 / segments as f32 == 0.0 / 0.0 == NaN
    let ( s, c ) = theta.sin_cos();                   // (NaN, NaN)
    positions.push( [ 0.5 * c, 0.5 * s ] );            // NaN vertex pushed into the RETURNED Vec
  }

  positions   // == [[NaN, NaN]] for segments=0
}
```

`circle_geometry( 0 )` returns `[[NaN, NaN]]` -- a single, silently-corrupted vertex, no error,
no panic.

## Impact

**Who is affected:** Any caller of `circle_geometry( 0 )`. `circle_geometry` is `pub`, exported
via `mod_interface!`'s `own use` tier at `helpers.rs`'s layer and re-exported again through
`joins.rs`'s `own use crate::helpers::circle_geometry;` -- confirmed via a workspace-wide grep
that it has **zero call sites anywhere in this workspace today**, so the practical blast radius
is limited to the crate's public API surface (any consumer depending on this crate directly, in
or outside this workspace) rather than any currently-executing internal code path.

**What breaks:** The returned position buffer contains a `NaN` vertex with no diagnostic.

**Magnitude:** 1 function (`circle_geometry`), 1 missing floor.

**Entity Scope:** None -- a code-level defect.

## How Discovered

While scouting `line_tools` for BUG-236 (this session), read `helpers.rs` in full immediately
after confirming `caps.rs`'s `round_cap_geometry` NaN-leak, and recognized the identical shape in
`circle_geometry`: an inclusive `0..=segments` range dividing by the same `segments` parameter
with no floor. Confirmed via grep that `circle_left_half_geometry`/`circle_right_half_geometry`
(the two structurally similar siblings in the same file) use an *exclusive* `0..segments` range
instead, which correctly degenerates to zero iterations (empty output, no `NaN`) at
`segments == 0` -- ruling them out as affected and confirming the inclusive-vs-exclusive range
distinction is exactly what separates safe from unsafe here (same reasoning already applied and
confirmed safe for `joins.rs`'s three geometry functions during this scouting pass).

## Minimum Reproducible Example

```rust
use line_tools::helpers::circle_geometry;

let positions = circle_geometry( 0 );
assert!( positions.iter().all( | p | p[ 0 ].is_finite() && p[ 1 ].is_finite() ) ); // pre-fix: fails, [[NaN, NaN]]
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/line_tools && cargo nextest run --all-features -E 'test(circle_geometry_with_zero_segments_does_not_produce_nan_bug_237)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `circle_geometry` divides by `segments` with no floor inside an inclusive `0..=segments` loop that still executes once at `segments == 0`, so `circle_geometry( 0 )` silently returns a `NaN` vertex instead of erroring. | ✅ Root Cause | Direct read of pre-fix `circle_geometry` shows the unguarded division inside a loop that runs at least once regardless of `segments`; confirmed empirically via temporary-revert-and-rerun (`circle_geometry( 0 )` produced `[[NaN, NaN]]`, test failed as predicted). | E1, E2, E3, E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/line_tools/src/helpers.rs`, `circle_geometry` (pre-fix, direct read) | `for wedge in 0..=segments { let theta = .. * wedge as f32 / segments as f32; ... positions.push(..); }` -- same inclusive-range-divides-by-its-own-bound shape as BUG-236's `round_cap_geometry`. | H1 ✅ |
| E2 | `module/helper/line_tools/src/helpers.rs`, `circle_left_half_geometry`/`circle_right_half_geometry` (direct read, same file) | Both use `for wedge in 0..segments` -- an *exclusive* range that yields zero iterations at `segments == 0`, so `positions` stays empty; these two are NOT affected, confirming the inclusive-vs-exclusive range distinction is the deciding factor. | H1 ✅ |
| E3 | Workspace-wide grep (`grep -rn "circle_geometry" --include="*.rs" .`) | Zero call sites anywhere in the workspace outside `circle_geometry`'s own definition and its `joins.rs`/`mod_interface!` re-exports -- confirms this is a live public-API defect with no currently-executing internal caller, informing the Severity rating (Low, not Medium like BUG-236). | H1 ✅ |
| E4 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting `circle_geometry`'s floor back to the unguarded `segments` reproduced `expected every vertex to be finite for circle_geometry( 0 ), got [[NaN, NaN]]` on the new test. | H1 ✅ |

## Root Cause

`circle_geometry` accepted a `usize` `segments` parameter and used it unchanged both as an
inclusive loop bound (`0..=segments`) and, inside that same loop, as a division's divisor
(`wedge as f32 / segments as f32`). For `segments == 0`, the inclusive range still yields exactly
one iteration (`wedge == 0`), and `0.0_f32 / 0.0_f32` evaluates to `NaN` per IEEE 754 -- Rust's
`f32` division never panics on a zero divisor, so the `NaN` silently propagated into the returned
position vector instead of erroring at the point the invalid input was actually used.

## Why Not Caught

No existing test constructed `circle_geometry` with `0` segments; the function has no doc comment
stating a minimum, and (per E3 above) this crate has no other caller of `circle_geometry` at all
-- the defect was reachable only through direct, unprivileged use of the crate's own public API,
which no test or in-tree caller exercised.

## Fix Location

`module/helper/line_tools/src/helpers.rs`: `circle_geometry` now floors its `segments` argument
with `segments.max( 1 )` before it's used as a loop bound or division's divisor, mirroring
BUG-236's identical fix for `round_cap_geometry` in the same crate.

## Prevention

`tests/webgl/helpers.rs::circle_geometry_with_zero_segments_does_not_produce_nan_bug_237` calls
`circle_geometry( 0 )` directly and asserts every returned vertex is finite. Two sibling tests
confirm the floor is a no-op for an already-valid segment count
(`circle_geometry_with_ordinary_segments_is_unaffected_by_the_floor`) and lock in that
`circle_left_half_geometry`/`circle_right_half_geometry` already degenerate safely at
`segments == 0` (`circle_half_geometry_with_zero_segments_stays_empty_and_finite`), guarding
against a future refactor toward `circle_geometry`'s inclusive-range shape silently
reintroducing this defect in either sibling.

## Pitfall

`f32`/`f64` division never panics on a zero divisor -- it silently returns `NaN` or `±inf` -- so
there is no language-level safety net that will surface a missing floor on a parameter that later
becomes a division's divisor inside a loop whose range includes the degenerate case. A public
function with zero in-tree callers is still a live defect: it is part of the crate's exported API
surface and reachable by any external consumer with no privilege beyond an ordinary function
call -- "nothing calls it yet" is a Severity signal, not a reason to decline filing (contrast
with this same session's `canvas_renderer::texture_set` leak, correctly declined because it was
*both* zero-reachability *and* untestable without a live GL context; `circle_geometry` fails
neither of those two conditions).

## Generalized Version

**Broken assumption:** "a loop bounded by the same parameter that's used as a divisor inside it
will naturally skip the divide-by-zero case when that parameter is 0."

**Confirmed general rule:** Same rule as BUG-236's (this is its second confirmed instance): this
only holds for an *exclusive* range (`0..N`), not an *inclusive* one (`0..=N`). Any `f32`/`f64`
parameter used both as an inclusive-range loop bound and as a division's divisor inside that loop
must be floored to at least `1` before use. Two confirmed instances of this exact shape in one
crate, discovered in the same scouting pass, is itself a signal worth recording: an inclusive
range paired with a same-named divisor is a recognizable, greppable pattern (`0..=`) worth a
deliberate sweep before considering a crate's geometry-generation surface fully scouted.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found immediately after BUG-236, in the same `line_tools` scouting pass, reading `helpers.rs` in full and recognizing the identical unguarded-divisor-inside-an-inclusive-range-loop shape. |
| 2026-08-17 | fixed | `circle_geometry` now floors `segments` with `.max( 1 )`, mirroring BUG-236's fix. |
| 2026-08-17 | verified | `cargo nextest run -p line_tools --all-features`: 98/98 passed, 0 skipped. `cargo test --doc -p line_tools --all-features`: 0 passed, 3 ignored (pre-existing crate convention, unrelated to this fix). `cargo clippy -p line_tools --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (`got [[NaN, NaN]]` pre-fix, passed post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE, `is_finite()` is an exact, non-flaky check. Adversarial pass: re-checked the zero-reachability claim (E3) with a second, independent grep pattern (`circle_geometry` unqualified, not scoped to a specific import style) before relying on it for the Severity rating -- came back empty both times. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly identified and cited BUG-236 as the same-scout sibling occurrence and BUG-233 as the fix-shape precedent, cross-linked in both directions; also correctly distinguished this filing decision from the declined `canvas_renderer::texture_set` leak (zero-reachability alone is not sufficient grounds to decline when the function remains natively testable and is genuine public API). | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of `circle_geometry` and its two exclusive-range siblings, plus empirical revert-rerun proof showing the exact predicted `NaN` value. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to `circle_geometry`'s single argument floor. Adversarial pass: re-read `circle_left_half_geometry`/`circle_right_half_geometry` line-by-line to confirm their exclusive-range loops are genuinely unaffected, not just superficially similar -- confirmed zero iterations at `segments == 0` for both, no `NaN` reachable. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `circle_geometry`; the function's public signature is unchanged (still takes `usize`, returns `Vec<[f32;2]>`), so no downstream caller needed updating (and there are none in this workspace, per E3). | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix with the exact predicted `NaN` value,
pass post-fix) and temporary direct-source-edit revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/line_tools/src/helpers.rs` | `circle_geometry` now floors `segments` with `segments.max( 1 )` (full `Fix(BUG-237)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/line_tools/tests/webgl/helpers.rs` | New file. Added `circle_geometry_with_zero_segments_does_not_produce_nan_bug_237` (`bug_reproducer(BUG-237)`, 5-section doc comment) plus `circle_geometry_with_ordinary_segments_is_unaffected_by_the_floor` and `circle_half_geometry_with_zero_segments_stays_empty_and_finite`. |
| `module/helper/line_tools/tests/webgl/mod.rs` | Added `mod helpers;`. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document, matching this session's established convention for fixed (not by-design) defects. |

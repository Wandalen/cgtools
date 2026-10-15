# BUG-236: `Cap::Round( 0 ).geometry()` divides by zero, producing a `NaN` cap vertex

- **Severity:** Medium (no crash, no panic -- `f32` division never panics -- but the returned
  cap mesh silently contains a `NaN` vertex, which then propagates into whatever consumes it,
  e.g. a GPU vertex buffer, with no diagnostic at the source)
- **state:** Completed
- **Affects:** Any `Cap::Round( segments )` constructed (directly, or via `Line::cap_set`) with
  `segments == 0`.
- **Component:** `module/helper/line_tools` (`src/caps.rs`, `round_cap_geometry`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** [BUG-237](./237_circle_geometry_zero_segments_produces_nan.md) -- the
  identical defect shape (unrescued division by a `segments`/`wedge`-count parameter inside an
  inclusive `0..=segments` range loop that still executes once at `segments == 0`), found in a
  different public function (`helpers::circle_geometry`) in the same scouting pass; same
  relationship as BUG-181/BUG-193's "defect class duplicated in a second file" precedent. Also
  mirrors the already-fixed BUG-142/BUG-233 (`.max( .. )`-floor-a-divisor-parameter convention).

## Symptom

```rust
// pre-fix
pub fn round_cap_geometry( segments : usize ) -> ( Vec< [ f32; 2 ] >, Vec< [ u32; 3 ] > )
{
  let mut positions = Vec::new();
  let mut indices = Vec::new();

  positions.push( [ 0.0; 2 ] );
  for i in 0..=segments                              // segments=0: still runs once (i=0)
  {
    let theta = std::f32::consts::PI * 0.5 + i as f32 / segments as f32 * std::f32::consts::PI;
    // i=0, segments=0: i as f32 / segments as f32 == 0.0 / 0.0 == NaN
    let ( y, x ) = theta.sin_cos();                   // (NaN, NaN)
    positions.push( [ 0.5 * x, 0.5 * y ] );            // NaN vertex pushed into the RETURNED Vec
  }

  for i in 0..segments { /* segments=0: zero iterations, indices stays [] */ }

  ( positions, indices )   // positions == [[0.0, 0.0], [NaN, NaN]] for segments=0
}
```

`Cap::Round( 0 ).geometry()` returns a mesh whose second vertex is `[NaN, NaN]`, with an empty
index buffer (0 triangles) -- silently corrupted data, no error, no panic.

## Impact

**Who is affected:** Any caller constructing `Cap::Round( 0 )`, whether a literal `0`, a
computed segment count that evaluates to `0` (e.g. derived from a zoom/LOD factor), or a
default/uninitialized value -- `Cap::Round( usize )` is a public tuple variant with no smart
constructor and no validation anywhere in the call chain (`cap_set` stores it unchanged;
`geometry()` dispatches to `round_cap_geometry` with the raw value).

**What breaks:** The returned position buffer contains a `NaN` vertex. Since `Cap::geometry()`'s
only real caller is `d2/line.rs`'s mesh-upload path, this `NaN` would reach a GPU vertex buffer
undetected -- typically rendering as degenerate/invisible geometry rather than crashing, making
the root cause hard to spot from symptoms alone.

**Magnitude:** 1 function (`round_cap_geometry`), 1 missing floor.

**Entity Scope:** None -- a code-level defect.

## How Discovered

This session's scouting pass of `line_tools`, reading `caps.rs` in full and noticing the
inclusive `0..=segments` loop divides by `segments` with no floor -- the same defect shape as
the already-fixed BUG-142/BUG-233 (an `f32`/`f64` constructor-or-entry-point parameter later
used as a division's divisor). Confirmed the doc comment states no minimum ("The `usize`
parameter specifies the number of segments used to approximate the curve.").

## Minimum Reproducible Example

```rust
use line_tools::Cap;

let ( vertices, _indices, _len ) = Cap::Round( 0 ).geometry();
assert!( vertices.iter().all( | v | v.is_finite() ) ); // pre-fix: fails, contains NaN
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/line_tools && cargo nextest run --all-features -E 'test(round_cap_geometry_with_zero_segments_does_not_produce_nan_bug_236)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `round_cap_geometry` divides by `segments` with no floor inside an inclusive `0..=segments` loop that still executes once at `segments == 0`, so `Cap::Round( 0 ).geometry()` silently returns a `NaN` vertex instead of erroring. | ✅ Root Cause | Direct read of pre-fix `round_cap_geometry` shows the unguarded division inside a loop that runs at least once regardless of `segments`; confirmed empirically via temporary-revert-and-rerun (`Cap::Round( 0 )` produced `[0.0, 0.0, NaN, NaN]`, test failed as predicted). | E1, E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/line_tools/src/caps.rs`, `round_cap_geometry` (pre-fix, direct read) | `for i in 0..=segments { let theta = .. + i as f32 / segments as f32 * ..; ... positions.push(..); }` -- the loop range is inclusive, so it runs once even when `segments == 0`, and the divisor is the unguarded parameter itself. | H1 ✅ |
| E2 | `module/helper/line_tools/src/caps.rs`, `Cap::Round( usize )` + `d2/line.rs`'s `cap_set` (direct read) | `Cap::Round` is a bare public tuple variant; `cap_set( &mut self, cap : Cap )` stores `self.cap = cap` with zero validation -- no guard anywhere between a caller's `Cap::Round( 0 )` and `round_cap_geometry`'s division. | H1 ✅ |
| E3 | Temporary direct-source-edit revert-and-rerun (this fix) | Reverting `round_cap_geometry`'s floor back to the unguarded `segments` reproduced `expected every vertex component to be finite for Cap::Round( 0 ), got [0.0, 0.0, NaN, NaN]` on the new test. | H1 ✅ |

## Root Cause

`round_cap_geometry` accepted a `usize` `segments` parameter and used it unchanged both as an
inclusive loop bound (`0..=segments`) and, inside that same loop, as a division's divisor
(`i as f32 / segments as f32`). For `segments == 0`, the inclusive range still yields exactly one
iteration (`i == 0`), and `0.0_f32 / 0.0_f32` evaluates to `NaN` per IEEE 754 -- Rust's `f32`
division never panics on a zero divisor, so the `NaN` silently propagated into the returned
position buffer instead of erroring at the point the invalid input was actually used.

## Why Not Caught

No existing test constructed `Cap::Round` with `0` segments -- `tests/webgl/mod.rs` had no test
module for `caps.rs` at all prior to this fix. `Cap::Round`'s own doc comment states no minimum
segment count.

## Fix Location

`module/helper/line_tools/src/caps.rs`: `round_cap_geometry` now floors its `segments` argument
with `segments.max( 1 )` before it's used as a loop bound or division's divisor, mirroring
`Tween::new`/`Step::new`'s established `.max( .. )` guard for the identical defect shape
(BUG-142/BUG-233).

## Prevention

`tests/webgl/caps.rs::round_cap_geometry_with_zero_segments_does_not_produce_nan_bug_236`
constructs `Cap::Round( 0 )` through the crate's real public entry point (`Cap::geometry`) and
asserts every returned vertex component is finite. A sibling test
(`round_cap_geometry_with_ordinary_segments_is_unaffected_by_the_floor`) confirms the floor is a
no-op for an already-valid segment count.

## Pitfall

`f32`/`f64` division never panics on a zero divisor -- it silently returns `NaN` or `±inf` -- so
there is no language-level safety net that will surface a missing floor on a parameter that later
becomes a division's divisor inside a loop whose range includes the degenerate case. An inclusive
range (`0..=N`) is a specific trap here: unlike an exclusive range (`0..N`), it still executes
once even when `N == 0`, so "the loop just won't run" is not a safe assumption to fall back on.

## Generalized Version

**Broken assumption:** "a loop bounded by the same parameter that's used as a divisor inside it
will naturally skip the divide-by-zero case when that parameter is 0."

**Confirmed general rule:** This only holds for an *exclusive* range (`0..N`); an *inclusive*
range (`0..=N`) still executes once at `N == 0`, and if the loop body divides by `N`, that one
iteration silently produces `NaN`. Any `f32`/`f64` parameter used both as an inclusive-range loop
bound and as a division's divisor inside that loop must be floored to at least `1` before use --
this is a specific variant of BUG-233's more general "any divisor-of-a-division constructor
parameter needs a floor" rule, worth naming separately because the inclusive-vs-exclusive range
distinction is exactly what determines whether the same-shaped code is safe or not (see
BUG-237's report for the sibling `circle_geometry`/`circle_left_half_geometry` comparison, where
the exclusive-range siblings are safe by construction and the inclusive-range one is not).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `line_tools` scouting pass, reading `caps.rs` in full and recognizing the unguarded-divisor-inside-an-inclusive-range-loop shape. |
| 2026-08-17 | fixed | `round_cap_geometry` now floors `segments` with `.max( 1 )`, mirroring BUG-233's convention. |
| 2026-08-17 | verified | `cargo nextest run -p line_tools --all-features`: 98/98 passed, 0 skipped. `cargo test --doc -p line_tools --all-features`: 0 passed, 3 ignored (pre-existing crate convention, unrelated to this fix). `cargo clippy -p line_tools --all-targets --all-features -- -D warnings`: clean. Fix verified via a temporary direct-source-edit revert-and-rerun (`got [0.0, 0.0, NaN, NaN]` pre-fix, passed post-fix). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | Confirming pass: deterministic MRE, `is_finite()` is an exact, non-flaky check. Adversarial pass: considered whether the `.max( 1 )` floor could itself change output for a previously-valid `segments >= 1` input -- confirmed `.max( 1 )` is a true no-op for any input already `>= 1`, only the pathological `0` input is affected. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly identified and cited BUG-233 as the fix-shape precedent and BUG-237 as the sibling same-scout occurrence, cross-linked in both directions. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of `round_cap_geometry`, `Cap::Round`, `cap_set`, plus empirical revert-rerun proof showing the exact predicted `NaN` values. | — |
| D5 | Execution Scope | — | 🟢 | Confirming pass: fix confined to `round_cap_geometry`'s single argument floor. Adversarial pass: grepped `caps.rs` and confirmed `square_cap_geometry` (no `segments` parameter) and `Cap::Square`/`Cap::Butt` (no division) are structurally unaffected -- only `Cap::Round`'s path needed the guard. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `round_cap_geometry`; the function's public signature is unchanged (still takes `usize`, returns the same tuple type), so no downstream caller needed updating. | — |

**Reproduced:** Confirmed via `cargo nextest` (fail pre-fix with the exact predicted `NaN`
values, pass post-fix) and temporary direct-source-edit revert-and-rerun. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/line_tools/src/caps.rs` | `round_cap_geometry` now floors `segments` with `segments.max( 1 )` (full `Fix(BUG-236)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/line_tools/tests/webgl/caps.rs` | New file. Added `round_cap_geometry_with_zero_segments_does_not_produce_nan_bug_236` (`bug_reproducer(BUG-236)`, 5-section doc comment) plus `round_cap_geometry_with_ordinary_segments_is_unaffected_by_the_floor`. |
| `module/helper/line_tools/tests/webgl/mod.rs` | Added `mod caps;`. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None -- the fix eliminates the trap rather than leaving a permanent API characteristic to document, matching this session's established convention for fixed (not by-design) defects. |

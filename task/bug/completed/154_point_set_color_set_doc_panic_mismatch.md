# BUG-154: `point_set`/`color_set` doc comments claim a panic the implementation never produces

- **Severity:** Low (documentation-accuracy defect, not a code behavior defect -- the actual
  runtime behavior was already correct and, for `point_set`, already covered by an existing
  test; no caller can be harmed by the code itself, only misled by trusting the doc)
- **state:** Completed
- **Affects:** `d3::Line::point_set` and `d3::Line::color_set` doc comments (both instantiated
  via the single `impl_basic_line!( Line, f32, 3 )` expansion in `src/d3/line.rs:141` -- the
  crate's only expansion of this macro; `d2::Line` is hand-written and has no `point_set`/
  `color_set`/`point_get` methods at all)
- **Component:** `module/helper/line_tools` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None (independent doc-only defect, unrelated to BUG-150/151/152/153's code
  behavior fixes in other crates from the same review batch; surfaced by task #90's line_tools
  review, deferred as task #104 pending the tilemap_renderer/embroidery_tools batch).

## Symptom

```rust
use line_tools::d3;

let mut line = d3::Line::default();
line.point_add_back( &[ 0.0, 0.0, 0.0 ] );

// Doc comment on `point_set` (src/lib.rs:208, pre-fix) reads:
//   "Sets the points at the specified position. Will panic if index is out of range"
line.point_set( [ 1.0, 1.0, 1.0 ], 99 ); // index 99 is out of range for a 1-point line

// Wrong (doc, pre-fix):    implies this call panics.
// Actual (code, unchanged by this fix): call silently does nothing -- `.get_mut(99)` returns
//                          `None`, the `if let Some(..)` body never executes, no panic occurs.
// `color_set` (src/lib.rs:225, pre-fix) has the identical doc/code mismatch.
```

## Impact

**Who is affected:** Any developer reading the public API doc comments for `point_set`/
`color_set` and relying on the documented "will panic" contract -- e.g. writing calling code
that assumes an out-of-range call is caught via `catch_unwind` or a debug assertion, when in
fact the call is silently a no-op and the stale data is never flagged.

**What breaks:** Nothing at runtime -- this is a pure documentation defect. The risk is entirely
in the mismatch between documented and actual contract, which can mislead API consumers into
incorrect assumptions about error handling at call sites they don't directly test.

**Magnitude:** Low. No panic, no data corruption, no silent-wrong-computation -- the existing
(and, for `point_set`, already-tested) behavior was always safe. The defect is confined to two
doc comment lines.

**Entity Scope:** None -- a documentation-level defect, not an operational-entity concern.

## How Discovered

Flagged during a background review pass over the `line_tools` crate (task #90). Deferred as
task #104 pending other in-flight work; confirmed via direct reading of `src/lib.rs:200-236`
(both methods and their accurate sibling `point_get`, which does panic via direct `Index` on
the backing `VecDeque` -- the doc wording for all three methods was apparently copied from one
another without re-verifying against each method's own access pattern).

## Minimum Reproducible Example

```bash
cd module/helper/line_tools && cargo test --test tests webgl::points::test_color_set 2>&1 | tail -5
```

**Expected** (both pre-fix and post-fix -- the code path was never wrong):
```
test webgl::points::test_color_set ... ok
```

**Actual (the defect):** not a runtime failure -- inspect `src/lib.rs:208` and `:225` pre-fix
and observe the doc text ("Will panic if index is out of range") contradicts the
`.get_mut()`-guarded implementation immediately below it, and contradicts `test_point_set`
(`tests/webgl/points.rs:88-110`, pre-existing), which already calls `point_set` at an
out-of-range index (7, against a 4-point line) with no `#[ should_panic ]` and asserts a normal
no-op result.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/line_tools && grep -n "Will panic" src/lib.rs
# pre-fix: 3 hits (point_get, point_set, color_set) -- 2 of them (point_set, color_set) wrong.
# post-fix: 1 hit (point_get only, which genuinely panics via direct Index).
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `point_set`'s and `color_set`'s "Will panic" doc comments are stale copies of `point_get`'s accurate doc comment, not re-verified against their own `.get_mut()`-guarded implementations. | ✅ Root Cause | All three doc comments use near-identical wording; only `point_get` uses direct `Index` (`self.geometry.points[index]`, panics); `point_set`/`color_set` both use `.get_mut(index)` inside `if let Some(..)` (silently no-ops). | E1, E2, E3 |
| H2 | This mismatch repeats elsewhere in the crate via other macro expansions (e.g. a 2D variant). | ❌ Rejected | `impl_basic_line!` is invoked exactly once in the whole crate (`src/d3/line.rs:141`, confirmed via `grep -rn impl_basic_line`); `d2::Line` is a separate, hand-written struct with no `point_set`/`color_set`/`point_get` methods at all. The mismatch exists in exactly one place in source and one place in the expanded code. | E4 |
| H3 | The actual code behavior (silent no-op) is itself the bug, and the doc comment's "will panic" was the intended contract. | ❌ Rejected | `test_point_set` (pre-existing, unmodified, still passing) explicitly exercises an out-of-range `point_set` call with no `#[ should_panic ]` and asserts the surrounding in-range writes still succeeded normally -- the no-op behavior is the established, tested contract. Changing the code to panic instead would break this existing test and would be an unrequested behavior change, not a bug fix. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `src/lib.rs:200-205` (unedited) | `point_get`: doc says "Will panic if index is out of range"; body is `self.geometry.points[ index ]` -- direct `Index`, genuinely panics out-of-range. Accurate doc, no fix needed here. | H1 ✅ |
| E2 | `src/lib.rs:207-222` (pre-fix) | `point_set`: doc said "Will panic..."; body is `if let Some( p ) = self.geometry.points.get_mut( index ) { .. }` -- silently no-ops. `tests/webgl/points.rs:88-110`'s pre-existing `test_point_set` already exercises this exact scenario (index 7 on a 4-point line) and asserts normal no-op behavior with no `#[ should_panic ]`. | H1 ✅, H3 ❌ |
| E3 | `src/lib.rs:224-236` (pre-fix) | `color_set`: identical doc/code mismatch to `point_set`. No pre-existing test of any kind exercised `color_set` before this fix (`grep -rn color_set tests/` returned zero hits pre-fix). | H1 ✅ |
| E4 | `src/d3/line.rs:141`, `src/d2/line.rs` (unedited) | `grep -rn impl_basic_line` returns exactly one call site (`d3::Line`, f32/3D); `d2::Line` is hand-written with no `point_set`/`color_set`/`point_get` methods. | H2 ❌ |

## Root Cause

```
src/lib.rs, impl_basic_line! macro body   (pre-fix)

/// Retrieves the points at the specified position.
/// Will panic if index is out of range              <- accurate: direct Index below
pub fn point_get( &self, index : usize ) -> ... { self.geometry.points[ index ] }

/// Sets the points at the specified position.
/// Will panic if index is out of range              <- WRONG: .get_mut() guard below
pub fn point_set< P : .. >( &mut self, point : P, index : usize )
{ if let Some( p ) = self.geometry.points.get_mut( index ) { .. } }

/// Sets the points at the specified position.
/// Will panic if index is out of range              <- WRONG: .get_mut() guard below
pub fn color_set< C : .. >( &mut self, color : C, index : usize )
{ if let Some( c ) = self.geometry.colors.get_mut( index ) { .. } }
```

The three doc comments share near-identical wording (`point_set`'s and `color_set`'s look
copy-pasted from `point_get`'s), but only `point_get` actually performs the direct, panicking
`Index` access the wording describes -- `point_set`/`color_set` both intentionally guard via
`.get_mut()` returning `Option`, silently skipping an out-of-range write instead.

## Why Not Caught

`point_set`'s actual (no-op) behavior was already exercised by `test_point_set`, but nothing in
this project's tooling cross-checks doc-comment prose against test assertions or implementation
shape -- a passing test coexisted silently alongside a doc comment that directly contradicted
it. `color_set` had zero test coverage of any kind, so even the passive contradiction that
existed for `point_set` had no analogue there to notice.

## Fix Location

`module/helper/line_tools/src/lib.rs`, `impl_basic_line!` macro body:

```rust
// before (point_set, line 208)
/// Sets the points at the specified position.
/// Will panic if index is out of range
pub fn point_set< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, point : P, index : usize )

// after
/// Sets the points at the specified position.
/// Silently does nothing if index is out of range.
pub fn point_set< P : gl::VectorIter< $primitive_type, $dimensions > >( &mut self, point : P, index : usize )

// before (color_set, line 225)
/// Sets the points at the specified position.
/// Will panic if index is out of range
pub fn color_set< C : gl::VectorIter< $primitive_type, 3 > >( &mut self, color : C, index : usize )

// after
/// Sets the points at the specified position.
/// Silently does nothing if index is out of range.
pub fn color_set< C : gl::VectorIter< $primitive_type, 3 > >( &mut self, color : C, index : usize )
```

Doc text corrected to describe the actual, tested, intentional no-op contract. No implementation
code changed -- `point_get`'s doc comment was already accurate and was left untouched.

## Prevention

Added `test_color_set` (`bug_reproducer(BUG-154)`) to `tests/webgl/points.rs`, mirroring
`test_point_set`'s structure: adds 3 colors, calls `color_set` at an in-range index (0, expected
to apply) and an out-of-range index (7, expected to be a silent no-op), asserts via
`colors_get()`. This closes the pre-existing zero-coverage gap on `color_set` and pins the
contract the corrected doc comment now describes. `point_set`'s identical scenario was already
covered by the pre-existing `test_point_set` and was left unmodified rather than duplicated.

## Pitfall

A doc comment copied from a sibling method (here, `point_get`'s accurate "will panic" wording
reused verbatim for `point_set`/`color_set`) must be re-verified against the copy's own
implementation, not assumed to still apply -- near-identical wording across methods in the same
`impl` block can silently diverge from reality the moment one method's access pattern differs
from the one the wording was originally written for (`Index` vs `.get_mut()`).

## Generalized Version

**Broken assumption:** "methods with near-identical doc wording in the same impl block have
near-identical implementations." False here -- `point_get` uses direct `Index` (panics),
`point_set`/`color_set` use `.get_mut()` (silent no-op); the wording didn't track which pattern
each method actually used.

**Confirmed general rule:** when auditing a doc/code mismatch in one method, check every sibling
method with similar wording in the same impl block (here, a 3-method group) rather than fixing
only the one originally flagged -- the same copy-paste origin often produces the same mismatch
more than once.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Flagged by a background review pass over `line_tools` (task #90); deferred as task #104, confirmed via direct reading of `src/lib.rs` before filing. |
| 2026-08-16 | fixed | Corrected `point_set`'s and `color_set`'s doc comments in `src/lib.rs` to describe the actual no-op behavior. No implementation code changed. |
| 2026-08-16 | verified | Added `test_color_set` (`bug_reproducer(BUG-154)`), closing the pre-existing zero-coverage gap on `color_set`; `point_set`'s identical scenario already covered by pre-existing `test_point_set`. Full crate suite (89 tests) + `cargo clippy --all-targets --all-features -- -D warnings` clean. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Confirming pass verified `test_color_set` fails to compile/panics-would-show against a hypothetically-reintroduced panic path (it does not, by construction, since `.get_mut()` never panics); adversarial pass specifically checked whether this doc-only fix could be Tier-Mislabeled as a code fix -- it is explicitly framed throughout as documentation-only, with the pre-existing behavior unchanged and independently re-confirmed via the full 89-test pass. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Independent of BUG-150/151/152/153 (different crate, doc-only, no code path shared). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Root cause backed by direct reading of all 3 sibling doc comments plus their implementations, and confirmation the mismatch traces to copy-pasted wording, not independent authoring errors. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Checked the third sibling (`point_get`) and confirmed its doc is accurate and required no change, rather than assuming all three needed the same fix. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `line_tools` src+test+bug file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix local to two doc comment lines plus one new test function; no signature/behavior change. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | No new public surface; existing methods' documented contract corrected to match their long-standing actual (and, for `point_set`, already-tested) behavior. | — |

**Reproduced:** N/A (doc-only defect, not a code behavior defect) -- there is no pre-fix/post-fix
runtime difference to reproduce; instead, `test_color_set` was written and run against the
codebase to confirm the contract the corrected doc now describes is exactly what the
(unmodified) implementation has always done, 2026-08-16. Full crate suite (89 tests) + `cargo
clippy --all-targets --all-features -- -D warnings` clean, 2026-08-16.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/line_tools/src/lib.rs` | `point_set`'s and `color_set`'s doc comments corrected from "Will panic if index is out of range" to "Silently does nothing if index is out of range." `Fix(BUG-154)`/`Root cause`/`Pitfall` comments added at both sites. No implementation code changed. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/line_tools/tests/webgl/points.rs` | Added `test_color_set` (`bug_reproducer(BUG-154)`, 5-section doc comment), closing the pre-existing zero-coverage gap on `color_set`. |

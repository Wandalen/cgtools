# BUG-215: `Glyph::from_glif`'s point type accumulator leaks across points within a contour

- **Severity:** Medium (silently corrupts curve geometry for any glyph containing an off-curve
  point that omits its `type` attribute -- the normal, spec-correct way to write one -- rather than
  crashing or erroring)
- **state:** Completed
- **Affects:** Every `primitive_generation` caller loading a `.glif` file whose contour mixes an
  explicitly-typed point with a following untyped (off-curve) point -- i.e. essentially every
  real-world glyph containing a curve, since the UFO/glif spec's normal encoding of an off-curve
  bezier control point omits `type` entirely rather than writing `type="offcurve"`.
- **Component:** `module/helper/primitive_generation` (`src/text/ufo.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Same function as the already-fixed BUG-128 (`b"typ"` vs `b"type"` attribute-name
  typo), but an independent root cause -- BUG-128 was about *matching* the attribute at all; this
  bug is about the *default* applied when the attribute is legitimately absent. BUG-128's own fix
  did not touch the accumulator's declaration scope.

## Symptom

```rust
// pre-fix -- Glyph::from_glif, src/text/ufo.rs
let mut typ = PointType::Move;   // declared once, OUTSIDE the point-parsing loop

loop
{
  match event
  {
    Ok( Event::Empty( e ) ) if e.starts_with( b"point" ) =>
    {
      // ... attribute loop only overwrites `typ` if a `type` attribute IS present ...
      contour_points.push( ContourPoint::new( x, y, typ, smooth, None, None ) );
    },
    Ok( Event::End( e ) ) if e.starts_with( b"contour" ) =>
    {
      typ = PointType::Move;   // reset once per CONTOUR, not once per POINT
      ...
```

A point with no `type` attribute silently inherited whatever type the *previous point in the same
contour* had, instead of defaulting to `PointType::OffCurve` (the UFO/glif spec's implicit default
for an untyped point).

## Impact

**Who is affected:** Any caller of `Glyph::from_glif` (directly, or transitively via `Font::new`'s
UFO-directory loader) on a `.glif` file containing a curve -- normal UFO/glif files encode an
off-curve bezier control point by omitting `type` entirely, so this is not an edge case.

**What breaks:** A point following a differently-typed point but itself omitting `type` gets
mis-tagged with the previous point's type instead of `OffCurve`. Depending on the surrounding
points, this either corrupts the resulting bezier path's shape (a wrong `PointType` changes which
`kurbo::PathEl` variant `norad::Contour::to_kurbo` emits for that point) or, as in this bug's own
MRE, starves a following `Curve`/`QCurve` point's `offs` control-point queue entirely, making
`to_kurbo` return `Err(BadPoint)` -- which `Glyph::from_glif` maps straight to `None`, silently
dropping the entire glyph.

**Magnitude:** 1 misscoped `let mut` (moved from per-contour to per-point) plus 1 now-redundant
reset line removed.

**Entity Scope:** None — a code-level defect.

## How Discovered

Continuing this session's bug-fixing sweep of `primitive_generation`'s UFO/glif text pipeline
(following the `browser_input` audit), read `Glyph::from_glif` end-to-end and cross-checked its
per-point state handling against `norad` 0.18.4's own reference glif parser (already established as
this exact function's verification method by the pre-existing `Fix(BUG-128)` comment) -- `norad`'s
`parse_point` declares its own `typ` default fresh, inside the per-point function itself, never
carried over between points; this crate's version declared it once outside the whole event loop.

## Minimum Reproducible Example

```rust
// module/helper/primitive_generation/tests/ufo_glif_point_type_test.rs
const GLIF : &[ u8 ] = br#"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="test" format="2">
  <outline>
    <contour>
      <point x="0" y="0" type="move"/>
      <point x="50" y="100"/>              <!-- no `type` -- should default to OffCurve -->
      <point x="100" y="0" type="curve"/>
    </contour>
  </outline>
</glyph>"#;

let glyph = Glyph::from_glif( GLIF, 'a' );
// pre-fix: None (the untyped point inherits Move, so the Curve point's control-point
//          queue is empty and norad::Contour::to_kurbo returns Err(BadPoint))
// post-fix: Some(..) (the untyped point correctly defaults to OffCurve)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/primitive_generation && cargo nextest run --features font-processing -E 'test(from_glif_defaults_an_untyped_point_to_offcurve_not_the_previous_points_type)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `typ`'s `let mut` is scoped per-contour (outside the point loop), so an untyped point inherits the previous point's type instead of defaulting fresh. | ✅ Root Cause | Confirmed by direct read: `let mut typ = PointType::Move;` sits before `loop { ... }`, and the only reset is at the `</contour>` end-tag arm, not per point. | E1 |
| H2 | `norad`'s own reference parser also carries `typ` across points within a contour, so this crate's behavior is spec-conformant and not a bug. | ❌ Falsified | `norad` 0.18.4's `glyph/parse.rs::parse_point` declares `let mut typ = PointType::OffCurve;` fresh, as a local inside the per-point parsing function itself -- structurally incapable of carrying a value from a prior point. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/primitive_generation/src/text/ufo.rs`, pre-fix `Glyph::from_glif` (direct read) | `typ` declared once outside `loop`, reset only at `</contour>` -- widened lifetime one loop level too coarse. | H1 ✅ |
| E2 | `~/.cargo/registry/.../norad-0.18.4/src/glyph/parse.rs::parse_point` (direct read, line 368) | `let mut typ = PointType::OffCurve;` declared fresh inside the per-point function -- the authoritative reference default, per-point, never per-contour. | H2 ❌ |

## Root Cause

The `typ` accumulator's declaration site (`let mut typ = PointType::Move;`, once, outside the whole
parsing `loop`) was one lexical scope too coarse for the semantics it needed: a value that must
reset to a fresh default *per point* was instead scoped to persist across an entire *contour*,
with a manual reset only at the contour boundary. Any point that omitted its own `type` attribute
therefore read whatever `typ` was left at by the immediately preceding point, not the spec's actual
implicit default.

## Why Not Caught

BUG-128's own regression test (`from_glif_honors_the_declared_curve_point_type`) exercises a curve
where *every* point carries an explicit `type` attribute, including the off-curve one
(`type="offcurve"`) -- exactly the one case that structurally cannot expose this bug, since the
attribute loop always overwrites `typ` when the attribute is present. No existing fixture omitted
a `type` attribute at all.

## Fix Location

`module/helper/primitive_generation/src/text/ufo.rs`, `Glyph::from_glif`:
- Removed the per-contour `let mut typ = PointType::Move;` declaration (and its `</contour>`-arm
  reset).
- Added a fresh `let mut typ = PointType::OffCurve;` local declaration inside the
  `Ok( Event::Empty( e ) ) if e.starts_with( b"point" )` arm, declared before the attribute-parsing
  loop -- matching `norad`'s own reference default exactly, per point.

## Prevention

New test `from_glif_defaults_an_untyped_point_to_offcurve_not_the_previous_points_type` in the
existing `tests/ufo_glif_point_type_test.rs` (same file as BUG-128's own regression test, same
responsibility: point-type parsing correctness in `from_glif`), constructing a contour where an
untyped point follows a `type="move"` point and precedes a `type="curve"` point -- the exact shape
that starves the `Curve` point's control-point queue pre-fix.

## Pitfall

A state-machine accumulator that must reset per-iteration needs its `let mut` declared *inside* the
loop body at the correct granularity -- declaring it one level too high (per-contour instead of
per-point, in this case) silently widens its lifetime, and the leak is invisible in any fixture
where every point happens to carry an explicit type attribute, which is also the easiest kind of
fixture to write by hand.

## Generalized Version

**Broken assumption:** "an attribute-driven state variable's declaration scope doesn't matter as
long as it gets reset *somewhere* before the next logical unit starts."

**Confirmed general rule:** An accumulator's `let mut` must be declared at exactly the granularity
of the state it represents -- a per-point default needs a per-point declaration, not a per-contour
one with a manual reset, since a manual reset at the wrong boundary silently permits leakage across
every intermediate iteration within that boundary.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `primitive_generation` UFO/glif pipeline audit, cross-checking `Glyph::from_glif`'s per-point state handling against `norad` 0.18.4's own reference parser. |
| 2026-08-17 | fixed | Moved `typ`'s declaration from per-contour (outside the loop) to per-point (inside the point-parsing match arm), defaulting to `PointType::OffCurve` per `norad`'s own reference; removed the now-unnecessary per-contour reset. 1 new regression test added. |
| 2026-08-17 | verified | `cargo nextest run -p primitive_generation --features font-processing`: 11/11 passed, 0 skipped. `cargo clippy -p primitive_generation --all-targets --features font-processing -- -D warnings`: clean. Temporary direct-source-edit revert-and-rerun: new test failed with `got None` pre-fix (exact match to hand-derived prediction), passed post-fix. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass checked whether the new per-point `OffCurve` default could regress a contour's very first point (whose type is spec-significant for `is_closed()`); confirmed `norad`'s own reference applies this same default unconditionally per-point regardless of position, so no special-casing was skipped. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE hand-traced against `norad::Contour::to_kurbo`'s actual match arms (`Curve` arm's `offs.make_contiguous()` on an empty queue returns `Err(BadPoint)`) before being encoded; confirmed by temporary-revert re-run producing the exact predicted `None`. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly scoped as independent of BUG-128 (different root cause: attribute *matching* vs. *default value*); correctly identified as a prerequisite investigation that also surfaced BUG-216 (unrelated function, filed separately). | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct read of pre-fix source (E1) and direct read of `norad`'s reference implementation (E2), not assumed from the bug's surface symptom alone. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to `typ`'s declaration scope; the also-hardcoded `let smooth = true;` in the same function was investigated (traced through `norad::Contour::to_kurbo`, which never reads `.smooth`) and correctly left untouched as inert, not swept into this fix. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `primitive_generation`; no downstream crate changes needed. | — |

**Reproduced:** YES — pre-fix, `from_glif_defaults_an_untyped_point_to_offcurve_not_the_previous_points_type`
failed with `got None`, matching the hand-derived prediction exactly; post-fix it passes. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/text/ufo.rs` | `Glyph::from_glif`: moved `typ`'s declaration from per-contour to per-point, defaulting to `PointType::OffCurve`; removed the now-unnecessary per-contour reset (`Fix(BUG-215)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/ufo_glif_point_type_test.rs` | Added `from_glif_defaults_an_untyped_point_to_offcurve_not_the_previous_points_type` and its `GLIF_WITH_OMITTED_TYPE_ATTRIBUTE` fixture; extended the module doc comment to cover BUG-215. |

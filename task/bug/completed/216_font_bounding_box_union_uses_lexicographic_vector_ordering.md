# BUG-216: `Font`'s bounding-box union uses lexicographic `Vector` ordering instead of component-wise min/max

- **Severity:** Medium (silently produces a geometrically wrong union bounding box for any
  multi-glyph font where no single glyph dominates every axis -- corrupts downstream layout/mesh
  sizing without ever erroring)
- **state:** Completed
- **Affects:** Every caller of `Font::from_glyphs` or `Font::new` building a font from more than one
  glyph of differing proportions (e.g. a tall/narrow glyph alongside a wide/short one) -- i.e. any
  real multi-character font.
- **Component:** `module/helper/primitive_generation` (`src/text/ufo.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Discovered during the same `primitive_generation` UFO/glif pipeline audit as
  BUG-215, but an independent root cause in a different function (`Font::from_glyphs`/`Font::new`'s
  bbox-union loop, vs. `Glyph::from_glif`'s per-point type parsing) -- filed separately per this
  repo's one-ID-per-root-cause convention. Covers both call sites (`Font::from_glyphs` and
  `Font::new`) under one ID since they share the identical root cause and fix shape (established
  precedent: BUG-181/193, BUG-207/208, BUG-209, BUG-213).

## Symptom

```rust
// pre-fix -- Font::from_glyphs / Font::new, src/text/ufo.rs (identical pattern at both sites)
if min > glyph.bounding_box.min
{
  min = glyph.bounding_box.min;
}
if max < glyph.bounding_box.max
{
  max = glyph.bounding_box.max;
}
```

`min`/`max` are `F32x3` (`ndarray_cg::Vector<f32, 3>`). `Vector`'s `>`/`<` operators route through
its `Ord`/`PartialOrd` impls, which delegate straight to `[f32; 3]`'s **lexicographic** array
comparison -- decided entirely by the x component, only inspecting y/z to break an x-tie. An AABB
union needs the unrelated **component-wise** per-axis min/max instead.

## Impact

**Who is affected:** Any caller building a multi-glyph `Font` via `Font::from_glyphs` or
`Font::new` where glyphs differ in proportion (tall-vs-wide is the general failure shape, not an
edge case -- most real alphabets contain both).

**What breaks:** Whenever one glyph's bounding box has a more extreme x-extent than another's in
both directions, that glyph's *entire* min/max vector wins the comparison wholesale -- silently
discarding the other glyph's y-extent even where it was the true extreme. The resulting
`Font::max_size` under-reports the font's real vertical (or, symmetrically, horizontal) extent,
corrupting anything downstream that sizes layout, atlas packing, or a bounding mesh off of it.

**Magnitude:** 2 near-identical 4-line `if`/`if` blocks (one per call site) replaced with 2 lines
of `.min()`/`.max()` calls each; 1 new public accessor added to make the defect observable at all.

**Entity Scope:** None — a code-level defect.

## How Discovered

Continuing this session's `primitive_generation` UFO/glif pipeline audit (immediately after
confirming BUG-215), read `Font::from_glyphs` and `Font::new`'s bbox-union loops and noticed both
used `Vector`'s bare comparison operators rather than an explicit component-wise method. Cross-
checked `ndarray_cg::Vector`'s actual `Ord`/`PartialOrd` implementation (`vector/general.rs`,
confirmed lexicographic, delegating to `[E; N]`'s own array `cmp`) against its own
`Vector::min`/`Vector::max` methods (`vector/arithmetics.rs`, confirmed component-wise) and against
`mingl::geometry::BoundingBox`'s own `compute`/`combine` methods (confirmed already using the
correct component-wise pattern for the identical kind of union) -- three-way cross-reference
confirmed the operator was the wrong choice for AABB math.

## Minimum Reproducible Example

```rust
// module/helper/primitive_generation/tests/font_bounding_box_union_test.rs
let font = Font::from_glyphs
(
  [
    ( 't', Glyph::from_glif( &glif_triangle_bytes( 2.0, 10.0 ), 't' ).expect( "tall glyph" ) ),
    ( 'w', Glyph::from_glif( &glif_triangle_bytes( 6.0, 2.0 ), 'w' ).expect( "wide glyph" ) ),
  ]
);
let bbox = font.max_size();
// pre-fix: bbox.min.y() == -1.0, bbox.max.y() == 1.0
//          ('w', the x-dominant glyph, wins the lexicographic comparison wholesale,
//           discarding 't's true y-extent)
// post-fix: bbox.min.y() == -5.0, bbox.max.y() == 5.0  (the real union)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/primitive_generation && cargo nextest run --features font-processing -E 'test(from_glyphs_unions_bounding_boxes_component_wise_not_lexicographically)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The union loop's `if min > ... { min = ...; }` / `if max < ... { max = ...; }` pattern uses `Vector`'s lexicographic `Ord`, not a component-wise min/max, so it can discard one glyph's y-extent when another glyph's x-extent dominates. | ✅ Root Cause | Confirmed by direct read of `ndarray_cg::vector::general`: `impl Ord for Vector`/`impl PartialOrd for Vector` both delegate to `self.0.cmp(&other.0)` — plain `[E; N]` array comparison, x-component-first. | E1 |
| H2 | `Vector` has no component-wise alternative, so the operator-based pattern is the only option and not actually a bug. | ❌ Falsified | `ndarray_cg::vector::arithmetics::Vector::min`/`Vector::max` are exactly the component-wise per-axis methods needed (`r[i] = a[i].min(b[i])`), and this crate's own dependency `mingl::geometry::BoundingBox::compute`/`combine` already use them for the identical union operation. | E2 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/vector/general.rs:55-75` (direct read) | `Ord`/`PartialOrd` for `Vector` delegate to `self.0.cmp(&other.0)` -- lexicographic, x-component-first array comparison. | H1 ✅ |
| E2 | `module/math/ndarray_cg/src/vector/arithmetics.rs:91-114` and `module/min/mingl/src/geometry.rs:68-136` (direct read) | `Vector::min`/`Vector::max` are the correct component-wise methods; `BoundingBox::compute`/`combine` (this crate's own dependency) already use exactly this pattern for AABB union. | H2 ❌ |

## Root Cause

`Vector<E, N>` intentionally exposes two unrelated orderings: a total, lexicographic one (via
`<`/`>`, backed by a derived-delegate `Ord`/`PartialOrd`, useful for e.g. canonical sort keys) and a
component-wise one (via `.min()`/`.max()` methods, useful for geometry). The union loop reached for
the operator instead of the method. Both compile and typecheck identically -- `Vector: PartialOrd`
makes `<`/`>` valid Rust for any two `Vector`s regardless of geometric intent, so there is no
type-level signal at the call site that the wrong ordering was selected.

## Why Not Caught

`Font::max_size` (the field this union loop writes) had **no public accessor at all** before this
fix -- nothing in the crate's public API surface could observe its value. Its only would-be public
readers (`text_to_mesh`/`text_to_countour_mesh`) gate their read of it behind `glyph.body:
Some(...)`, a field nothing in the public API populates for a `Font::from_glyphs`-built font
(`Glyph::new` always leaves `body: None`; `Font::from_glyphs` never calls
`contours_to_fill_geometry`). The union arithmetic was therefore entirely untested.

## Fix Location

`module/helper/primitive_generation/src/text/ufo.rs`:
- `Font::from_glyphs` and `Font::new` (identical pattern at both sites): replaced
  `if min > glyph.bounding_box.min { min = glyph.bounding_box.min; }` /
  `if max < glyph.bounding_box.max { max = glyph.bounding_box.max; }` with
  `min = min.min( glyph.bounding_box.min ); max = max.max( glyph.bounding_box.max );`.
- Added `Font::max_size( &self ) -> BoundingBox`, a minimal read-only accessor (returns by value,
  `BoundingBox: Copy`) mirroring the crate's own pre-existing `Glyph::contours()` precedent for
  exposing an otherwise-private field for testing.

## Prevention

New test file `tests/font_bounding_box_union_test.rs`, constructing two glyphs whose bounding
boxes are tall-vs-wide (one dominant in x, the other in y) -- the exact shape that exposes
lexicographic-vs-component-wise divergence, since a same-direction-dominant pair cannot distinguish
the two orderings.

## Pitfall

The bug is invisible whenever the bounding boxes being unioned happen to already agree on which one
has the more extreme x *and* y in the same direction (the common case for same-script glyphs of
similar proportions) -- it only surfaces when one glyph's x-extent and another's y-extent are the
two that matter, which is common for real alphabets (e.g. `l` vs `w`) but easy to miss by accident
in a hand-picked test fixture.

## Generalized Version

**Broken assumption:** "a type implementing `PartialOrd`/`Ord` means its `<`/`>` operators are a
valid choice for any comparison need involving that type, including geometric per-axis extremes."

**Confirmed general rule:** A vector/tuple type may legitimately support two unrelated orderings --
a total lexicographic one (via `Ord`, useful for sort keys) and a component-wise one (via dedicated
methods, useful for geometry) -- and the compiler cannot distinguish which one a call site *meant*,
since both are equally well-typed. Any AABB/bounding-box union, intersection, or clamp must use the
component-wise method explicitly, never the bare comparison operator, regardless of whether the
operator happens to produce a plausible-looking result on the specific fixture at hand.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `primitive_generation` UFO/glif pipeline audit, immediately after BUG-215; cross-referenced `ndarray_cg::Vector`'s `Ord`/`PartialOrd` impl against its own `.min()`/`.max()` methods and against `mingl::BoundingBox`'s own correct union pattern. |
| 2026-08-17 | fixed | Replaced both union loops' `if min > ... / if max < ...` operator-based comparisons with `min.min(...)` / `max.max(...)` component-wise calls at both `Font::from_glyphs` and `Font::new`; added `Font::max_size()` accessor to make the fix testable. 1 new test file added. |
| 2026-08-17 | verified | `cargo nextest run -p primitive_generation --features font-processing`: 11/11 passed, 0 skipped. `cargo clippy -p primitive_generation --all-targets --features font-processing -- -D warnings`: clean. Temporary direct-source-edit revert-and-rerun: new test failed with `got -1` pre-fix (exact match to hand-derived lexicographic-comparison prediction), passed post-fix. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | Adversarial pass checked both call sites (`from_glyphs` AND `Font::new`) were fixed, not just the one exercised by the new test (which only calls `from_glyphs`) -- confirmed via grep that `Font::new`'s identical pattern was also replaced identically. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE hand-traced against `Vector`'s lexicographic `cmp` before being encoded (predicted `w`'s x-dominant vector would win wholesale, producing `min.y=-1`/`max.y=1` instead of the true `-5`/`5`); confirmed by temporary-revert re-run producing exactly that predicted value. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly filed as a separate ID from BUG-215 (different function, different root cause) despite being found in the same audit pass; correctly scoped to cover both call sites under one ID per established multi-site precedent. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct reads of 3 independent sources (`Vector`'s `Ord` impl, `Vector`'s `.min()`/`.max()` methods, `BoundingBox`'s own correct union pattern) rather than assumed from the symptom alone. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to the union comparison itself; `Font::max_size()` accessor addition is a minimal, justified testability enabler (no other public path exists to observe the field for a `from_glyphs`-built font), not scope creep. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `primitive_generation`; the dependency crates read for evidence (`ndarray_cg`, `mingl`) were read-only references, not modified. | — |

**Reproduced:** YES — pre-fix, `from_glyphs_unions_bounding_boxes_component_wise_not_lexicographically`
failed with `got -1` for both `min.y`/`max.y` assertions, matching the hand-derived lexicographic-
comparison prediction exactly; post-fix it passes. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/text/ufo.rs` | `Font::from_glyphs` and `Font::new`: replaced operator-based bbox union comparison with `.min()`/`.max()` component-wise calls (`Fix(BUG-216)` comment blocks, both sites); added `Font::max_size()` public accessor. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/font_bounding_box_union_test.rs` | New file: `from_glyphs_unions_bounding_boxes_component_wise_not_lexicographically`, with a tall-vs-wide two-glyph fixture designed to distinguish lexicographic from component-wise union. |
| `module/helper/primitive_generation/Cargo.toml` | Registered `font_bounding_box_union_test` as a `[[test]]` entry, `required-features = ["font-processing"]`. |

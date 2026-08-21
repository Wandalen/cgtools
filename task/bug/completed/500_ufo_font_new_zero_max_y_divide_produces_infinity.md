# BUG-500: `Font::new`'s glyph rescale divides by `max_y` with no zero-guard, producing `Infinity`-poisoned glyph coordinates

- **Severity:** Low (requires a UFO font directory where every loaded glyph is zero-height, or
  zero glyphs load at all -- an unusual but real edge case for a malformed/incomplete font
  directory; no crash, but silently corrupts every glyph coordinate to `Infinity`/`NaN`)
- **state:** Completed
- **Affects:** Any UFO font directory loaded via `Font::new` where every loaded glyph is
  zero-height, or where zero glyphs load at all (e.g. a bad `path`, or an entirely empty glyph
  directory).
- **Component:** `module/helper/primitive_generation` (`src/text/ufo.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None. Found in the same sweep as BUG-499 (same crate) but a different
  mechanism and a different source file -- filed separately.

## Symptom

```rust
// pre-fix -- src/text/ufo.rs, Font::new
let [ mut max_x, mut max_y ] = [ 0.0, 0.0 ];
for glyph in glyphs.values() { /* raises max_x/max_y from each glyph's bbox */ }

let scale = 250.0;
for glyph in glyphs.values_mut()
{
  glyph.scale( scale / max_y ); // no guard against max_y == 0.0
}
```

If every loaded glyph is zero-height (or `glyphs` is empty, leaving `max_y` at its `0.0` seed),
`scale / max_y` is `250.0 / 0.0`. Rust float division by zero does not panic -- it silently
produces `f32::INFINITY`, which every subsequent `glyph.scale(...)` call then multiplies every
glyph coordinate by, poisoning them to `Infinity`/`NaN`.

## Impact

**Who is affected:** Any caller loading a UFO font whose glyph directory is malformed, pointed at
the wrong path, or genuinely contains only zero-height glyphs.

**What breaks:** Every glyph's contour/bounding-box coordinates become `Infinity`/`NaN` after the
rescale loop -- silently, with no error signal -- corrupting all downstream text-mesh generation
(`text_to_mesh`/`text_to_countour_mesh`) for that font.

**Consumer audit:** `Font::new` is the only site performing this division; `Font::from_glyphs`
(the synchronous, non-loading constructor used by tests) explicitly skips the rescale step
entirely and is unaffected.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX sweep of `module/helper/primitive_generation`.

## Minimum Reproducible Example

```rust
// module/helper/primitive_generation/tests/ufo_font_scale_test.rs
let factor = glyph_rescale_factor( 250.0, 0.0 );
assert!( factor.is_finite() ); // pre-fix: 250.0 / 0.0 == f32::INFINITY
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/primitive_generation && cargo nextest run -E 'binary(ufo_font_scale_test)'
```

## Root Cause

`max_y` is a "max of measured values" seeded at `0.0` -- a safe default for the *measuring* loop
itself (any real glyph height is `>= 0.0`) -- but that safety does not carry over to the *later*
use of the result as a divisor. The seed value must be re-examined at every place the measured
max gets consumed, not just where it gets produced; nothing did so at the division site.

## Why Not Caught

The division lived inline inside the async, wasm-only, real-file-I/O `Font::new` -- not natively
unit-testable at all (this crate's own `Font::max_size()` / BUG-216 precedent exists for exactly
this reason: extracting a pure accessor to make otherwise GL/IO-bound logic independently
testable). Nothing exercised the `max_y == 0.0` edge case because nothing could reach the
arithmetic without also standing up real UFO font files on disk.

## Fix Location

`module/helper/primitive_generation/src/text/ufo.rs`: extracted the division into a pure
`glyph_rescale_factor( target_scale : f32, max_y : f32 ) -> f32` function that floors `max_y` at
`f32::EPSILON` via `.max(...)` before dividing, and wired `Font::new` to call it instead of the
raw `scale / max_y` expression. `glyph_rescale_factor` is independently testable with plain `f32`
inputs -- no font loading, no WebGL context required. Exposed as `pub` via `mod_interface`.

## Prevention

New test file `ufo_font_scale_test.rs` with 3 tests: `zero_max_y_yields_finite_scale_not_infinity`
(the reproducer), `positive_max_y_divides_normally` (regression guard for the normal case), and
`negative_max_y_also_yields_finite_scale` (locks in that the guard is `.max( EPSILON )`, not
`.abs().max( EPSILON )`, against a hypothetical negative input).

## Pitfall

Float division by zero doesn't panic in Rust -- it silently returns `Infinity`/`NaN`, so a
missing guard gives zero compile-time or runtime signal; the defect only surfaces later as
corrupted (infinite/NaN) glyph geometry, far from its actual cause.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of `module/helper/primitive_generation`. |
| 2026-08-20 | fixed | Extracted `glyph_rescale_factor` with an `f32::EPSILON` floor; wired into `Font::new`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted `glyph_rescale_factor` to the raw unguarded `target_scale / max_y` and confirmed `zero_max_y_yields_finite_scale_not_infinity` fails; restored the fix and confirmed 35/35 crate tests pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-500)`/`Root cause`/`Pitfall` 3-field comment applied at `glyph_rescale_factor`'s definition. | — |
| D3 | Scope containment | — | 🟢 | Changes confined to `ufo.rs` (new function + 1-line call-site change); no unrelated files touched. | — |

**Reproduced:** YES -- temporarily reverted `glyph_rescale_factor` to `target_scale / max_y`
(no guard); `zero_max_y_yields_finite_scale_not_infinity` failed (`factor.is_finite()` is false
for `250.0 / 0.0`). Restored the fix; full crate suite (35/35) passes with 0 warnings.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/text/ufo.rs` | Added `glyph_rescale_factor` with an `f32::EPSILON` floor; `Font::new` now calls it instead of dividing directly. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/ufo_font_scale_test.rs` | New file: 3 tests covering the zero-`max_y` reproducer, the normal case, and a negative-`max_y` edge case. |

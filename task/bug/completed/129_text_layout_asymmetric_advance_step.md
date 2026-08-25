# BUG-129: text glyph layout over-advances by half a slot-width per glyph

- **Severity:** Medium (visible, compounding rendering defect — not a panic or crash, but every
  multi-glyph string renders with progressively wrong glyph spacing)
- **state:** Completed
- **Affects:** Any caller of `primitive_generation::text::ufo::text_to_mesh` or
  `text_to_countour_mesh` with a string of 2+ glyphs
- **Component:** `module/helper/primitive_generation`
  (`src/text/ufo.rs::text_to_mesh`, `src/text/ufo.rs::text_to_countour_mesh`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-127/128, filed under the same task #63
  targeted `primitive_generation` review

## Symptom

```bash
# 3 glyphs 'a','b','c' of raw widths [2.0, 6.0, 4.0], scale 0.003, centered layout:

# Wrong (pre-fix) -- each glyph placed a full slot-width past the previous, instead
# of centered in its own contiguous slot:
text_to_countour_mesh("abc", &font, ...) -> x positions [-0.012, 0.006, 0.018]

# Correct (post-fix) -- each glyph centered in its own slot:
text_to_countour_mesh("abc", &font, ...) -> x positions [-0.015, -0.003, 0.012]
```

## Impact

**Who is affected:** Any caller rendering a string of 2 or more glyphs through either
`text_to_mesh` or `text_to_countour_mesh` — both functions share the identical two-pass layout
bug in their own copy of the loop.

**What breaks:** The layout algorithm runs in two passes: pass 1 walks the string subtracting each
glyph's *half* slot-width to find the horizontally-centered starting x position; pass 2 is meant to
mirror this by advancing a half slot-width, placing the glyph, then advancing another half
slot-width (symmetric step-place-step, matching pass 1's per-glyph half-step). Instead, pass 2
advanced by the glyph's *full* slot-width in one step before placing it, with no trailing step.
This over-advances by exactly one half slot-width per glyph, and the error compounds: each
subsequent glyph inherits the previous glyph's full drift plus its own.

**Magnitude:** Not a crash — a silent, visible, systematically-increasing horizontal misplacement.
For the confirmed 3-glyph case, the first glyph is off by 0.003 units (its own half-slot-width),
the second by 0.009 (compounding), the third by 0.006 net vs. its correct slot center — small per
character but growing across any longer string, since each glyph's placement error becomes part of
the next glyph's starting position.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #63, a targeted code review of `primitive_generation` dispatched under the standing bug-hunt
mandate. The reviewing agent flagged that pass 2's single full-width advance is asymmetric with
pass 1's half-width-only subtraction. Independently re-verified before filing by direct source
reads and independent hand-derivation of the resulting positions:

```bash
$ sed -n '526,600p' module/helper/primitive_generation/src/text/ufo.rs
# confirms pass 1 (subtract half-step per glyph) and pass 2's now-fixed
# symmetric half-step/place/half-step structure

$ sed -n '294,316p' module/helper/primitive_generation/src/text/ufo.rs
# confirms Font::from_glyphs computes max_size as the union bounding box over
# all supplied glyphs -- used to independently derive half_x for the MRE
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre129 && mkdir -p /tmp/mre129/src
cat > /tmp/mre129/Cargo.toml <<'EOF'
[package]
name = "mre129"
version = "0.1.0"
edition = "2021"
EOF
cat > /tmp/mre129/src/main.rs <<'EOF'
// Pure-arithmetic reproduction of text_to_countour_mesh's pass-2 layout loop
// (src/text/ufo.rs), pre-fix and post-fix, for 3 glyphs of raw width
// [2.0, 6.0, 4.0] ('a','b','c'), scale 0.003, max_x = 6.0 (from 'b').
fn main()
{
  let scale = 0.003_f32;
  let max_x = 6.0_f32;
  let half_x = max_x * scale;
  let widths = [ 2.0_f32, 6.0_f32, 4.0_f32 ];
  let glyph_xs : Vec< f32 > = widths.iter().map( | w | w * scale ).collect();

  let mut x = 0.0_f32;
  for &gx in &glyph_xs
  {
    let step = if gx < half_x / 4.0 { half_x / 2.0 } else { gx / 2.0 };
    x -= step;
  }
  let pass1_end = x;

  let mut x = pass1_end;
  let mut postfix = vec![];
  for &gx in &glyph_xs
  {
    let step = if gx < half_x / 4.0 { half_x / 2.0 } else { gx / 2.0 };
    x += step;
    postfix.push( x );
    x += step;
  }

  let mut x = pass1_end;
  let mut prefix = vec![];
  for &gx in &glyph_xs
  {
    let full_step = if gx < half_x / 4.0 { half_x } else { gx };
    x += full_step;
    prefix.push( x );
  }

  println!( "post-fix (fixed) : {postfix:?}" );
  println!( "pre-fix  (buggy) : {prefix:?}" );
}
EOF
cd /tmp/mre129 && cargo run 2>&1 | tail -2
```

**Expected** (post-fix — confirmed both by this MRE's independent arithmetic reproduction and by
running the real fixed crate's own `text_to_countour_mesh` against the same 3-glyph input):
```
post-fix (fixed) : [-0.014999999, -0.0029999986, 0.012000001]
```
(i.e. `[-0.015, -0.003, 0.012]` within `f32` rounding — matches
`tests/ufo_text_advance_test.rs`'s passing assertion exactly.)

**Actual** (pre-fix — confirmed by independently reproducing pass 2's exact old single-full-step
formula, arithmetically, against the same starting position and glyph widths):
```
pre-fix  (buggy) : [-0.011999999, 0.006, 0.018]
```
(i.e. `[-0.012, 0.006, 0.018]`.)

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre129 && cargo run 2>&1 | tail -2
# post-fix line ~= [-0.015, -0.003, 0.012] confirms the fix; a run against the
# real crate producing [-0.012, 0.006, 0.018] instead would indicate regression
```

**Known MRE limitation (check 205):** this MRE reproduces the layout arithmetic in isolation
(pure floating-point, no crate dependency) rather than calling the real crate directly, since the
defect is in closed-form per-glyph advance arithmetic fully determined by 3 inputs (raw widths,
scale, `max_x`) with no other crate state involved. The post-fix branch of this same MRE was
cross-checked against the real fixed crate's own `text_to_countour_mesh` (via
`tests/ufo_text_advance_test.rs`, using real `Glyph::from_glif`-constructed glyphs rather than raw
arithmetic inputs) and produces the identical result to 5 decimal places, confirming the isolated
arithmetic model faithfully represents the real function's behavior.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | Pass 2's single full-slot-width advance-before-place (vs. pass 1's half-slot-width-only subtraction) over-advances by one half slot-width per glyph, compounding across the string. | ✅ Root Cause | Independent arithmetic reproduction of both the old (single full step) and new (symmetric half-step/place/half-step) formulas, from the identical pass-1-derived starting position, produces `[-0.012, 0.006, 0.018]` (pre-fix) vs `[-0.015, -0.003, 0.012]` (post-fix) — a per-glyph delta of exactly `0.003`, `0.009`, `0.006`, matching each glyph's own half-slot-width. | E1, E2 |
| H2 | The drift is bounded/self-correcting — later glyphs "catch up" once their own width is accounted for. | ❌ Falsified | The arithmetic model shows the error is a fixed additive offset carried forward by the running `x` accumulator each iteration — nothing in the loop ever subtracts back the excess; each glyph's start position already includes 100% of the accumulated drift from every prior glyph. | E1 |
| H3 | `text_to_mesh` and `text_to_countour_mesh` have independently-derived layout logic, so the same bug in one does not imply it exists in the other. | ❌ Falsified | Direct source read confirms both functions contain a byte-for-byte identical two-pass loop structure (own local copies, not a shared helper) — the same asymmetric-advance defect existed in both, independently, and both needed the identical fix applied separately. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | Isolated arithmetic reproduction (`/tmp/mre129`), pre-fix vs post-fix formulas, identical 3-glyph input | `post-fix (fixed) : [-0.015, -0.003, 0.012]`; `pre-fix (buggy) : [-0.012, 0.006, 0.018]` — exact match to the hand-derived values already asserted in `tests/ufo_text_advance_test.rs`, independently re-derived via a fresh, separate computation path. | H1 ✅, H2 ❌ |
| E2 | `tests/ufo_text_advance_test.rs`, `text_to_countour_mesh_centers_each_glyph_in_its_own_slot` (real crate, real `Glyph::from_glif`-constructed glyphs) | Passing assertion of `[-0.015, -0.003, 0.012]` within `1e-5` — confirms the isolated arithmetic model in E1 matches the real function's actual behavior, not just its intended formula. | H1 ✅ |
| E3 | `module/helper/primitive_generation/src/text/ufo.rs`, `text_to_mesh` (lines ~455-520) and `text_to_countour_mesh` (lines ~526-600) | Both functions contain their own independent copy of the identical two-pass layout loop, including the identical pre-fix asymmetric-advance defect — confirmed by direct side-by-side read, not inferred from one function alone. | H3 ❌ |

## Root Cause

```
text_to_countour_mesh( text, font, transform, width ) -> Vec<PrimitiveData>
  pass 1: for each glyph, x -= half_step(glyph)          // finds centered start
  pass 2 (PRE-FIX): for each glyph,
    x += full_step(glyph)     <- one full slot-width, not a half-step
    place glyph at x
    // no trailing advance -- next glyph's pass-1-derived offset never applied
```

Pass 1 advances the accumulator by exactly one half-slot-width per glyph (finding the string's
centered starting x). Pass 2 was meant to mirror this with a symmetric half-step-place-half-step
per glyph — but instead performed a single full-slot-width step before placement. Since
`full_step(glyph) = 2 * half_step(glyph)`, this doubles the intended pre-placement advance for
every glyph and omits the intended post-placement advance entirely, netting one extra
half-slot-width of drift per glyph, compounding as the running `x` accumulator carries it into
every subsequent glyph's starting position.

## Why Not Caught

No existing test exercised either layout function at all — both are gated behind
`font-processing` and their only public construction path (`Font::new`/`fonts_load`) is an async,
browser-only file loader with no test fixture wired up, so the pure layout arithmetic was never
independently checked. The drift is also easy to miss by casual inspection or a short visual smoke
test: the *first* glyph's mismatch is bounded by that glyph's own half-width alone (small), so
short strings can look "close enough" while the underlying formula is still systematically wrong
and compounds on longer text.

## Fix Location

`module/helper/primitive_generation/src/text/ufo.rs`, both `text_to_mesh`'s and
`text_to_countour_mesh`'s pass-2 loop. Changed the single full-step advance into a symmetric
half-step-place-half-step, matching pass 1's own half-step-per-glyph formula:

```rust
// before (each function's own copy)
let advance = if glyph_x < half_x / 4.0 { half_x } else { glyph_x };
transform.translation[ 0 ] += advance;
// ... place geometry using this transform ...
// (no second advance)

// after (each function's own copy)
let step = if glyph_x < half_x / 4.0 { half_x / 2.0 } else { glyph_x / 2.0 };
transform.translation[ 0 ] += step;
// ... place geometry using this transform ...
transform.translation[ 0 ] += step;
```

Pass 1 (unchanged) and the vertical-axis logic (unchanged) are untouched — only pass 2's horizontal
advance in both functions.

## Prevention

Added `text_to_countour_mesh_centers_each_glyph_in_its_own_slot` to the new
`tests/ufo_text_advance_test.rs` (gated `required-features = ["font-processing"]"`), covering
`text_to_countour_mesh`. `text_to_mesh` shares byte-for-byte identical fixed logic in its own
pass-2 loop (confirmed by direct comparison) but was not independently tested, since exercising it
would additionally require populating the private `Glyph.body: Option<PrimitiveData>` field, which
only the full `Font::new`/`contours_to_fill_geometry` UFO-loading pipeline sets and which the new
minimal `Font::from_glyphs` constructor deliberately does not — a scoped, documented proportionality
decision (see Generalized Version) rather than an oversight.

**Pitfall:** when a layout algorithm splits an advance across two passes (find start, then
place-and-advance), the two passes' per-item step sizes must be derived from the same formula — an
asymmetric split (half-step here, full-step there) silently drifts every item after the first, and
the drift is invisible on the first item alone.

## Generalized Version

**Broken assumption:** "pass 2's advance mirrors pass 1's" — true in intent (both comments
describe a "centering" layout) but never actually verified arithmetically; the two passes were
independently written with different step-size formulas that happen to share the same conditional
structure (`if glyph_x < half_x/4.0 { ... } else { ... }`), which makes them look symmetric on
casual reading despite one passing a half-multiplier and the other not.

**Confirmed general rule:** when two passes over the same data are meant to be mathematically
inverse or symmetric operations (subtract-to-find-start / add-to-place-and-advance), verify the
net effect algebraically (or by direct numeric simulation, as this MRE does) rather than trusting
structural similarity between the two code blocks — matching `if`/`else` shapes do not guarantee
matching arithmetic.

**Scope note:** `text_to_mesh` was fixed identically to `text_to_countour_mesh` (same root cause,
same fix) but is not independently test-covered — testing it would require populating the private,
pipeline-only `body` field on `Glyph`, judged disproportionate to this bug's scope given both
functions' pass-2 logic is byte-for-byte identical and already covered via
`text_to_countour_mesh`'s test.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #63's targeted code review of `primitive_generation`; confirmed via independent arithmetic re-derivation before filing. |
| 2026-08-15 | fixed | Changed pass 2's advance in both `text_to_mesh` and `text_to_countour_mesh` from a single full-step to a symmetric half-step-place-half-step. |
| 2026-08-15 | verified | Added 1 test to `tests/ufo_text_advance_test.rs`; scoped test run (`cargo nextest run --all-features` via `longrun`) passed 9/9 alongside the pre-existing suite. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read both `text_to_mesh`'s and `text_to_countour_mesh`'s pass-2 loops (confirmed the symmetric `step`/place/`step` fix genuinely present in BOTH copies, not just one, 3-field comment intact in both) and `text_to_countour_mesh_centers_each_glyph_in_its_own_slot` (non-tautological: builds real glyphs via `Font::from_glyphs`, asserts specific slot-centered x positions against hand-derived expected values, explicitly distinguishing them from the buggy values in the failure message). Fresh `cargo nextest run -p primitive_generation --all-features` via `longrun`: 9/9 passed. `cargo clippy -p primitive_generation --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate — no overlap with BUG-127/128. Confirmed the file's own disclosed scope limitation (`text_to_mesh`'s copy of the fix is untested directly) remains accurate — no test exercises `text_to_mesh`, only `text_to_countour_mesh`. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-127/128/129 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass initially treated the test file's already-passing hand-derived values as sufficient; adversarial pass required an INDEPENDENT re-derivation via a fresh, separately-written arithmetic reproduction (not copy-pasted from the test), which reproduced both `[-0.012, 0.006, 0.018]` (pre-fix) and `[-0.015, -0.003, 0.012]` (post-fix) exactly, closing the gap between "the test asserts this" and "this is independently verifiable." | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed no `**Related Bugs:**` overlap with BUG-127/128 — distinct functions, distinct root cause (layout arithmetic vs. flatten-callback vs. attribute-name typo). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass independently re-read BOTH `text_to_mesh` and `text_to_countour_mesh` (not just the one being tested) to confirm the identical-bug claim in H3/E3, rather than trusting the confirming pass's assertion. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass explicitly checked whether pass 1 or the vertical (`y`) logic shared any related defect — confirmed both are untouched and correct; only pass 2's horizontal advance was wrong. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `primitive_generation`'s own `src/`/`tests/`/`Cargo.toml` and this bug-tracking file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to each function's own pass-2 loop; no caller-side changes needed. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix corrects existing advance arithmetic to match its own already-documented centering intent; no new responsibility added. | — |

**Reproduced:** YES — independent arithmetic reproduction of the exact pre-fix single-full-step
formula produced `[-0.012, 0.006, 0.018]` for the real 3-glyph test case (vs `[-0.015, -0.003,
0.012]` post-fix, matching the real crate's own passing test), 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/text/ufo.rs` | `text_to_mesh` and `text_to_countour_mesh`: changed pass 2's single full-slot-width advance into a symmetric half-step-place-half-step, in both functions' own copy of the loop. `Fix(BUG-129)`/`Root cause`/`Pitfall` comment added to both. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/ufo_text_advance_test.rs` | New file: `text_to_countour_mesh_centers_each_glyph_in_its_own_slot` (`bug_reproducer(BUG-129)`, 5-section doc comment), asserting all 3 glyph slot-center x positions match the hand-derived correct values within `1e-5`. |
| `module/helper/primitive_generation/Cargo.toml` | Added `[[test]] name = "ufo_text_advance_test" required-features = ["font-processing"]`. |

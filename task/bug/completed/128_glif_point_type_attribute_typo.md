# BUG-128: `Glyph::from_glif` never recognizes any `.glif` point's declared type

- **Severity:** High (silent data corruption — every UFO glyph outline loaded through the crate's
  only glyph-parsing entry point is misinterpreted, with no error, warning, or panic)
- **state:** Completed
- **Affects:** Any caller of `primitive_generation::text::ufo::Glyph::from_glif` — i.e. every
  glyph loaded through `Font::new`'s real UFO-directory pipeline, the crate's only production
  path into text rendering
- **Component:** `module/helper/primitive_generation` (`src/text/ufo.rs::Glyph::from_glif`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-127/129, filed under the same task #63
  targeted `primitive_generation` review

## Symptom

```bash
# Same 3-point curve contour (move -> offcurve -> curve), parsed by from_glif:

# Wrong (pre-fix) -- every point silently misread as PointType::Move:
Glyph::from_glif(GLIF_WITH_CURVE, 'a').contours() -> flattens to 3 points
# (3 disconnected single-point "contours" -- the curve is entirely lost)

# Correct (post-fix) -- point types honored, curve properly flattened:
Glyph::from_glif(GLIF_WITH_CURVE, 'a').contours() -> flattens to 14 points
```

## Impact

**Who is affected:** Every caller of `Glyph::from_glif` — which is every glyph loaded by
`Font::new`, the crate's only production UFO-loading entry point (confirmed: `Font::new` calls
`Glyph::from_glif` once per glyph file read from the `.ufo` directory's `glyphs/` folder).

**What breaks:** The `.glif` XML point-attribute loop matched `attr.key.0` against the byte
literal `b"typ"` instead of the UFO/glif spec's real attribute name, `type` (confirmed directly
against `norad` 0.18.4's own glif parser, which reads exactly `b"type"`). `b"typ"` can never match
a real `.glif` file's `type="..."` attribute — every point silently kept the loop's
`PointType::Move` default regardless of its actual declared type in the file.

**Magnitude:** Not a partial/degraded result — every non-trivial glyph outline (anything with a
curve, or more than one straight-line contour point) is corrupted into a series of disconnected
single-point "contours" instead of a connected outline, since `norad::Contour::to_kurbo` (per its
own match on `pt.typ`) turns every point typed `Move` into a fresh, disconnected `MoveTo` rather
than the intended `LineTo`/`QuadTo`/`CurveTo`. This silently produces near-degenerate (and, for
single-point contours, zero-area) glyph geometry for essentially every real font, with no error or
panic to signal the corruption — the pipeline completes "successfully" with wrong output.

**Entity Scope:** None — a code-level defect, not an operational-entity concern.

## How Discovered

Task #63, a targeted code review of `primitive_generation` dispatched under the standing bug-hunt
mandate. The reviewing agent flagged the `b"typ"` match arm as inconsistent with the UFO/glif
format's documented `type` attribute name. Independently re-verified before filing by direct
source reads of both the crate and the exact pinned `norad` dependency version:

```bash
$ sed -n '130,225p' module/helper/primitive_generation/src/text/ufo.rs
# confirms the pre-fix match arm: `b"typ" => { ...; typ = t; }` -- no `b"type"` arm exists

$ grep -n '"type"\|b"type"' ~/.cargo/registry/src/index.crates.io-*/norad-0.18.4/src/glyph/parse.rs
# confirms norad's own reference glif parser reads the attribute as exactly `b"type"`

$ sed -n '308,358p' ~/.cargo/registry/src/index.crates.io-*/norad-0.18.4/src/glyph/mod.rs
# confirms Contour::to_kurbo's match on pt.typ: Move => new disconnected move_to,
# Line => line_to, OffCurve => queued for a following Curve/QCurve's quad_to/curve_to
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre128 && mkdir -p /tmp/mre128/src
cat > /tmp/mre128/Cargo.toml <<'EOF'
[package]
name = "mre128"
version = "0.1.0"
edition = "2021"

[dependencies]
primitive_generation = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/primitive_generation", default-features = false, features = [ "font-processing" ] }
EOF
cat > /tmp/mre128/src/main.rs <<'EOF'
use primitive_generation::text::ufo::Glyph;

const GLIF_WITH_CURVE: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<glyph name="test" format="2">
  <outline>
    <contour>
      <point x="0" y="0" type="move"/>
      <point x="50" y="100" type="offcurve"/>
      <point x="100" y="0" type="curve"/>
    </contour>
  </outline>
</glyph>"#;

fn main()
{
  let glyph = Glyph::from_glif( GLIF_WITH_CURVE, 'a' ).expect( "well-formed glif must parse" );
  let total : usize = glyph.contours().iter().map( Vec::len ).sum();
  println!( "flattened point count: {total}" );
}
EOF
cd /tmp/mre128 && cargo run 2>&1 | tail -1
```

**Expected** (post-fix — confirmed by running this exact MRE against the real fixed crate):
```
flattened point count: 14
```

**Actual** (pre-fix — confirmed by independently reproducing the exact old `b"typ"` match arm,
byte-for-byte, against the same real `GLIF_WITH_CURVE` bytes and the same pinned
`quick-xml`/`norad` dependency versions, in an isolated scratch crate not touching the real
already-fixed source):
```
pre-fix flattened point count: 3
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre128 && cargo run 2>&1 | tail -1
# 14 = fixed; 3 = bug present
```

**Known MRE limitation (check 205):** `primitive_generation` is this workspace's own crate; the
MRE path-depends on it locally rather than a registry version, matching BUG-116/118-127's own
documented exception. The pre-fix count (3) was independently confirmed by reproducing the exact
old match arm against the real pinned `quick-xml 0.41`/`norad 0.18.4` in a separate scratch crate
(see How Discovered's methodology), not by reverting the actual crate source.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The glif point-type attribute match arm's byte literal (`b"typ"`) does not match the real UFO/glif spec's attribute name (`type`), so the `typ` loop variable never leaves its `PointType::Move` default. | ✅ Root Cause | `norad`'s own reference parser reads `b"type"`. Isolated reproduction of the exact old `b"typ"` arm against real `quick-xml`/`norad`, fed the same 3-point curve contour, produces 3 flattened points (all-Move); the real fixed crate, fed the identical input, produces 14. | E1, E2, E3 |
| H2 | Even if `typ` defaults to `Move` for every point, `norad::Contour::to_kurbo` still produces a usable (if suboptimal) path — the corruption is cosmetic, not structural. | ❌ Falsified | `to_kurbo`'s own match on `pt.typ` treats every `Move`-typed point (other than a contour's genuine first point) as the start of a brand-new, disconnected subpath — not a connected line/curve segment. A 3-point curve contour collapses to 3 disconnected single points instead of one continuous flattened curve. | E3, E4 |
| H3 | The bug is confined to curve-containing glyphs; straight-line-only glyphs (letters like "L", "I") are unaffected since `Line` and `Move` produce similar-looking output. | ❌ Falsified | `to_kurbo` treats `Move` as *always* starting a new disconnected subpath regardless of type, while `Line` connects to the previous point — a straight-line glyph with more than one contour point is equally corrupted into N disconnected single points instead of one connected polyline. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `~/.cargo/registry/.../norad-0.18.4/src/glyph/parse.rs` | `norad`'s own reference glif parser reads the point-type attribute as exactly `b"type"`, confirming the real UFO/glif spec's attribute name. | H1 ✅ |
| E2 | Isolated scratch-crate reproduction of the exact pre-fix `b"typ"` match arm, run against real `quick-xml 0.41`/`norad 0.18.4`, fed the real `GLIF_WITH_CURVE` fixture | `pre-fix flattened point count: 3` — every point silently defaulted to `PointType::Move`. | H1 ✅ |
| E3 | `~/.cargo/registry/.../norad-0.18.4/src/glyph/mod.rs`, `Contour::to_kurbo` (lines ~308-358) | Match on `pt.typ`: `Move` always starts a new disconnected `move_to`; only `Line`/`OffCurve`+`Curve`/`QCurve` connect to form a continuous path. Confirms an all-`Move` contour cannot produce a connected outline regardless of point count or straight-vs-curve content. | H2 ❌, H3 ❌ |
| E4 | Real fixed crate, same `GLIF_WITH_CURVE` fixture, via `/tmp/mre128` | `post-fix flattened point count: 14` — same input, correct types honored, produces a properly flattened continuous curve. | H1 ✅, H2 ❌ |

## Root Cause

```
Glyph::from_glif( glif_bytes, character ) -> Option<Self>
  for attr in point_element.attributes() {
    match attr.key.0 {
      b"x" => ...
      b"y" => ...
      b"typ" => { typ = PointType::from_str(&value)... }   <- never matches; real attr is b"type"
      _ => {}
    }
  }
  // typ stays PointType::Move for every point, always
```

The match arm's byte-string literal was a one-letter typo of the real UFO/glif attribute name.
Because the surrounding `match` has a silent `_ => {}` catch-all (correctly handling genuinely
unrecognized attributes), the typo produced no error, warning, or panic — it simply never fired,
and every point kept the loop's `PointType::Move` initialization value.

## Why Not Caught

No existing test exercised `Glyph::from_glif` at all — the crate's only public entry point into
UFO loading, `Font::new`/`fonts_load`, reads real `.ufo` directories via an async, browser-only
file loader (`mingl::web::file::load`) with no test fixture wired up, so the byte-level `.glif`
parser was entirely untested. The corruption also produces no error or panic — glyph loading
"succeeds" with silently wrong geometry, giving no runtime signal that anything was wrong.

## Fix Location

`module/helper/primitive_generation/src/text/ufo.rs`, `Glyph::from_glif`'s point-attribute match.
Changed the byte-string literal from `b"typ"` to `b"type"`:

```rust
// before
match attr.key.0
{
  b"x" => x = value.parse::< f64 >().ok(),
  b"y" => y = value.parse::< f64 >().ok(),
  b"typ" =>
  {
    let Ok( t ) = PointType::from_str( &value ) else { continue };
    typ = t;
  }
  _ => {}
}

// after
match attr.key.0
{
  b"x" => x = value.parse::< f64 >().ok(),
  b"y" => y = value.parse::< f64 >().ok(),
  b"type" =>
  {
    let Ok( t ) = PointType::from_str( &value ) else { continue };
    typ = t;
  }
  _ => {}
}
```

Every previously-parsed `x`/`y` coordinate is bit-for-bit unchanged; only the previously-ignored
`type` attribute is now honored.

## Prevention

Added `from_glif_honors_the_declared_curve_point_type` to the new
`tests/ufo_glif_point_type_test.rs` (gated `required-features = ["font-processing"]`). Making this
test possible at all required a minimal, justified public-API widening (see Refs: src/ below) —
`Glyph::from_glif` was private with no public field/constructor access to its parsed contours; a
workspace-wide grep confirmed zero precedent anywhere in this codebase for in-source `#[cfg(test)]`
tests, ruling out that alternative, so a `contours()` getter and a `pub` visibility change on
`from_glif` itself were added, scoped strictly to what this bug's regression test requires.

**Pitfall:** an unmatched byte-string arm in a `match` with a `_ => {}` catch-all fails silently —
it never panics or errors, it just never fires. Cross-check hardcoded attribute-name literals
against the format spec or a reference parser (`norad`, in this case), not just internal
self-consistency within the same file.

## Generalized Version

**Broken assumption:** "this byte-string literal matches the attribute name because it looks
approximately right and the code compiles" — a silently-never-firing match arm is indistinguishable
from a correctly-firing one at both compile time and (absent a dedicated test) run time, since the
surrounding catch-all swallows the mismatch without any observable signal.

**Confirmed general rule:** whenever a `match` scrutinizes a raw string/byte-string literal against
external format data (file formats, wire protocols, attribute names) and includes a silent
catch-all arm, that match is untestable-by-inspection — a typo produces no compiler error and no
runtime error, only silently wrong output. Such matches need dedicated tests asserting the
*post-parse value*, not just that parsing "succeeds" without erroring.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #63's targeted code review of `primitive_generation`; confirmed via a reference-parser cross-check (`norad`'s own glif parser) before filing. |
| 2026-08-15 | fixed | Changed the match arm in `Glyph::from_glif` from `b"typ"` to `b"type"`. |
| 2026-08-15 | verified | Added `contours()` accessor, made `from_glif` `pub`, added 1 test to `tests/ufo_glif_point_type_test.rs`; scoped test run (`cargo nextest run --all-features` via `longrun`) passed 9/9 alongside the pre-existing suite. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `Glyph::from_glif`'s attribute match (confirmed `b"type"` genuinely replaces the old `b"typ"` typo, 3-field comment intact) and `from_glif_honors_the_declared_curve_point_type` (non-tautological: parses real `.glif` XML bytes with declared `move`/`offcurve`/`curve` types, asserts the flattened point count against the expected value). Fresh `cargo nextest run -p primitive_generation --all-features` via `longrun`: 9/9 passed. `cargo clippy -p primitive_generation --all-features --all-targets -- -D warnings`: clean. `**Related Bugs:** None` confirmed accurate — no overlap with BUG-127/129. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-127/128/129 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass initially relied only on the mre128 probe's abstract `Contour::new` construction; adversarial pass required an isolated reproduction of the ACTUAL old match-arm code, byte-for-byte, parsing the ACTUAL `GLIF_WITH_CURVE` XML through real `quick-xml`, closing the gap between "the underlying norad behavior" and "this specific parser's pre-fix output" — confirmed 3 (pre) vs 14 (post) against the identical fixture. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed no `**Related Bugs:**` overlap with BUG-127/129 — distinct function, distinct file region, distinct root cause (attribute-name typo vs `unreachable!()` premise vs advance-step asymmetry). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass independently re-read `norad`'s own `parse.rs` (not just trusted the confirming pass's paraphrase) to confirm `b"type"` is the format's real attribute name. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked the rest of `from_glif`'s attribute match (`b"x"`, `b"y"`) against the same typo risk — both correct, no further hidden mismatches. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `primitive_generation`'s own `src/`/`tests/`/`Cargo.toml` and this bug-tracking file touched. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `from_glif`'s own match arm; no caller-side changes needed. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | The 3 new `pub` API additions (`contours()`, `from_glif` visibility, `Font::from_glyphs`) are scoped strictly to making this bug's own regression test possible, each with a doc comment explaining its scope — not a general API expansion. | — |

**Reproduced:** YES — isolated reproduction of the exact pre-fix `b"typ"` match arm produced 3
flattened points against the real `GLIF_WITH_CURVE` fixture (vs 14 for the real fixed crate on the
identical input), using real pinned `quick-xml 0.41`/`norad 0.18.4`, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/src/text/ufo.rs` | `Glyph::from_glif`: fixed the point-type attribute match from `b"typ"` to `b"type"`. `Fix(BUG-128)`/`Root cause`/`Pitfall` comment added. Also (testability-enabling, shared with BUG-129): added `pub fn contours(&self) -> &[Vec<[f32;2]>]`, changed `from_glif` from private to `pub fn` (with `# Panics` doc + `#[must_use]`), added `pub fn Font::from_glyphs`. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/primitive_generation/tests/ufo_glif_point_type_test.rs` | New file: `from_glif_honors_the_declared_curve_point_type` (`bug_reproducer(BUG-128)`, 5-section doc comment), asserting the flattened point count exceeds the degenerate all-Move count (3). |
| `module/helper/primitive_generation/Cargo.toml` | Added `[[test]] name = "ufo_glif_point_type_test" required-features = ["font-processing"]`. |

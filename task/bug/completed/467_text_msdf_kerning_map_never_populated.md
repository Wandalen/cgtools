# BUG-467: `text_msdf`'s `font_parse` never populates its kerning map -- every kerning pair silently dropped

- **Severity:** High (kerning data is parsed from the font JSON but never reaches the returned
  `MSDFFont`, so every consumer of `MSDFFont::kernings` reads an always-empty map -- text layout
  silently renders with zero kerning applied, for every font this crate loads)
- **state:** Completed
- **Affects:** `examples/minwebgl/text_msdf`
- **Component:** `examples/minwebgl/text_msdf/src/json.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None.

## Symptom

```rust
// pre-fix -- text_msdf/src/json.rs, font_parse
let mut kerning_map : HashMap< u8, HashMap< u8, f32 > > = HashMap::new();

// If present, build a map of offsets between possible pair of letters
for k in &res.kernings
{
  if let Some( map ) = kerning_map.get_mut( &k.first )
  {
    map.insert( k.second, k.amount );
  }
}
```

`kerning_map.get_mut( &k.first )` looks up an entry that was never inserted anywhere -- no code
path in this function ever calls `kerning_map.insert`/`entry` for the outer map before this loop
runs. `get_mut` on a key that was never inserted always returns `None`, so the `if let Some( map )`
guard never matches, for any input. Every iteration of the loop is a silent no-op: the parsed
`res.kernings` (from the font's own JSON) is read and then discarded, and `kerning_map` is returned
empty regardless of how many kerning pairs the source JSON actually contained.

## Impact

**Who is affected:** Every consumer of `MSDFFontJSON::font_parse` -- currently this crate's own
`text_msdf` demo, the only call site in the workspace.

**What breaks:** Text rendered via this crate's MSDF pipeline never applies inter-glyph kerning
adjustments, regardless of what the loaded font's own kerning table specifies. Glyph spacing is
visually uniform (advance-width only) instead of kerned, which is most noticeable on pairs a real
font typically tightens (e.g. "AV", "To", "We").

**Magnitude:** 1 function, 1 always-empty map, 0 kerning pairs ever surviving into the returned
`MSDFFont` regardless of input size.

**Entity Scope:** None -- a code-level defect confined to this crate's own JSON parsing.

## How Discovered

Found during a repo-wide bug/UX sweep of the minwebgl example crates, tracing every `HashMap`
lookup back to its corresponding insert -- `kerning_map`'s only mutation site was a `get_mut`
inside the loop meant to populate it, with no insert/entry call anywhere in the function.

## Minimum Reproducible Example

```rust
// examples/minwebgl/text_msdf/src/json.rs, inline #[cfg(test)] mod tests (this crate is a
// fn main()-only WebGL demo binary with no [lib] target, so an external tests/*.rs integration
// test cannot reach font_parse regardless of pub visibility -- see the local rulebook's Test
// Placement rule).
let font_json = r#"{ "kernings": [ { "first": 65, "second": 66, "amount": -1.5 } ] }"#;
let font = MSDFFontJSON::font_parse( font_json );
// pre-fix: font.kernings.len() == 0 -- the pair above is silently dropped.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p minwebgl_text_msdf -- json::tests::font_parse_populates_kerning_map_for_every_pair
```

## Root Cause

A lookup-only accessor (`get_mut`) was used where an upsert (`entry`/`or_default`) was needed. The
`if let Some( map ) = kerning_map.get_mut( &k.first )` guard reads as an intentional "kerning data
is optional" check, but it is dead code -- `kerning_map` starts empty and nothing ever calls
`entry`/`insert` on it before this loop, so the guard's condition is unconditionally `false` for
every possible input, not just inputs that happen to lack kerning data.

## Why Not Caught

No test file existed for this crate before this fix -- it is a `fn main()`-only WebGL demo binary
with no lib target. The bug produces no crash and no visibly-wrong glyph *positions* under casual
inspection (glyphs still render, just without kerning's typically-subtle spacing adjustment), so it
has no obvious visual symptom distinguishing it from a font that genuinely has no kerning pairs.

## Fix Location

`examples/minwebgl/text_msdf/src/json.rs`: the loop body now reads
`kerning_map.entry( k.first ).or_default().insert( k.second, k.amount );`, creating the inner map
on first use of a given `first` key instead of requiring it to already exist.

## Prevention

Added `json::tests::font_parse_populates_kerning_map_for_every_pair` -- a fixture with 3 kerning
pairs across 2 distinct `first` keys (two pairs sharing `first == 65` to exercise the outer map's
`entry` being reused across pairs, one pair with a distinct `first == 66` to confirm outer keys
stay independent), asserting the exact resulting nested-map contents for all 3 pairs.

## Pitfall

`HashMap<K, HashMap<K2, V>>` nested maps need `entry(..).or_default()` (or equivalent) at the outer
level before inserting into the inner map -- `get_mut` alone can never populate a key that was
never inserted, and silently no-ops instead of panicking, which hides the bug. A guard shaped like
`if let Some( map ) = map.get_mut( &key )` reads as defensive/optional-data handling, but is dead
code if nothing upstream ever inserts that key -- always check the corresponding insert site exists
before trusting a `get`/`get_mut` guard's apparent intent.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of minwebgl example crates; discovery, fix, and test landed together in one session. |
| 2026-08-20 | fixed | Replaced the dead `get_mut`-guarded loop body with `entry(..).or_default().insert(..)`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted `font_parse` to the pre-fix `get_mut`-guarded body (test left in place); `cargo test -p minwebgl_text_msdf` confirmed the new test fails (`kerning_map` empty, `len() == 0` instead of the expected `2`). Restored the fix; test passes. Final combined pass: `cargo test && cargo clippy --all-targets --all-features -- -D warnings && cargo check --target wasm32-unknown-unknown`, all clean (exit 0). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-467)`/`Root cause`/`Pitfall` 3-field format applied at the fix site in `json.rs`. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `font_parse`'s kerning loop body; no other function or file touched. Confirmed via re-reading the diff before verification. | — |

**Reproduced:** YES -- temporarily reverting `font_parse`'s fixed loop body back to the
`get_mut`-guarded version caused `font_parse_populates_kerning_map_for_every_pair` to fail
(`kerning_map.len()` was `0`, not the expected `2` distinct `first` keys); restoring the fix passes.
2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/text_msdf/src/json.rs` | `font_parse`'s kerning loop now uses `entry(..).or_default().insert(..)` instead of a dead `get_mut`-guarded branch. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/minwebgl/text_msdf/src/json.rs` (inline `#[cfg(test)] mod tests`, no `lib.rs` in this crate) | Added `font_parse_populates_kerning_map_for_every_pair`, asserting exact nested-map contents for 3 kerning pairs across 2 `first` keys. |

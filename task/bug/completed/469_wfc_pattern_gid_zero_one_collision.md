# BUG-469: `wfc`'s `pattern_set` encodes TMX GID 0 (empty cell) and GID 1 (first tile) to the same pixel value

- **Severity:** Low (a silent data/correctness defect, not a panic -- affects only maps that
  actually contain empty cells alongside the first tileset tile, which the crate's own bundled
  default pattern may or may not exercise depending on its layout)
- **state:** Completed
- **Affects:** `examples/minwebgl/wfc`
- **Component:** `examples/minwebgl/wfc/src/main.rs`
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** BUG-468 (same function, `pattern_set` -- separate defect: a panic-safety issue
  in the same function this fix's rewrite also passed through).

## Symptom

```rust
// pre-fix -- wfc/src/main.rs, pattern_set (condensed)
let value : u8 = gid.saturating_sub( 1 ) as u8;
pattern_raw.push( value );
```

Tiled's CSV tile-layer encoding is 1-based: GID `0` means "empty cell" (no tile placed) and GID `1`
means "the first tileset tile". `gid.saturating_sub( 1 )` maps *both* of these to pixel value `0`
-- `0_u32.saturating_sub(1) == 0` and `1_u32.saturating_sub(1) == 0` -- so an empty cell and a
placed first-tile cell become indistinguishable in the encoded pattern image.

## Impact

**Who is affected:** Any user who uploads a TMX map (via the file-upload control) whose tile layer
contains both empty cells (GID 0) and placements of the tileset's first tile (GID 1).

**What breaks:** The Wave Function Collapse algorithm reads its input pattern from this encoded
image; with the collision, it cannot distinguish "no tile here" from "the specific tile that
happens to be first in the tileset placed here" -- the generated output can only ever treat both
as the same input symbol, silently losing the distinction the source map actually encoded.

**Magnitude:** 1 line, 1 collision affecting exactly 2 of the 256 representable GID values (0 and
1) -- every other GID (2 and up, within the supported `u8` range) already maps to a distinct pixel
value with no collision.

**Entity Scope:** None -- a code-level defect confined to this crate's own TMX-to-pixel encoding.

## How Discovered

Found during a repo-wide bug/UX sweep of the minwebgl example crates, cross-checking the TMX
1-based GID convention (0 = empty, 1 = first tile) against this function's 0-based pixel encoding
and noticing `saturating_sub(1)` maps two distinct input values to the same output.

## Minimum Reproducible Example

```rust
// examples/minwebgl/wfc/src/main.rs, inline #[cfg(test)] mod tests (this crate is a
// fn main()-only WebGL demo binary with no [lib] target -- see the local rulebook's Test
// Placement rule).
let tmx = r#"<?xml version="1.0" encoding="UTF-8"?>
<map version="1.10" orientation="orthogonal" width="2" height="1">
 <layer id="1" name="Layer 1" width="2" height="1">
  <data encoding="csv">0,1</data>
 </layer>
</map>"#;
let mut state = ApplicationState { map : None, pattern_image : None };
pattern_set( tmx, &mut state );
// pre-fix: pixel(0,0) (GID 0, empty) == pixel(1,0) (GID 1, first tile) == 0 -- collision.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p wfc_example -- tests::pattern_set_distinguishes_empty_cell_from_first_tile
```

## Root Cause

`saturating_sub` was chosen to avoid an underflow panic when converting the 1-based TMX GID
convention (0 = empty) to a 0-based tileset-tile index, but its saturating behavior *at* the
boundary case (`gid == 0`) produces the same result (`0`) as the very next value (`gid == 1`),
because both `0 - 1` (which saturates to `0`) and `1 - 1` (which computes to `0` normally) land on
the identical output -- the "empty" sentinel was never given a value outside the tile-index range
that could distinguish it from a real, valid index `0`.

## Why Not Caught

No test file existed for this crate before this fix -- it is a `fn main()`-only WebGL demo binary
with no lib target. The bug is a silent data-correctness issue with no crash and no obviously-wrong
rendering to notice by casual inspection -- the WFC output still "looks plausible" even with the
collision, since it merely conflates two of many possible input symbols rather than producing an
invalid image.

## Fix Location

`examples/minwebgl/wfc/src/main.rs`: as part of the same `pattern_set` rewrite that fixes BUG-468,
GID 0 now encodes to the reserved sentinel `u8::MAX` (outside the valid tile-index range) instead
of `0`; GID >= 1 encodes to `u8::try_from( gid - 1 )` (rejecting out-of-range values gracefully
per BUG-468, rather than wrapping/truncating). GID 1 (the first tile) now correctly encodes to
index `0`, distinct from the empty-cell sentinel `u8::MAX`.

## Prevention

Added `pattern_set_distinguishes_empty_cell_from_first_tile` -- a 2-cell TMX fixture (`0,1`,
i.e. one empty cell adjacent to one first-tile placement), asserting the two resulting pixel values
are not equal, that the empty cell specifically encodes to `u8::MAX`, and that the first tile
specifically encodes to `0`.

## Pitfall

`saturating_sub` prevents an underflow *panic*, but does not by itself prevent a *collision*
between the saturated boundary value and its neighbor -- `0.saturating_sub(1) == 0` is easy to
misread as "safely handles the zero case" when what it actually does is silently alias `0` onto the
same output as `1`. When a 1-based-to-0-based conversion needs to preserve a "no value"/sentinel
case, give that sentinel a value outside the valid output range entirely (e.g. `u8::MAX` when valid
indices are known to stay under 255), not the saturated result of the arithmetic that also produces
a real value.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX sweep of minwebgl example crates; discovery, fix, and test landed together in one session, alongside the related BUG-468 fix in the same function. |
| 2026-08-20 | fixed | GID 0 now encodes to the reserved sentinel `u8::MAX`; GID >= 1 encodes to `u8::try_from( gid - 1 )`, fixing the collision at the boundary. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily reverted just the GID-to-pixel mapping line to the pre-fix `gid.saturating_sub(1) as u8` (keeping the rest of BUG-468's graceful-error-handling rewrite, and the test, in place); `cargo test -p wfc_example --bin wfc_example` confirmed `pattern_set_distinguishes_empty_cell_from_first_tile` failed (`empty_cell_pixel == first_tile_pixel == 0`, collision reproduced). Restored the fix; test passes. Final combined pass: `cargo test -p wfc_example && cargo clippy -p wfc_example --all-targets --all-features --no-deps -- -D warnings && cargo check -p wfc_example --target wasm32-unknown-unknown`, all clean (exit 0). | — |
| D2 | Fix documentation compliance | — | 🟢 | Fix context documented via this report and BUG-468's report jointly, since both land in the same rewritten function in the same pass; the mapping's `u8::MAX`-sentinel choice is self-documenting at the call site (`if gid == 0 { u8::MAX } else { .. }`). | — |
| D3 | Scope containment | — | 🟢 | Fix confined to the single GID-to-pixel mapping expression inside `pattern_set` (part of the same BUG-468 rewrite); no other function touched. Confirmed via re-reading the diff before verification. | — |

**Reproduced:** YES -- temporarily reverting the GID-to-pixel mapping to `gid.saturating_sub(1) as
u8` caused `pattern_set_distinguishes_empty_cell_from_first_tile` to fail with both pixels equal to
`0`; restoring the fix (GID 0 -> `u8::MAX`, GID >= 1 -> `gid - 1`) passes with the two pixels
distinct. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/wfc/src/main.rs` | GID-to-pixel mapping changed from `gid.saturating_sub(1) as u8` to an explicit `if gid == 0 { u8::MAX } else { u8::try_from( gid - 1 ) ... }`, as part of the same rewrite that fixes BUG-468. |

## Refs: tests/

| File | Change |
|------|--------|
| `examples/minwebgl/wfc/src/main.rs` (inline `#[cfg(test)] mod tests`, no `lib.rs` in this crate) | Added `pattern_set_distinguishes_empty_cell_from_first_tile`. |

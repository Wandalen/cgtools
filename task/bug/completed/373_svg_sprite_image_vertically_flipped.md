# BUG-373: SVG backend's draw-time Y-up→Y-down flip also mirrors already-correctly-oriented raster sprite images, rendering them upside-down

- **Severity:** Low (zero real, non-test callers anywhere in the workspace submit a
  `RenderCommand::Sprite`/batch sprite command through `adapter-svg` today — confirmed via
  exhaustive grep; `examples/scene_script/pingpong_animation` is the only real, non-test consumer
  of `SvgBackend` and draws no sprite/image content at all — but a live public-API defect
  affecting 100% of this backend's own raster-sprite draws, once any exist)
- **state:** Completed
- **Affects:** Any current or future caller that draws a `RenderCommand::Sprite` or sprite-batch
  instance through `adapter-svg` where the sprite's sheet is `ImageSource::Bitmap`/`Path` (raster)
  content — i.e. effectively all real raster-sprite usage of the SVG backend, once any exists.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/svg.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-18
- **Related Bugs:** BUG-210 (`./completed/210_webgl_bitmap_upload_y_flip_asymmetry.md`) is the same
  general category of defect (raw bitmap data is Y-down/top-origin natively; the crate's Y-up
  convention requires a compensating flip somewhere in the pipeline) in a different backend and via
  a completely different mechanism (WebGL `UNPACK_FLIP_Y_WEBGL` texture-upload flag vs. this bug's
  SVG `<use>` transform) — no overlap, cross-checked directly. BUG-374 (filed same session,
  `./completed/374_svg_sprite_use_missing_dimensions_overscale.md`) is a distinct defect at the same
  two SVG sprite call sites (missing `width`/`height` causing over-scale, not orientation) — the
  over-scale bug had been masking this one's own visual symptom during investigation (see How
  Discovered) but the two are independent root causes, independently fixed.

## Symptom

```rust
// pre-fix -- src/adapters/svg.rs, sprites_load
let img_def = format!
(
  "<symbol id=\"sprite_{}\" viewBox=\"{} {} {} {}\"><use href=\"#img_{}\" width=\"{}\" height=\"{}\"/></symbol>",
  sprite.id.inner(),
  sprite.region[ 0 ], sprite.region[ 1 ], sprite.region[ 2 ], sprite.region[ 3 ],
  sprite.sheet.inner(),
  sheet.width, sheet.height
);
```

The symbol's inner `<use href="#img_N">` (referencing the raster `<image>`) carries no counter-flip.
Every draw-time `<use href="#sprite_N">` referencing that symbol is emitted by
`transform_to_svg_static`, which always appends `scale(sx,-sy)` to convert the crate's Y-up world
convention to SVG's native Y-down convention. This is correct for vector content authored directly
in Y-up coordinates, but `<image>` elements (and the `region` rectangle selecting a sub-area of
them) are natively Y-down/top-origin already — see `SpriteAsset::region`'s own doc comment ("SVG:
`<symbol viewBox="x y w h">`"). The outer flip therefore mirrors already-correctly-oriented raster
content a second time, rendering it upside-down.

## Impact

**Who is affected:** Any caller of `adapter-svg`'s `Backend::submit` on a `RenderCommand::Sprite`
or `AddSpriteInstance` whose sprite sheet is raster (`ImageSource::Bitmap`/`Path`) content —
currently none: exhaustive grep (`grep -rln "SvgBackend\|adapter-svg"`, excluding
`tilemap_renderer`'s own `src`/`tests`) found exactly one real consumer,
`examples/scene_script/pingpong_animation`, which submits no `Sprite`/image commands at all (paths
and shapes only).

**What breaks:** every raster sprite drawn through `adapter-svg` renders vertically mirrored — the
top of the source image appears at the bottom of the rendered sprite and vice versa. Vector content
(paths, meshes) is unaffected; only raster `<image>`-backed sprites are mirrored.

**Magnitude:** 1 function (`SvgResources::sprites_load`), single root cause (missing counter-flip
on the symbol's inner `<use>`), fixed by a single added `transform` attribute.

**Entity Scope:** None — a code-level defect.

## How Discovered

Self-directed investigation into a gap documented in `roadmap.md`'s "svg adapter gaps" section:
"Image Y-flip: SVG `<image>` elements are Y-down natively; sprites rendered from them may appear
flipped." Confirmed via `SpriteAsset::region`'s doc comment (`src/assets.rs`) that `region` is
measured in the same Y-down/top-origin convention as `<image>` itself, then cross-checked
`transform_to_svg_static` and `sprites_load` directly to confirm no compensating flip existed
anywhere in the pipeline. Initial investigation attempts were confounded by BUG-374 (the sprite
`<use>` over-scale bug, fixed first): with sprites rendering at ~100x their intended size, a single
source pixel filled the entire sampled area, making any real orientation difference visually
undetectable in browser pixel readback until BUG-374 was fixed first.

## Minimum Reproducible Example

```rust
// module/helper/tilemap_renderer/src/adapters/svg.rs -- sprites_load, pre-fix
// A 2x2 bitmap: row 0 (top, as loaded) = RED,GREEN; row 1 (bottom) = BLUE,WHITE.
// pre-fix:  rendered sprite shows row 0 (RED,GREEN) at the BOTTOM, row 1 (BLUE,WHITE) at the TOP
//           -- vertically mirrored relative to the source.
// post-fix: rendered sprite shows row 0 at the TOP, row 1 at the BOTTOM -- matches source exactly.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_renderer && cargo test -p tilemap_renderer --features adapter-svg --test svg_backend_test -- sprite_symbol_use_counter_flips_image_orientation
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The draw-time outer `scale(sx,-sy)` (Y-up→Y-down world conversion) also mirrors raster `<image>` content, which is already Y-down natively, with no compensating counter-flip anywhere in the pipeline. | ✅ Root Cause | `SpriteAsset::region`'s doc comment confirms Y-down/top-origin convention matching `<image>`/viewBox; `transform_to_svg_static` unconditionally emits `scale(sx,-sy)`; `sprites_load`'s pre-fix symbol definition has no `transform` on its inner `<use href="#img_N">` at all. | E1, E2, E3 |
| H2 | The counter-flip must re-center on the sprite's own crop-window extent (`region.y`..`region.y+region.h`), not the full sheet's height, so which sub-rectangle `viewBox` selects stays unaffected. | ✅ Confirmed | Algebraic boundary-point substitution: with `flip_y = 2*region.y + region.h`, the crop window's own top/bottom boundaries map onto each other reversed; a naive full-sheet-height flip would instead select the wrong sub-rectangle for any `region.y != 0`. | E4 |
| H3 | This is the same defect as BUG-210 (WebGL bitmap upload Y-flip). | ❌ Rejected | BUG-210 is a texture-upload pixel-storage flag (`UNPACK_FLIP_Y_WEBGL`) in a completely different backend (WebGL); this bug is an SVG `<use>` transform attribute. Same general category (raster Y-down vs. crate Y-up), different backend, different mechanism, independently fixed. | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tilemap_renderer/src/assets.rs`, `SpriteAsset::region` doc comment (direct read) | "A rectangular region within a loaded image ... SVG: `<symbol viewBox="x y w h">`" — confirms `region` and `<image>` share the same Y-down/top-origin measurement convention. | H1 ✅ |
| E2 | `module/helper/tilemap_renderer/src/adapters/svg.rs`, `transform_to_svg_static` (direct read) | Unconditionally appends `scale(sx,-sy)` to every draw-time `<use href="#sprite_N">`, with no branch distinguishing raster from vector sprite content. | H1 ✅ |
| E3 | `module/helper/tilemap_renderer/src/adapters/svg.rs`, pre-fix `sprites_load` (direct read) | Symbol definition's inner `<use href="#img_N">` carries only `width`/`height`, no `transform` attribute at all. | H1 ✅ |
| E4 | Algebraic derivation (hand proof, pre-implementation) | Substituting `y = region.y` and `y = region.y + region.h` into `y' = flip_y - y` with `flip_y = 2*region.y + region.h` yields `y' = region.y + region.h` and `y' = region.y` respectively — the crop window's own boundaries swap, proving the sub-rectangle selection is preserved (reversed internally, not relocated). | H2 ✅ |
| E5 | Real-browser pixel readback (Chromium + CDP `.eval` canvas `getImageData`, via `browsee`), pre- and post-fix, on a 4-quadrant RGBW test bitmap | Pre-fix: rendered quadrant colors vertically mirrored relative to source. Post-fix: exact orientation match, all 4 quadrants in source-correct positions. | H1 ✅ |

## Root Cause

`sprites_load` builds the sprite's `<symbol>` definition with an inner `<use href="#img_N">` that
sizes the referenced `<image>` to the sheet's own pixel dimensions but applies no `transform`. Every
draw-time reference to that symbol (`cmd_sprite`, `cmd_draw_batch`) goes through
`transform_to_svg_static`, which always emits an outer `scale(sx,-sy)` to convert the crate's Y-up
world convention into SVG's native Y-down convention — correct for vector content, but it also
mirrors the already-Y-down raster `<image>` a second time, since nothing in the pipeline ever
compensates for that double conversion.

## Why Not Caught

No existing test rendered an asymmetric (non-uniform-color) bitmap and checked pixel orientation —
every pre-existing sprite test used uniform-color or string-content-only assertions, neither of
which can detect a visual mirror. Only a real-browser pixel readback could surface it, and that
readback was itself confounded by the unrelated BUG-374 over-scale defect until that bug was fixed
first (see How Discovered).

## Fix Location

`module/helper/tilemap_renderer/src/adapters/svg.rs`, `SvgResources::sprites_load`: the symbol
definition's inner `<use href="#img_N">` now carries `transform="translate(0,{flip_y})
scale(1,-1)"`, where `flip_y = 2.0 * sprite.region[1] + sprite.region[3]` — algebraically derived to
re-center the flip on the crop window's own vertical extent (`region.y`..`region.y+region.h`), not
the full sheet, so which sub-rectangle `viewBox` selects stays completely unaffected; only the
visual orientation within that already-selected window flips.

## Prevention

1 new regression test, `module/helper/tilemap_renderer/tests/svg_backend_test.rs`:
`sprite_symbol_use_counter_flips_image_orientation` — uses a deliberately non-trivial region
(`region.y = 2.0 != 0`) so the `2*region.y + region.h` formula is meaningfully exercised rather than
only its `region.y == 0` special case, asserting the symbol's inner `<use>` carries the exact
counter-flip transform (`translate(0,10) scale(1,-1)` for `region=[4,2,8,6]`).

## Pitfall

The counter-flip must be centered on the *crop window's* own extent (`2*region.y + region.h`), not
the full sheet's height — centering on the sheet instead would correctly un-mirror a full-sheet
sprite but silently select the wrong sub-rectangle for any sprite whose region doesn't start at the
sheet's own origin. Separately: this bug's own visual symptom was masked by BUG-374's over-scale
defect during investigation — when a sprite renders at ~100x its intended size, a single source
pixel fills the entire sampled area, making orientation differences visually undetectable until the
over-scale bug is fixed first. When investigating a suspected pixel-level rendering defect, rule out
gross scale/size defects on the same call path before trusting a negative visual read.

## Generalized Version

**Broken assumption:** "A single world-space-convention conversion (Y-up→Y-down) applied uniformly
at draw time is correct for all sprite content types."

**Confirmed general rule:** Raster content (bitmap images) and vector content (paths, meshes) can
have different native coordinate conventions even within the same rendering pipeline — a Y-up
world-to-device conversion correct for one can silently double-flip the other. Any adapter mixing
raster and vector content must verify each content type's own native orientation independently
before assuming a single outer transform is sufficient. (Same general lesson as BUG-210, in a
different backend and via a different mechanism.)

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found via self-directed investigation into `roadmap.md`'s documented "Image Y-flip" gap in the SVG adapter; root-caused via `SpriteAsset::region`'s doc comment plus direct reads of `transform_to_svg_static` and `sprites_load`; confirmed via algebraic boundary-point proof and real-browser pixel readback (Chromium + CDP, 4-quadrant RGBW test bitmap). Renumbered from an earlier informal "BUG-370" working label to this file only after confirming ID 370 was still unclaimed at filing time (a sibling over-scale bug found in the same investigation collided with a concurrent actor's ID 369 and was renumbered to BUG-371; this bug's own ID was unaffected). |
| 2026-08-18 | fixed | `sprites_load`'s symbol definition now emits `transform="translate(0,{flip_y}) scale(1,-1)"` on the inner `<use href="#img_N">`, with `flip_y = 2*region.y + region.h`. 1 new regression test added. |
| 2026-08-18 | verified | `cargo test -p tilemap_renderer --features adapter-svg --test svg_backend_test -- sprite_symbol_use_counter_flips_image_orientation`: 1/1 passed. Also independently confirmed via real-browser pixel readback (Chromium + CDP `.eval` canvas `getImageData`) on a 4-quadrant RGBW bitmap: post-fix orientation exactly matches the source image. |
| 2026-08-18 | renumbered | Renumbered 370→373 (file + all source/test/registry references) after a live re-verification of `task/readme.md`'s `highest_id` mid-session found it had jumped to 372: a concurrent actor had independently filed `task/verifying/370_...`/`371_...`/`372_...` (a `task_360` split series), colliding with both this bug's ID (370) and BUG-371/374's. Per this session's established policy (never touch a concurrent actor's in-flight files), this bug's own lighter-footprint references were renumbered instead; 373/374 confirmed free via a fresh live `find` immediately before renumbering. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE isolates the exact pre-fix/post-fix orientation difference for an asymmetric 2x2 test bitmap, distinct from every pre-existing SVG sprite test (all use uniform-color content or string-only assertions). | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Checked against `tilemap_renderer`'s existing sprite/orientation-adjacent bugs (BUG-210, BUG-374) via direct `bug/readme.md` grep — BUG-210 is a distinct mechanism in a different backend; BUG-374 is a distinct root cause (over-scale, not orientation) at the same call sites; no overlap. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by a doc-comment cross-reference (`SpriteAsset::region`), direct source reads of both the flip-emitting and flip-receiving code, an algebraic boundary-point proof for the fix formula (not just plausible-looking derivation), and real-browser pixel readback confirming the fix empirically. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to `sprites_load`'s symbol-definition string; `cmd_sprite`/`cmd_draw_batch`'s own draw-time `<use>` emission is untouched — re-confirmed via all 3 new/existing sprite-sizing tests passing unchanged. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `tilemap_renderer` (`src/adapters/svg.rs` + `tests/svg_backend_test.rs`); freshly grepped this round confirming `sprites_load` has no callers outside `assets_load` and no other file duplicates the symbol-definition string-building logic. | — |

**Reproduced:** YES — pre-fix, real-browser pixel readback showed vertically mirrored quadrant
colors on a 4-quadrant RGBW test bitmap; post-fix, the same readback shows an exact orientation
match, and the new `sprite_symbol_use_counter_flips_image_orientation` test passes on first run
against the derived formula. 2026-08-18.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/svg.rs` | `SvgResources::sprites_load`: symbol definition's inner `<use href="#img_N">` now carries a counter-flip `transform` (`Fix(BUG-373)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/svg_backend_test.rs` | Added `sprite_symbol_use_counter_flips_image_orientation`. |

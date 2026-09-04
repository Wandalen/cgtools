# BUG-374: SVG sprite draw-time `<use>` carries no explicit `width`/`height`, defaulting to 100% of the viewport and compounding multiplicatively with the transform's own scale

- **Severity:** Low (zero real, non-test callers anywhere in the workspace submit a
  `RenderCommand::Sprite`/batch sprite command through `adapter-svg` today — confirmed via
  exhaustive grep; `examples/scene_script/pingpong_animation` is the only real, non-test consumer
  of `SvgBackend` and draws no sprite/image content at all — but a live public-API defect affecting
  100% of this backend's own sprite draws, once any exist)
- **state:** Completed
- **Affects:** Any current or future caller that draws a `RenderCommand::Sprite` or sprite-batch
  instance through `adapter-svg` where the sprite's `<symbol>` has a `viewBox` (always true —
  `sprites_load` always sets one) — i.e. effectively all real sprite usage of the SVG backend, once
  any exists.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/svg.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-18
- **Related Bugs:** BUG-373 (`./completed/373_svg_sprite_image_vertically_flipped.md`, filed same
  session) is a distinct defect at the same two SVG sprite call sites (missing counter-flip causing
  vertical mirroring, not scale) — this bug's over-scale had been masking BUG-373's own visual
  symptom during investigation (a sprite rendered at ~100x its intended size samples only a single
  source pixel, hiding any orientation difference) but the two are independent root causes,
  independently fixed; this one was fixed first. BUG-240
  (`../completed/240_native_backend_sprite_quad_unscaled_by_region.md`) is the same general category
  of defect (a backend's sprite draw silently diverging from the region-scaled size convention the
  other backends share) in `NativeBackend` instead of `SvgBackend`, via a completely different
  mechanism (unscaled local quad geometry vs. this bug's missing SVG attribute) — no overlap,
  cross-checked directly. No other `tilemap_renderer`-component bug touches sprite `<use>` sizing.

## Symptom

```rust
// pre-fix -- src/adapters/svg.rs, cmd_sprite
let use_el = format!( "<use href=\"#sprite_{}\"{transform}{clip}{tint}/>", s.sprite.inner() );
```

The draw-time `<use href="#sprite_N">` carries no `width`/`height` attribute. Per SVG 1.1/2, a
`<use>` referencing a `<symbol>` that has a `viewBox` but no explicit size on the `<use>` itself
defaults to 100% of the *containing viewport* — not the symbol's own `viewBox` size. `sprites_load`
always gives every sprite symbol a `viewBox` (sized to `region`'s pixel dimensions), so every
draw-time `<use>` silently triggers this 100%-of-viewport auto-fit. That auto-fit scale then
compounds multiplicatively with the `<use>`'s own explicit `transform` (`transform_to_svg_static`'s
world-to-SVG `scale(sx,-sy)`), producing a gross over-scale — 100x or more in a typical
200px-viewport / 2px-sprite case — that renders sprites as a solid-color blob deep inside a single
source pixel rather than at their intended on-screen size.

## Impact

**Who is affected:** Any caller of `adapter-svg`'s `Backend::submit` on a `RenderCommand::Sprite` or
`AddSpriteInstance` — currently none: exhaustive grep (`grep -rln "SvgBackend\|adapter-svg"`,
excluding `tilemap_renderer`'s own `src`/`tests`) found exactly one real consumer,
`examples/scene_script/pingpong_animation`, which submits no `Sprite`/image commands at all (paths
and shapes only).

**What breaks:** every sprite drawn through `adapter-svg` renders at a wildly incorrect on-screen
size — the sprite's own `viewBox`-sized `<use>` auto-fits to 100% of the SVG root's `width`/`height`
(e.g. 200x200 for a typical test viewport) and *then* the draw-time `transform`'s scale is applied
on top of that already-wrong size, compounding the error multiplicatively rather than additively. A
`Transform`/`region` combination that would render at the correct size on WebGL/WebGPU renders at a
completely different, viewport-and-sprite-size-dependent size on this backend.

**Magnitude:** 2 call sites (`cmd_sprite`, `cmd_draw_batch`), same shared root cause (missing
`width`/`height` on the draw-time `<use>`), each fixed independently since the two build their
`<use>` strings via separate code paths.

**Entity Scope:** None — a code-level defect.

## How Discovered

Self-directed investigation into a gap documented in `roadmap.md`'s "svg adapter gaps" section
concerning SVG `<image>` Y-flip behavior (see BUG-373). While setting up a real-browser pixel
readback to verify sprite orientation, the rendered sprite occupied nearly the entire test viewport
instead of its configured small size — visibly wrong independent of orientation. Traced to the
draw-time `<use>` string in both `cmd_sprite` and `cmd_draw_batch` carrying no `width`/`height`,
confirmed against the SVG 1.1/2 specification's documented default-sizing behavior for a
`<use>`-referencing-`<symbol>-with-viewBox` with no explicit size on the `<use>` itself.

## Minimum Reproducible Example

```rust
// module/helper/tilemap_renderer/src/adapters/svg.rs -- cmd_sprite, pre-fix
// sprite region = 16x16px, viewport = 800x600, transform.scale = [100,100]
// pre-fix:  <use href="#sprite_0" transform="...scale(100,-100)"/>  (no width/height)
//           -- <use> auto-fits its *unset* size to 100% of the 800x600 viewport, THEN the
//              draw-time scale(100,-100) is applied on top -- compounding, not the intended size.
// post-fix: <use href="#sprite_0" width="16" height="16" transform="...scale(100,-100)"/>
//           -- <use> is explicitly sized to the region's native 16x16px, matching every other
//              backend's convention (region size * transform.scale = intended on-screen size).
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_renderer && cargo test -p tilemap_renderer --features adapter-svg --test svg_backend_test -- sprite_use_carries_explicit_dimensions_matching_region sprite_batch_use_carries_explicit_dimensions_matching_region
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | The draw-time `<use href="#sprite_N">` at both `cmd_sprite` and `cmd_draw_batch` carries no explicit `width`/`height`, triggering SVG's 100%-of-viewport auto-fit default. | ✅ Root Cause | Direct read of both call sites' pre-fix `<use>`-building code shows no `width`/`height` attribute anywhere in either format string; SVG 1.1/2 spec confirms the 100%-of-viewport default for an unsized `<use>` referencing a `viewBox`-bearing `symbol`. | E1, E2 |
| H2 | The two call sites (`cmd_sprite`, `cmd_draw_batch`) require two independent fixes since they build their `<use>` strings via separate code paths. | ✅ Confirmed | `cmd_sprite` and `cmd_draw_batch` are distinct functions with independently-constructed format strings — no shared helper existed prior to the fix; each needed its own `sprite_dims` lookup and its own regression test. | E1, E3 |
| H3 | `cmd_draw_batch`'s call site needs the same existence guard as `cmd_sprite`'s (`sprite_defs.contains`, from BUG-209) before this fix can apply. | ❌ Rejected | `cmd_draw_batch` has no pre-existing existence guard at all (unlike `cmd_sprite`'s BUG-209 check) — adding one would be new, unrelated scope; the fix instead uses `.unwrap_or((1.0,1.0))`, matching this call site's existing dangling-reference behavior for that already-separate gap. | E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tilemap_renderer/src/adapters/svg.rs`, pre-fix `cmd_sprite` (direct read) | `<use href="#sprite_{}" {transform}{clip}{tint}/>` — no `width`/`height` term anywhere in the format string. | H1 ✅ |
| E2 | SVG 1.1 §5.6 / SVG 2 §5.6 (`<use>` element sizing rules, cited from specification) | A `<use>` referencing a `<symbol>` with a `viewBox` but no explicit `width`/`height` on the `<use>` itself defaults to 100% of the containing viewport, not the symbol's own `viewBox` size. | H1 ✅ |
| E3 | `module/helper/tilemap_renderer/src/adapters/svg.rs`, pre-fix `cmd_draw_batch` (direct read) | Independent `<use href="#sprite_{}" ...>` construction inside the batch-instance loop, textually separate from `cmd_sprite`, also missing `width`/`height`; no `sprite_defs.contains`-style guard present at this call site. | H1 ✅, H2 ✅, H3 ✅ |
| E4 | Real-browser pixel readback (Chromium + CDP `.eval` canvas `getImageData`, via `browsee`), pre- and post-fix | Pre-fix: sprite rendered as a viewport-filling solid-color blob. Post-fix: sprite renders at the size implied by `region` pixel dimensions × `transform.scale`, matching WebGL/WebGPU/`NativeBackend`'s shared convention. | H1 ✅ |

## Root Cause

Neither `cmd_sprite` nor `cmd_draw_batch` emitted an explicit `width`/`height` on the draw-time
`<use href="#sprite_N">` element. `sprites_load` always gives every sprite's `<symbol>` a `viewBox`
(sized to the sprite's `region` pixel dimensions), and per the SVG specification, a `<use>`
referencing such a symbol with no explicit size of its own defaults to 100% of the *containing
viewport* — never the symbol's own `viewBox` size. That auto-fit scale then compounds
multiplicatively with the `<use>`'s own explicit `transform` (the draw call's world-to-SVG
`scale(sx,-sy)`), rather than the two combining to produce the intended on-screen size.

## Why Not Caught

Every existing sprite test asserted only on the `<use href="#sprite_N"` prefix or a match count,
never on the presence of an explicit `width`/`height` attribute — so the auto-fit fallback was
silently exercised without any test noticing which SVG default it triggered. No pixel-render
infrastructure exists in this crate's unit tests; only a real-browser pixel readback (external to
the test suite) could surface the visual effect.

## Fix Location

`module/helper/tilemap_renderer/src/adapters/svg.rs`:
- Added `SvgResources::sprite_dims : IntMap<ResourceId<asset::Sprite>, (f32, f32)>`, populated in
  `sprites_load` alongside the existing `sprite_defs` set, storing each sprite's region pixel
  `(width, height)`.
- `cmd_sprite`: emits `width="{w}" height="{h}"` on the draw-time `<use>`, looked up via
  `.expect(...)` (safe — the pre-existing `sprite_defs.contains` guard, from BUG-209, guarantees
  `sprite_dims` was populated alongside it).
- `cmd_draw_batch`: emits the same `width`/`height`, looked up via `.unwrap_or((1.0,1.0))` (no
  pre-existing existence guard at this call site, so this fallback matches its existing
  dangling-reference behavior for that already-separate, unrelated gap rather than introducing a new
  panic path).

## Prevention

2 new regression tests, `module/helper/tilemap_renderer/tests/svg_backend_test.rs`:
- `sprite_use_carries_explicit_dimensions_matching_region` (`cmd_sprite` path) — asserts the
  draw-time `<use>` carries `width`/`height` matching the region's pixel size exactly.
- `sprite_batch_use_carries_explicit_dimensions_matching_region` (`cmd_draw_batch` path) — same
  assertion at the independent batch-instance call site. Initially failed with the fallback's
  `width="1" height="1"` instead of the expected region size; traced to the test's asset bitmap
  bytes being deliberately malformed (a byte-count mismatch copy-pasted from a pre-existing sibling
  test, `sprite_batch_create_draw`, that `bitmap_to_png` silently rejects — see Pitfall) rather than
  a defect in the fix itself; corrected to a properly-sized buffer, after which the test passes
  against the fix as originally implemented.

## Pitfall

`bitmap_to_png` (`images_load`) silently returns `None` and skips registration entirely when a
bitmap's byte count doesn't match `width*height*4` — no warning, no error, the image and any sprite
referencing it simply never appear in `defs`. The pre-existing `sprite_batch_create_draw` test uses
exactly this malformed pattern and "passes" only because it asserts exclusively on `body` (which
unconditionally emits the `<use>` reference regardless of whether the referenced `<symbol>` actually
exists in `defs`), never on `defs` itself — silently exercising a dangling SVG reference without
noticing. When writing a new sprite test by copying an existing one's asset setup, verify the
bitmap byte count actually matches `width*height*4`, and assert on `defs` (not just `body`) if the
symbol's own existence matters to the test.

## Generalized Version

**Broken assumption:** "Referencing a `<symbol>` via `<use>` inherits that symbol's own `viewBox`
size automatically."

**Confirmed general rule:** SVG's `<use>` sizing is independent of the referenced `<symbol>`'s
`viewBox` — an unsized `<use>` defaults to 100% of its own containing viewport, not the symbol's
intrinsic size. Any code generating `<use>` elements against `viewBox`-bearing symbols must emit
explicit `width`/`height` matching the intended size; omitting them is not a no-op, it silently
substitutes a viewport-relative default that compounds with any additional transform scale already
applied.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Found via self-directed investigation into `roadmap.md`'s documented SVG adapter gaps, while setting up a real-browser pixel readback for BUG-370; traced to both `cmd_sprite` and `cmd_draw_batch` omitting `width`/`height` on their draw-time `<use>` elements, confirmed against the SVG specification's documented default-sizing behavior. Originally filed under a working label of "BUG-369"; renumbered to 371 after a concurrent actor independently claimed ID 369 for an unrelated task (`task/draft/369_register_curve_surface_rendering_from_angle_y_fix_closes_bug311_split_of_task_360.md`) partway through this session — since this bug's references were already embedded across `src/adapters/svg.rs` and `tests/svg_backend_test.rs` while the concurrent claim was a single fresh draft file, this bug's own references were renumbered rather than the concurrent actor's in-flight work. |
| 2026-08-18 | fixed | Added `SvgResources::sprite_dims` bookkeeping, populated in `sprites_load`; `cmd_sprite` and `cmd_draw_batch` both now emit explicit `width`/`height` on their draw-time `<use>`. 2 new regression tests added (1 initially failed due to an unrelated test-authoring defect in its own asset bytes, corrected same session — see Prevention). |
| 2026-08-18 | verified | `cargo test -p tilemap_renderer --features adapter-svg --test svg_backend_test -- sprite_use_carries_explicit_dimensions_matching_region sprite_batch_use_carries_explicit_dimensions_matching_region`: 2/2 passed. Also independently confirmed via real-browser pixel readback: sprites render at their intended region-scaled size post-fix, no longer as a viewport-filling blob. |
| 2026-08-18 | renumbered | Renumbered 371→374 (file + all source/test/registry references) after a live re-verification of `task/readme.md`'s `highest_id` mid-session found it had jumped to 372: a concurrent actor had independently filed `task/verifying/370_...`/`371_...`/`372_...` (a `task_360` split series), colliding with both this bug's ID (371) and BUG-373's (370). Per this session's established policy (never touch a concurrent actor's in-flight files), this bug's own references were renumbered instead; 373/374 confirmed free via a fresh live `find` immediately before renumbering. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE isolates the exact compounding-scale mechanism (unset `<use>` size defaulting to viewport, then multiplied by `transform`'s own scale), distinct from every pre-existing SVG sprite test (all assert only on `<use>` presence/count, never size). | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Checked against `tilemap_renderer`'s existing sprite-sizing-adjacent bugs (BUG-240, BUG-373) via direct `bug/readme.md` grep — BUG-240 is the same general category (region-scaled sizing convention) in a different backend (`NativeBackend`) via a completely different mechanism (quad geometry vs. SVG attribute); BUG-373 is a distinct root cause (orientation, not scale) at the same call sites; no overlap. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct source reads of both affected call sites, the SVG 1.1/2 specification's documented default-sizing rule (not assumed behavior), and real-browser pixel readback confirming the fix empirically restores the intended on-screen size. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to `sprite_dims` bookkeeping plus one added attribute at each of the 2 call sites; each site's existing guard/fallback semantics (BUG-209's `.expect` guard at `cmd_sprite`, the pre-existing dangling-reference fallback at `cmd_draw_batch`) preserved exactly, not altered. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `tilemap_renderer` (`src/adapters/svg.rs` + `tests/svg_backend_test.rs`); freshly grepped this round confirming `cmd_sprite`/`cmd_draw_batch` have no callers outside `submit`'s own command dispatch and no other file duplicates their `<use>`-string-building logic. | — |

**Reproduced:** YES — pre-fix, real-browser pixel readback showed a viewport-filling solid-color
blob instead of the sprite's intended small size; post-fix, the same readback shows the sprite at
its correctly region-scaled size, and both new regression tests pass against the derived fix.
2026-08-18.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/svg.rs` | Added `SvgResources::sprite_dims`; `cmd_sprite` and `cmd_draw_batch` both emit explicit `width`/`height` on their draw-time `<use>` (`Fix(BUG-374)` comment blocks). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/svg_backend_test.rs` | Added `sprite_use_carries_explicit_dimensions_matching_region` and `sprite_batch_use_carries_explicit_dimensions_matching_region`. |

# BUG-210: `bitmap_texture_upload` never sets `UNPACK_FLIP_Y_WEBGL`, so `ImageSource::Bitmap` sprites render upside-down

- **Severity:** Medium (visually wrong output for one of two supported image-source paths, no
  crash or data loss)
- **state:** Completed
- **Affects:** Every `adapter-webgl` caller loading an image via `ImageSource::Bitmap` (sync, raw
  bytes) and drawing it through a `Sprite` command — the async `ImageSource::Path` path was
  already correct.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/webgl.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-16
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** Found in the same session as BUG-209/BUG-211 (same crate, same audit pass);
  independent root cause and fix, no shared code path with either.

## Symptom

```rust
// pre-fix -- webgl.rs, bitmap_texture_upload (ImageSource::Bitmap sync upload path)
gl.pixel_storei( gl::UNPACK_ALIGNMENT, unpack_alignment );
gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
  gl::TEXTURE_2D, 0, gl_fmt as i32, width, height, 0, gl_fmt, gl::UNSIGNED_BYTE, Some( &bytes_owned ),
).unwrap();
gl.pixel_storei( gl::UNPACK_ALIGNMENT, 4 ); // restored; UNPACK_FLIP_Y_WEBGL never touched
```

A sprite loaded via `ImageSource::Bitmap` and drawn through `Sprite`/`ScreenSpaceSprite` rendered
vertically flipped relative to the same image loaded via `ImageSource::Path`.

## Impact

**Who is affected:** Every caller mixing `ImageSource::Bitmap` (e.g. procedurally generated
textures, embedded/compiled-in image bytes, or any caller supplying raw decoded pixels
synchronously) with the sprite-drawing commands — the two supported image sources silently
disagreed on vertical orientation.

**What breaks:** `sprite.vert`/`sprite_batch.vert` compute `v_uv.y = 1 - (region.y + (1 -
quad.y) * region.h) / tex.y` — a formula that only produces correct output when the underlying
texture was uploaded with `UNPACK_FLIP_Y_WEBGL = 1` (the `ImageSource::Path` convention, via
`minwebgl::texture::d2::upload`). `bitmap_texture_upload` left the flag at its WebGL default (`0`),
so every `Bitmap`-sourced sprite rendered upside-down through this same shader math.

**Magnitude:** 1 function (`bitmap_texture_upload`), 1 missing `pixel_storei` call.

**Entity Scope:** None — a code-level defect.

## How Discovered

This session's audit of `tilemap_renderer`'s WebGL adapter, cross-checking every place
`pixel_storei`/texture-upload state is set against the sprite shaders' own UV-flip assumption —
found via direct comparison of `bitmap_texture_upload`'s upload call against
`minwebgl::texture::d2::upload`'s (the `Path`-path uploader), which already sets the flag.

## Minimum Reproducible Example

Not unit-testable in this crate (no live `WebGl2RenderingContext` in `cargo test`, see
Prevention). Reproduction is by direct source inspection:

```rust
// webgl.rs -- ImageSource::Bitmap path (bitmap_texture_upload), pre-fix
// no gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, ... ) call anywhere in the function

// contrast: ImageSource::Path path, via minwebgl::texture::d2::upload (unchanged, already correct)
// -- sets UNPACK_FLIP_Y_WEBGL = 1 before its own tex_image_2d call
```

**Verify Command** (<=3 lines, standalone — source-inspection check, no live GL context available):
```bash
cd module/helper/tilemap_renderer && grep -n "UNPACK_FLIP_Y_WEBGL" src/adapters/webgl.rs
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `bitmap_texture_upload` never sets `UNPACK_FLIP_Y_WEBGL`, while the sprite shaders' UV formula assumes it is always set (matching the `Path`-path convention). | ✅ Root Cause | Direct read of `bitmap_texture_upload` (no `pixel_storei(UNPACK_FLIP_Y_WEBGL, ...)` call anywhere) versus `minwebgl::texture::d2::upload` (sets it to 1) and `sprite.vert`'s `v_uv.y` formula (only correct under the flipped convention). | E1, E2, E3 |
| H2 | `mesh.vert` is also affected, since it passes `a_uv` straight through. | ❌ Falsified — scope note, not a defect | `mesh.vert` doesn't compensate for either convention; it was already correct-by-construction for callers authoring UVs to match whichever upload path they use. Not a bug — a pre-existing authoring contract, out of scope for this fix (see Fix Location). | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tilemap_renderer/src/adapters/webgl.rs`, `bitmap_texture_upload` (pre-fix, direct read) | No `pixel_storei(gl::UNPACK_FLIP_Y_WEBGL, ...)` call anywhere in the function body. | H1 ✅ |
| E2 | `minwebgl::texture::d2::upload` (dependency source, direct read) | Sets `UNPACK_FLIP_Y_WEBGL = 1` before its own `tex_image_2d` call — the convention `ImageSource::Path`-sourced textures already follow. | H1 ✅ |
| E3 | `module/helper/tilemap_renderer/src/adapters/webgl.rs` (or its shader source), `sprite.vert`/`sprite_batch.vert` | `v_uv.y = 1 - (region.y + (1 - quad.y) * region.h) / tex.y` — only produces the documented, correct output when the source texture was uploaded with the flip flag set. | H1 ✅ |
| E4 | `module/helper/tilemap_renderer/src/adapters/webgl.rs` (or its shader source), `mesh.vert` | Passes `a_uv` through unchanged — no compensation for either upload convention, by design; this is a caller-authoring contract, not a defect. | H2 ❌ |

## Root Cause

`bitmap_texture_upload` (the synchronous, raw-bytes `ImageSource::Bitmap` upload path) never
called `gl.pixel_storei(gl::UNPACK_FLIP_Y_WEBGL, 1)`, unlike the asynchronous
`ImageSource::Path` path (`minwebgl::texture::d2::upload`), which does. Both upload paths feed
the same sprite shaders, which are written against exactly one of the two possible conventions —
the flipped one. The two upload paths silently disagreed on which convention they produced.

## Why Not Caught

No automated test exists for either WebGL upload path — `tests/webgl_backend_test.rs` has zero
live `WebGl2RenderingContext` tests by this crate's own established design (constructing a real
`WebGlBackend` requires a browser GL context this workspace's `cargo test` infrastructure cannot
provide). The asymmetry was only visible by directly comparing the two upload functions' source
against each other and against the shader's own UV-flip assumption — exactly the audit that
found it this session.

## Fix Location

`module/helper/tilemap_renderer/src/adapters/webgl.rs`: `bitmap_texture_upload` now calls
`gl.pixel_storei(gl::UNPACK_FLIP_Y_WEBGL, 1)` immediately before its
`tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array` call, and restores
it to `0` (the WebGL default) immediately afterward — mirroring the function's own existing
`UNPACK_ALIGNMENT` set/restore pattern in the same function, so the fix introduces no new
state-management convention. `mesh.vert`'s pass-through behavior (Hypothesis H2) was confirmed
correct-by-design and deliberately left unchanged.

## Prevention

No automated regression test — `webgl_backend_test.rs` has zero live `WebGl2RenderingContext`
tests by this crate's own established design (see Why Not Caught; the same limitation already
documented for BUG-209's WebGL half and BUG-200's history). The fix is a direct, minimal,
single-call addition matching an already-correct sibling convention in the same function
(`UNPACK_ALIGNMENT`'s existing restore pattern) and the already-correct `Path`-path uploader's
own established behavior — reducing but not eliminating the residual risk left by the missing
live test. `tilemap_renderer/readme.md`'s "WebGL texture upload Y-flip asymmetry" section
(now marked "fixed, BUG-210") documents the fix and both upload paths' now-shared convention for
future readers, so the asymmetry cannot silently regress unnoticed by any contributor reading
that section before touching either upload path.

## Pitfall

Two functions that upload to the same GPU resource type (a 2D texture) and feed the same
consuming shaders can still silently disagree on GL-state conventions like unpack flags — neither
function's own type signature or the shader's own correctness on ONE path proves consistency
across BOTH paths. A per-path state-setting convention (like `UNPACK_FLIP_Y_WEBGL`) has to be
checked at every upload call site individually, not assumed shared once one site is known
correct.

## Generalized Version

**Broken assumption:** "if the shader renders correctly for images loaded through the common
path, it renders correctly for images loaded through any path."

**Confirmed general rule:** GL unpack state (`pixel_storei` flags) is call-site-scoped, not
texture-type-scoped or shader-scoped — every distinct upload code path touching the same GL
context must independently set every flag the consuming shader assumes, since GL provides no
mechanism to enforce or inherit that convention across call sites.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-16 | filed | Found via this session's WebGL adapter audit, comparing `bitmap_texture_upload` against `minwebgl::texture::d2::upload` and the sprite shaders' own UV-flip assumption. |
| 2026-08-16 | fixed | Added `gl.pixel_storei(gl::UNPACK_FLIP_Y_WEBGL, 1)` before the upload call and restored to `0` after, mirroring the function's existing `UNPACK_ALIGNMENT` restore pattern. |
| 2026-08-16 | documented | `tilemap_renderer/readme.md`'s pre-existing "WebGL texture upload Y-flip asymmetry" section updated: heading marked "(fixed, BUG-210)", prose converted to past tense, speculative "**Fix**:" paragraph replaced with a "**Fixed**:" paragraph describing the applied change. |
| 2026-08-17 | verified | `cargo nextest run -p tilemap_renderer --all-features --no-fail-fast`: 144/144 passed, 0 skipped. `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings`: clean. Fix itself verified by direct source inspection only — no live-`WebGl2RenderingContext` test exists in this crate to run (see Prevention section). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present, including an explicit no-test-coverage note rather than a fabricated test claim. | — |
| D2 | MRE Validity & Reproducibility | 🟠 | 🟢 | Confirming pass initially reached for a unit-test MRE. Adversarial pass checked whether a live-GL-context test is actually possible in this crate — re-confirmed via `webgl_backend_test.rs`'s own top-of-file doc comment that it is not — and corrected the MRE to a source-inspection form instead of fabricating an untestable claim. | Replaced the planned unit-test MRE with a direct `grep`-verifiable source-inspection MRE. |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly distinguished from BUG-209 (different root cause, different functions, no shared code path) despite being found in the same audit pass; correctly cross-referenced to the already-updated `tilemap_renderer/readme.md` section. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct comparison of both upload functions' source plus the consuming shader's own UV formula — not assumed from the readme's pre-existing (accurate) description alone. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to the single missing `pixel_storei` call plus its restore; `mesh.vert`'s pass-through behavior (Hypothesis H2) explicitly investigated and confirmed out of scope, not silently left unexamined. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `tilemap_renderer`; no downstream crate changes needed. | — |

**Reproduced:** Confirmed via direct source inspection only (both pre-fix asymmetry and post-fix
symmetry) — no live-`WebGl2RenderingContext` test exists in this crate to produce a pass/fail
signal (see Prevention). 2026-08-16/17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/webgl.rs` | `bitmap_texture_upload`: added `gl.pixel_storei(gl::UNPACK_FLIP_Y_WEBGL, 1)` before the `tex_image_2d` call, restored to `0` after (full `Fix(BUG-210)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| — | None — no live `WebGl2RenderingContext` test exists in this crate; see Prevention section for why and what mitigates the residual risk. |

## Refs: docs/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/readme.md` | "WebGL texture upload Y-flip asymmetry" section: heading marked "(fixed, BUG-210)"; prose converted to past tense; speculative "**Fix**:" paragraph replaced with a "**Fixed**:" paragraph describing the applied change. |

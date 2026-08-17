# BUG-240: `NativeBackend`'s sprite quad ignores `region` pixel size and anchors at the sprite's center instead of a corner, diverging from WebGL/WebGPU/SVG

- **Severity:** Low (zero real, non-test callers of `NativeBackend` exist anywhere in the
  workspace today, confirmed via exhaustive grep — but a live public-API defect affecting 100% of
  this backend's own sprite draws, natively testable with no browser/GPU-context caveats)
- **state:** Completed
- **Affects:** Any current or future caller that draws a `RenderCommand::Sprite` through
  `adapter-native` where the sprite's `region` pixel size differs from `1x1`, or where corner-vs-
  center placement matters — i.e. effectively all real `NativeBackend` sprite usage, once any
  exists.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/native.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Related Bugs:** Found during this session's `tilemap_renderer` crate scout (task #173),
  immediately after closing BUG-239. Checked against the crate's 5 existing
  `module/helper/tilemap_renderer`-component bugs (BUG-153, 209, 210, 211, 239) — BUG-210 is the
  only other bug touching sprite-quad rendering (WebGL texture-upload `UNPACK_FLIP_Y_WEBGL`), a
  completely different mechanism (texture sampling, not vertex geometry) in a different backend;
  no overlap.

## Symptom

```rust
// pre-fix -- src/adapters/native.rs, NativeBackend::quad_vertices
let corners = [ ( -0.5f32, -0.5f32 ), ( 0.5, -0.5 ), ( 0.5, 0.5 ), ( -0.5, 0.5 ) ];
for ( i, ( lx, ly ) ) in corners.into_iter().enumerate()
{
  let world_x = m[ 0 ] * lx + m[ 3 ] * ly + m[ 6 ];
  let world_y = m[ 1 ] * lx + m[ 4 ] * ly + m[ 7 ];
  // ...
}
```

With `region = [0,0,8,8]` (an 8x8-pixel sprite), `scale = [1,1]`, `position = [0,0]`: the pre-fix
world-space footprint spans `[-0.5, 0.5]` on each axis (extent 1, centered at `position`). Every
other backend's equivalent computation (`webgl.rs`'s `sprite.vert`: `world = u_transform *
vec3(quad * u_region.zw, 1.0)`; `webgpu.rs`'s WGSL `vs_main`, identical shape; `svg.rs`'s
`sprites_load`, sizing a `<symbol viewBox="region...">` to the region's own pixel dimensions)
instead scales the local quad by `region`'s pixel size before the transform, giving a footprint of
`[0, 8]` (extent 8, anchored at `position` as a corner, not centered on it).

## Impact

**Who is affected:** Any caller of `adapter-native`'s `Backend::submit` on a `RenderCommand::Sprite`
where `region`'s pixel width/height isn't exactly `1`, or where `Transform::position` is expected
to land on a specific sprite corner rather than its center — currently none: exhaustive grep
(`grep -rln "NativeBackend" module/ examples/`, excluding `adapters/native.rs` itself and
`tests/`) found exactly one hit, a doc-comment cross-reference in `webgpu.rs`, and zero real
construction/usage sites anywhere in the workspace.

**What breaks:** every sprite drawn through `NativeBackend` renders at the wrong on-screen size
(`Transform::scale` alone instead of `region.{width,height} * scale`) and, independently, anchored
at the wrong point (`Transform::position` as the sprite's center instead of one of its corners) —
both diverge from the WebGL/WebGPU/SVG-consistent convention. A `Transform` value that renders
correctly on any other backend renders at the wrong size and place on this one.

**Magnitude:** 1 function (`NativeBackend::quad_vertices`), single shared root cause producing two
observable symptoms (size, anchor) — both fixed by the same 2-line change to how the local quad's
already-existing `fx`/`fy` values feed the transform.

**Entity Scope:** None — a code-level defect.

## How Discovered

This session's `tilemap_renderer` crate scout (task #173), reading `src/adapters/native.rs`
immediately after closing BUG-239. `quad_vertices`'s local quad (`[-0.5, 0.5]`, unscaled by
`region`) looked suspicious against the crate's own doc comment claiming `NativeBackend` uses "the
same minimal command family the WebGPU adapter translates" — a claim about command support, not
per-vertex geometry. Cross-checked by directly reading all 3 sibling backends' actual
vertex-generation code: `webgl.rs`'s `adapters/shaders/sprite.vert` (region-scaled, `[0,1]`-ranging
quad — explicit comment: "Scale unit quad to sprite's pixel size (region.zw), then apply
transform"), `webgpu.rs`'s inline WGSL `vs_main` (`let scaled = local_position *
uniforms.region.zw;`, identical shape), and `svg.rs`'s `sprites_load` (`<symbol id="sprite_N"
viewBox="region.x region.y region.w region.h">` — the symbol's own coordinate system is sized to
the region). All three agree with each other and disagree with `native.rs`. The anchor-point
divergence was confirmed separately by direct matrix substitution: at the WebGL/WebGPU quad's
`(0,0)` corner, `world = transform * (0,0,1)` reduces to exactly the transform's translation term
(`Transform::position`) with no other contribution — proving `position` is that corner, never the
center, on those two backends; `NativeBackend`'s corners are symmetric `±0.5` around `(0,0)`,
proving `position` is the center there.

## Minimum Reproducible Example

```rust
// module/helper/tilemap_renderer/src/adapters/native.rs -- quad_vertices, pre-fix
let t = Transform { position : [ 0.0, 0.0 ], scale : [ 1.0, 1.0 ], ..Default::default() };
let region = [ 0.0, 0.0, 8.0, 8.0 ]; // an 8x8-pixel sprite region
// pre-fix:  world-space footprint spans [-0.5, 0.5]  on each axis (extent 1, centered at position)
// post-fix: world-space footprint spans [ 0.0, 8.0 ] on each axis (extent 8 = region size * scale,
//           anchored at position as a corner) -- matches sprite.vert / vs_main / sprites_load.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/tilemap_renderer && cargo nextest run --all-features --test native_backend_test -E 'test(sprite_footprint_scales_with_region_pixel_size) + test(sprite_and_corner_pixels_match_configured_colors)'
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `quad_vertices`'s local quad is unscaled by `region`'s pixel dimensions, making on-screen sprite size `Transform::scale` alone instead of `region.{w,h} * scale`. | ✅ Root Cause | `sprite.vert`/`vs_main` both explicitly scale their local quad by `region.zw` before the transform; `svg.rs` sizes its `<symbol>` to `region`'s own dimensions; `native.rs`'s corners are raw `±0.5`, never touching `region`. | E1, E2, E3, E4 |
| H2 | `quad_vertices`'s local quad is centered at `Transform::position`, while WebGL/WebGPU anchor `position` at one corner of the sprite. | ✅ Confirmed | Direct matrix substitution: WebGL/WebGPU's `quad=(0,0)` corner reduces to exactly `Transform::position` with zero linear-term contribution (corner, not center); Native's symmetric `±0.5` corners average to `position` (center, not corner). | E1, E2, E5 |
| H3 | The two symptoms (size, anchor) require two separate fixes in two separate locations. | ❌ Rejected | Both trace to the same 2 lines (`world_x`/`world_y`'s corner-scaling), fixed by the same edit: reusing the function's own pre-existing `fx`/`fy` (`[0,1]`-mapped, already computed for UV) scaled by `region`, instead of the raw `[-0.5,0.5]` `lx`/`ly`. | E4 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/tilemap_renderer/src/adapters/shaders/sprite.vert` (direct read, 43 lines) | `vec3 world = u_transform * vec3( quad * u_region.zw, 1.0 );` with `quad` from `gl_VertexID`, ranging `{0,1}x{0,1}` — comment: "Scale unit quad to sprite's pixel size (region.zw), then apply transform." | H1 ✅, H2 ✅ |
| E2 | `module/helper/tilemap_renderer/src/adapters/webgpu.rs`, WGSL `vs_main` (direct read, 423 lines) | `let scaled = local_position * uniforms.region.zw;` then `world = uniforms.transform * vec4(scaled, 0.0, 1.0);`, `local_position` from `QUAD_VERTICES` (`{0,1}x{0,1}`, not centered) — identical shape to E1. | H1 ✅, H2 ✅ |
| E3 | `module/helper/tilemap_renderer/src/adapters/svg.rs`, `sprites_load` (direct read) | `<symbol id="sprite_N" viewBox="region.x region.y region.w region.h">` — the symbol's own coordinate system is sized to `region`'s pixel dimensions, structurally parallel to E1/E2. | H1 ✅ |
| E4 | `module/helper/tilemap_renderer/src/adapters/native.rs`, pre-fix `quad_vertices` (direct read) | `corners = [(-0.5,-0.5), (0.5,-0.5), (0.5,0.5), (-0.5,0.5)]` fed directly into the transform's linear part with no `region` term anywhere in the function. | H1 ✅, H3 ❌ |
| E5 | `module/helper/tilemap_renderer/tests/native_backend_test.rs`, `sprite_footprint_scales_with_region_pixel_size` (new test, real GPU pixel-readback, post-fix) | `region=[0,0,8,8]`, `scale=[2,2]` (footprint 16, half-extent 8): a point 6px from the sprite's center reads the sprite's color; under the pre-fix formula (footprint `scale`=2, half-extent 1) that same point would be outside. Passed on first run against the derived fix (151/151 total). | H1 ✅, H2 (indirectly, via the corrected `centered_sprite_command` fixture also passing) |

## Root Cause

`NativeBackend::quad_vertices` computed each corner's world position as `transform_linear_part *
(lx, ly) + position`, using the raw `[-0.5, 0.5]`-ranging local corner (`lx`, `ly`) directly —
never multiplying by `region`'s pixel width/height, and never shifting off center. `webgl.rs`'s
`sprite.vert`, `webgpu.rs`'s `vs_main`, and `svg.rs`'s `sprites_load` all instead use a local quad
scaled to `region`'s own pixel size and anchored so `Transform::position` lands on one corner, not
the center. `quad_vertices` already computed exactly the values needed to match this (`fx`, `fy` —
the same corner mapped to `[0, 1]`, used for UV) but never reused them for the position
calculation.

## Why Not Caught

The pre-existing `sprite_and_corner_pixels_match_configured_colors` test sampled only the
viewport's exact center and a far corner, both under a fixture (`centered_sprite_command`) where
`Transform::position` was itself set to the viewport's center — a configuration where the old
(centered, unscaled) and a hypothetically-fixed (corner-anchored, region-scaled) formula can both
still place *something* at that same center pixel, since the test never sampled a point whose
in/out status actually depends on `region`'s pixel size or on which specific point `position`
anchors. No test compared `NativeBackend`'s output against the other 3 backends' shader/SVG
source directly.

## Fix Location

`module/helper/tilemap_renderer/src/adapters/native.rs`, `NativeBackend::quad_vertices`: `world_x`/
`world_y` now scale the corner's already-computed `fx`/`fy` (`[0, 1]`-ranging, previously used only
for UV) by `region[2]`/`region[3]` before applying the transform's linear part, instead of using
the raw `[-0.5, 0.5]` `lx`/`ly` directly. The UV-output lines are unchanged in formula, only
restated in terms of the same `fy` (with the pre-existing `1.0 - fy` flip preserved exactly) to
avoid a duplicate local variable.

## Prevention

1 new regression test, `module/helper/tilemap_renderer/tests/native_backend_test.rs`:
`sprite_footprint_scales_with_region_pixel_size` — asserts a point 6px from a sprite's center is
inside a `region`-scaled 16px-wide footprint but would be outside the pre-fix 2px-wide one,
deliberately kept symmetric about the viewport's own center so the assertions hold regardless of
the renderer's NDC-to-pixel Y-axis direction (never independently confirmed — see Pitfall). The
pre-existing `centered_sprite_command` fixture (`scale: [24,24]`, `position: [size/2, size/2]`) was
corrected to `scale: [3,3]`, `position: [size/2 - 12, size/2 - 12]` to preserve its own "center
covered, corner clear" assertions under the new corner-anchored, region-scaled semantics — both its
tests re-verified passing unchanged (151/151 total, clippy clean).

## Pitfall

A backend's own doc comment claiming behavioral parity with a sibling ("the same minimal command
family the WebGPU adapter translates") is a claim about *command support*, not *per-vertex
geometry* — the two are independent properties, and a backend can honestly support the same
command set while silently computing different geometry for it. Verify parity claims against the
sibling's actual vertex-generation math (shader source, WGSL, symbol viewBox), not the doc
comment's framing. Separately: this fix's regression test deliberately avoids asserting which
literal screen-space corner (e.g. "top-left" vs "bottom-left") `Transform::position` anchors to —
only that a symmetric configuration behaves correctly regardless of that direction — because the
renderer's NDC-to-pixel Y-axis convention was not independently confirmed by reading `gpu_hal`'s
own viewport/readback code, which was out of this crate's scope. A future caller relying on a
*specific* corner (not just "some consistent corner") should confirm that direction empirically
first.

## Generalized Version

**Broken assumption:** "A backend's doc comment claiming it translates 'the same minimal command
family' as a sibling backend implies the two compute matching per-vertex geometry for that
family."

**Confirmed general rule:** Command-level API parity and geometry-level rendering-math parity are
independent properties across backend adapters implementing the same abstract command set — a
parity claim in a doc comment must be checked against each backend's actual vertex-generation code
(shader source or equivalent), not trusted at face value from the command-support list alone.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `tilemap_renderer` crate scout (task #173), immediately after closing BUG-239, while reading `src/adapters/native.rs`; cross-checked against `webgl.rs`'s `sprite.vert`, `webgpu.rs`'s WGSL `vs_main`, and `svg.rs`'s `sprites_load` as independent ground truth, all three mutually consistent and all three diverging from `native.rs`. |
| 2026-08-17 | fixed | `quad_vertices`'s `world_x`/`world_y` now scale by `region[2]`/`region[3]` via the function's own pre-existing `fx`/`fy`; UV math unchanged. 1 new regression test added; 1 pre-existing test's fixture corrected to match the new semantics. |
| 2026-08-17 | verified | `cargo nextest run -p tilemap_renderer --all-features` (via `verb/test_only pkg::tilemap_renderer`, `longrun`-detached, log `module/-0015_longrun.log`): 151/151 passed, 0 skipped, including the new test and the corrected fixture's test, both on real GPU pixel readback. `cargo clippy -p tilemap_renderer --all-targets --all-features -- -D warnings` (`longrun`-detached, log `module/-0016_longrun.log`): clean, exit 0. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present. | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | MRE isolates a single-axis, identity-scale case (`region=[0,0,8,8]`, `scale=1`, `position=0`) exposing both the size divergence (footprint 1 vs 8) and the anchor divergence (span `[-0.5,0.5]` vs `[0,8]`) in one minimal example, distinct from every pre-existing `native_backend_test.rs` test (all use the pre-fix-calibrated centered fixture). | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Checked against the crate's 5 `tilemap_renderer`-component bugs (BUG-153/209/210/211/239, re-derived via a component-scoped grep of `task/bug/readme.md` rather than trusted from memory) — BUG-210 is the only other sprite-quad-adjacent bug (WebGL texture-upload Y-flip, a distinct mechanism); no overlap. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by 3 independent, mutually-consistent cross-backend sources (`sprite.vert`, `webgpu.rs`'s `vs_main`, `svg.rs`'s `sprites_load`), direct matrix-substitution proof for the anchor claim (not just visual pattern-matching), and confirmed empirically against real GPU pixel readback post-fix (not merely a plausible-looking derivation) — the new test passed on its first run against the derived formula. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to `world_x`/`world_y`'s corner-scaling in `quad_vertices`; the already-correct UV computation is untouched in formula (only restated to reuse `fy` without a duplicate variable), re-confirmed via all pre-existing UV-dependent pixel assertions still passing unchanged. | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `tilemap_renderer` (`src/adapters/native.rs` + `tests/native_backend_test.rs`); freshly grepped `native.rs`/`webgpu.rs`/`webgl.rs` this round (not carried over from a prior claim) confirming `quad_vertices` has no callers outside this file and no other file duplicates its corner math. | — |

**Reproduced:** YES — pre-fix, hand-derivation shows the MRE's footprint as `[-0.5, 0.5]`; post-fix,
the real-GPU-backed `sprite_footprint_scales_with_region_pixel_size` test confirms the corrected
`[0, 8]`-equivalent (region-scaled) footprint on first run. 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/native.rs` | `NativeBackend::quad_vertices`: `world_x`/`world_y` scaled by `region[2]`/`region[3]` via the existing `fx`/`fy` (`Fix(BUG-240)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/native_backend_test.rs` | Added `sprite_footprint_scales_with_region_pixel_size`; corrected `centered_sprite_command`'s fixture (`position`/`scale`) to match the new corner-anchored, region-scaled semantics. |

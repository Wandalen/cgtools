# BUG-539: An occluder drawn last left the colour mask off after `Renderer::render`

- **Severity:** High (with an occluder last in the opaque queue, the skybox, every
  post-processing pass and the next frame's clear wrote no colour; no error, no crash)
- **state:** Completed
- **Affects:** Every `Renderer::render` caller whose opaque queue ends with a material whose
  `Material::color_write_mask` turns colour writes off, an `is_occluder` material in practice.
  Occluders sort first in the opaque queue, so this needs a queue holding only occluders: an
  AR scene before its product has loaded, or one whose only visible node is the occluder.
- **Component:** `module/helper/renderer` (`src/webgl/renderer.rs`)
- **repo_identity:** self
- **Filed:** 2026-10-19
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-19
- **Related Bugs:** BUG-439 (the same class: a pass leaving global GL state behind).

## Symptom

`Renderer::render` applies each material's colour mask before its draw
(`material_color_mask_enable`) and never reset it afterwards. An occluder's mask is all off. When
it was the last draw of the opaque pass, the mask stayed off for the skybox, for the
transparent pass's composite, for every post-processing pass and for the next frame's clear.

## Impact

**Who is affected:** AR pages using depth-only occluders (PR #219, `Material::is_occluder`).

**What breaks:** The skybox and everything after the opaque pass draw nothing, and the canvas
keeps whatever the previous frame left, until a material with colour writes on is drawn last
again.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Writing the occluder tests for PR #219 (2026-10-14). The fix landed in that PR (`32655c8f`).
Round 3 of the review (2026-10-17) asked for a record.

## Minimum Reproducible Example

```rust
renderer.skybox_set( Some( red_skybox( &gl ) ) );
scene.add( rect_node( &gl, [ -5.0, -5.0, 0.0, 5.0 ], 1.0, Box::new( Occluder( PbrMaterial::new( &gl ) ) ) ) );
renderer.render( &gl, &mut scene, &camera() ).unwrap();
// pre-fix: the pixel beside the occluder holds the clear colour, not the skybox's red,
// and COLOR_WRITEMASK reads [ false; 4 ]
```

**Verify Command:**
```bash
cd module/helper/renderer && cargo test --target wasm32-unknown-unknown --test ar_alpha_readback_test -- an_occluder_drawn_last
```

## Root Cause

The colour mask is per-material state like depth and face state, but nothing reset it. Depth
and face state are set before every opaque draw and reset at the end of the transparent pass;
the colour mask was new with `Material::color_write_mask` and got no reset at all.

## Why Not Caught

No test drew an occluder last, and the depth tests read depth, not colour.

## Fix Location

`module/helper/renderer/src/webgl/renderer.rs`: `gl.color_mask( true, true, true, true )` after
the opaque pass, before the skybox, and again at the end of the transparent pass, each with a
`Fix(BUG-539)` comment. (Commit `32655c8f`; comments added in review round 3.)

## Prevention

`an_occluder_drawn_last_does_not_leave_the_colour_mask_off` in
`module/helper/renderer/tests/ar_alpha_readback_test.rs` sets a red skybox, draws an occluder
over the left half only, and requires the skybox's red on the right half. The skybox draws
right after the opaque pass, so only the first reset can make it show. The test also requires
`COLOR_WRITEMASK` to be all on after the frame.

## Pitfall

Every piece of per-material GL state the opaque pass applies has to be reset before a pass that
does not set it itself. A `?` that returns from a draw loop early still skips these resets, for
the colour mask as for depth and face state.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-10-14 | fixed | Found while writing the PR #219 occluder tests; fixed in `32655c8f` without a record. |
| 2026-10-19 | filed | Round 3 of the PR #219 review asked for a record. |
| 2026-10-19 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `ar_alpha_readback_test` passes 6/6. Adversarial pass: with the reset after the opaque pass removed, the reproducer fails (5/6), the right half reading the clear colour instead of the skybox. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-539)` / `Root cause` / `Pitfall` comment at both reset sites; 5-section doc comment and `bug_reproducer(BUG-539)` marker on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to the two resets in `Renderer::render`'s opaque and transparent passes. | — |

**Reproduced:** YES -- the reverted run above. 2026-10-19.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/renderer.rs` | Colour mask reset after the opaque and the transparent pass; `Fix(BUG-539)` comments. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/ar_alpha_readback_test.rs` | `an_occluder_drawn_last_does_not_leave_the_colour_mask_off`, marked `bug_reproducer(BUG-539)`. |

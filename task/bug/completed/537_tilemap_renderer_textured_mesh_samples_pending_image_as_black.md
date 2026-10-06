# BUG-537: WebGL2 textured meshes drew opaque black while their image decoded

- **Severity:** Medium (a textured mesh flashed black until its image decoded, again after every
  `assets_load`, and stayed black if the load failed; no crash)
- **state:** Completed
- **Affects:** Every `Mesh` command and mesh batch on `WebGlBackend` textured by an
  `ImageSource::Path` or `ImageSource::Encoded` image, e.g. the textured object quads in
  `examples/minwebgl/hexagonal_map`.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/webgl.rs`,
  `src/adapters/webgl/webgl_renderers.rs`)
- **repo_identity:** self
- **Filed:** 2026-10-06
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-06
- **Related Bugs:** BUG-538 was fixed in the same PR (#218).

## Symptom

`Path` and `Encoded` images decode asynchronously. Until `on_load` uploads the decoded image,
the adapter registers the texture as 0×0 with no level-0 image, and `on_error` installs no
fallback. Both sprite paths skip such a texture, but `cmd_mesh` and `MeshRenderer::batch_draw`
bound it and set `use_texture`. WebGL2 samples an incomplete texture as ( 0, 0, 0, 1 ), so
`mesh.frag`'s `tex * color` drew opaque black.

## Impact

**Who is affected:** Any WebGL2 scene with a textured mesh on an image loaded from a path or
from encoded bytes.

**What breaks:** The mesh draws black for the first frames after every `assets_load`, and for
good if the image fails to load.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Round 1 of the PR #218 review (2026-10-01). The binding was unchanged from master.

## Minimum Reproducible Example

```rust
backend.assets_load( &assets_with_encoded_white_png ).unwrap();
// Same task: the decode can't have landed yet.
backend.submit( &[ clear_grey, textured_mesh_over_the_viewport ] ).unwrap();
// pre-fix: pixel ( 0, 0 ) reads ( 0, 0, 0, 255 ) instead of the grey clear
```

**Verify Command:**
```bash
cd module/helper/tilemap_renderer && wasm-pack test --headless --chrome -- --features adapter-webgl,test_internals --test webgl_pending_image_test
```

## Root Cause

The 0×0 "still decoding" state was checked on the sprite paths only; the mesh paths assumed every
registered texture had pixels.

## Why Not Caught

No test drew a mesh on an image that loads asynchronously, and the black frames last only until
the decode lands.

## Fix Location

- `module/helper/tilemap_renderer/src/adapters/webgl.rs`: `cmd_mesh` returns early on a 0×0
  texture, as `cmd_sprite` does. `Fix(BUG-537)` comment there. (Commit `dab0907f`.)
- `module/helper/tilemap_renderer/src/adapters/webgl/webgl_renderers.rs`:
  `MeshRenderer::batch_draw` does the same. `Fix(BUG-537)` comment there.

## Prevention

`module/helper/tilemap_renderer/tests/webgl_pending_image_test.rs` draws a textured mesh and a
textured mesh batch in the same task as `assets_load` and requires the clear, and a third case
waits for the decode and requires the white texel.

## Pitfall

Every draw path that binds an image texture has to check that it has pixels.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-10-01 | fixed | Raised by round 1 of the PR #218 review; fixed in `dab0907f` without a record. |
| 2026-10-06 | filed | Round 2 of the review asked for a record. |
| 2026-10-06 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `webgl_pending_image_test` passes 3/3. Adversarial pass: with both 0×0 checks removed, all three cases fail (0/3), reading ( 0, 0, 0, 255 ). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-537)` / `Root cause` / `Pitfall` comment in `cmd_mesh`, `Fix(BUG-537)` note in `MeshRenderer::batch_draw`; 5-section doc comment and `bug_reproducer(BUG-537)` marker on each of the three cases. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to the two early returns. | — |

**Reproduced:** YES -- the removed-check run above. 2026-10-06.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/webgl.rs` | `cmd_mesh` skips a 0×0 texture; `Fix(BUG-537)` comment. |
| `module/helper/tilemap_renderer/src/adapters/webgl/webgl_renderers.rs` | `MeshRenderer::batch_draw` skips a 0×0 texture; `Fix(BUG-537)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/webgl_pending_image_test.rs` | `textured_mesh_waits_for_its_image`, `textured_mesh_batch_waits_for_its_image`, `textured_mesh_draws_once_its_image_decodes`, each marked `bug_reproducer(BUG-537)`. |

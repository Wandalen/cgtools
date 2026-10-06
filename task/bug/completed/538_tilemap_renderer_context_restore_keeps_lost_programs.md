# BUG-538: After a real WebGL context loss, `assets_load` left the backend drawing nothing

- **Severity:** High (after a real context loss and restore, every draw was lost for the rest of
  the backend's life while `submit()` kept returning `Ok`; no crash)
- **state:** Completed
- **Affects:** Every consumer of `tilemap_renderer`'s `WebGlBackend` whose context is really lost
  and restored (a GPU driver reset, a backgrounded mobile tab, too many contexts open) and that
  calls `assets_load` again, as the restore warning tells it to.
- **Component:** `module/helper/tilemap_renderer` (`src/adapters/webgl.rs`)
- **repo_identity:** self
- **Filed:** 2026-10-06
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-06
- **Related Bugs:** BUG-441 (moved the clearing of `context_lost` into `assets_load`; this bug is
  what that re-upload still missed). BUG-537 was fixed in the same PR (#218).

## Symptom

A restored WebGL context has none of the objects created before the loss and starts from default
GL state. `assets_load` re-uploaded images, sprites and geometries and then cleared
`context_lost`, but the sprite and mesh programs and the GL state (`BLEND`, `DEPTH_TEST` /
`LEQUAL`) were built only in `WebGlBackend::new`. Every later draw used a program from the lost
context with blending and depth testing off, so only `Clear` showed. Called while the context
was still lost, `assets_load` also "succeeded" and cleared the flag over an empty context.

## Impact

**Who is affected:** Long-running WebGL2 pages, mobile especially, where context loss happens in
normal use.

**What breaks:** Nothing but the clear colour renders after the restore, and no error says why.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Round 1 of the PR #218 review (2026-10-01). Unchanged from master.

## Minimum Reproducible Example

```rust
lose_context_call( &extension, "loseContext" );    // WEBGL_lose_context
// ... wait for webglcontextlost ...
lose_context_call( &extension, "restoreContext" );
// ... wait for the context to come back ...
backend.assets_load( &red_sprite_assets() ).unwrap();
backend.submit( &[ clear_blue, red_sprite_over_the_viewport ] ).unwrap();
// pre-fix: pixel ( 0, 0 ) reads the blue clear instead of red
```

**Verify Command:**
```bash
cd module/helper/tilemap_renderer && wasm-pack test --headless --chrome -- --features adapter-webgl,test_internals --test webgl_context_loss_test
```

## Root Cause

Only `new` built the shader programs and the GL state, and BUG-441's fix had `assets_load`
re-upload the assets alone before clearing `context_lost`, without checking that the context had
actually come back.

## Why Not Caught

BUG-441's test simulates the loss by setting the private flag, so the context and its programs
were never actually lost.

## Fix Location

- `module/helper/tilemap_renderer/src/adapters/webgl.rs`: after a loss, `assets_load` returns
  `RenderError::ContextLost` while `gl.is_context_lost()`, and otherwise rebuilds both renderers
  and re-applies `new`'s GL state through the shared `renderers_new` / `gl_state_init` before
  re-uploading. `Fix(BUG-538)` comment there. (Commit `4c005a00`.)

## Prevention

`assets_load_after_real_restore_draws_again` in
`module/helper/tilemap_renderer/tests/webgl_context_loss_test.rs` drives a real loss and restore
through `WEBGL_lose_context` and requires both the `ContextLost` error during the loss and a
drawn sprite after it.

## Pitfall

A restored context keeps nothing from before the loss, not even programs or enabled
capabilities; a simulated loss can't show that.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-10-01 | fixed | Raised by round 1 of the PR #218 review; fixed in `4c005a00` without a record. |
| 2026-10-06 | filed | Round 2 of the review asked for a record. |
| 2026-10-06 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `webgl_context_loss_test` passes 2/2. Adversarial passes: with the whole post-loss block removed, the reproducer fails on the `ContextLost` check (1/2); with only the rebuild removed, it fails on the drawn pixel, reading the blue clear ( 0, 0, 255, 255 ) instead of red (1/2). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-538)` / `Root cause` / `Pitfall` comment in `assets_load`; 5-section doc comment and `bug_reproducer(BUG-538)` marker on the reproducer. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to the post-loss block at the top of `assets_load` and the extraction of `renderers_new` / `gl_state_init` from `new`. | — |

**Reproduced:** YES -- both reverted runs above. 2026-10-06.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/src/adapters/webgl.rs` | `assets_load` rebuilds programs and GL state after a loss and reports `ContextLost` while it lasts; `Fix(BUG-538)` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/tilemap_renderer/tests/webgl_context_loss_test.rs` | `assets_load_after_real_restore_draws_again`, marked `bug_reproducer(BUG-538)`. |

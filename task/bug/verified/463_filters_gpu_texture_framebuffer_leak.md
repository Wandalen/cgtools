# BUG-463: `filters`' `Renderer`/`Framebuffer` never free replaced GPU textures/framebuffers, leaking on every image upload and every resize

- **Severity:** High (unbounded GPU memory growth during ordinary interactive use -- every image
  upload, drag-drop, Apply, Revert, and canvas resize leaks GPU resources with no way to reclaim
  them short of a full page reload)
- **state:** Verified
- **Affects:** `examples/minwebgl/filters`'s `Renderer` (`image_texture`/`original_texture`
  replacement) and `Framebuffer` (replaced wholesale on every canvas resize, including the
  resize-sync check that runs after every single filter apply).
- **Component:** `examples/minwebgl/filters` (`src/renderer.rs`, `src/framebuffer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Fix Task:** [507](../../verifying/507_register_filters_gpu_texture_framebuffer_leak_fix_closes_bug463.md)

## Symptom

```rust
// pre-fix -- src/renderer.rs
pub fn image_texture_set( &mut self, image_texture : Option< WebGlTexture > )
{
  self.image_texture = image_texture; // old value just dropped -- GPU texture never freed
}
```

```rust
// pre-fix -- src/framebuffer.rs
pub struct Framebuffer { handle : WebGlFramebuffer, color_attachment : WebGlTexture, .. } // no Drop impl
```

`WebGlTexture`/`WebGlFramebuffer` are thin JS handle wrappers -- dropping the Rust-side value only
releases the Rust/JS reference wrapper, it never tells the GL driver to free the underlying GPU
resource. That requires an explicit `gl.delete_texture`/`gl.delete_framebuffer` call, which neither
type ever made before this fix, on any of their replacement paths.

## Impact

**Who is affected:** Any user of the `filters` demo interacting with it for more than a few
operations -- every image upload, drag-and-drop, Apply-filter click, Revert click, and canvas
resize (including the automatic resize-sync check `filter_apply` runs after every single filter
application) replaces a texture and/or a framebuffer without freeing the outgoing GPU resource.

**What breaks:** Unbounded GPU memory growth for the duration of the page session -- no crash
within ordinary use, but a long interactive session (many filter applies, many image swaps) can
exhaust GPU memory or degrade performance, with no way to reclaim the leaked resources short of a
full page reload.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `examples/minwebgl/filters`, auditing every call site
that replaces `Renderer`'s texture fields or `Framebuffer` wholesale, and cross-checking each
against whether the outgoing GL object was ever explicitly deleted -- none were, workspace-wide
convention for owned GL handles (`renderer::webgl::shadow::ShadowMap`) does implement `Drop` for
exactly this reason.

## Manual Reproduction / Verification

No dedicated automated MRE test was added -- GPU resource lifetime is only observable through a
real WebGL context (e.g. `WEBGL_lose_context`'s extension state, or a GPU memory profiler), which
this crate has no scaffolding for, consistent with this sweep's granted exception for example
crates. Verified instead by:

1. A 6-scenario hand-trace of every `image_texture`/`original_texture` mutation path in the
   crate's actual call-site ordering (`main.rs`'s `image_handler_create`, `apply_button_setup`,
   `cancel_button_setup`, `revert_button_setup`, `bg_removal_image_handler_create`) confirming the
   post-fix aliasing guard (compare against the sibling field before deleting) neither deletes a
   texture the sibling field still needs nor leaves a genuinely-orphaned texture undeleted, across
   every one of those call paths.
2. `cargo check -p filters --target wasm32-unknown-unknown` -- clean, no errors.

**Verify Command:**
```bash
cd examples/minwebgl/filters && cargo check --target wasm32-unknown-unknown
```

## Root Cause

Neither `Renderer::image_texture_set`/`original_texture_set` (and the two restore methods that
used to assign `self.image_texture` directly instead of routing through a setter) nor `Framebuffer`
ever called `gl.delete_texture`/`gl.delete_framebuffer` on the GL object being replaced --
replacing a `WebGlTexture`/`WebGlFramebuffer` field only drops the Rust-side handle wrapper, never
the GPU-side resource it refers to.

## Why Not Caught

No existing test or manual-testing checklist entry exercised GPU memory over an extended
interactive session -- a leak of this kind produces no error, no crash, and no visible symptom
within a short testing session; it only manifests as gradual, session-length-dependent resource
growth that requires a GPU profiler or very long interactive use to notice.

## Fix Location

- `examples/minwebgl/filters/src/renderer.rs`: `image_texture_set`/`original_texture_set` now
  delete the outgoing texture (`self.gl.delete_texture`) unless it aliases the sibling field
  (`image_texture`/`original_texture` can point at the exact same GL object right after upload, or
  after `original_texture_restore`) -- checked via `WebGlTexture`'s `PartialEq` (reference
  equality). `original_texture_restore`/`previous_texture_restore` now route through
  `image_texture_set` instead of assigning `self.image_texture` directly, so their outgoing texture
  is properly deleted too.
- `examples/minwebgl/filters/src/framebuffer.rs`: `Framebuffer` now stores its own `gl : GL` handle
  and implements `Drop`, deleting both `handle` (the framebuffer object) and `color_attachment`
  (its backing texture) -- freeing the GPU resources every time `Renderer::framebuffer_size_update`
  replaces `self.framebuffer` with a new one (including the post-filter-apply resize-sync check).

## Prevention

None added beyond the fix itself and the wasm32 compile check, per this sweep's exception for
example crates -- `Framebuffer`'s `Drop` impl now makes every future replacement path
automatically safe (matching this workspace's own `ShadowMap` idiom, so no call site can forget to
free it), and `Renderer`'s two setter functions centralize every texture-replacement call site
through the same aliasing-safe logic instead of leaving each call site to remember it
individually.

## Pitfall

A WebGL object handle (`WebGlTexture`, `WebGlFramebuffer`, etc.) needs an explicit `gl.delete_*`
call to free its GPU resource -- letting Rust's own `Drop` glue run on the handle wrapper alone is
not enough. Any type that owns one should implement `Drop` itself (or route every replacement
through a setter that does the deletion) rather than relying on field assignment to be safe by
default. Separately: `image_texture`/`original_texture` can alias the same underlying GL object at
certain points in this crate's own lifecycle (immediately after upload, and after
`original_texture_restore`) -- deleting unconditionally on every replace would delete a texture
the *sibling* field still needs, corrupting whatever renders from it next; always check the
sibling field before deleting an aliasable handle, never delete blindly.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide bug/UX-DX sweep of `examples/minwebgl/filters`. |
| 2026-08-20 | fixed | Aliasing-safe texture deletion added to `Renderer`'s two setters (and its two restore methods routed through them); `Drop` impl added to `Framebuffer`. Documented with `Fix(BUG-463)`/`Root cause`/`Pitfall` at 5 sites. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 2/2

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Fix correctness (aliasing hand-trace + compile) | — | 🟢 | Adversarial pass: specifically hunted for a use-after-delete regression given `image_texture`/`original_texture` can alias -- traced all 5 real call-site sequences (`image_handler_create`, `apply_button_setup`, `cancel_button_setup`, `revert_button_setup`, `bg_removal_image_handler_create`) against the aliasing guard and found no case where the guard both under-deletes (leak persists) or over-deletes (use-after-delete). `cargo check -p filters --target wasm32-unknown-unknown` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-463)`/`Root cause`/`Pitfall` 3-field format applied at all 5 edit sites across `renderer.rs`/`framebuffer.rs`, modeled on this workspace's own `renderer::webgl::shadow::ShadowMap` `Drop` idiom. | — |

**Reproduced:** Confirmed via hand-trace of GL-handle-replacement call sites against the pre-fix
code, cross-referenced against this workspace's own established `Drop`-for-GL-handles idiom (not a
live GPU-memory-profiler observation -- see Manual Reproduction / Verification for why an automated
MRE was not added). 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `examples/minwebgl/filters/src/renderer.rs` | `image_texture_set`/`original_texture_set`: aliasing-safe `gl.delete_texture` on replace. `original_texture_restore`/`previous_texture_restore`: route through `image_texture_set`. All 4 sites carry `Fix(BUG-463)` comments. |
| `examples/minwebgl/filters/src/framebuffer.rs` | Added `gl : GL` field and `impl Drop for Framebuffer` deleting the framebuffer + color-attachment texture, with `Fix(BUG-463)`/`Root cause`/`Pitfall` comment. |

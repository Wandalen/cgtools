# BUG-493: `CanvasRenderer::render` never restores `DEPTH_TEST`/`BLEND`/`depth_mask`/`front_face` before returning

- **Severity:** Medium (no crash -- purely a global GL state leak, masked in the 3 known real
  call sites by a subsequent renderer that happens to re-establish its own state before it
  matters, same masking-by-luck shape as the already-fixed BUG-342)
- **state:** Completed
- **Affects:** Any caller of `CanvasRenderer::render` whose own subsequent GL work assumes
  default state (`DEPTH_TEST` disabled, `BLEND` whatever it was, `depth_mask` writable,
  `front_face` CCW) rather than re-asserting its own state explicitly. Confirmed via
  workspace-wide audit: the 3 real call sites (`examples/minwebgl/{lottie,animation,curve}
  _surface_rendering`) all immediately follow `canvas_renderer.render(..)` with
  `renderer::webgl::Renderer::render`, which does explicitly set its own `DEPTH_TEST`/`BLEND`/
  `depth_mask`/`front_face` at multiple points during its own draw calls (confirmed via direct
  read of `module/helper/renderer/src/webgl/renderer.rs`) -- so the leaked state is very likely
  masked by luck for these 3 callers today, same as BUG-342's framebuffer-binding leak was before
  its own fix. Any caller that does *not* immediately follow with a renderer that re-asserts its
  own full state (a UI overlay pass, a differently-ordered pipeline, a future consumer) would be
  silently affected.
- **Component:** `module/helper/canvas_renderer` (`src/renderer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Same function, same general leaked-global-GL-state shape as BUG-342 (which
  fixed this same function's framebuffer-binding leak) -- but a genuinely different mechanism (4
  enable-flag/mask/winding state bits vs. a framebuffer binding) and a separate fix, filed
  separately per this bug's own discovery (found as a distinct, explicitly-named item in a
  repo-wide sweep, not as a direct extension of re-reviewing BUG-342 itself).

## Symptom

```rust
// pre-fix -- src/renderer.rs, CanvasRenderer::render
pub fn render( &self, gl : &GL, scene : &mut Scene, camera : &Camera, colors : &[ F32x4 ] ) -> Result< (), gl::WebglError >
{
  scene.world_matrix_update();

  gl.enable( gl::DEPTH_TEST );
  gl.disable( gl::BLEND );
  gl.depth_mask( true );
  gl.clear_depth( 1.0 );
  gl.front_face( gl::CCW );

  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );
  // .. draw ..
  gl.bind_framebuffer( GL::FRAMEBUFFER, None );  // Fix(BUG-342) -- already restored

  Ok( () )
  // DEPTH_TEST / BLEND / depth_mask / front_face left exactly as this function set them,
  // regardless of what they were before this call
}
```

Any caller with `BLEND` enabled, or `CW` winding, or `depth_mask( false )`, before calling
`render()` silently has that state overwritten and left overwritten after `render()` returns.

## Impact

**Who is affected:** See Affects above -- currently-known callers are very likely masked by a
subsequent renderer's own explicit state-setting, but nothing in `CanvasRenderer::render`'s own
contract guarantees this, and the framebuffer-binding sibling of this exact leak (BUG-342) was
fixed on the basis that relying on a caller's incidental re-bind is not itself a fix.

**What breaks:** Visual only, and only for a caller that doesn't itself re-assert the 4 affected
state bits before relying on them.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Assigned as part of a repo-wide bug/UX sweep of `line_tools`, `canvas_renderer`, and
`browser_input`, explicitly contrasting `render()`'s 4 unrestored state-mutating calls (lines
303-307 in the pre-fix source) against the framebuffer binding just below them, which BUG-342
specifically restores (line 371) -- asking for the same restore treatment to be extended to the 4
additional state bits.

## Minimum Reproducible Example

No live `WebGl2RenderingContext` test infrastructure exists in this crate (no
`wasm-bindgen-test` dev-dependency -- same limitation BUG-227/BUG-342 already documented for this
exact crate), so this bug (like BUG-342) is demonstrated structurally instead of behaviorally: a
source-inspection test extracts `render`'s body verbatim from `src/renderer.rs` at test-run time
and asserts the 4 state-restoring snapshot reads and restore calls are present, in the correct
position (after the existing BUG-342 framebuffer restore, before returning).

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/canvas_renderer && cargo nextest run -E 'test(render_restores_depth_test_blend_depth_mask_and_front_face_before_returning)'
```

## Root Cause

`render` unconditionally overwrites 4 pieces of global WebGL context state --
`DEPTH_TEST`/`BLEND` enable flags (`gl.enable`/`gl.disable`), `depth_mask( true )`, and
`front_face( gl::CCW )` -- as an unconditional setup step at the top of the function, with no
snapshot of what the state was beforehand and no restore before returning. WebGL enable-flag,
write-mask, and winding-order state all persist on the context until explicitly changed by some
later call -- there is no automatic scoping. The framebuffer binding this same function also
mutates was already given exactly this snapshot/restore treatment under BUG-342; the other 4
state bits this function equally mutates were not.

## Why Not Caught

Same structural gap as BUG-342: no live `WebGl2RenderingContext` test infrastructure exists in
this crate, and all 3 real call sites happen to be immediately followed by a different renderer
(`renderer::webgl::Renderer::render`) that explicitly re-asserts its own `DEPTH_TEST`/`BLEND`/
`depth_mask`/`front_face` state as part of its own per-material/per-pass draw logic -- masking the
leak by luck, not by any restore `render()` itself performs, identical in kind to how BUG-342's
framebuffer leak was masked by that same subsequent renderer explicitly rebinding its own target.

## Fix Location

`module/helper/canvas_renderer/src/renderer.rs`: `render` now snapshots all 4 state bits before
overwriting them --
`gl.is_enabled( gl::DEPTH_TEST )`, `gl.is_enabled( gl::BLEND )` (both infallible `bool`-returning
WebGL queries), `gl.get_parameter( gl::DEPTH_WRITEMASK )` and `gl.get_parameter( gl::FRONT_FACE )`
(both `Result<JsValue, JsValue>`-returning; decoded via `.as_bool()`/`.as_f64()` with a safe
fallback to this function's own pre-existing default if the query itself ever fails) -- and
restores all 4 in the same trailing restore block as the existing BUG-342 framebuffer restore,
immediately before returning `Ok( () )`.

## Prevention

Added a new test to the existing `module/helper/canvas_renderer/tests/renderer_test.rs` (reusing
its own `render_fn_body()` helper, the same structural-extraction technique BUG-342's own test
uses), asserting: (1) all 4 snapshot-read calls are present in `render`'s current body, and (2)
all 4 corresponding restore calls appear after the BUG-342 framebuffer restore point, in the same
trailing restore block.

## Pitfall

`render()` already restored one piece of global state it mutates (the framebuffer binding, per
BUG-342) -- fixing that one restore did not guarantee the other 4 pieces of state this same
function mutates were also restored. Each piece of global GL state a function changes has to be
individually audited for its own snapshot/restore; a passing test for one piece of leaked state
is not evidence about any other piece, even within the very same function.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Assigned as part of a repo-wide bug/UX sweep, explicitly naming the contrast between this function's already-restored framebuffer binding (BUG-342) and its 4 still-unrestored state bits. |
| 2026-08-20 | fixed | Added snapshot reads (`is_enabled`/`get_parameter`) before the existing state-setting calls, and a restore block after the existing BUG-342 framebuffer restore. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily stripped the new restore block (kept the snapshot reads, to isolate exactly the restore-presence assertion) and confirmed the new test fails with the expected message (`cargo nextest run -E 'test(render_restores_depth_test...)'` -- FAIL, "restore not found after the BUG-342 framebuffer restore point"); restored the fix and confirmed it passes again, byte-identical to the original via `diff` against a pre-edit backup. Full scoped suite after restore: `cargo nextest run -p line_tools -p canvas_renderer -p browser_input --all-features` -- 139/139 pass; `cargo clippy -p line_tools -p canvas_renderer -p browser_input --all-targets --all-features -- -D warnings` clean (one real `clippy::map_unwrap_or` finding against this fix's own `front_face_was` computation was caught and fixed -- `.map(..).unwrap_or(..)` to `.map_or(..)`); `cargo test --doc` clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-493)`/`Root cause`/`Pitfall` 3-field format applied at the snapshot site in `src/renderer.rs`, matching this crate's own BUG-342/BUG-227 comment convention. | — |
| D3 | Scope containment | — | 🟢 | Confirmed via `git diff` that only `src/renderer.rs` (fix) and `tests/renderer_test.rs` (new test, appended to the existing file rather than creating a new one) were touched for this bug. | — |

**Reproduced:** YES -- temporarily removing the restore block (while leaving the snapshot reads in
place, to isolate exactly the condition under test) caused the new test to fail with the exact
expected assertion message; restoring the fix (confirmed byte-identical to the pre-revert state
via `diff`) passes again. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/canvas_renderer/src/renderer.rs` | Snapshot `DEPTH_TEST`/`BLEND`/`DEPTH_WRITEMASK`/`FRONT_FACE` before `render()`'s existing state-setting calls; restore all 4 after the existing BUG-342 framebuffer restore, before returning. `Fix(BUG-493)`/`Root cause`/`Pitfall` comment at the snapshot site. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/canvas_renderer/tests/renderer_test.rs` | Added `render_restores_depth_test_blend_depth_mask_and_front_face_before_returning`, reusing the file's existing `render_fn_body()` structural-extraction helper. |

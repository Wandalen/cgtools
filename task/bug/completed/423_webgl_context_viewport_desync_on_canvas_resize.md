# BUG-423: WebGL2 viewport never re-set when a canvas is resized via `mingl::web::canvas`'s `ResizeObserver`

- **Severity:** High (no crash -- but every consumer of `minwebgl::context::from_canvas`/
  `from_canvas_with` whose canvas can resize after creation renders into a stale, clipped/stretched
  viewport rectangle from that point forward, for the remaining lifetime of the context)
- **state:** Completed
- **Affects:** Any consumer of `minwebgl::context::from_canvas`/`from_canvas_with` whose canvas
  resizes after context creation (window resize, flex/grid reflow, devtools docking, orientation
  change, ...) -- i.e. any non-fixed-size canvas, which is the common case for a full-viewport WebGL
  app.
- **Component:** `module/min/minwebgl` (`src/context.rs`), `module/min/mingl` (`src/web/canvas.rs`,
  visibility-only change)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- isolated to this crate's own GL-context/resize integration, no shared
  root cause with any other bug filed this sweep.

## Symptom

```rust
// pre-fix -- src/context.rs, from_canvas_with
let context = canvas.get_context_with_context_options( "webgl2", &context_options )?..;
let gl : GL = context.dyn_into()?;
Ok( gl ) // no viewport call anywhere -- initial or on resize
```

`mingl::web::canvas`'s own `ResizeObserver` (the one `canvas::make()` attaches) correctly kept a
canvas's `width`/`height` attributes synced to its CSS box on every resize -- but nothing, anywhere
in `minwebgl`, ever called `gl.viewport(..)` in response, either at context-creation time or on any
subsequent resize. A WebGL2 context's viewport is never implied by a drawing-buffer resize; it stays
wherever it was last explicitly set (or its `0,0,canvas_width,canvas_height` default from context
creation) until something calls `gl.viewport(..)` again.

## Impact

**Who is affected:** Any consumer of `from_canvas`/`from_canvas_with` whose canvas is not a fixed,
never-resized size -- i.e. any typical full-viewport or responsively-sized WebGL app.

**What breaks:** After any CSS-driven resize (window resize, flex reflow, devtools docking,
orientation change), the drawing buffer itself correctly resizes (via `mingl::web::canvas`'s
`ResizeObserver`), but the GL viewport keeps rendering into its stale, pre-resize rectangle --
visually, the rendered image stays clipped into a stale small rectangle (if the canvas grew) or
stretched/scaled incorrectly (if the canvas shrank), rather than filling the new drawing buffer.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`,
tracing `mingl::web::canvas`'s `ResizeObserver` callback through to every consumer that binds a GL
context to a canvas it watches -- `minwebgl::context` never re-applies the viewport on resize, and
`mingl::web::canvas` is deliberately GL-unaware (shared substrate reused by `minwebgl::canvas`,
`minwebgpu::canvas`, and `minwebgl::texture::d2::sprite_upload`'s own temporary 2D-context canvas),
so no existing call site anywhere already covered this.

## Minimum Reproducible Example

A full pixel-verified live-browser reproduction of the resize path itself was attempted this session
and found genuinely infeasible from this workspace's existing, unmodified example crates -- see
**Live Verification Investigation** below for the complete, honest record of that attempt (both the
render-loop confound that made a Firefox pixel check uninformative, and the pre-existing,
unrelated Chromium failure that blocked the state-introspection alternative). The Minimum
Reproducible Example is therefore the source-level fact the fix addresses, confirmed by direct
reading of `context.rs` before the fix existed: `from_canvas_with` constructed a `GL` context and
returned it with no `gl.viewport(..)` call anywhere in the function, while `mingl::web::canvas`'s
`ResizeObserver` (verified by reading `canvas.rs`) resizes the canvas's `width`/`height` attributes
on every CSS-driven resize with nothing GL-aware ever reacting to it.

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minwebgl && cargo nextest run -p minwebgl -E 'test(from_canvas_with_syncs_viewport_initially_and_on_every_resize)'
```

## Live Verification Investigation

Documented in full, honestly, because an earlier draft of this fix's source comment briefly claimed
a live pixel-verification had already succeeded -- it had not, and that claim was corrected (see
History) before this report was filed. The actual investigation:

1. Confirmed `browsee`/`trunk` tool availability (both healthy).
2. Launched `trunk serve --release` for `examples/minwebgl/context_triangle_smoke` (the only example
   crate covering `context::from_canvas` + a draw call), read-only -- no edits made to the example
   crate, per this fix's edit-scope boundary.
3. Launched Firefox via `browsee`, confirmed correct initial render (screenshot: centered red
   triangle, `rendered::rgb 44 45 43`).
4. Issued a scripted window resize (`.window do::resize`) -- the call itself reported failure, but a
   follow-up `.windows` query confirmed the resize DID actually apply (`532x412` -> `900x700`).
5. Post-resize screenshot came back blank white -- neither the expected "clipped triangle" (broken
   state) nor "correctly-scaled triangle" (fixed state) signal.
6. Diagnosed via `.console` (Firefox: driver warnings only, no page JS output -- `.console`/`.eval`
   are Chromium/CDP-only per the tool's own docs), the `trunk` server log (no reload occurred, so the
   blank result wasn't a build error), and a second `.wait for::render` (a genuine 20s timeout,
   confirming the blank state was not merely transient).
7. Read `examples/minwebgl/context_triangle_smoke/src/main.rs` in full: it draws exactly once
   (`fn main() { app_run().unwrap(); }`, `app_run` performs one `clear` + `draw_arrays` and returns,
   no `requestAnimationFrame` loop). Per the HTML canvas spec, resizing a canvas's `width`/`height`
   attribute always clears its drawing buffer, regardless of context type -- since nothing redraws
   after the resize, the canvas renders identically blank whether this fix's viewport re-sync is
   present or absent. This example is therefore structurally uninformative for a pixel-based
   fixed-vs-broken discrimination of this specific defect, independent of any code change made here.
8. Attempted a Chromium-based alternative: CDP's `.eval` would allow reading
   `gl.getParameter(gl.VIEWPORT)` directly, sidestepping the redraw confound entirely. Killed the
   Firefox session, launched Chromium against the same `trunk` server -- got a 60s render timeout with
   console error `"Uncaught RuntimeError: unreachable"` from the wasm module. This is a pre-existing,
   environment-specific Chromium WebGL failure (matching a prior, independently-recorded observation
   of the same failure signature in this environment), unrelated to any change made in this fix, and
   it blocked this second avenue too.
9. Searched for an alternative render-loop-bearing example crate in this workspace (e.g. a
   spinning-cube style demo) -- none exists among the example crates reachable without modifying an
   `examples/` crate, which is outside this fix's edit scope.
10. Cleaned up: killed the Chromium session, then terminated the `trunk` dev server by its exact
    confirmed PID (never by process-name pattern, since multiple `trunk` invocations across crates in
    this workspace share the same invocation form) -- confirmed both the supervisor and the actual
    `trunk` process had exited before concluding the investigation.

**Conclusion:** live pixel/state verification of this specific fix is not achievable from this
workspace's existing, unmodified example crates in this environment. The fix itself is applied on the
strength of direct source-level reasoning (confirmed by reading both `canvas.rs`'s `ResizeObserver`
wiring and `context.rs`'s prior total absence of any `gl.viewport(..)` call) plus a source-inspection
regression test, following the same fallback already established in this workspace for BUG-290
(`minvulkan/tests/context_test.rs`) and this session's own BUG-424/BUG-425 for defects that are real
but structurally unreachable from the available native/in-scope test surface.

## Root Cause

`mingl::web::canvas` is deliberately GL-unaware -- shared substrate reused by both `minwebgl::canvas`
and `minwebgpu::canvas`, and used internally by `minwebgl::texture::d2::sprite_upload`'s own
temporary 2D-context canvas -- so its `ResizeObserver` correctly keeps `canvas.width()`/`height()`
synced to the CSS box but cannot itself call `gl.viewport(..)` without hard-coding a WebGL assumption
into infrastructure that WebGPU consumers and 2D-context call sites also depend on.
`minwebgl::context::from_canvas_with`, the one place that already knows both the canvas *and* the GL
context bound to it, never filled that gap -- it created the context and returned it with no
`gl.viewport(..)` call anywhere, initial or resize-driven.

## Why Not Caught

No existing test or example exercised a resize of a canvas already bound to a live WebGL2 context --
`minwebgl`'s existing example crates either never resize after creation, or (per the Live
Verification Investigation above) draw only once with no render loop, making a resize's visual effect
unobservable by design even when reproduced live. The defect is invisible in any fixed-size-canvas
usage, which is common enough in example/demo code to have left this path unexercised.

## Fix Location

`module/min/minwebgl/src/context.rs`, `from_canvas_with`: added an initial
`gl.viewport( 0, 0, canvas.width() as i32, canvas.height() as i32 )` call immediately after context
creation, plus a second, GL-aware `ResizeObserver` -- independent of `mingl::web::canvas`'s own --
whose callback calls `mingl::web::canvas::canvas_resize` (now `pub`, exposing the exact same
width/height computation `canvas::make()`'s own observer uses, so the two can never independently
diverge) and then re-applies `gl.viewport(..)` with the freshly recomputed size.
`module/min/mingl/src/web/canvas.rs`: `canvas_resize`'s visibility widened from private to `pub`,
pure visibility change, no behavior change, to allow the above reuse.

## Prevention

New source-inspection test `from_canvas_with_syncs_viewport_initially_and_on_every_resize` in
`module/min/minwebgl/tests/context_viewport_resize_test.rs`: asserts (via `include_str!`) exact
occurrence counts of the initial `gl.viewport(..)` call, the second `ResizeObserver`'s
`canvas::canvas_resize(..)` call, its `gl_clone.viewport(..)` re-application, and the `ResizeObserver`
construction itself. RED state empirically confirmed by temporarily stripping the entire added block
back to a bare `Ok(gl)` and re-running -- genuinely failed; restored, re-ran GREEN. The cross-crate
`pub` precondition on `canvas_resize` was deliberately **not** separately re-tested: reverting it
(confirmed both for full-private and for `pub(crate)`) makes `mingl` itself fail to compile via its
own `mod_interface!`-generated `own use canvas_resize;` re-export (`E0432`/`E0364`) before this crate
could even build, so a dedicated source-inspection assertion for that specific precondition would be
unreachable-in-failure and therefore pure duplication of a guarantee the macro already enforces at
compile time -- confirmed empirically this session, not assumed.

## Pitfall

A canvas's drawing-buffer size (`width`/`height` attributes) and a bound WebGL context's viewport are
two independent pieces of state -- resizing one never implies resizing the other, and a
`ResizeObserver` that only updates the former (as shared, GL-unaware canvas substrate correctly does)
will silently desync any GL viewport bound to that canvas unless something GL-aware explicitly
re-applies it on every resize, not just at context creation. Separately: an unverified "already
confirmed via live testing" claim must never be allowed to stand in shipped source code once its
absence of supporting evidence is noticed -- correcting it (rather than trusting or silently
preserving it) surfaced the genuine, still-useful investigation this report now documents in full.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`, tracing `mingl::web::canvas`'s `ResizeObserver` through to its GL-context consumers. |
| 2026-08-20 | fixed | Added initial + resize-driven `gl.viewport(..)` calls in `from_canvas_with`, backed by a second GL-aware `ResizeObserver` reusing `mingl::web::canvas::canvas_resize` (widened to `pub`). |
| 2026-08-20 | corrected | An earlier draft of this fix's source comment in `context.rs` stated "Verified via live `browsee` reproduction against `examples/minwebgl/context_triangle_smoke`" with no supporting evidence (no logs, screenshots, or artifacts) anywhere. Rather than trust or silently preserve that claim, a genuine live-verification attempt was carried out (see Live Verification Investigation above); the claim was then corrected in-source to accurately describe what was and wasn't achievable, deferring the full record to this report. Confirmed via `grep` that no other stray "browsee"/"Verified via live" claims existed elsewhere in the touched crates' `src/`. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 4/4

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily stripped the entire fix block from `context.rs` back to a bare `Ok(gl)`, re-ran the test -- genuinely failed (nextest exit 100, 1 failed, `left: 0 / right: 1`). Restored the fix, re-ran GREEN (1/1 pass). Separately confirmed (twice: full-private and `pub(crate)`) that reverting `canvas_resize`'s visibility fails `mingl`'s own compile via `mod_interface!`'s re-export, independent of this test -- informed the decision not to duplicate that guarantee in a second test. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-423)` source comment in `context.rs` corrected to remove the unsubstantiated live-verification claim; `Fix(BUG-423)` comment in `mingl`'s `canvas.rs` documents the visibility-only change. Test carries the mandated 5-section doc block (`bug_reproducer(BUG-423)`). | — |
| D3 | Scope containment | — | 🟢 | Only `context.rs` (fix), `mingl/src/web/canvas.rs` (visibility change), and `minwebgl/tests/context_viewport_resize_test.rs`/`tests/readme.md` (test + Responsibility Table) touched -- confirmed via `git diff`, all within the authorized `module/min/{mingl,minwebgl}` edit scope. The live-verification investigation itself touched no files in either crate -- `examples/minwebgl/context_triangle_smoke` was launched and observed read-only, never edited. | — |
| D4 | Integrity of prior claims | — | 🟢 | The false "Verified via live browsee reproduction" claim, discovered while preparing this report, was corrected in-source (not left standing, not silently rewritten without record) -- see History above for the full, honest account, including that a genuine attempt was made and why it could not reach a pixel-verified conclusion in this environment. | — |

**Reproduced:** YES (compile-time / source-level) -- temporary removal of the entire fix block from
`context.rs` caused the new test to fail with a missing-viewport-call assertion; restoring the fix
passes. Live pixel/state reproduction was genuinely attempted and found infeasible from this
workspace's existing example crates and this environment's Chromium instance -- see Live Verification
Investigation above for the full record. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minwebgl/src/context.rs` | `from_canvas_with` now sets the viewport immediately after context creation and re-applies it on every subsequent resize via a second, GL-aware `ResizeObserver`; `Fix(BUG-423)` source comment corrected to remove an unsubstantiated live-verification claim. |
| `module/min/mingl/src/web/canvas.rs` | `canvas_resize` visibility widened from private to `pub` (pure visibility change, no behavior change) so `minwebgl::context` can reuse its exact width/height computation. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minwebgl/tests/context_viewport_resize_test.rs` | New file. Source-inspection reproducer `from_canvas_with_syncs_viewport_initially_and_on_every_resize`, RED/GREEN-confirmed via a temporary full-block revert. |
| `module/min/minwebgl/tests/readme.md` | Added a row for `context_viewport_resize_test.rs`. |

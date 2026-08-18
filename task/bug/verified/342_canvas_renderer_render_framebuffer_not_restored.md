# BUG-342: `CanvasRenderer::render` never restores the default framebuffer binding before returning

- **Severity:** Medium (latent — not currently manifesting in any real call site — but a
  genuine WebGL state-leakage defect: any future or third-party caller that issues a GL call
  right after `render()` without itself rebinding a framebuffer first silently targets the
  internal offscreen texture instead of the intended target)
- **state:** Verified
- **Affects:** Every call to `CanvasRenderer::render`, for any caller that does not itself
  rebind a framebuffer immediately afterward
- **Component:** `module/helper/canvas_renderer` (`src/renderer.rs`, `CanvasRenderer::render`)
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **Related Bugs:** BUG-227 (same crate, same file — `framebuffer_create`'s renderbuffer-creation
  panic; independent defect, no shared code path — see Dedup note in Generalized Version)

## Symptom

```rust
// render (renderer.rs, pre-fix) -- binds self.framebuffer, never restores None
gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &self.framebuffer ) );   // bound
...
scene.traverse( &mut draw_node )?;
Ok( () )                                                             // returns -- framebuffer still bound

// sibling functions in the same file both restore None before returning:
// framebuffer_create (line 78):  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
// texture_set        (line 395): gl.bind_framebuffer( gl::FRAMEBUFFER, None );
```
`render`'s own doc comment describes it as rendering "to the internal framebuffer" — an
offscreen operation the caller should be able to treat as self-contained. Its two siblings that
also bind `self.framebuffer` both explicitly restore the default binding when they finish;
`render` binds it and simply returns.

## Impact

**Who is affected:** any code that calls `CanvasRenderer::render` and then issues further GL
calls without itself rebinding a framebuffer first — not any of the 3 *current* real call sites
(see Why Not Caught), but any future consumer, and any external crate depending on
`canvas_renderer` as a library (it is published to crates.io per its `Cargo.toml`
`documentation`/`repository` metadata).

**What breaks:** WebGL's `bindFramebuffer` state persists on the context until explicitly
changed. After `render()` returns with the internal offscreen framebuffer still bound, any
subsequent draw, clear, or read call issued by code that assumes the default (visible canvas)
framebuffer is active instead silently writes into `self.framebuffer`'s attached texture — no
error, no panic, just wrong pixels ending up in the wrong place (or the intended target
appearing to receive nothing).

**Magnitude:** 1 function (`render`), 1 missing restore call, matching the exact pattern its 2
siblings in the same file both already handle correctly.

**Entity Scope:** `None` — a code-level defect.

## How Discovered

A prior investigation pass for this session's bug-hunt read `CanvasRenderer::render` in full and
compared its `gl.bind_framebuffer` usage against its two siblings (`framebuffer_create`,
`texture_set`) in the same file, finding `render` is the only one of the three that binds
`self.framebuffer` without a matching restore. This report independently re-confirms the
asymmetry by direct reading of the current source (`render`: lines 292-361; bind at line 309; no
restore before `Ok(())` at line 360 — pre-fix line numbers) and by checking this crate's existing
test infrastructure and BUG-227's own precedent for how a no-live-GL-context defect in this exact
crate was previously verified (see MRE below).

## Minimum Reproducible Example

Not behaviorally unit-testable in this crate: no live `WebGl2RenderingContext` test
infrastructure exists here (confirmed — no `wasm-bindgen-test` dev-dependency in
`module/helper/canvas_renderer/Cargo.toml`; `tests/renderer_test.rs`'s own header comment
states "every `CanvasRenderer` method takes `&GL`, so nothing else here is natively
exercisable" outside a live context), the exact limitation BUG-227's own Prevention section
already documented for this same crate. Per that established precedent, and since adding
`wasm-bindgen-test` browser infrastructure for a one-line restore-call fix would be
disproportionate (same cost/benefit judgment BUG-227's own MAAV D2 pass made explicitly), this
is verified by a structural/source-inspection regression test rather than a behavioral one — see
Prevention for why this is still a real, permanent, automatically fail→pass reproducer rather
than a purely manual check.

**Verify Command** (run from the crate root; ≤3 lines):
```bash
cd module/helper/canvas_renderer
cargo test --test renderer_test render_restores_default_framebuffer_binding_before_returning -- --exact
```
**What:** `render`'s body must contain a `bind_framebuffer( GL::FRAMEBUFFER, None )` restore
call after the `Some( &self.framebuffer )` bind, matching `framebuffer_create`/`texture_set`'s
own convention.

**Expected** (fixed): test passes.

**Actual** (pre-fix, directly confirmed by running the same test against the current, unfixed
source before applying the fix below):
```
thread 'render_restores_default_framebuffer_binding_before_returning' panicked at tests/renderer_test.rs:...:
render() must call `bind_framebuffer( GL::FRAMEBUFFER, None )` after binding self.framebuffer and
before returning (matching framebuffer_create/texture_set's own restore convention) -- next
bind_framebuffer call found after the self.framebuffer bind was: None
test render_restores_default_framebuffer_binding_before_returning ... FAILED
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `render` (renderer.rs:292-361, pre-fix; now 292-374 post-fix) binds `self.framebuffer` at line 309 but never calls `gl.bind_framebuffer(..., None)` before returning at line 360-361 (pre-fix — now restores at line 371) | ✅ Root Cause | Direct read of `render`'s full body: exactly one `bind_framebuffer` call, at line 309, binding `Some(&self.framebuffer)`; no second call anywhere in the function | E1 |
| H2 | This is inconsistent with the rest of the file — `framebuffer_create` and `texture_set` both explicitly restore `None` at the end of their own framebuffer-binding logic | ✅ Verified | Direct read: `framebuffer_create` line 78, `texture_set` line 395, both `gl.bind_framebuffer( gl::FRAMEBUFFER, None )` as their last GL state-changing call | E2, E3 |
| H3 | No currently-real call site is affected because each one immediately follows `canvas_renderer.render(...)` with a different renderer's own `.render(...)` call that rebinds its own target first | ✅ Verified | `grep -rn "canvas_renderer.*\.render(\|\.render(.*canvas" examples/` (3 hits: `animation_surface_rendering`, `curve_surface_rendering`, `lottie_surface_rendering`) — each followed immediately by `renderer::webgl::Renderer::render`, which rebinds before drawing | E4 |
| H4 | No live `WebGl2RenderingContext` test infrastructure exists in this crate, so no behavioral (as opposed to structural) test could catch this | ✅ Verified | `module/helper/canvas_renderer/Cargo.toml` has no `wasm-bindgen-test` dev-dependency; `tests/renderer_test.rs`'s own header comment states the same limitation; BUG-227 (same crate) already documented this exact gap | E5 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/canvas_renderer/src/renderer.rs:292-361` (`render`, pre-fix, direct read) | Single `bind_framebuffer` call at line 309 (`Some(&self.framebuffer)`); function returns at line 360-361 with no restoring call in between | H1 ✅ |
| E2 | `module/helper/canvas_renderer/src/renderer.rs:78` (`framebuffer_create`, direct read) | `gl.bind_framebuffer( gl::FRAMEBUFFER, None );` — restores default binding before the function returns | H2 ✅ |
| E3 | `module/helper/canvas_renderer/src/renderer.rs:395` (`texture_set`, direct read) | `gl.bind_framebuffer( gl::FRAMEBUFFER, None );` — same restore convention, second sibling | H2 ✅ |
| E4 | `grep -rn "canvas_renderer" examples/*/src/main.rs` (3 example call sites) | Every real caller pairs `canvas_renderer.render(...)` with an immediately-following `renderer::webgl::Renderer::render(...)` call, which rebinds its own framebuffer target before drawing — masking the leak by luck | H3 ✅ |
| E5 | `module/helper/canvas_renderer/Cargo.toml` (`[dev-dependencies]`) + `tests/renderer_test.rs:1-8` (direct read) | No `wasm-bindgen-test` entry; header comment states "every `CanvasRenderer` method takes `&GL`, so nothing else here is natively exercisable" | H4 ✅ |

## Root Cause

`render` performs an offscreen-rendering operation by binding `self.framebuffer`, drawing into
it, and (per its own doc comment's framing as a self-contained operation) implicitly promising
to leave the WebGL context in a clean state afterward — the same contract its two siblings
(`framebuffer_create`, `texture_set`) both honor explicitly by restoring `None` as their last
GL state change. `render` binds the framebuffer at line 309 but was never given the matching
restore call before its `Ok(())` return at line 360-361 (pre-fix — the function now restores at
line 371 and closes at line 374, see Fix Location); the asymmetry is a straightforward omission,
not a difference in what `render` is supposed to do relative to its siblings.

## Why Not Caught

No automated test exists for any WebGL-context-requiring code in this crate —
`tests/renderer_test.rs`'s own header comment states this explicitly (only the pure,
GL-context-free `mesh_colors_resolve` helper is natively testable; BUG-227 already documented
the same limitation for `framebuffer_create` in this exact file). Separately, all 3 real call
sites happen to immediately chain a *different* renderer's own `.render(...)` call right after
`canvas_renderer.render(...)`, and that second call rebinds its own framebuffer target before
drawing anything — coincidentally masking the state leak in every currently-exercised code path,
by luck of call-site ordering rather than by any restore `render` itself performs.

## Fix Location

**`module/helper/canvas_renderer/src/renderer.rs:358-374`** (`render`, end of function body,
post-fix — was lines 357-360 pre-fix; the restore call is now at line 371, function closes at
line 374):

```rust
// Before:
      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      Ok( () )
    }

// After:
      // Traverse the scene and draw all opaque objects.
      scene.traverse( &mut draw_node )?;

      gl.bind_framebuffer( GL::FRAMEBUFFER, None );

      Ok( () )
    }
```
Source comment (`Fix(BUG-342)`/`Root cause`/`Pitfall`) added immediately above the new line.
`GL::FRAMEBUFFER` (matching `render`'s own existing bind call's path form at line 309) is used
rather than `gl::FRAMEBUFFER` (the sibling functions' form) for local consistency within the same
function — both resolve to the same constant.

**`module/helper/canvas_renderer/tests/renderer_test.rs`** (new test appended): extracts
`render`'s body verbatim from the real `src/renderer.rs` at test-run time via brace-counting
(so it always exercises the actual current implementation, never a copy that could drift stale)
and asserts a `bind_framebuffer( ..., None )` call appears after the `Some( &self.framebuffer )`
bind.

## Prevention

No live-`WebGl2RenderingContext` behavioral regression test — this crate has no live-GL test
infrastructure (see Why Not Caught; the same limitation BUG-227 already documented for this
exact crate). The new structural test in `tests/renderer_test.rs` mitigates the residual risk
for this specific regression (a future edit removing the restore call again) without requiring
disproportionate new browser-test infrastructure, but it proves only that the restore call's
*text* is present in `render`'s body, not that it executes on every code path (see Pitfall).

Detection command for the general pattern (a function binding a non-default framebuffer without
a matching restore before every return):
```bash
grep -n "bind_framebuffer" module/helper/canvas_renderer/src/renderer.rs
```
This is a starting point for human review, not a precise check — it cannot by itself confirm a
restore call exists on *every* return path of a function, only that the string appears somewhere
in the file; each function using it still needs to be read to confirm placement.

**Pitfall:** when several functions in the same file share a "bind a non-default GL object,
do work, restore the default" shape, fixing (or correctly writing) the restore on some of them
doesn't guarantee the rest were caught — each function has to be individually audited against
the shape it shares with its siblings, not assumed consistent once most of them look handled
(the same general pitfall BUG-227 already recorded for this file's `framebuffer_create`, applied
here one level up at the whole-function-contract scope instead of a single resource-creation
call).

## Generalized Version

**Broken assumption:** "if most functions that bind a non-default GL resource in this file
already restore the default before returning, all of them do."

**Confirmed general rule:** a "bind non-default, do work, restore default" contract shared by
several sibling functions in the same file must be individually verified on each function, not
assumed consistent from the majority — the exact same generalized rule BUG-227 already
established for this file's *resource-creation* call sites (`create_texture`/
`create_renderbuffer`/`create_framebuffer`, each independently honoring or violating a
`None`-on-failure contract), now confirmed to also apply one level up, at the
*framebuffer-binding-restore* contract shared by whole functions (`framebuffer_create`,
`texture_set`, `render`) rather than individual calls within one function.

**Detection invariant:**
```
for every function in this file that calls `gl.bind_framebuffer( GL::FRAMEBUFFER, Some(...) )`:
  every return path must be preceded by `gl.bind_framebuffer( GL::FRAMEBUFFER, None )`,
  unless the function's own contract explicitly documents leaving the binding in place
```
Single confirmed instance in this workspace (`render` is the only function in
`canvas_renderer`, and the only function found workspace-wide via
`grep -rn "bind_framebuffer.*Some" --include=*.rs .` paired with no matching `None` restore in
the same function body, that binds a non-default framebuffer without restoring it). Not a
duplicate of BUG-227: that bug is `framebuffer_create`'s `create_renderbuffer().unwrap()` panic
(a resource-*creation* failure-handling defect); this bug is `render`'s missing binding
*restore* (a state-leakage defect) — different functions, different failure classes, same file,
same general "audit every sibling of a shared shape individually" lesson.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed | Re-confirmed via direct source reading, following up a prior investigation pass's finding; verified by a new structural/source-inspection reproducer test per BUG-227's own established no-live-GL-test precedent for this crate |
| 2026-08-18 | VERIFY Gate run, PASS | File was placed directly in `bug/verified/` at filing time without the formal PROC1-S9 VERIFY Gate ever having been run/recorded (state field still read `Unverified`, no `## Verification Record` present) — a state/location consistency violation caught during a repo-wide reach-consistency sweep. Ran the 8-dimension Tier 2 Dual-Role Self-Check now: re-executed the Verify Command fresh (`cargo test --test renderer_test render_restores_default_framebuffer_binding_before_returning -- --exact`) — exit 0, test passes, matching the documented Expected block exactly. Adversarial pass caught one real defect: Evidence Table's Hypothesis column cited bare H-IDs with no state symbols (checklist 304 requires symbols, per BUG-114's own precedent) — fixed by annotating all 5 rows `✅`. Dimension 2's `/tmp/mreNNN/`-path checklist item (203) and MRE self-containment item (205) are both met via the same documented, precedented structural-test exception BUG-227 (same crate, already ✅ Completed) already established for this no-live-WebGL2-context crate, not a fresh exception invented here. All 8 dimensions 🟢 — see `## Verification Record`. VERIFY_PASS fired; state → `Verified` (file already correctly resided in `bug/verified/`, no move needed). |

## Verification Record

**VERIFY Gate (2026-08-18) — Tier 2 Dual-Role Self-Check, 8 dimensions, verdict: PASS (8/8).**

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | Completeness | — | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | — | 🟢 | 203/205 met via BUG-227's precedented no-live-GL structural-test exception, not literal `/tmp/mreNNN/` | — |
| D3 | Cross-Reference Integrity | — | 🟢 | Evidence Table Hypothesis column had bare H-IDs, no state symbols (304) | Added `✅` to all 5 rows |
| D4 | Root Cause Quality | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 issue | 1 fix |

**Reproduced:** YES — `cargo test --test renderer_test render_restores_default_framebuffer_binding_before_returning -- --exact`, exit 0, 2026-08-18.

## Refs: src/

- `module/helper/canvas_renderer/src/renderer.rs` — `render` now restores the default (`None`) framebuffer binding before returning, mirroring `framebuffer_create`/`texture_set`'s existing convention

## Refs: tests/

- `module/helper/canvas_renderer/tests/renderer_test.rs` — new structural regression test: extracts `render`'s body from `src/renderer.rs` and asserts the restore call is present after the `self.framebuffer` bind (no live-`WebGl2RenderingContext` test infrastructure exists in this crate; see MRE/Prevention)

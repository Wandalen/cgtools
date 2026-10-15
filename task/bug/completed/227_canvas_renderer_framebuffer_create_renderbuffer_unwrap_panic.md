# BUG-227: `framebuffer_create`'s renderbuffer creation panics instead of returning `None` like its siblings

- **Severity:** Medium (panics instead of the documented graceful failure; requires WebGL
  context loss to trigger, not everyday input)
- **state:** Completed
- **Affects:** Every `CanvasRenderer::new` caller (`lottie_surface_rendering`,
  `curve_surface_rendering`, `animation_surface_rendering` examples, and any future consumer) —
  a WebGL context lost at the moment of construction panics instead of returning the documented
  `Err`.
- **Component:** `module/helper/canvas_renderer` (`src/renderer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-17
- **Related Bugs:** None — independent discovery, found while scouting `canvas_renderer` for the
  first time this session; no shared code path with any other filed bug.

## Symptom

```rust
// pre-fix -- renderer.rs, framebuffer_create
let color = gl.create_texture()?;                    // graceful: None -> function returns None
...
let depthbuffer = gl.create_renderbuffer().unwrap();  // panics: None -> panic
...
let framebuffer = gl.create_framebuffer()?;           // graceful: None -> function returns None
```

`framebuffer_create`'s own doc comment promises: "or `None` if creation fails." Two of its three
identical-purpose WebGL resource-creation calls honor that; the third panics instead.

## Impact

**Who is affected:** Any caller of `CanvasRenderer::new` whose WebGL2 context is lost (GPU
driver reset, mobile tab backgrounding, OS reclaiming GPU memory, an explicit
`WEBGL_lose_context`) at the moment construction runs.

**What breaks:** Per the WebGL2 spec, `createRenderbuffer` (like `createTexture` and
`createFramebuffer`) returns `null` when the context is lost. `CanvasRenderer::new` already
turns `framebuffer_create`'s `None` into a proper `Err( WebglError::FailedToAllocateResource(
"Framebuffer" ) )` (`renderer.rs:197-201`) — but only if `framebuffer_create` itself returns
`None` rather than panicking first. The renderbuffer call panics before that `Err` conversion
ever runs, turning a documented, handleable failure mode into an unrecoverable panic for library
code.

**Magnitude:** 1 function (`framebuffer_create`), 1 of 3 identical-shape calls left unguarded.

**Entity Scope:** None — a code-level defect.

## How Discovered

This session's scouting pass of `canvas_renderer` (previously unaudited), reading
`framebuffer_create` in full and comparing its three WebGL resource-creation calls against its
own doc comment's stated contract.

## Minimum Reproducible Example

Not unit-testable in this crate — `tests/renderer_test.rs`'s own doc comment states "every
`CanvasRenderer` method takes `&GL`, so nothing else here is natively exercisable" (no live
`WebGl2RenderingContext` in `cargo test`; see Prevention). `create_renderbuffer()` returning
`None` specifically requires WebGL context loss, which even a real-browser test cannot trigger
deterministically without the `WEBGL_lose_context` extension racing actual construction — out of
proportion to a one-line defensive fix. Reproduction is by direct source inspection:

```rust
// renderer.rs -- framebuffer_create, pre-fix
let color = gl.create_texture()?;                    // guarded
let depthbuffer = gl.create_renderbuffer().unwrap();  // NOT guarded -- the defect
let framebuffer = gl.create_framebuffer()?;           // guarded
```

**Verify Command** (<=3 lines, standalone — source-inspection check, no live GL context
available):
```bash
cd module/helper/canvas_renderer && grep -n "create_texture()\|create_renderbuffer()\|create_framebuffer()" src/renderer.rs
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `create_renderbuffer()` is `.unwrap()`'d while the other two calls in the same function propagate `None` via `?`, against the function's own documented "or `None` if creation fails" contract. | ✅ Root Cause | Direct read of `framebuffer_create` (lines 43-73) confirms the asymmetry; `CanvasRenderer::new` (lines 197-201) confirms the `None` conversion path exists and works for the guarded calls. | E1, E2 |
| H2 | This is intentional — renderbuffer creation is assumed infallible for some reason the other two aren't. | ❌ Falsified | All three calls are the same WebGL "create a GPU object" shape, documented by the same spec section, with identical `null`-on-context-loss failure semantics; no comment or doc anywhere in the crate singles out the renderbuffer call as special. | E1 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/helper/canvas_renderer/src/renderer.rs:43-73` (`framebuffer_create`, pre-fix, direct read) | `create_texture()?` and `create_framebuffer()?` propagate `None`; `create_renderbuffer().unwrap()` panics on the identical failure class. Doc comment (lines 40-42) promises `None` on any creation failure. | H1 ✅ |
| E2 | `module/helper/canvas_renderer/src/renderer.rs:197-201` (`CanvasRenderer::new`, direct read) | `let Some(...) = framebuffer_create(...) else { return Err(...) }` — confirms the graceful-`None` contract is real and relied upon by the only caller; it just never reaches this point for the renderbuffer failure case. | H1 ✅ |

## Root Cause

`framebuffer_create` creates three WebGL GPU resources (texture, depth renderbuffer,
framebuffer) using the identical "may return `null` on context loss" API shape, and the
function's own doc comment commits to surfacing every such failure as `None`. Two of the three
calls do; the renderbuffer call was written with `.unwrap()` instead of `?`, breaking that
contract for exactly one of the three resources.

## Why Not Caught

No automated test exists for any WebGL-context-requiring code in this crate —
`tests/renderer_test.rs`'s own doc comment states this explicitly (only the pure,
GL-context-free `mesh_colors_resolve` is natively testable). The asymmetry was only visible by
directly comparing the three resource-creation calls in `framebuffer_create` against each other
and against the function's own doc comment — exactly the audit that found it this session.

## Fix Location

`module/helper/canvas_renderer/src/renderer.rs`: `framebuffer_create`'s `create_renderbuffer()`
call now uses `?` instead of `.unwrap()`, matching the sibling `create_texture()?` /
`create_framebuffer()?` calls in the same function and honoring the function's own documented
`None`-on-failure contract.

## Prevention

No automated regression test — this crate has no live `WebGl2RenderingContext` test
infrastructure (see Why Not Caught; the same limitation already documented for BUG-210's
`tilemap_renderer` case and this crate's own `renderer_test.rs`). The fix is a direct,
one-character-shape change (`.unwrap()` → `?`) matching the two already-correct sibling calls in
the same function, reducing but not eliminating the residual risk left by the missing live test.

## Pitfall

When several calls of the same resource-creation shape sit in one function, fixing (or
correctly writing) the failure contract on some of them doesn't guarantee the rest were caught —
each call site has to be individually audited against the shape it shares with its siblings, not
assumed consistent once most of them look handled.

## Generalized Version

**Broken assumption:** "if most calls of a given API shape in a function already honor its
documented failure contract, all of them do."

**Confirmed general rule:** A doc comment's contract ("returns `None` if creation fails") covers
every call inside the function that can produce that failure, not just the calls a reader
happens to check — verify every call of the shared shape individually.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed | Found via this session's `canvas_renderer` scouting pass, comparing `framebuffer_create`'s three WebGL resource-creation calls against each other and against its own doc comment. |
| 2026-08-17 | fixed | Changed `gl.create_renderbuffer().unwrap()` to `gl.create_renderbuffer()?`, matching the sibling `create_texture()?`/`create_framebuffer()?` calls. |
| 2026-08-17 | verified | `cargo nextest run -p canvas_renderer --all-features --no-fail-fast`: 1/1 passed, 0 skipped. `cargo clippy -p canvas_renderer --all-targets --all-features -- -D warnings`: clean. Fix itself verified by direct source inspection only — no live-`WebGl2RenderingContext` test exists in this crate (see Prevention section). |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 6/6

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | — | 🟢 | All FI008 sections present, including an explicit no-test-coverage note rather than a fabricated test claim. | — |
| D2 | MRE Validity & Reproducibility | 🟠 | 🟢 | Confirming pass initially considered a `WEBGL_lose_context`-based browser test. Adversarial pass weighed the cost (new browser harness, non-deterministic context-loss timing) against the fix's triviality and this crate's own established no-live-test precedent (BUG-210) — concluded a source-inspection MRE is correct, not a shortcut. | Confirmed source-inspection MRE instead of building disproportionate browser test infrastructure. |
| D3 | Cross-Reference Integrity | — | 🟢 | Correctly identified as independent of all prior bugs; correctly modeled on BUG-210's no-live-GL-test precedent rather than duplicating its content. | — |
| D4 | Root Cause Quality | — | 🟢 | Backed by direct read of all three resource-creation calls plus the function's own doc comment and its only caller's `None`-handling code. | — |
| D5 | Execution Scope | — | 🟢 | Fix confined to the single `.unwrap()` → `?` change; no other call in the function needed touching (the other two were already correct). | — |
| D6 | Crate Scope Unity | — | 🟢 | Fix lives entirely in `canvas_renderer`; no downstream crate changes needed (return type unchanged, still `Option<...>`). | — |

**Reproduced:** Confirmed via direct source inspection only (both pre-fix asymmetry and post-fix
symmetry) — no live-`WebGl2RenderingContext` test exists in this crate to produce a pass/fail
signal (see Prevention). 2026-08-17.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/canvas_renderer/src/renderer.rs` | `framebuffer_create`: changed `gl.create_renderbuffer().unwrap()` to `gl.create_renderbuffer()?` (full `Fix(BUG-227)` comment block). |

## Refs: tests/

| File | Change |
|------|--------|
| — | None — no live `WebGl2RenderingContext` test exists in this crate; see Prevention section for why and what mitigates the residual risk. |

## Refs: docs/

| File | Change |
|------|--------|
| — | None — no pre-existing doc section described this defect (unlike BUG-210's `tilemap_renderer/readme.md` case). |

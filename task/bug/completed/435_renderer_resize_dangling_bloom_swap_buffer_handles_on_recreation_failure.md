# BUG-435: `Renderer::resize()` can leave `bloom_effect`/`swap_buffer` holding dangling handles to already-freed GPU resources

- **Severity:** Medium (not a leak -- a potential use-after-free / dangling-handle bind on the
  *next* frame's `render()` call, on an error path only; a resize that fails partway could leave
  the renderer drawing with already-deleted texture/program handles instead of failing loudly)
- **state:** Completed
- **Affects:** Any consumer of `Renderer::resize()` where `UnrealBloomPass::new` can return
  `Err` -- in practice, shader compilation failure. On this crate's fixed `include_str!` bloom
  shader source, that branch cannot occur in a real browser (see the disclosed test-coverage
  limitation below), so no live deployment of this crate is known to have hit the observable
  failure mode -- this is a structural correctness fix, not a fix for a reported crash.
- **Component:** `module/helper/renderer` (`src/webgl/renderer.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- an ordering defect specific to `resize()`'s error-path control flow,
  unrelated in mechanism to the Drop-leak family (BUG-432/433/436/437/438/440) found in the same
  sweep.

## Symptom

```rust
// pre-fix -- src/webgl/renderer.rs, Renderer::resize
self.resizable_resources_free( gl ); // deletes old framebuffer_ctx/bloom_effect/swap_buffer GL objects
self.framebuffer_ctx = FramebufferContext::new( gl, width, height, samples );
if self.use_emission
{
  self.bloom_effect = Some( UnrealBloomPass::new( gl, width, height, gl::RGBA16F )? ); // early-return on Err
  self.swap_buffer = Some( SwapFramebuffer::new( gl, width, height ) );
}
else
{
  self.bloom_effect = None;
  self.swap_buffer = None;
}
```

If `use_emission` is true and `UnrealBloomPass::new` returns `Err`, the `?` early-returns from
`resize` *before* `self.bloom_effect`/`self.swap_buffer` are reassigned -- but
`resizable_resources_free` above had already deleted both structs' underlying GL
textures/programs moments earlier. `self` is left holding `Some(..)` values whose handles are
already GPU-deleted.

## Impact

**Who is affected:** Any consumer calling `Renderer::resize()` on a code path where
`UnrealBloomPass::new` can fail with emission enabled.

**What breaks:** `render()`'s `composite()` step would bind and draw with the dangling
`bloom_effect`/`swap_buffer` handles on the next frame, instead of the caller ever observing the
`resize()` error surfaced cleanly -- a use-after-free of already-deleted GL objects, likely
producing driver-dependent undefined behavior (silently wrong output, a WebGL error spammed to
console, or in the worst case a context loss) rather than the clean, catchable `Err` the `?`
operator was supposed to provide.

**Magnitude:** One dangling framebuffer/texture/program bind per failed resize while
`use_emission` is true.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during the same repo-wide bug/UX-defect discovery sweep as BUG-432/433/434 -- auditing
every fallible (`?`-using) step in `resize()` against what state had already been mutated
before that step, looking specifically for "free old state, then a fallible step, then assign
new state" orderings where the fallible step's early return skips the reassignment.

## Minimum Reproducible Example

A live, unmocked reproduction of the exact original failure requires forcing
`UnrealBloomPass::new`'s shader compilation to fail, which (see Prevention below) has no
legitimate real-GL trigger on this crate's fixed shader source -- so the MRE below documents the
control-flow shape of the bug rather than a runnable failing-then-passing assertion:

```rust
// src/webgl/renderer.rs -- illustrative, not a literal runnable reproducer (see Prevention)
renderer.use_emission = true;
renderer.resize( &gl, new_w, new_h, samples ); // if UnrealBloomPass::new returns Err here,
// pre-fix: bloom_effect/swap_buffer still hold Some(..) wrapping already-deleted GL handles.
// post-fix: same early-return, but bloom_effect/swap_buffer are cleared to None *before* the
// fallible step runs, so the dangling-Some state is structurally impossible.
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/helper/renderer && cargo test -p renderer --lib --target wasm32-unknown-unknown -- renderer_resize_replaces_bloom_and_swap_buffer_cleanly_across_repeated_resizes
```

## Root Cause

The previous code only assigned `self.bloom_effect`/`self.swap_buffer` fresh values inside the
`if self.use_emission` / `else` branches. When `use_emission` was true and
`UnrealBloomPass::new` returned `Err`, the `?` early-returned from `resize` before either
assignment ran -- but `resizable_resources_free` above had already deleted both structs'
underlying GL textures/programs. Freeing a resource and clearing the handle that refers to it
must happen atomically from the caller's point of view; splitting them across a fallible
recreation step lets an error path leave a dangling handle mistaken for a live one.

## Why Not Caught

No test previously exercised `resize()`'s error path at all -- existing tests
(`webgl_renderer_pass_cycle_test.rs`) exercise successful resizes only, and
`UnrealBloomPass::new`'s only fallible step (shader compilation from a fixed `include_str!`
source) always succeeds in a real browser, so the failure branch has never been exercised by any
test in this crate, before or after this fix.

## Fix Location

`module/helper/renderer/src/webgl/renderer.rs`, `Renderer::resize`: moved
`self.bloom_effect = None; self.swap_buffer = None;` to run unconditionally, immediately before
the fallible `UnrealBloomPass::new(...)?` call, rather than only inside the `else` branch after
it.

## Prevention

**Disclosed test-coverage limitation:** `UnrealBloomPass::new`'s only fallible step is shader
compilation from a fixed `include_str!` source that always succeeds against a real WebGL2
context -- there is no legitimate, non-mocked way to force it to return `Err` (mocking is
forbidden in this project). The original defect's *exact* branch (an `Err` from
`UnrealBloomPass::new` inside `resize`) is therefore not directly exercisable by a real-GL test.

In place of that unforceable branch, the new inline test
`renderer_resize_replaces_bloom_and_swap_buffer_cleanly_across_repeated_resizes` (in
`renderer.rs`'s `mod tests`, wasm32-gated, needing private-field access per
`rulebook.md § Test placement`) verifies the closest achievable real-GL functional invariant:
across several repeated resizes with `use_emission` toggled between calls, `bloom_effect`/
`swap_buffer` are always in the state implied by the current `use_emission` value (`Some` when
true, `None` when false) and every previous cycle's GL handles are confirmed deleted before the
next cycle's are created -- i.e., the fix's ordering change is exercised on every successful
resize, even though the specific `Err`-early-return interior to it is not. This is a genuine,
disclosed gap in this reproducer's coverage of the original branch, not a claim of full coverage.

## Pitfall

"Free old state, then attempt a fallible recreation, then assign the new state" is a natural but
unsafe ordering whenever the fallible step can early-return via `?` -- the assignment that would
have overwritten the stale `Some(..)` never runs. The safe pattern is to clear the field (to
`None`, or to a definitely-valid placeholder) *before* the fallible step, so any early return
leaves the field in a state matching what was actually freed, not what the code intended to
assign next.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during the repo-wide bug/UX-defect discovery sweep of `module/helper/renderer/`/`module/helper/tilemap_renderer/`. |
| 2026-08-20 | fixed | Reordered `bloom_effect`/`swap_buffer` clearing to run before the fallible `UnrealBloomPass::new` call; added `Fix(BUG-435)`/`Root cause`/`Pitfall` source comment and inline functional-invariant test (with disclosed coverage limitation for the exact original error branch). |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `cargo check --target wasm32-unknown-unknown --tests -p renderer` compiles clean; test reuses the `Renderer::new(&gl,64,64,4)`/`EXT_color_buffer_float` pattern from `webgl_renderer_pass_cycle_test.rs`. Adversarial pass: attempted to find a real-GL way to force `UnrealBloomPass::new` to `Err` (env-var-gated shader source swap, malformed dimensions, etc.) -- none exist without mocking; this gap is disclosed above rather than papered over with a fabricated failure path. | Substituted the closest real-GL functional invariant; documented the gap explicitly in Prevention. |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-435)`/`Root cause`/`Pitfall` 3-field source comment; 5-section test doc comment on the reproducer, including the disclosed limitation. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `renderer.rs`'s `Renderer::resize` method plus its own inline test module. | — |

**Reproduced:** PARTIAL -- the ordering defect itself is reproduced and verified structurally (a
`git diff`-visible reordering of two statements relative to a fallible call, confirmed correct
by direct code inspection: pre-fix, an early `?`-return between the free and the reassignment
was possible; post-fix, it is not, since the reassignment-to-None now precedes the fallible
call). The new test exercises the surrounding successful-resize invariant on real GL but cannot
exercise the exact original `Err` branch without mocking (forbidden) -- see the disclosed
limitation in Prevention. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/renderer.rs` | Reordered `bloom_effect`/`swap_buffer` clearing in `resize()` with `Fix(BUG-435)`/`Root cause`/`Pitfall` comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/renderer.rs` | Added inline `mod tests::renderer_resize_replaces_bloom_and_swap_buffer_cleanly_across_repeated_resizes` (wasm32-gated; documents the unforceable-error-branch limitation in its doc comment). |

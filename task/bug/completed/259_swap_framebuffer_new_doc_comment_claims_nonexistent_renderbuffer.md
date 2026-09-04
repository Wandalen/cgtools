# BUG-259: `SwapFramebuffer::new`'s doc comment claims it creates a depth/stencil
renderbuffer that the function body has never created

- **Severity:** Low (documentation-vs-code contract drift, not an observable rendering defect --
  no panic, no wrong pixels; the risk is a future caller trusting the stated contract)
- **state:** Completed
- **Affects:** `webgl::post_processing::SwapFramebuffer::new`
- **Component:** `module/helper/renderer` (`src/webgl/post_processing/pass.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`SwapFramebuffer::new`'s doc comment states it creates "its WebGL framebuffer, renderbuffer, and
the primary output texture", and that the framebuffer "is configured with a single color
attachment point and a depth/stencil renderbuffer". The function body contains no
`create_renderbuffer`, `renderbuffer_storage`, or `framebuffer_renderbuffer` call anywhere --
only a single-color-attachment framebuffer and one `RGBA16F` output texture are actually created.
The struct itself has no `renderbuffer` field to hold one.

## Impact

**Who is affected:** Any future maintainer or caller reading `SwapFramebuffer::new`'s doc comment
to decide whether depth testing is safe to rely on when rendering into a `SwapFramebuffer`-backed
target.

**What breaks:** Nothing at runtime today -- every existing `Pass` implementation in this crate
(`blend.rs`, `tonemapping.rs`, `to_srgb.rs`, `shadow_to_color.rs`, `unreal_bloom.rs`,
`narrow_outline.rs`, `normal_depth_outline.rs`) already explicitly calls
`gl.disable( gl::DEPTH_TEST )` before drawing, so the missing renderbuffer never manifests as an
observable defect currently. The risk is purely forward-looking: a new `Pass` implementation
written against the doc comment's promise of a depth/stencil renderbuffer, without independently
verifying the function body, would silently get incorrect (default, always-passing) depth
behavior with no attachment backing it, or would call depth-related GL functions against a
framebuffer that cannot satisfy them.

**Entity Scope:** `None` -- source-level documentation-vs-code contract gap, not entity directory
instances.

## How Discovered

During this session's Group G review of `module/helper/renderer/src/webgl/post_processing/`
(`blend.rs`, `mod.rs`, `pass.rs`, `shadow_to_color.rs`, `tonemapping.rs`, `to_srgb.rs`,
`unreal_bloom.rs`, `outline/mod.rs`, `outline/narrow_outline.rs`,
`outline/normal_depth_outline.rs`). `pass.rs`'s `SwapFramebuffer::new` doc comment was checked
directly against its own function body and found to claim a renderbuffer that is never created.
`git log --follow -p` on `pass.rs` (then named `composer.rs`) confirmed a real renderbuffer field
and its `create_renderbuffer`/`renderbuffer_storage`/`framebuffer_renderbuffer` calls existed
originally (added in commit `568e4732`) and were deliberately removed in commit `a54d680b`
("Added bloom", 2025-05-28) once depth testing was no longer needed for post-processing passes --
but the doc comment describing them was never revisited. Cross-checked that `gbuffer.rs`'s own
`GBuffer` (out of this review's scope, already fixed as BUG-243-adjacent work earlier this
session) correctly creates and documents a real renderbuffer, confirming this is not a
repo-wide copy-paste pattern but a one-off drift specific to `pass.rs`.

## Minimum Reproducible Example

Pure source-text comparison, no GPU context needed -- the doc comment and the function body it
sits above are both available as static text via `include_str!`. See
`tests/webgl/pass.rs`'s `swap_framebuffer_new_doc_comment_renderbuffer_claim_matches_body`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --features webgl --test tests webgl::pass::
```
**Expected** (fixed): the test passes. **Actual** (pre-fix, confirmed via temporary
direct-source-edit revert-and-rerun of `pass.rs`'s doc comment back to its original wording): the
test failed, since the doc's first sentence claimed "renderbuffer" while the body never called
`create_renderbuffer`.

## Root Cause

`SwapFramebuffer::new` (pre-fix doc comment, current body -- verbatim):
```rust
/// Creates a new `SwapFramebuffer` instance, initializing its WebGL framebuffer,
/// renderbuffer, and the primary output texture.
///
/// The framebuffer is configured with a single color attachment point and a
/// depth/stencil renderbuffer. An initial `output_texture` is created with
/// `RGBA16F` format for high precision.
pub fn new( gl : &gl::WebGl2RenderingContext, width : u32, height : u32 ) -> Self
{
  let framebuffer = gl.create_framebuffer();
  gl.bind_framebuffer( gl::FRAMEBUFFER, framebuffer.as_ref() );
  gl::drawbuffers::drawbuffers( gl, &[ 0 ] );
  gl.bind_renderbuffer( gl::RENDERBUFFER, None );  // unbinds -- does not create one
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );
  let output_texture = gl.create_texture();
  // ... texture setup only, no renderbuffer creation anywhere in this function
}
```
The `bind_renderbuffer( gl::RENDERBUFFER, None )` call unbinds any currently-bound renderbuffer
(global GL state hygiene) -- it does not create one. Commit `a54d680b` ("Added bloom",
2025-05-28) removed this struct's `renderbuffer` field along with its creation code once depth
testing was no longer needed for post-processing passes, but the doc comment (written in an
earlier commit, `568e4732`) was never updated to match.

## Why Not Caught

Rust doc comments are free-form prose with no compiler-enforced contract against the function
body beneath them, so a stale claim compiles cleanly forever. Every `Pass` implementation in this
crate explicitly disables `DEPTH_TEST` before drawing regardless of what `SwapFramebuffer`
provides, so the missing depth/stencil buffer never manifested as an observable rendering defect
-- the mismatch was purely a documentation trap for a future caller trusting the stated contract
enough to skip disabling depth testing.

## Fix Applied (2026-08-17)

**`src/webgl/post_processing/pass.rs`:** Rewrote `SwapFramebuffer::new`'s doc comment to state
only what the function actually does -- creates a framebuffer with a single color attachment and
the output texture, no renderbuffer -- and added an explicit note that any `Pass` rendering into
it must not rely on depth testing, naming that every existing `Pass` implementation in this crate
already disables `DEPTH_TEST` before drawing. No functional code was changed; this is a
comment-only fix.

**`tests/webgl/pass.rs`** (new file, module registered as `mod pass;` in `tests/webgl/mod.rs`):
extracts the real `///` doc comment above `SwapFramebuffer::new` (filtering out plain `//`
comments, including this crate's own `Fix( BUG-NNN )` annotations, so a fix's own explanatory
prose is never mistaken for the doc contract it sits above) and the function's own body text via
`include_str!`, then asserts the doc's initialization-enumeration sentence claims a
`renderbuffer` if and only if the body actually calls `create_renderbuffer`.

## Verification

`longrun`-detached, from repo root, isolated `CARGO_TARGET_DIR` (concurrent-session build
contention observed via `ps aux` against the shared target directory during this session; ruled
out as the cause of an earlier, unrelated compile failure -- see History):
- `cargo test -p renderer --features webgl --test tests webgl::pass::` -- pre-fix (temporary
  direct-source-edit revert of `pass.rs`'s doc comment to its original wording, restored
  immediately after): test failed. Post-fix (restored): test passed.
- `cargo test -p renderer --features webgl --test tests` (full scoped suite): all tests pass.
- `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean.

## Generalized Version

**Broken assumption:** a doc comment, once written, continues to describe the function body
correctly through later refactors. It doesn't -- nothing type-checks prose against the code it
sits above, so a refactor that deletes a resource (here, a renderbuffer) leaves the doc comment
describing a contract the function no longer honors, with zero compiler signal. This is the same
generalized lesson as BUG-258 (a cache's staleness key must cover every input that shaped the
cached value) applied to documentation instead of caching: a claim about "what this function
does" is itself a piece of state that can go stale, and the only way to catch that drift is a
direct comparison between the claim and the code, not a runtime-behavior test (runtime behavior
here never changed).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group G review of `src/webgl/post_processing/` (10 files: `blend.rs`, `mod.rs`, `pass.rs`, `shadow_to_color.rs`, `tonemapping.rs`, `to_srgb.rs`, `unreal_bloom.rs`, `outline/mod.rs`, `outline/narrow_outline.rs`, `outline/normal_depth_outline.rs` -- all clean except this one). Root cause: `SwapFramebuffer::new`'s doc comment claimed renderbuffer creation that was deliberately removed in commit `a54d680b` ("Added bloom", 2025-05-28) and never revisited in the doc comment. Fixed by rewriting the doc comment to match actual behavior; no functional code changed. Verified via a new `include_str!`-based source-text comparison test (confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun) plus a clean scoped `cargo test` and `cargo clippy`. During verification, transiently hit a compile error in the shared `tests/webgl/mod.rs` caused by a concurrent sibling review agent's in-flight `mod renderer;` (later renamed to `mod program_needs_recompile;` -- see BUG-258) shadowing the file's own `use renderer::webgl as the_module;` import; resolved on its own once the sibling agent completed its rename, requiring no action here. Filed as BUG-259 after a fresh on-disk scan found BUG-258 (filed concurrently by that same sibling agent) had already claimed the provisionally-reserved 258; renumbered this bug's own `Fix( BUG-NNN )` source comment and test self-references from 258 to 259 accordingly.

# BUG-424: `Swapchain::rebuild()` leaks the new `VkSwapchainKHR` handle and any already-created image views on an error path

- **Severity:** Medium (no crash on the happy path; a genuine native resource leak -- driver-level
  handles, not just Rust memory -- on an error path that a resizing/minimizing/GPU-hot-unplug
  window can realistically hit repeatedly during a long-running session)
- **state:** Completed
- **Affects:** Any consumer of `minvulkan::Swapchain::rebuild()` -- called on every window resize /
  swapchain-out-of-date event, per this crate's own render-loop convention.
- **Component:** `module/min/minvulkan` (`src/swapchain.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verification_date:** 2026-08-20
- **Related Bugs:** Structurally identical to the already-fixed BUG-290 (`Context`'s `VkInstance`
  leaked on an error path after successful creation, in `context.rs`) -- same defect *class*
  (an FFI handle created successfully, then leaked because a *later* fallible step in the same
  function errors before the handle is committed to `self`/wrapped in its owning `Drop` type), same
  `*Guard`-RAII fix pattern, different crate module (`swapchain.rs` vs `context.rs`).

## Symptom

```rust
// pre-fix -- src/swapchain.rs, rebuild()
let swapchain = unsafe { self.loader.create_swapchain( &create_info, None ) }?;
let images = unsafe { self.loader.get_swapchain_images( swapchain ) }?;
let image_views = images
.iter()
.map( | image | image_view_create( &self.device, *image, format.format ) )
.collect::< Result< Vec< _ >, Error > >()?;
// if get_swapchain_images or any image_view_create call fails, `swapchain`
// (and any views already created before the failing one) are never destroyed
```

If `get_swapchain_images` fails, or any `image_view_create` call in the `.collect()` chain fails
after one or more earlier calls in the same iteration already succeeded, the function returns `Err`
via `?` -- but `swapchain` (already a live `VkSwapchainKHR` handle at that point) is never passed to
`vkDestroySwapchainKHR`, and any image views already created by earlier iterations of the `.map()`
are discarded along with the `Vec` that would have owned them, never passed to
`vkDestroyImageView` either.

## Impact

**Who is affected:** Any consumer whose render loop calls `rebuild()` on resize/out-of-date events --
this crate's own documented usage pattern. A single failed `rebuild()` call leaks one
`VkSwapchainKHR` plus zero or more `VkImageView`s (however many the `.collect()` chain had already
successfully created before the failing one) every time the error path is hit.

**What breaks:** No immediate crash -- the function correctly returns `Err` and the caller can react
(e.g. retry, or tear down and recreate the whole `Context`). But each leaked handle is a real
driver-side allocation that outlives this process's own Rust-level bookkeeping; repeated resize
events hitting a transient error condition (e.g. surface temporarily zero-sized during a window
minimize, a brief out-of-memory condition on a loaded system) could accumulate leaked handles across
a long-running session.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`,
specifically while auditing every FFI-handle-creating function in `minvulkan` against the
already-documented BUG-290 defect class (successful handle creation followed by a later fallible
step with no cleanup-on-error) -- `rebuild()`'s `.collect::<Result<Vec<_>,Error>>()` over
`image_view_create` calls, following a separate, already-successful `create_swapchain` call, is the
exact same shape.

## Minimum Reproducible Example

Live reproduction needs a real windowing surface and a way to force `get_swapchain_images` or
`image_view_create` to fail after `create_swapchain` succeeds (e.g. a format/extent combination
valid enough to create the swapchain but invalid enough to fail image-view creation) -- not
constructible from this crate's native test surface, which is why `surface_test.rs` (pre-existing,
same crate) already documents `Swapchain` construction itself as needing a real window handle this
crate cannot produce natively. See `swapchain_test.rs`'s own doc comment for the full reasoning
behind the source-inspection fallback used instead, mirroring `Fix(BUG-290)`'s own precedent
(`context_test.rs`).

**Verify Command** (<=3 lines, standalone):
```bash
cd module/min/minvulkan && cargo nextest run -p minvulkan -E 'test(swapchain_rebuild_guards_handle_and_views_on_error_path)'
```

## Root Cause

`rebuild()` created the new `VkSwapchainKHR` handle and its image views as two separate fallible
steps, with the handle's lifetime not tied to any `Drop`-implementing guard until the very end of
the function (implicit assignment to `self.swapchain`, which only happens after every earlier
fallible step has already succeeded) -- so any `?` between `create_swapchain` succeeding and that
final assignment leaks the handle, and the `.collect::<Result<Vec<_>,Error>>()` idiom compounds this
for image views specifically: unlike pure Rust values, discarding a `Vec<VkImageView>` on collect
failure does not run any real cleanup -- an FFI handle has no `Drop` impl of its own, it's just an
opaque integer, so the underlying driver resource silently outlives the discarded `Vec`.

## Why Not Caught

No existing test exercised `rebuild()`'s error path at all (`surface_test.rs` documents that live
`Swapchain` construction/rebuild needs a real window handle unavailable in this crate's native test
environment), and the happy path -- the only path any existing test or real usage has ever
exercised -- never surfaces a leak, since nothing observable from outside the driver changes when a
handle merely goes uncollected by `vkDestroy*`.

## Fix Location

`module/min/minvulkan/src/swapchain.rs`: added `SwapchainGuard<'a>` (RAII wrapper holding the loader,
device, swapchain handle, and a `Vec` of image views, destroying all of them in its `Drop` impl
unless `disarm()`ed first). `rebuild()` now constructs the guard immediately after `create_swapchain`
succeeds, pushes each successfully-created image view onto it via an explicit `for` loop (replacing
the `.collect::<Result<Vec<_>,Error>>()` chain, specifically because the loop lets each iteration's
view be pushed onto the guard -- and thus become droppable -- the moment it's created, where
`.collect()`'s all-or-nothing semantics would not), and calls `guard.disarm()` only once every
fallible step has succeeded, committing the handle and views to `self`.

## Prevention

New source-inspection test `swapchain_rebuild_guards_handle_and_views_on_error_path` in
`module/min/minvulkan/tests/swapchain_test.rs`: asserts (via `include_str!("../src/swapchain.rs")`)
exactly 1 occurrence each of `struct SwapchainGuard`, `impl Drop for SwapchainGuard`,
`SwapchainGuard::new( &self.loader, &self.device, handle )`, and `guard.disarm()`; exactly 0
occurrences of the leak-prone `.collect::< Result< Vec< _ >, Error > >()` pattern; and that the
guard's `Drop` impl calls both `destroy_image_view` and `destroy_swapchain`.

## Pitfall

`.collect::<Result<Vec<_>, E>>()` is safe and idiomatic for pure Rust values (a discarded `Vec` on
error just drops its already-collected elements normally, freeing their memory) but is an anti-
pattern for FFI handles specifically -- an FFI handle has no destructor, so "dropping" it via a
discarded collection is not cleanup, it's a leak. Any FFI-handle-producing `.map(..).collect()`
chain needs an explicit RAII guard (or an explicit fallible loop pushing onto one) instead, exactly
because the values being collected don't clean up after themselves the way normal Rust values do.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during a repo-wide bug/UX-DX sweep of `module/min/{mingl,minwebgl,minwebgpu,minvulkan}`, auditing every FFI-handle-creating function against the already-fixed BUG-290 defect class. |
| 2026-08-20 | fixed | Added `SwapchainGuard` RAII wrapper mirroring `Fix(BUG-290)`'s `InstanceGuard`; replaced the leak-prone `.collect()` chain with an explicit guarded loop; added `Fix(BUG-424)`/`Root cause`/`Pitfall` source comment. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Source-inspection test asserts both the guard's presence (construction, `disarm()`, `Drop` impl with both destroy calls) and the leak-prone pattern's absence (0 occurrences of the old `.collect()` chain) -- a partial revert of either half would fail at least one assertion. Full-crate pass: `cargo nextest run -p minvulkan` -- 11/11 pass. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-424)`/`Root cause`/`Pitfall` 3-field format applied at the guard-construction call site in `rebuild()`. | — |
| D3 | Scope containment | — | 🟢 | Only `swapchain.rs` (fix), `tests/swapchain_test.rs` (new test), and `tests/readme.md` (new, previously missing entirely) touched -- confirmed via `git diff`/`git status`, all within `module/min/minvulkan`. | — |

**Reproduced:** Source-inspection only -- live reproduction is not achievable from this crate's
native test surface (no real window handle available; see Minimum Reproducible Example above). The
test's own construction ( asserting both presence of the fix's exact guard mechanics and absence of
the pre-fix leak-prone pattern ) is the closest available substitute for a RED/GREEN cycle: reverting
either half of the fix would independently fail the corresponding assertion. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/min/minvulkan/src/swapchain.rs` | Added `SwapchainGuard<'a>` (RAII wrapper, `new`/`view_push`/`disarm`/`Drop`); `rebuild()` now guards the new swapchain handle and its image views from construction through commit, replacing the leak-prone `.collect::<Result<Vec<_>,Error>>()` chain with an explicit guarded loop. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/min/minvulkan/tests/swapchain_test.rs` | New file. Source-inspection reproducer `swapchain_rebuild_guards_handle_and_views_on_error_path`, mirroring `Fix(BUG-290)`'s `context_test.rs` precedent. |
| `module/min/minvulkan/tests/readme.md` | New file (previously missing). Responsibility Table covering `context_test.rs`, `surface_test.rs`, `swapchain_test.rs`. |

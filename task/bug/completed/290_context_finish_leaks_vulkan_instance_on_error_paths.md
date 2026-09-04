# BUG-290: `Context::builder()...context_finish()` leaks the `VkInstance` handle on all 3 of its error paths

- **Severity:** Medium (a genuine resource leak on real error paths -- `vkDestroyInstance` is
  Vulkan-spec-mandated cleanup, not optional; narrow in practical trigger frequency since it only
  fires when instance creation succeeds but subsequent physical-device/logical-device setup fails)
- **state:** Completed
- **Affects:** `ContextBuilder< DeviceBuilder >::context_finish()`
  (`module/min/minvulkan/src/context.rs`)
- **Component:** module/min/minvulkan
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`context_finish()`'s 3 fallible steps after `instance` is already live --
`instance.enumerate_physical_devices()`, the graphics-queue-family search (`.ok_or(
Error::NoSuitableDevice )`), and `instance.create_device(..)` -- each propagated their error via
a bare `?`/`.ok_or()?`. On any of these 3 paths, the already-created `ash::Instance` was simply
dropped as an ordinary Rust value instead of having `vkDestroyInstance` called on it.

## Impact

**Who is affected:** any caller of `Context::builder().instance_make()?.context_finish()` on a
system where the Vulkan loader loads successfully and an instance is created, but subsequent
physical/logical device setup fails (no graphics-capable device present, or `vkCreateDevice`
itself fails for a driver/resource reason) -- every such failed attempt leaks one `VkInstance`.

**What breaks:** Vulkan's own explicit-cleanup contract (`vkDestroyInstance` must be called on
every instance no longer in use) -- a real, spec-visible resource leak, not merely undesirable
Rust style. Repeated failed construction attempts (e.g. retry loops, or repeated test-harness
setup against a flaky driver) would accumulate leaked instances for the life of the process.

**Entity Scope:** `None` -- library resource-management defect, not entity directory instances.

## How Discovered

Systematic bug-hunting pass across `minvulkan`'s only 2 source files (parent task: bug-hunting the
3 workspace crates with no prior recorded investigation -- `browser_tools`, `minvulkan`,
`gl_uniforms`). While reading `context_finish`, noticed all 3 of its early-return `?`/`.ok_or()?`
sites occur strictly after `instance` is bound to a real, live `ash::Instance`, and cross-checked
against `Context`'s own `Drop` impl -- which exists specifically *because* `ash`'s handle wrapper
types perform no automatic cleanup. Confirmed by reading `ash` 0.38.0's own crate source
(`~/.cargo/registry/.../ash-0.38.0+1.3.281/src/{entry,instance}.rs`): zero `impl Drop for` blocks
anywhere in the crate for `Entry`, `Instance`, or `Device`.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -c "instance_cleanup_on_error( &instance )" module/min/minvulkan/src/context.rs
```
**Expected** (fixed): `3` -- one call site per error-producing branch in `context_finish`.
**Actual** (pre-fix): `0` -- no cleanup call existed anywhere in the file; each of the 3 branches'
`?`/`.ok_or()?` dropped `instance` as an inert value with no `vkDestroyInstance` call.

Note: the leak itself (an un-destroyed `VkInstance` handle) is not practically observable from a
black-box test against a real driver without forcing one of the 3 failure conditions, none of
which are portably triggerable from outside `context_finish` against a real local Vulkan
implementation without faking the Vulkan layer (see Why Not Caught / Verification).

## Root Cause

`ash`'s handle wrapper types (`Entry`, `Instance`, `Device`) do not implement `Drop` -- Vulkan
itself requires explicit `vkDestroyInstance`/`vkDestroyDevice` calls, and `ash` deliberately leaves
that to the caller rather than pretending ordinary Rust ownership provides it "for free." (`Entry`
is the one partial exception : it internally holds `_lib_guard : Option<Arc<Library>>`, so the
*dynamic library* does unload once the last clone drops -- but that is unrelated to, and does not
generalize to, `vkDestroyInstance`/`vkDestroyDevice`.) `context_finish`'s 3 early-return sites were
written assuming ordinary drop semantics would clean up `instance`, which is false for this type.

## Why Not Caught

`context_test.rs`'s existing tests (T01-T03) only exercise the success path -- none force
`enumerate_physical_devices`/`create_device` to fail, or arrange for every enumerated physical
device to lack a graphics-capable queue family. All 3 of `context_finish`'s failure conditions are
impractical to trigger portably against a real local Vulkan implementation (Lavapipe, in this
environment) from outside the function without faking the Vulkan layer, which this crate's real
(non-mocked), driver-backed test philosophy does not do. A resource leak also produces no visible
symptom in an ordinary black-box test regardless -- the function still returns the correct
`Err( Error::X )` either way; nothing in the public API surfaces whether `vkDestroyInstance` ran.

## Fix Applied (2026-08-18)

**`module/min/minvulkan/src/context.rs`:**
- Added `instance_cleanup_on_error( instance : &ash::Instance )`, a one-line `unsafe`-wrapping
  helper calling `instance.destroy_instance( None )`, placed immediately above `Context`'s own
  `Drop` impl with a `Fix(BUG-290)`/Root cause/Pitfall source comment.
- Called it from all 3 error-producing points in `context_finish`: the `enumerate_physical_devices`
  `.map_err(..)`, the graphics-queue-family search's `.ok_or_else(..)` (changed from `.ok_or(..)`
  specifically because `.ok_or` evaluates its argument eagerly -- using it here would have
  destroyed the instance on the *success* path too, a strictly worse bug than the leak it fixes),
  and the `create_device` `.map_err(..)`.

**New regression test** (`tests/context_test.rs`):
`context_finish_destroys_instance_on_every_error_path` -- a source-inspection test (via
`include_str!`, the same approach BUG-287/BUG-288 used for their own hard-to-runtime-test doc-only
defects, here adapted to a cleanup-only defect) asserting `instance_cleanup_on_error` is defined
exactly once and called with `( &instance )` exactly 3 times in `context.rs`.

## Verification

`longrun`-detached, from repo root. No `git stash`/`git show` revert was needed for this proof --
the regression test's own baseline check (does `instance_cleanup_on_error` exist at all) is
naturally RED against the already-untouched-at-that-point source, so the test was written and run
before the fix existed, then again after.

- **Pre-fix (RED):** `cargo test -p minvulkan --test context_test -- \
  context_finish_destroys_instance_on_every_error_path`: `0 passed; 1 failed` -- failed on the
  `helper_def_count == 1` assertion (found 0), confirming the fix did not yet exist.
- **Post-fix (GREEN):** same command: `1 passed; 0 failed`. Full crate suite: `cargo nextest run
  -p minvulkan --all-features` -- `4 tests run: 4 passed, 0 skipped` (the new regression test plus
  all 3 pre-existing real-Vulkan-backed integration tests, confirming no regression to the success
  path). `cargo clippy -p minvulkan --all-targets --all-features -- -D warnings` -- clean.

## Generalized Version

An FFI/driver handle wrapper type not implementing `Drop` is a deliberate, common pattern (the
underlying resource's destruction often needs a specific call with specific arguments, or must
happen in a specific order relative to sibling resources, which a blanket `Drop` impl can't always
get right) -- but it means every early-return path between "resource created" and "resource handed
off to its final owner" must destroy it explicitly, and this is easy to miss because Rust's own
ownership/move semantics silently accept the drop without complaint; there is no compiler warning
for "you dropped a value that needed manual cleanup." Before trusting `?`/`.ok_or()?` to be a safe
way to propagate an error past an already-acquired FFI resource, check whether that resource's
type implements `Drop` for the specific cleanup being assumed. Also worth noting for any future
fix of this shape: prefer `.ok_or_else()`/`.map_err()` (lazy) over `.ok_or()` (eager) whenever the
"or" branch has a side effect -- an eager combinator runs its argument unconditionally, which for
a destructive side effect like this one would corrupt the success path, not just the error path.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found during a systematic bug-hunting pass across the 3 workspace crates with no prior recorded investigation this session (`module/alias/browser_tools`, `module/min/minvulkan`, `module/helper/gl_uniforms`) -- `browser_tools` and `gl_uniforms` were investigated and found clean (trivial re-export crate; thin wrapper whose one plausible lead, a null-uniform-location panic, was traced into `minwebgl`'s actual trait impls and disproven), `minvulkan` yielded this genuine leak. Root cause: `ash`'s `Instance`/`Device`/`Entry` types implement no `Drop`, so `context_finish`'s 3 early-return error paths (all occurring after `instance` was already live) silently leaked the `VkInstance` handle instead of calling `vkDestroyInstance`, confirmed by reading `ash` 0.38.0's own source. Fixed by adding a small `instance_cleanup_on_error` helper called from all 3 error sites, using `.ok_or_else`/`.map_err` (not eager `.ok_or`) to keep the cleanup confined to the error paths only. Verified via a source-inspection regression test (the same class of technique BUG-287/288 used for doc-only defects, here adapted to a cleanup-only defect that resists black-box runtime triggering against a real, non-mocked Vulkan driver), confirmed RED against the pristine source then GREEN post-fix, plus the full 4-test crate suite (including all 3 pre-existing real-Vulkan integration tests, confirming no regression) and clean clippy. `task/readme.md`'s `highest_id` stood at 289 at filing time, confirmed via a fresh on-disk scan across all `task/bug/` lifecycle subdirectories plus a task-side namespace cross-check immediately before filing. |

# BUG-262: `weights_sequence` panics on `[T]::chunks( 0 )` when a glTF mesh has morph-target
animation channels but zero declared morph targets

- **Severity:** High (crashes the whole glTF animation load for any valid glTF asset combining a
  `weights` animation channel with an empty/absent `mesh.weights` default array -- not a rare
  malformed-input edge case, but a legitimate combination the glTF spec allows)
- **state:** Completed
- **Affects:** `webgl::animation::loaders::gltf::weights_sequence`
  (`src/webgl/animation/loaders/gltf.rs`), reached via `load`'s `Property::MorphTargetWeights` arm
- **Component:** `module/helper/renderer` (`src/webgl/animation/loaders/gltf.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`weights_sequence` computes `weights.chunks( components * targets )` to split a flat morph-weight
output buffer into one chunk per keyframe. `[T]::chunks` panics unconditionally
(`"chunk size must be non-zero"`) when given a chunk size of `0`. A glTF mesh can legitimately
carry a `weights` animation channel targeting its morph targets while omitting the mesh's own
optional `weights` default-value array -- in that case `DisplacementsData::morph_weights_get()`
stays empty, and `load()`'s `Property::MorphTargetWeights` arm passes that length straight through
as `targets`, reaching `weights_sequence` with `targets == 0` and `components * targets == 0`.

## Impact

**Who is affected:** any consumer loading a glTF asset via
`webgl::animation::loaders::gltf::load` whose mesh has a morph-target `weights` animation channel
but no `mesh.weights` default array -- a legitimate, spec-permitted authoring combination (some
export pipelines omit the default-weights array when every weight is fully driven by animation).

**What breaks:** the entire glTF load panics with `"chunk size must be non-zero"` at the
`weights.chunks(..)` call site -- not a graceful `None`/skip for the one affected animation
channel, but an unrecoverable panic that aborts loading the whole asset, including every other
channel/mesh/node that would otherwise have loaded successfully.

**Entity Scope:** `None` -- source-level missing-guard defect in a loader function, not entity
directory instances.

## How Discovered

During this session's Group H review of `module/helper/renderer/src/webgl/animation/loaders/*`,
direct trace of `weights_sequence`'s `targets` parameter back to its caller (`load`'s
`Property::MorphTargetWeights` arm, which derives it from
`DisplacementsData::morph_weights_get().len()`) revealed no zero-check before the value reaches
`weights.chunks( components * targets )` -- combined with confirming `[T]::chunks` panics
unconditionally on a zero chunk size (`std` documentation/behavior), and that
`morph_weights_get()` can legitimately return an empty slice when a mesh's `weights` default array
is absent.

## Minimum Reproducible Example

No GL context or asset files are needed -- an inline-JSON glTF fixture (matching this test file's
existing `gltf::Gltf::from_slice` pattern) with one `weights` animation channel is enough to
obtain a real `Channel`, then call `weights_sequence` directly with `targets: 0`. See
`tests/gltf_animation_loader_test.rs::weights_sequence_returns_none_instead_of_panicking_when_targets_is_zero`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --test gltf_animation_loader_test --features animation
```
**Expected** (fixed): 5 passed. **Actual** (pre-fix, confirmed via temporary direct-source-edit
revert of the `targets == 0` guard and rerun): 4 passed, 1 failed -- panicked at
`module/helper/renderer/src/webgl/animation/loaders/gltf.rs:319:19: chunk size must be non-zero`.

## Root Cause

`weights_sequence` (pre-fix):
```rust
fn weights_sequence
(
  channel : &Channel< '_ >,
  buffers : &[ Vec< u8 > ],
  targets : usize
)
-> Option< Sequence< Tween< Vec< f64 > > > >
{
  let ( components, times, values ) = channel_decode( channel, buffers )?;
  // ... eventually:
  weights.chunks( components * targets )
  // ...
}
```
No guard existed for `targets == 0` before it reached `.chunks(..)`. `components` is always `≥ 1`
(`channel_decode` returns `1` or `3`), so `components * targets == 0` exactly when `targets == 0`
-- and `[T]::chunks` panics unconditionally on a zero chunk size, rather than returning an empty
iterator.

## Why Not Caught

No test exercised `weights_sequence` at all prior to this bug --
`gltf_animation_loader_test.rs` only covered `channel_decode`/`vec3_sequence`, and no existing
glTF test fixture combined morph-target animation channels with an absent `mesh.weights` array.
`weights_sequence` was also module-private (no `pub`), so it could not previously be exercised
directly from an external test file at all.

## Fix Applied (2026-08-17)

**`src/webgl/animation/loaders/gltf.rs`:** `weights_sequence` now returns `None` immediately when
`targets == 0`, before reaching `.chunks(..)`:
```rust
pub fn weights_sequence
(
  channel : &Channel< '_ >,
  buffers : &[ Vec< u8 > ],
  targets : usize
)
-> Option< Sequence< Tween< Vec< f64 > > > >
{
  if targets == 0
  {
    return None;
  }

  let ( components, times, values ) = channel_decode( channel, buffers )?;
  // ...
}
```
Every call site already handles `None` gracefully via `let Some( sequence ) = weights_sequence(
.. ) else { continue; }`, so the fix simply lets the pre-existing skip-on-`None` path handle the
zero-targets case instead of panicking before ever reaching it. Also changed from module-private
to `pub` (required so the new regression test, which lives in `tests/`, can call it directly), and
added to the `mod_interface!`'s `own use` export list alongside `load`/`channel_decode`/
`vec3_sequence`.

**`tests/gltf_animation_loader_test.rs`** (edited): added `weights_sequence` to the existing
import list; added a `MORPH_WEIGHTS_FIXTURE` inline-JSON glTF constant (one mesh, one morph-target
`weights` animation channel) and a `morph_weights_buffers()` helper encoding 2 keyframes' worth of
weight values; added 1 new native `#[ test ]` function,
`weights_sequence_returns_none_instead_of_panicking_when_targets_is_zero`, calling
`weights_sequence` directly with `targets: 0` and asserting it returns `None` instead of
panicking.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --test gltf_animation_loader_test --features animation` -- pre-fix
  (temporary direct-source-edit revert of the `targets == 0` guard): 4 passed, 1 failed, panicked
  at `gltf.rs:319:19: chunk size must be non-zero`. Post-fix (guard restored): 5 passed, 0 failed.
- `cargo test -p renderer --test blender_tests --test gltf_animation_loader_test --features
  animation` (combined scoped run, post-fix, alongside BUG-261's own fix): 23 passed + 5 passed, 0
  failed.
- `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean (see final
  workspace-scoped verification run below).

## Generalized Version

**Broken assumption:** a chunk-size parameter derived from optional/external input (here, a
mesh's own morph-target count, which depends on an optional `mesh.weights` array that authoring
tools may omit) will always be nonzero because the call site "usually" supplies a nonzero value.
`[T]::chunks( 0 )` panics unconditionally rather than returning an empty iterator, so any chunk
size computed from external/optional data must be checked for zero before use, regardless of how
rare the zero case seems in practice -- the glTF spec explicitly permits the combination that
triggers it here. Whenever a loader function derives a `.chunks(..)`/`.chunks_exact(..)` argument
from optional source data, add the zero-check at the top of the function, in the single place all
call paths pass through, rather than trusting every caller to have validated it first.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group H review of `renderer::webgl::animation::loaders::gltf`. Root cause: `weights_sequence` computed `weights.chunks( components * targets )` with no guard against `targets == 0`, which `[T]::chunks` panics on unconditionally; a glTF mesh with a morph-target `weights` animation channel but no `mesh.weights` default array legitimately reaches this with `targets == 0`. Fixed by returning `None` immediately when `targets == 0`, letting every call site's existing `let Some(..) else { continue; }` skip path handle it gracefully instead of panicking; also promoted the function from module-private to `pub` so a regression test in `tests/` could call it directly. Verified via 1 new native unit test (confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun -- pre-fix panics with `chunk size must be non-zero`), the combined scoped suite alongside BUG-261, and clean clippy. Filed as BUG-262, not BUG-259, after a concurrent session actor independently claimed BUG-259 (`SwapFramebuffer::new` doc-comment fix) between this session's initial claim to 259 and file-write time -- verified via a fresh repo-wide-grep re-scan immediately before writing, which also renumbered this fix's own source/test comments from BUG-259 to BUG-262. Closed same-session (Tier 2 Dual-Role Self-Check). |

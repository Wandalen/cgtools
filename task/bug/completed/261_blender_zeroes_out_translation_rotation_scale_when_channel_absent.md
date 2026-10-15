# BUG-261: `Blender`'s `translation_blend`/`rotation_blend`/`scale_blend` zero out a node's
transform whenever no blended `Sequencer` targets that channel

- **Severity:** High (silently corrupts skeletal pose data on every `Blender::set` call for any
  joint lacking a translation/rotation/scale channel -- extremely common in glTF rigs where a
  joint is animated via rotation only; no panic, but visibly wrong output: affected joints snap to
  `(0,0,0)` translation, `(0,0,0)` scale, or identity rotation instead of staying at their
  authored/previous pose)
- **state:** Completed
- **Affects:** `Blender::translation_blend`, `Blender::rotation_blend`, `Blender::scale_blend`
  (`src/webgl/animation/blending.rs`), reached via `AnimatableComposition::set` on every
  `Blender::set` call for every node in the target `FxHashMap`
- **Component:** `module/helper/renderer` (`src/webgl/animation/blending.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`Blender::set` calls `translation_blend`, `rotation_blend`, and `scale_blend` for every node in
the skeleton, each of which builds a `values : Vec< (T, f32) >` by scanning every weighted
`Sequencer` for a channel matching that node's name plus the relevant suffix
(`TRANSLATION_PREFIX`/`ROTATION_PREFIX`/`SCALE_PREFIX`). When none of the weighted animations
carry that channel for this node (`values` stays empty), all three functions previously fell
straight through to their accumulation loop and unconditionally called `node.borrow_mut()
.{translation,rotation,scale}_set(..)` with the loop's untouched `Default::default()` seed --
`F32x3::default()` (`(0,0,0)`) for translation/scale, `QuatF32::default()` (identity) for
rotation's explicit `else` branch. Every sibling `AnimatableComposition` implementation in this
module (`Sequencer`, `Pose`, `Scaler`, `Transition`) instead skips the `_set()` call entirely when
its channel is absent, per an established "skip-if-absent" convention this code silently violated.

## Impact

**Who is affected:** any consumer of `Blender` blending a `Sequencer` whose animation clips don't
cover every TRS channel for every joint -- the normal case for glTF rigs, where e.g. a joint
animated purely by rotation (no translation/scale keyframes at all) is extremely common. Every
`Blender::set` call on such a rig zeroed that joint's translation and collapsed its scale to zero,
while forcing its rotation to identity whenever *no* weighted animation targeted rotation either.

**What breaks:** affected joints visibly snap to the world origin (translation), disappear
(`(0,0,0)` scale collapses the joint's subtree to a point), or reset to identity orientation on
every single `set` call -- not a one-time initialization glitch, but a per-frame overwrite that
permanently discards whatever pose the joint previously had (its bind pose, or a pose set by a
different, non-`Blender` system).

**Entity Scope:** `None` -- source-level logic defect in a blending function, not entity directory
instances.

## How Discovered

During this session's Group H review of `module/helper/renderer/src/webgl/animation/*`, direct
comparison of `translation_blend`/`rotation_blend`/`scale_blend` against the sibling
`AnimatableComposition` implementations in `pose.rs`/`scaling.rs`/`transition.rs` (all of which
guard their own `_set()` call behind a channel-present check) revealed that `blending.rs`'s three
functions had no equivalent guard -- each built its `values` vector, then applied the accumulated
result to the node unconditionally regardless of whether `values` was ever populated.

## Minimum Reproducible Example

No GL context is needed -- `Blender::set` operates purely on `Node`'s CPU-side transform fields.
Construct a `Blender` with one weighted `Sequencer` carrying only a rotation channel for a node,
give that `Node` a known non-default translation/scale, call `Blender::set`, and assert the
translation/scale are unchanged. See
`tests/blender_tests.rs::test_blender_leaves_translation_and_scale_untouched_when_only_rotation_channel_present`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --test blender_tests --features animation
```
**Expected** (fixed): 23 passed. **Actual** (pre-fix, confirmed via temporary direct-source-edit
revert of all three `is_empty()` guards and rerun): 22 passed, 1 failed -- the new test's
translation/scale assertions failed (`translation.x() == 3.0` etc. failed because `translation_set`
had overwritten the node's translation with `(0,0,0)`).

## Root Cause

`translation_blend`/`scale_blend` (pre-fix), after the values-gathering loop:
```rust
if self.normalize { weights_normalize( &mut values ); }
let mut translation = F32x3::default();
for ( t, w ) in values { translation += t * w; }
node.borrow_mut().translation_set( translation );
```
With `values` empty, this loop body never executes, and `translation` stays at
`F32x3::default()` (`(0,0,0)`) -- which is then unconditionally written to the node.
`rotation_blend`'s `else` branch on the `values_iter.next()` match was even more explicit about
the empty case, but explicitly *wrong*: it force-set the node's rotation to `QuatF32::default()`
(identity) instead of leaving it untouched. None of the three functions distinguished "no
contribution" (should leave the node alone) from "contribution summing to zero" (a legitimately
computed zero) -- for translation/scale those two states collapse to the identical `(0,0,0)`
value, so an accumulator seeded from `Default::default()` cannot be applied safely without an
explicit emptiness check first.

## Why Not Caught

No existing test in `blender_tests.rs` constructed a `Sequencer` that omitted a channel for an
otherwise-present node -- every existing test either supplied all three channels or checked
behavior unrelated to partial channel coverage (weight normalization, hemisphere alignment,
completion tracking). The bug produces no panic and no compiler warning; it is only observable by
comparing a node's transform before and after `Blender::set` when a channel is deliberately
absent.

## Fix Applied (2026-08-17)

**`src/webgl/animation/blending.rs`:** all three functions now return immediately, before applying
any accumulated result, when `values.is_empty()`:
```rust
if values.is_empty()
{
  return;
}
```
(`translation_blend`/`scale_blend` use this exact guard; `rotation_blend`'s existing `else` branch
on `values_iter.next()` was changed from force-setting identity to simply `return;`.) This matches
the "skip-if-absent" convention already used by `Sequencer`/`Pose`/`Scaler`/`Transition` elsewhere
in this module.

**`tests/blender_tests.rs`** (edited): 1 new native `#[ test ]` function,
`test_blender_leaves_translation_and_scale_untouched_when_only_rotation_channel_present`,
constructing a `Blender` with a single weighted `Sequencer` carrying only a rotation channel,
giving the target `Node` known non-default translation/scale, calling `Blender::set`, and
asserting translation/scale are unchanged while rotation did actually blend away from identity
(proving the fix doesn't also suppress the channel that *is* present).

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --test blender_tests --features animation` -- pre-fix (temporary
  direct-source-edit revert of all three `is_empty()`/`return` guards): 22 passed, 1 failed (the
  new test's translation/scale assertions). Post-fix (guards restored): 23 passed, 0 failed.
- `cargo test -p renderer --test blender_tests --test gltf_animation_loader_test --features
  animation` (combined scoped run, post-fix, alongside BUG-262's own fix): 23 passed + 5 passed, 0
  failed.
- `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: clean (see final
  workspace-scoped verification run below).

## Generalized Version

**Broken assumption:** an accumulator seeded from a type's `Default::default()` value can always
be applied to its target unconditionally after a "gather contributions" loop -- this only holds
when "no contributions gathered" and "the type's default/identity value" are the same observable
outcome. For a blending function whose job is "leave untouched when absent, blend when present",
`Default::default()` (`(0,0,0)` for a translation/scale vector) is not equivalent to "no
contribution" (should skip the write entirely) -- the two must be distinguished explicitly via an
emptiness check on the contributions collection before applying the accumulated result. Whenever
adding a new per-channel blend function to a composition system that already has an established
skip-if-absent convention among its siblings, verify the new function follows the same convention
rather than assuming the accumulator's own default naturally produces a no-op.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during Group H review of `renderer::webgl::animation::blending`. Root cause: `translation_blend`/`rotation_blend`/`scale_blend` applied their accumulated blend result to the target node unconditionally, even when no weighted `Sequencer` carried that channel for the node -- overwriting the node's existing transform with the accumulator's `Default::default()` seed (`(0,0,0)` translation/scale, identity rotation) instead of leaving it untouched, unlike every sibling `AnimatableComposition` impl's established skip-if-absent convention. Fixed by adding an explicit `values.is_empty()` guard (or, for rotation, correcting the existing empty-case branch) before applying the accumulated result. Verified via 1 new native unit test (confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun), the combined scoped suite alongside BUG-262, and clean clippy. Filed as BUG-261, not BUG-258, after discovering two concurrent session actors had already claimed BUG-258 (an IBL program-cache bug) and, after this session's own initial claim to 258 collided, BUG-259 was also independently claimed by a third concurrent actor's `SwapFramebuffer::new` doc-comment fix between this session's scans -- verified via a fresh repo-wide grep re-scan immediately before writing. Closed same-session (Tier 2 Dual-Role Self-Check). |

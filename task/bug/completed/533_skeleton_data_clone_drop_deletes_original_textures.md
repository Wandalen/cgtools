# BUG-533: Dropping a clone of an uploaded `TransformsData`/`DisplacementsData` deleted the original's textures

- **Severity:** Medium (once a clone taken after `upload()` is dropped first, the original
  binds deleted textures every frame: its skinning / morph-target data is uploaded into, and
  sampled from, whatever textures the units still hold)
- **state:** Completed
- **Affects:** Every clone of an uploaded `TransformsData` / `DisplacementsData` -- they are cloned
  whenever a skinned or morph-target mesh is cloned (`Skeleton` / `Mesh` / `Node::tree_clone`).
- **Component:** `module/helper/renderer` (`src/webgl/skeleton.rs`)
- **repo_identity:** self
- **Filed:** 2026-09-28
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-10-01
- **Related Bugs:** Follow-up to BUG-437, whose `Drop` made the aliased `Clone` destructive; same
  clone-must-not-arm-`Drop` rule as BUG-440 (`IBL`).

## Symptom

After `upload()`, `TransformsData` holds `global_texture` / `inverse_texture` and `gl = Some`
(`DisplacementsData`: `displacements_texture`). `Clone` copied the texture handles (aliasing the
same GPU textures, re-allocated only by the clone's own first `upload()`) **and** `gl`. Dropping
such a clone before its first `upload()` ran the BUG-437 `Drop`, which deleted the textures the
original was still bound to.

## Impact

**Who is affected:** Any scene that clones a skinned or morph-target mesh and drops the clone (or
a subtree containing it) before it has been rendered once.

**What breaks:** Every frame the original binds its deleted textures to upload and sample them.
WebGL rejects each bind with `INVALID_OPERATION` and keeps the previous binding, so
`TransformsData::upload`'s `texture_data_4f_load` writes the joint matrices (RGBA32F, with
NEAREST / CLAMP parameters) into whatever texture unit 0 still holds, possibly another object's,
and the skinning / morph-target samplers read whatever their units held before. They read zero
(an incomplete texture) only when nothing was bound there.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Code review of PR #208 (renderer `Drop` work), which already carried the fix without a record.

## Minimum Reproducible Example

```rust
let original = TransformsData::new_owning_for_test( global_texture.clone(), inverse_texture.clone(), &gl );
let clone = original.clone();
drop( clone );
// pre-fix: false -- the clone's Drop deleted the original's texture
assert!( gl.is_texture( global_texture.as_ref() ) );
```

**Verify Command:**
```bash
cd module/helper/renderer && wasm-pack test --headless --chrome -- --features test_internals --test tests
```

## Root Cause

`impl Clone for TransformsData` / `DisplacementsData` copied the texture handles together with
`gl : self.gl.clone()`. `gl` is what arms `Drop` (BUG-437), so the clone deleted handles it only
aliased.

## Why Not Caught

The BUG-437 reproducers only drop an owning value; none clones one first. The BUG-437 pitfall
comment covered only the opposite order (original dropped before the clone's first `upload()`).

## Fix Location

- `module/helper/renderer/src/webgl/skeleton.rs`: both `Clone` impls reset the texture handles and
  `gl` to `None`, so a clone never holds the original's textures; its first `upload()` allocates its
  own through the existing `is_none()` path, as for a new value, and the former `need_clone_inner`
  reallocation is gone. `Fix(BUG-533)` comments with root cause and pitfall sit on both impls.
  (The first fix only reset `gl`, which stayed safe only through `upload()`'s reallocation order.)

## Prevention

Two reproducers in `module/helper/renderer/tests/webgl/skeleton_gl_lifecycle.rs`
(`transforms_data_clone_drop_keeps_original_textures`,
`displacements_data_clone_drop_keeps_original_texture`): clone an owning value, assert the clone
holds no texture handle, drop the clone, assert the original's textures are still live, then drop
the original and assert they are freed. Two more
(`transforms_data_clone_uploads_its_own_textures_in_either_drop_order`,
`displacements_data_clone_uploads_its_own_texture_in_either_drop_order`) upload clones and drop
them in both orders. Restoring the copied handles and `gl` in either `Clone` fails all four.

## Pitfall

`Clone` must never copy a GPU handle that `Drop` deletes -- reset it and let the clone allocate its
own; `IBL` follows the same rule (BUG-440).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-09-28 | filed | Raised by the PR #208 review; the `gl : None` fix was already in the PR without a record. |
| 2026-09-28 | fixed | `Fix(BUG-533)` comments on both `Clone` impls, two reproducers. |
| 2026-10-01 | fixed | Second PR #208 review: `Clone` resets the texture handles too, `need_clone_inner` removed, two upload-after-clone tests. |
| 2026-10-01 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: `wasm-pack test --headless --chrome -- --features test_internals --test tests` passes 26/26. Adversarial pass: with both `Clone` impls reverted to copying the texture handles and `gl`, the four clone tests fail and the two BUG-437 drop tests still pass (22/26). | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-533)` / `Root cause` / `Pitfall` comments on both `Clone` impls; the BUG-437 `Drop` Pitfalls name the handle reset; 5-section doc comments on both reproducers. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to the two `Clone` impls, the removed `need_clone_inner` field and its `upload()` branches, and two `test_internals` accessors; no caller reads a clone's handles before its `upload()`. | — |

**Reproduced:** YES -- the reverted-`Clone` run above fails both original reproducers (the
original's textures are gone after the clone drops). 2026-10-01.

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/skeleton.rs` | `Clone` for `TransformsData` / `DisplacementsData` resets the texture handles and `gl` to `None`; `need_clone_inner` removed; `Fix(BUG-533)` comments; `textures_for_test` / `texture_for_test` accessors. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/skeleton_gl_lifecycle.rs` | Added the two `bug_reproducer(BUG-533)` tests and the two upload-after-clone tests. |

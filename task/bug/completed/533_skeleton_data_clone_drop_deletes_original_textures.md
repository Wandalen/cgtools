# BUG-533: Dropping a clone of an uploaded `TransformsData`/`DisplacementsData` deleted the original's textures

- **Severity:** Medium (the original keeps sampling deleted textures: skinning / morph targets
  silently read nothing once any clone taken after `upload()` is dropped first)
- **state:** Completed
- **Affects:** Every clone of an uploaded `TransformsData` / `DisplacementsData` -- they are cloned
  whenever a skinned or morph-target mesh is cloned (`Skeleton` / `Mesh` / `Node::tree_clone`).
- **Component:** `module/helper/renderer` (`src/webgl/skeleton.rs`)
- **repo_identity:** self
- **Filed:** 2026-09-28
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

**What breaks:** The original mesh samples deleted textures; WebGL treats them as incomplete, so
joint transforms / morph displacements read as zero.

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

`impl Clone for TransformsData` / `DisplacementsData` copied `gl : self.gl.clone()`. `gl` is what
arms `Drop` (BUG-437), so the clone believed it owned handles that were only aliases.

## Why Not Caught

The BUG-437 reproducers only drop an owning value; none clones one first. The BUG-437 pitfall
comment covered only the opposite order (original dropped before the clone's first `upload()`).

## Fix Location

- `module/helper/renderer/src/webgl/skeleton.rs`: both `Clone` impls set `gl : None`; a clone owns
  no GPU texture until its own `upload()` allocates fresh ones (`need_clone_inner`). `Fix(BUG-533)`
  comments with root cause and pitfall sit on both impls.

## Prevention

Two reproducers in `module/helper/renderer/tests/webgl/skeleton_gl_lifecycle.rs`
(`transforms_data_clone_drop_keeps_original_textures`,
`displacements_data_clone_drop_keeps_original_texture`): clone an owning value, drop the clone,
assert the original's textures are still live, then drop the original and assert they are freed.
Restoring `gl : self.gl.clone()` in either `Clone` fails the matching test.

## Pitfall

Whatever field makes a value delete GPU resources in `Drop` must be reset, never copied, by a
`Clone` that aliases the handles -- the same rule `IBL` follows (BUG-440).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-09-28 | filed | Raised by the PR #208 review; the `gl : None` fix was already in the PR without a record. |
| 2026-09-28 | fixed | `Fix(BUG-533)` comments on both `Clone` impls, two reproducers, BUG-437 comment indentation restored. |

## Refs: src/

| File | Change |
|------|--------|
| `module/helper/renderer/src/webgl/skeleton.rs` | `Clone` for `TransformsData` / `DisplacementsData` resets `gl` to `None`; `Fix(BUG-533)` comments. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/helper/renderer/tests/webgl/skeleton_gl_lifecycle.rs` | Added the two `bug_reproducer(BUG-533)` tests. |

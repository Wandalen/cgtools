# BUG-256: `Mesh::clone`'s `skeleton` field aliases the original's `Rc< RefCell< Skeleton > > `
instead of producing an independent clone

- **Severity:** Medium (not a crash/panic, but breaks `Node::tree_clone`'s documented "new
  independent scene graph subtree" invariant: two "independent" `Mesh` instances silently share
  one `Skeleton`, so posing/animating one instance's skeleton silently affects the other's too)
- **state:** Completed
- **Affects:** `Mesh::clone` (`src/webgl/mesh.rs`), reached via `Node::tree_clone`
  (`src/webgl/node.rs`) for any node whose `Object3D::Mesh` carries `Some( skeleton )` -- 6 real
  call sites workspace-wide (see Impact)
- **Component:** `module/helper/renderer` (`src/webgl/mesh.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`Mesh`'s manual `Clone` impl builds each field independently: `primitives` correctly wraps a
fresh `Rc::new( RefCell::new( p.borrow().clone() ) )` per element, but `skeleton` instead cloned
the `Skeleton` *value*, wrote that clone back into `self`'s own `RefCell` (mutating the object
being cloned -- a side effect `Clone::clone( &self )` should never have), and then returned
`self`'s original `Rc` for the "cloned" `Mesh`'s `skeleton` field too -- so the original and the
clone ended up sharing the exact same `Rc< RefCell< Skeleton > > ` allocation.

## Impact

**Who is affected:** any consumer of `Node::tree_clone` (`src/webgl/node.rs`, doc-commented
"Clones the node and all of its descendants, creating a new independent scene graph subtree") on
a node whose `Object3D::Mesh` carries `Some( skeleton )` -- i.e. any skinned/animated mesh
instance. Confirmed via workspace-wide grep: `tree_clone` has 6 real call sites across
`examples/minwebgl/{lottie_surface_rendering, pbr_lighting (x2), animation_surface_rendering (x2),
curve_surface_rendering}`, plus internal recursive use in `scene.rs` and `skeleton.rs`'s own
`TransformsData::clone` (joint cloning).

**What breaks:** two "independent" `Mesh` instances produced by cloning end up sharing one
`Skeleton` -- posing/animating one instance's skeleton silently poses/animates the other's too,
since both `RefCell`s are the same allocation. Independently, since `Clone::clone` is documented/
expected to take `&self` without observable side effects, this implementation mutated the source
`Mesh`'s own skeleton in place as an unintended side effect of cloning it.

**Entity Scope:** `None` -- source-level `Clone`-impl correctness defect, not entity directory
instances.

## How Discovered

During this session's `renderer` crate scout (task #174), direct full-file review of
`src/webgl/mesh.rs` compared `Mesh::clone`'s `skeleton` arm against its own `primitives` arm two
lines above (and against `Primitive::clone` in `primitive.rs`, which deep-clones both `geometry`
and `material` via the identical `Rc::new( RefCell::new( x.borrow().clone() ) )` pattern) -- the
`skeleton` arm's `let clone = s.borrow().clone(); *s.borrow_mut() = clone; s.clone();` shape stood
out as the only place in the whole crate using this pattern (confirmed via workspace-wide grep for
`borrow_mut() = clone`, zero other matches), and traced back to `Node::tree_clone`'s own doc
comment plus `skeleton.rs`'s `TransformsData::clone` (which deliberately deep-clones `joints` via
`tree_clone` and sets `need_clone_inner`/`need_update_inverse` specifically to recreate GPU
textures on next upload) to confirm independent-clone semantics were clearly the intended
contract -- never actually exercised, because `Mesh::clone` never produced an independent `Rc` in
the first place.

## Minimum Reproducible Example

No GL context is needed -- `Skeleton::default()` (`transforms: None, displacements: None`) is
enough to construct a `Mesh` carrying `Some( skeleton )` and observe `Mesh::clone`'s `Rc` handling
directly via `Rc::strong_count`/`Rc::ptr_eq`. See `tests/webgl/mesh.rs`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --test tests webgl::mesh::
```
**Expected** (fixed): 1 passed. **Actual** (pre-fix, confirmed via temporary direct-source-edit
revert-and-rerun of the `skeleton` arm): 1 failed -- panics at the `!Rc::ptr_eq( &original_skeleton,
&cloned_skeleton )` assertion ( `Rc::ptr_eq` reads `true`, i.e. the clone aliased the original's
allocation ); the test's final `Rc::strong_count( &original_skeleton ) == 2` assertion never
executes on this path, since `assert!` panics immediately at the first failing check.

## Root Cause

`Mesh::clone` (pre-fix):
```rust
skeleton : self.skeleton.as_ref()
.map
(
  | s |
  {
    let clone = s.borrow().clone();
    *s.borrow_mut() = clone;
    s.clone()
  }
),
```
This clones the `Skeleton` *value* (`s.borrow().clone()`), immediately writes that clone back into
`self`'s own `RefCell` (`*s.borrow_mut() = clone` -- a value-identical overwrite of the source,
with no observable effect other than mutating `self` as a side effect of `.clone()`), and then
returns `s.clone()` -- which, since `s : &Rc< RefCell< Skeleton > > `, clones the `Rc` *pointer*,
bumping its strong count and handing the new `Mesh` a second reference to the same allocation
`self.skeleton` still points to. The `primitives` field two lines above does this correctly --
`Rc::new( RefCell::new( p.borrow().clone() ) )` -- wrapping the cloned value in a brand-new `Rc`;
the `skeleton` arm never performed the equivalent `Rc::new( RefCell::new( .. ) )` wrap.

## Why Not Caught

`Mesh` had zero test coverage of any kind prior to this bug -- no existing test constructed a
`Mesh` at all, let alone cloned one with `skeleton` set. The bug produces no panic and no compiler
warning (`Rc< RefCell< T > > ::clone()` type-checks identically whether it duplicates the pointer
or the pointee), and is only observable by comparing `Rc` identity across a clone, which nothing
in the codebase did.

## Fix Applied (2026-08-17)

**`src/webgl/mesh.rs`:** the `skeleton` arm now mirrors `primitives`'s own pattern exactly:
```rust
skeleton : self.skeleton.as_ref()
.map( | s | Rc::new( RefCell::new( s.borrow().clone() ) ) ),
```
A fresh `Rc` wrapping a clone of the pointee -- `self` is never mutated as a side effect of
cloning it, and the new `Mesh` gets its own independent `Skeleton` allocation, matching
`Primitive::clone`'s identical `geometry`/`material` pattern elsewhere in this same crate.
`TransformsData::clone`'s own already-existing `need_clone_inner`/`need_update_inverse` machinery
(deliberately built to recreate GPU textures the first time an independently-cloned `Skeleton`
next uploads, since a shallow `WebGlTexture` handle clone would otherwise alias the same GPU
texture) is now actually exercised as designed, since a genuine `Skeleton::clone()` finally
produces a value that gets wrapped in its own `Rc` instead of being discarded back into the
source.

**`tests/webgl/mesh.rs`** (new file): 1 new native `#[ test ]` function,
`clone_gives_the_clone_its_own_independent_skeleton`, constructing a `Mesh` with
`Some( Skeleton::default() )`, cloning it, and asserting via `Rc::ptr_eq`/`Rc::strong_count` that
the clone's `skeleton` is a distinct allocation from the original's, and that cloning does not
bump the original's strong count.

**Not touched:** `src/webgl/skeleton.rs`'s `TransformsData::clone` deep-clones each joint via
`Node::tree_clone`, producing joint `Rc< RefCell< Node > > `s disconnected from whatever scene
graph the original joints belonged to (rather than re-wiring them to the newly-cloned subtree) --
this is a separate, pre-existing design question in a file outside this bug's scope (`skeleton.rs`
was not part of this review's assigned file set), and is not newly introduced or worsened by this
fix; noted here for visibility, not fixed.

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --test tests webgl::mesh::` -- pre-fix (temporary direct-source-edit
  revert of the `skeleton` arm back to the aliasing version): 1 failed, panicking at the
  `!Rc::ptr_eq( .. )` assertion (the clone aliased the original's `Skeleton` allocation). Post-fix
  (arm restored): 1 passed, 0 failed.
- `cargo test -p renderer --test tests` (full scoped suite, post-fix): first run caught an
  unrelated failure, `webgl::camera::from_bounding_box_accepts_a_degenerate_zero_radius_box` --
  a concurrent session actor's own uncommitted, in-progress work in `tests/webgl/camera.rs`
  (confirmed absent from `git show HEAD:module/helper/renderer/tests/webgl/camera.rs`, and the
  underlying files' mtimes were more recent than this fix's own edits), entirely unrelated to
  `Mesh`/`Skeleton`/this fix. A later re-run (after that actor's own work progressed further, not
  touched by this fix) came back **71 passed, 0 failed**.
- `cargo clippy -p renderer --all-targets --all-features -- -D warnings`: exit 0, clean (checked
  both before and after the concurrent `camera.rs` failure resolved itself).

## Generalized Version

**Broken assumption:** a `Clone` impl's per-field arms are each independently correct just because
the overall `impl Clone` compiles and the struct's own `Clone` bound is satisfied -- an
`Rc< RefCell< T > > ` field's clone arm can silently alias instead of duplicate (`s.clone()` on the
`Rc` itself vs. `Rc::new( RefCell::new( s.borrow().clone() ) )`) with identical type-checking, and
nothing but a runtime identity check (`Rc::ptr_eq`/`Rc::strong_count`) or a side-by-side comparison
against a sibling field's already-correct arm in the same `impl` catches the divergence. Whenever a
struct's manual `Clone` impl has more than one `Rc< RefCell< _ > > `-shaped field, verify every arm
follows the same wrap-in-a-fresh-`Rc` pattern -- do not assume a later arm inherited the same
correctness as an earlier one just because they sit in the same literal.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found by direct full-file review of `mesh.rs` during task #174's `renderer` crate scout. Root cause: `Mesh::clone`'s `skeleton` arm cloned the `Skeleton` value but discarded the clone back into the source's own `RefCell` instead of a new `Rc`, then returned the source's original `Rc` for the "cloned" `Mesh` too -- both ended up sharing one `Skeleton` allocation, unlike the correctly-independent `primitives` arm two lines above. Fixed by wrapping the cloned value in a fresh `Rc::new( RefCell::new( .. ) )`, mirroring `primitives`'s own pattern. Verified via 1 new native unit test (confirmed fail pre-fix / pass post-fix via temporary revert-and-rerun -- pre-fix panics at the `!Rc::ptr_eq( .. )` assertion), the full scoped suite (a first run hit an unrelated concurrent actor's in-flight `camera.rs` failure, confirmed via `git show HEAD` absence and file mtimes; a later re-run came back 71/71 passing), and clean clippy. Filed as BUG-256, not BUG-255, after discovering the concurrent session actor had already claimed BUG-255 (`lights_spot_push_equal_cone_angles_nan`) between this session's initial disk scan and file-write time -- verified via a fresh re-scan immediately before writing. Closed same-session (Tier 2 Dual-Role Self-Check). |

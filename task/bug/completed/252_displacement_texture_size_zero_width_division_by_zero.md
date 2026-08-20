# BUG-252: `Skeleton`'s morph-target displacement-texture sizing formula collapses row width to
`0` for primitives with few vertices relative to their attribute/target count, dividing by zero
and permanently disabling the displacement-texture update

- **Severity:** Medium (no crash/panic -- the saturating `f32 as u32` cast absorbs the resulting
  `+inf`, and the caller's own size-limit check catches the saturated value -- but the affected
  primitive's morph-target displacement update is then silently and permanently abandoned every
  frame, with a misleading "texture too large" console error masking the real cause)
- **state:** Completed
- **Affects:** `Skeleton`'s `displacements_update` (called whenever a primitive with morph targets
  needs its displacement data texture rebuilt), via the new `displacement_texture_size_compute`
- **Component:** `module/helper/renderer` (`src/webgl/skeleton.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`displacements_update` sizes its displacement data texture by computing `i`, the number of whole
`vertex_displacement_len`-wide ( `attributes_count * targets_count` texels-per-vertex ) blocks
that fit in `sqrt(data.len())`, via a plain `.floor()`. Whenever a primitive's vertex count is
small relative to its attribute/target count -- concretely, whenever `vertices_count * 4 <
attributes_count * targets_count` -- `sqrt(data.len()) < vertex_displacement_len`, so the only
representable floored multiple is the zeroth one: `i = 0`, collapsing the texture row width `a`
to `0`. The height `b` is then computed as `(data.len() as f32 / a as f32).ceil()` -- a division
by zero -- which for any positive numerator produces `f32::INFINITY`, and `f32::INFINITY as u32`
saturates ( Rust 1.45+ semantics, no panic ) to `u32::MAX`.

## Impact

**Who is affected:** Any glTF asset containing a primitive whose vertex count is small relative to
its combined attribute-count × morph-target-count ( e.g. a handful of vertices carrying several
morph targets across position/normal/tangent -- a shape more likely in small decorative or
facial-rig sub-meshes than in bulk geometry, but not contrived: `attributes_count=1,
targets_count=10, vertices_count<=2` is enough to trigger it ).

**What breaks:** `a.max(b) > max_size` -- the very next check after the sizing call -- always
catches the saturated `u32::MAX`, so `displacements_update` returns early without writing the
texture and, critically, without clearing whatever flag gates the update (`need_update_displacement`
stays `true`). The affected primitive's morph-target displacement is never applied, silently,
every single frame, indistinguishable from "this primitive legitimately has no morph targets" --
while the console repeats a "texture too large" error that has nothing to do with the actual
0-width division-by-zero root cause.

**Entity Scope:** `None` -- source-level arithmetic defect, not entity directory instances.

## How Discovered

During this session's `renderer` crate scout (task #174), a dispatched sub-review of
`skeleton.rs` -- looking specifically for skinning/bone-transform math bugs -- found no defects in
the actual GPU-skinning matrix logic ( that composition happens in shader code, not this file;
this file only does CPU-side data-texture packing/sizing ), but hand-deriving the boundary
condition of the ( structurally similar, newly-added ) displacement-texture sizing formula
alongside the file's pre-existing, already-tested `data_texture_size_calculate` surfaced this
zero-width degenerate case.

## Minimum Reproducible Example

The formula is pure `usize`/`f32` arithmetic with no GL dependency, so it was extracted into its
own function, `displacement_texture_size_compute`, and made `pub` ( exported via `mod_interface!`,
alongside the file's existing `data_texture_size_calculate` precedent ), directly unit-testable.
See `tests/webgl/displacement_texture_size.rs`.

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p renderer --test tests webgl::displacement_texture_size
```
**Expected** (fixed): all 3 tests pass. **Actual** (pre-fix, confirmed via temporary direct-edit
revert-and-rerun of the `.max(1.0)` clamp): 2 of 3 fail -- the reproducer asserts `a != 0` and
gets `a == 0`; the capacity-invariant sweep sees `a * b * 4` collapse to `0` for the same input
(the third test, covering an already-safe ordinary vertex count, is unaffected either way).

## Root Cause

`displacements_update` (pre-fix, inlined):
```rust
let v = vertex_displacement_len as f32;
let i = ( ( data.len() as f32 ).sqrt() / v ).floor();
let a = ( v * i ) as u32;
let b = ( data.len() as f32 / a as f32 ).ceil() as u32;
```
`i` is "how many whole `v`-wide blocks fit in `sqrt(data.len())`," rounded down. Rounding down is
correct in general, but when `sqrt(data.len()) < v` -- i.e. `data.len() < v * v` -- the only
representable floored count is `0`, and nothing in the formula guards against that: `a` becomes
`0`, and `b`'s division by `a` is a division by zero. `f32` division never panics on a zero
divisor; `data.len() as f32 / 0.0` for any positive `data.len()` is `+inf`, and the subsequent
`as u32` cast saturates ( no UB, no panic, Rust 1.45+ ) to `u32::MAX` -- which then reads as a
plausible-looking ( if enormous ) texture height to every caller downstream, until the size-limit
check catches it for an unrelated-looking reason.

## Why Not Caught

No test exercised the sizing formula with a vertex count small relative to the attribute/target
count -- the closest existing coverage, `skeleton_tests.rs`'s `data_texture_size_calculate` suite,
targets a different, already-safe formula ( a clean power-of-4 growth with no floor-to-zero
boundary ) in the same file. The failure mode has no crash and no obviously-wrong render: the
existing `a.max(b) > max_size` guard ( intended purely as an oversized-texture safety limit )
incidentally also catches this unrelated division-by-zero's `u32::MAX` symptom, so the update is
cleanly abandoned with a console error that reads exactly like a legitimate size-limit hit --
nothing signals that `a` itself had already collapsed to `0` via unrelated arithmetic before the
limit check ever ran.

## Fix Applied (2026-08-17)

**`src/webgl/skeleton.rs`:** extracted the computation into its own function,
`displacement_texture_size_compute( data_len : usize, vertex_displacement_len : usize ) -> ( u32,
u32 )`, and changed `.floor()` to `.floor().max( 1.0 )`:
```rust
pub fn displacement_texture_size_compute( data_len : usize, vertex_displacement_len : usize ) -> ( u32, u32 )
{
  let v = vertex_displacement_len as f32;
  let i = ( ( data_len as f32 ).sqrt() / v ).floor().max( 1.0 );
  let a = ( v * i ) as u32;
  let b = ( data_len as f32 / a as f32 ).ceil() as u32;
  ( a, b )
}
```
guaranteeing at least one `vertex_displacement_len`-wide block is always chosen whenever there is
data to store -- `a` can now only be `0` when `vertex_displacement_len` itself is `0`, a case
already excluded by the caller's own `vertex_displacement_len != 0` guard before this function is
ever reached. `displacements_update` now calls
`displacement_texture_size_compute( data.len(), vertex_displacement_len )` instead of inlining the
formula. Both the new function and its call site carry a `Fix(BUG-252)`/root-cause/pitfall source
comment.

**`tests/webgl/displacement_texture_size.rs`** (new file): 3 native `#[ test ]` functions --
the reproducer ( a shape with `sqrt(data_len) < vertex_displacement_len` no longer collapses `a`
to `0` ), a capacity-invariant sweep across several vertex/attribute/target combinations
( `a * b * 4 >= data_len` must hold whenever `data_len > 0`, guarding the downstream
`data.extend(vec![0.0; a*b*4 - data.len()])` against ever underflowing ), and an ordinary-case
lock-in ( a vertex count large enough that `.max(1.0)` is a no-op, confirming the fix doesn't
change already-correct behavior ).

## Verification

`longrun`-detached, from repo root:
- `cargo test -p renderer --test tests webgl::displacement_texture_size` -- pre-fix ( temporary
  direct-source-edit revert of `.max( 1.0 )` only ): 1 passed, 2 failed. Post-fix ( restored ):
  3 passed, 0 failed.
- `cargo test -p renderer --test tests` ( full scoped suite, post-fix ): **62 tests run: 62
  passed, 0 failed** ( includes this bug's 3 new tests ).
- `cargo clippy -p renderer --all-features --all-targets -- -D warnings`: exit 0, clean -- no new
  lints triggered by promoting `displacement_texture_size_compute` to `pub`.

## Generalized Version

**Broken assumption:** "round down to the nearest whole multiple of N" is safe by construction --
`floor()` always produces a valid, representable value, so nothing about the formula *looks*
dangerous. But flooring a ratio can legitimately round all the way down to zero multiples when the
numerator is small relative to `N`, and if that rounded-down count is then used as a divisor
downstream ( here, `a` sizes both a texture width and a division ), the zero silently propagates
into a division-by-zero whose `f32` result ( `+inf`, saturating to `T::MAX` on cast ) is
plausible-looking enough to be caught by an unrelated bounds check instead of surfacing as the
real defect. Any "floor a ratio to get a count, then divide by that count" formula must be checked
at its own smallest legitimate input -- specifically, whenever `numerator < N * N` for a
`sqrt(numerator) / N` style ratio -- and floored counts intended for later use as a divisor should
carry an explicit `.max( 1.0 )` ( or equivalent ) lower bound at the point of computation, not left
implicit and rediscovered per call site.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during a dispatched review of `skeleton.rs`'s skinning/morph-target logic in task #174's `renderer` crate scout -- no bugs found in the actual GPU-skinning matrix math ( composed in shader code, outside this file's scope ), but hand-deriving the boundary condition of the newly-added displacement-texture sizing formula, alongside the file's own pre-existing `data_texture_size_calculate` precedent, surfaced this zero-width division-by-zero case. Root cause: `.floor()` on the block-count ratio can round all the way down to `0` when the vertex count is small relative to `attributes_count * targets_count`, and the resulting `a = 0` is then used as a divisor for the texture height, producing `+inf` that saturates to `u32::MAX` on cast -- silently caught by an unrelated size-limit check, masking the real cause and permanently disabling the primitive's morph-target update. Fixed by extracting the formula into its own `pub` function, `displacement_texture_size_compute`, and adding `.max( 1.0 )` to the floored block count. Verified via 3 new native unit tests (2 confirmed to fail pre-fix / pass post-fix via temporary revert-and-rerun), the full 62/62 scoped suite, and clean clippy. Closed same-session (Tier 2 Dual-Role Self-Check). Filed as BUG-252, not the originally-planned BUG-246: `task/readme.md`'s `highest_id: 245` was stale against actual on-disk allocations (TASK-246 through TASK-251 and BUG-249/BUG-250 already existed from a concurrent actor's activity), re-derived via a repo-wide scan for the true max ID before filing to avoid a fresh collision. |

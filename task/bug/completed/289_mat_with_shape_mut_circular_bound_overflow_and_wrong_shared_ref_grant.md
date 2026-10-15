# BUG-289: `MatWithShapeMut`'s reference impls carry a self-referential where-clause, overflowing trait resolution (E0275) for `&mut T` and wrongly permitting `&T`

- **Severity:** Medium (a genuine compile-time defect -- the trait's own legitimately-intended
  `&mut T` usage cannot actually be used as a generic bound without an E0275 overflow error, and a
  shared `&T` was separately, wrongly granted the same "mutable shape access" marker; currently
  reachable only via 2 commented-out/dead call sites, not live production code)
- **state:** Completed
- **Affects:** `MatWithShapeMut`'s `&T` and `&mut T` blanket impls
  (`module/math/ndarray_cg/src/d2/mat/general.rs`)
- **Component:** module/math/ndarray_cg
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Both of `MatWithShapeMut`'s reference-type impls --

```rust
impl< T, const ROWS : usize, const COLS : usize > MatWithShapeMut< ROWS, COLS > for &T
where
  Self : MatWithShape< ROWS, COLS > + MatWithShapeMut< ROWS, COLS > +,
{
}

impl< T, const ROWS : usize, const COLS : usize > MatWithShapeMut< ROWS, COLS > for &mut T
where
  Self : MatWithShape< ROWS, COLS > + MatWithShapeMut< ROWS, COLS >,
{
}
```

-- bound `Self : MatWithShapeMut< ROWS, COLS >` as a premise of implementing
`MatWithShapeMut< ROWS, COLS > for Self` -- the exact fact being proven, required as its own
justification. Attempting to actually use either `&T` or `&mut T` as a `M : MatWithShapeMut<..>`
generic bound fails with `error[E0275]: overflow evaluating the requirement`, rather than
succeeding for `&mut T` (the intended case) or cleanly failing with an ordinary
"trait not satisfied" error for `&T` (the intended-to-be-excluded case). Additionally, the `&T`
impl exists at all, which -- independent of the overflow -- wrongly grants a shared/immutable
reference the "supports mutable shape access" marker.

## Impact

**Who is affected:** any future generic code written against `M : MatWithShapeMut<ROWS,COLS>`
expecting to accept `&mut T` -- exactly the pattern `d2/rotation.rs`'s own commented-out
`inplace_between_vectors`/`inplace_look_at` function signatures use (`Dst : IndexingMut +
MatWithShapeMut<SIZE,SIZE>`). As currently written, calling such a function would fail to compile
with an opaque trait-resolution overflow instead of working as documented.

**What breaks:** the trait's own advertised capability (a mutable reference to a shape-bearing
type should satisfy `MatWithShapeMut`) is unusable in practice; separately, had the overflow not
masked it, a shared reference would have wrongly been accepted wherever mutable access was assumed.

**Entity Scope:** `None` -- library trait-definition defect, not entity directory instances.

## How Discovered

Systematic bug-hunting pass across `ndarray_cg`'s matrix/vector/quaternion modules (parent task:
scouting `module/math` crates for defects). While reading `d2/mat/general.rs`, noticed the `&T`
impl's where-clause bound `Self : MatWithShape<..> + MatWithShapeMut<..> +` (self-referential, plus
a stray trailing `+`) was structurally anomalous compared to the correct, non-circular
`MatWithShape for &T`/`&mut T` pair immediately above it (`where T : MatWithShape<..>`, bound on
the referent, not `Self`). Writing a doctest to prove `&T` shouldn't satisfy the trait first
revealed (diagnostically, by temporarily converting `compile_fail` to a plain fence) that the
failure was `E0275` overflow, not the expected "trait not implemented" -- and that `&mut T`
independently hit the identical overflow under the pristine source, confirming the circular
where-clause broke both reference kinds, not just wrongly-granted `&T`.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cat > /tmp/bug289_repro.rs << 'EOF'
fn requires_mut_shape< M : ndarray_cg::MatWithShapeMut< 2, 2 > >( _m : M ) {}
fn main() {
  let mut m = ndarray_cg::F32x2x2::default();
  requires_mut_shape( &mut m );
}
EOF
# against the fixed source, this compiles; against a git-show-restored pristine
# module/math/ndarray_cg/src/d2/mat/general.rs, rustc reports:
#   error[E0275]: overflow evaluating the requirement `&Mat<2, 2, f32, DescriptorOrderColumnMajor>: MatWithShape<2, 2>`
```
**Expected** (fixed): `&mut m` satisfies `MatWithShapeMut<2,2>` and the snippet compiles cleanly;
`&m` (shared reference) fails to compile with an ordinary trait-not-satisfied error.
**Actual** (pre-fix): both `&mut m` and `&m` fail to compile, but with `error[E0275]: overflow
evaluating the requirement`, not a trait-not-satisfied error -- masking the separate, real defect
that `&m` was also wrongly *permitted* by the (unreachable, due to the overflow) `&T` impl.

## Root Cause

The `&T` impl was copy-pasted from the `&mut T` impl below it with only the impl target changed
from `&mut T` to `&T`, leaving the same self-referential `where Self : MatWithShape<..> +
MatWithShapeMut<..>` bound untouched on both -- unlike the correct sibling `MatWithShape for
&T`/`&mut T` pair earlier in the same file, which each correctly bound `T : MatWithShape<..>` (the
referent type, not `Self`). Requiring `Self : MatWithShapeMut<..>` to prove `Self :
MatWithShapeMut<..>` is circular by construction; Rust's trait solver detects this during
obligation resolution and reports `E0275: overflow evaluating the requirement` rather than either
succeeding or failing cleanly.

## Why Not Caught

The only 2 real call sites anywhere in the crate that would have exercised `M : MatWithShapeMut<..>`
as a generic bound -- `d2/rotation.rs`'s `inplace_between_vectors`/`inplace_look_at` -- are both
commented out, so the overflow was never triggered by any compiled code path. No test in the crate
constructs a generic function bounded by `MatWithShapeMut` and calls it with a concrete reference
type, which is the only way this class of circular-bound defect surfaces.

## Fix Applied (2026-08-18)

**`module/math/ndarray_cg/src/d2/mat/general.rs`:**
- Removed the `&T` impl of `MatWithShapeMut` entirely -- a shared reference cannot provide mutable
  access, matching the trait's own documented contract.
- Corrected the remaining `&mut T` impl's where-clause from the circular `Self : MatWithShape<..> +
  MatWithShapeMut<..>` to `T : MatWithShapeMut<..>` (bound on the referent, not `Self`), mirroring
  the correct, non-circular pattern already used by the sibling `MatWithShape for &mut T` impl.
  `MatWithShapeMut`'s own supertrait bound (`Self : MatWithShape<..>`) is satisfied transitively:
  `T : MatWithShapeMut<..>` already implies `T : MatWithShape<..>` (by `MatWithShapeMut`'s own
  supertrait), which combines with the correct `MatWithShape for &mut T where T : MatWithShape<..>`
  impl to give `&mut T : MatWithShape<..>`.
- Added a `Fix(BUG-289)`/`Root cause`/`Pitfall` source comment above the corrected `&mut T` impl.
- Expanded the trait's doc comment to state the `&T`-exclusion contract explicitly, with 2
  regression doctests attached (see below).

**New regression tests** (doctests on `MatWithShapeMut`'s own doc comment,
`d2/mat/general.rs`):
- A plain (non-`compile_fail`) doctest asserting a generic function bounded by `M :
  MatWithShapeMut<2,2>` compiles and runs when called with `&mut F32x2x2::default()` -- this is
  the genuine, RED/GREEN-discriminating proof, since this is exactly the case that overflowed
  pre-fix.
- A `compile_fail` doctest asserting the same generic function fails to compile when called with a
  shared `&F32x2x2::default()` -- a forward-looking safety net against a *future* correct (i.e.
  non-circular) reintroduction of a `&T` impl; note this doctest "passes" (correctly fails to
  compile) in both the pre-fix and post-fix states, just for different underlying reasons (E0275
  overflow pre-fix, ordinary trait-not-satisfied post-fix) -- it does not by itself discriminate
  this specific historical bug, which is why the plain doctest above is the primary proof.

## Verification

`longrun`-detached, from repo root. Revert-and-rerun proof used a scratchpad copy of the fully
fixed `general.rs` (including both new doctests) plus a partial, Edit-based reinstatement of just
the buggy impl blocks -- not a full-file `git show`/`git stash` revert, since that would also have
erased the new doctests themselves, making a RED check against the doc-comment-attached tests
impossible.

- **Pre-fix (RED):** with both buggy circular impls reinstated (new doctests otherwise untouched),
  `cargo test --doc -p ndarray_cg --all-features -- MatWithShapeMut`: `1 passed; 1 failed` -- the
  primary plain doctest failed to compile with `error[E0275]: overflow evaluating the requirement`
  exactly as diagnosed, confirming the bug before the fix existed. (The secondary `compile_fail`
  doctest "passed" in this state too, for the reason noted above -- expected, not a false negative.)
- **Post-fix (GREEN):** same targeted command: `2 passed; 0 failed`. Full crate doctest suite:
  `7 passed; 0 failed; 2 ignored` (pre-existing ignores unrelated to this bug).
- **Full scoped confirmation:** `cargo nextest run -p ndarray_cg --all-features` (280/280 passed)
  and `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings` (clean), both run
  once after all 3 bugs found in this same investigation pass (BUG-287, BUG-288, BUG-289) were
  fixed together.

## Generalized Version

A `where Self : SameTrait<..>` (or any cycle through 2+ traits back to `Self`) bound on a blanket
impl is a structural red flag independent of whether it happens to compile in isolation -- it
doesn't inertly do nothing, and it doesn't necessarily error at the `impl` declaration site either
(Rust doesn't validate where-clause satisfiability at declaration time, only when something tries
to actually use the impl to discharge an obligation). It can instead silently overflow trait
resolution the moment any concrete type tries to use it, for both intended and unintended
implementors alike -- and if the only real consumers of the affected trait happen to be commented
out or otherwise dead, that overflow can go undetected indefinitely. When a blanket reference impl
(`impl<T> Trait for &T` / `&mut T`) is written, the where-clause should bound the *referent* `T`
(`T : Trait<..>`), matching the correct sibling `MatWithShape` pair in this same file, never `Self`
circularly back to the trait being implemented.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found during a systematic bug-hunting pass across `ndarray_cg`'s matrix/vector/quaternion modules. Root cause: `MatWithShapeMut`'s `&T` and `&mut T` blanket impls both carried a self-referential `where Self : MatWithShapeMut<..>` bound (the `&T` impl copy-pasted from `&mut T` with only the impl target changed), which doesn't just wrongly grant `&T` the trait -- it overflows trait resolution (E0275) for both reference kinds whenever anything actually tries to use `M : MatWithShapeMut<..>` as a generic bound, including the legitimately-intended `&mut T` case. Undetected because the only 2 real consumers in the crate (`d2/rotation.rs`'s `inplace_between_vectors`/`inplace_look_at`) are commented out. Fixed by removing the `&T` impl and correcting `&mut T`'s where-clause to the non-circular `T : MatWithShapeMut<..>`, matching the correct sibling `MatWithShape for &mut T` pattern. Verified via 2 new doctests (one plain, proving `&mut T` now compiles where it previously overflowed -- the primary RED/GREEN-discriminating proof; one `compile_fail`, a forward-looking guard against `&T` being wrongly reintroduced), the plain one confirmed failing with the exact E0275 overflow against a partially-reinstated pristine source (scratchpad copy + targeted Edit-based revert, not `git show`/`git stash`, since a full-file revert would have erased the new doctests too) then passing post-fix, plus the full 280-test suite and clean clippy. `task/readme.md`'s `highest_id` stood at 288 at filing time, confirmed via a fresh on-disk scan across all `task/bug/` lifecycle subdirectories immediately before filing. |

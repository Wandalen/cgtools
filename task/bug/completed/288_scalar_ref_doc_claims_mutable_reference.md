# BUG-288: `ScalarRef::scalar_ref` and its mirrored `Mat::scalar_ref` both document a `&self`-returning `&Scalar` method as returning "A mutable reference"

- **Severity:** Low (pure documentation/code contradiction -- both methods' actual runtime behavior
  was always correctly immutable and already covered by existing tests; only the doc comments'
  prose was wrong)
- **state:** Completed
- **Affects:** `ScalarRef::scalar_ref`'s doc comment (`module/math/ndarray_cg/src/md/access.rs`) and
  its mirrored inherent wrapper `Mat::scalar_ref`'s doc comment
  (`module/math/ndarray_cg/src/d2/mat/access_mirror.rs`)
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

`ScalarRef::scalar_ref( &self, .. ) -> &Self::Scalar` -- a `&self`-receiver method returning a
shared, immutable reference -- documented its own `# Returns` section as "A mutable reference to
the scalar at the specified index." The mirrored inherent forwarding method
`Mat::scalar_ref` (`d2/mat/access_mirror.rs`), which calls straight through to the trait method and
also returns `&<Self as Collection>::Scalar`, carried the identical wrong claim.

## Impact

**Who is affected:** anyone reading either method's rendered rustdoc or IDE hover to learn whether
`scalar_ref` grants write access -- the doc actively asserts the wrong answer for both the trait
method and its public inherent wrapper.

**What breaks:** documentation/code parity only. Both methods' actual runtime behavior (an
immutable borrow through a shared reference) was already correct and already exercised by
`test_scalar_ref_generic`/`test_scalar_ref_row_major`/`test_scalar_ref_column_major`
(`tests/inc/d2_test/access_test/scalar_test.rs`) -- no computation, call site, or consumer was
affected; the type system already prevents any actual mutation through the returned `&Scalar`.

**Entity Scope:** `None` -- library source doc-comment defect, not entity directory instances.

## How Discovered

Systematic bug-hunting pass across `ndarray_cg`'s matrix/vector/quaternion modules (parent task:
scouting `module/math` crates for defects). Ran a crate-wide `grep -rn "mutable reference" src/`
sweep and individually cross-referenced every hit against its method's own receiver
(`&self`/`&mut self`) and return type; both `scalar_ref` occurrences (the trait definition and its
mirrored inherent wrapper) were the sole true positives -- every other hit correctly described a
genuinely `_mut`-suffixed method.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "fn scalar_ref" -B5 module/math/ndarray_cg/src/md/access.rs module/math/ndarray_cg/src/d2/mat/access_mirror.rs
```
**Expected** (fixed): both preceding `# Returns` lines read "A reference to the scalar at the
specified index."
**Actual** (pre-fix): both preceding `# Returns` lines read "A mutable reference to the scalar at
the specified index," despite both methods taking `&self` and returning an immutable `&Scalar`.

## Root Cause

`ScalarRef::scalar_ref`'s doc comment (`md/access.rs`) was copy-pasted from the adjacent, correctly
`&mut self`-receiver `ScalarMut::scalar_mut` in the same file, without updating "mutable" to match
`ScalarRef`'s own immutable contract. `d2/mat/access_mirror.rs`'s inherent `Mat::scalar_ref`
wrapper then propagated the same wrong wording when it was written to mirror the trait method.

## Why Not Caught

`test_scalar_ref_generic` (`tests/inc/d2_test/access_test/scalar_test.rs`) already exercises
`scalar_ref`'s correct immutable-read behavior across both descriptors, but no test read either doc
comment's own text -- a doc string carries zero compiler enforcement, so a behaviorally-correct,
genuinely-immutable method can carry an arbitrarily wrong "mutable" claim indefinitely with every
runtime test still green.

## Fix Applied (2026-08-18)

**`module/math/ndarray_cg/src/md/access.rs`:** reworded `ScalarRef::scalar_ref`'s `# Returns` line
from "A mutable reference to the scalar at the specified index" to "A reference to the scalar at
the specified index." Added a 3-field `Fix(BUG-288)`/`Root cause`/`Pitfall` source comment directly
above the doc comment.

**`module/math/ndarray_cg/src/d2/mat/access_mirror.rs`:** identical rewording for the mirrored
inherent `Mat::scalar_ref`, plus the same 3-field source comment.

No behavioral change in either file.

**New regression test** (`module/math/ndarray_cg/tests/inc/d2_test/access_test/scalar_test.rs`):
`scalar_ref_doc_does_not_claim_mutable` -- reads both source files via `include_str!`, locates the
doc comment line immediately preceding each `scalar_ref` definition, and asserts neither mentions
"mutable." Covers both locations in one test since both share the identical defect and fix.

## Verification

`longrun`-detached, from repo root. Revert-and-rerun proof used scratchpad copies of both fixed
files plus `git show HEAD:<path>` to temporarily restore pristine content one file at a time --
never `git stash`.

- **Pre-fix (RED), location 1:** with only `md/access.rs` reverted to pristine (`access_mirror.rs`
  still fixed), `cargo test -p ndarray_cg --test tests -- scalar_ref_doc_does_not_claim_mutable`:
  `0 passed; 1 failed`, panic message confirms the `md/access.rs`-specific assertion failed.
- **Pre-fix (RED), location 2:** with `md/access.rs` restored to fixed and only
  `access_mirror.rs` reverted to pristine, same command: `0 passed; 1 failed`, panic message
  confirms the `d2/mat/access_mirror.rs`-specific assertion failed -- proving the test genuinely
  covers both locations independently, not just one.
- **Post-fix (GREEN):** both files restored to fixed, same targeted command plus the 4 pre-existing
  `test_scalar_ref_*`/`test_scalar_mut_*` tests: `5 passed; 0 failed` -- confirms the fix and that
  `scalar_mut`'s own (already-correct) "mutable" doc claims were left untouched.
- **Full scoped confirmation:** `cargo nextest run -p ndarray_cg --all-features` and
  `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings`, both clean (see
  `task/bug/readme.md`'s Closed Bugs row for this bug's final combined pass count, recorded once
  after all bugs from this same investigation pass were fixed).

## Generalized Version

A crate-wide `grep -rn "mutable reference" src/` sweep, cross-referenced against each hit's own
receiver and return type, is a cheap way to catch this exact defect class (a `_ref`/`_mut` doc pair
where one was copy-pasted from the other without updating the mutability claim) across an entire
codebase at once -- used here to confirm these were the only 2 true positives among many correctly
`_mut`-labeled hits in the same file family (`md/access.rs`, `d2/mat/access_mirror.rs`).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found during a systematic bug-hunting pass across `ndarray_cg`'s matrix/vector/quaternion modules, via a crate-wide `grep -rn "mutable reference"` sweep cross-referenced against each hit's own receiver/return type. Root cause: `ScalarRef::scalar_ref`'s doc comment copy-pasted from the adjacent `ScalarMut::scalar_mut` without updating the mutability claim; its mirrored inherent wrapper in `d2/mat/access_mirror.rs` propagated the same wrong text. Fixed by rewording both doc comments; no behavioral change (both methods were always correctly immutable). Verified via a new doc-text regression test covering both locations, each independently confirmed failing against a temporarily-restored pristine source (scratchpad copy + `git show HEAD:<path>`, no `git stash`) then passing post-fix, plus the 4 pre-existing sibling scalar-access tests. `task/readme.md`'s `highest_id` stood at 287 at filing time, confirmed via a fresh on-disk scan across all `task/bug/` lifecycle subdirectories immediately before filing. |

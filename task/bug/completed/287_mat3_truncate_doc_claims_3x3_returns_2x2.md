# BUG-287: `Mat3::truncate()`'s doc comment claims it converts "into the 3x3 matrix," but it returns (and has always correctly returned) a 2x2 matrix

- **Severity:** Low (pure documentation/code contradiction -- the function's actual runtime behavior
  was always correct and already covered by existing tests; only the doc comment's prose was wrong)
- **state:** Completed
- **Affects:** `Mat3::truncate()`'s doc comment (`module/math/ndarray_cg/src/d2/mat3x3/general.rs`)
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

`Mat< 3, 3, E, Descriptor >::truncate()`'s doc comment read "Convertes this matrix into the 3x3
matrix" while its own signature returns `Mat< 2, 2, E, Descriptor >` -- the doc claimed the wrong
output shape (and repeated the "3x3" self-reference nonsensically, since the method is defined on
`Mat3` itself).

## Impact

**Who is affected:** anyone reading `Mat3::truncate()`'s rendered rustdoc (docs.rs or IDE hover) to
learn what shape it returns -- the doc actively asserts the wrong answer.

**What breaks:** documentation/code parity only. The method's actual runtime behavior (drop the last
row and column, producing a 2x2 sub-matrix) was already correct and already exercised by
`test_truncate_row_major`/`test_truncate_column_major` (`tests/inc/mat3x3_test/general_test.rs`) --
no computation, call site, or consumer was affected.

**Entity Scope:** `None` -- library source doc-comment defect, not entity directory instances.

## How Discovered

Systematic bug-hunting pass across `ndarray_cg`'s matrix/vector/quaternion modules (parent task:
scouting `module/math` crates for defects). While reading `d2/mat3x3/general.rs` and
`d2/mat4x4/general.rs` side by side to verify `truncate()`'s cross-descriptor correctness, noticed
both methods share the identical doc string "Convertes this matrix into the 3x3 matrix" -- correct
for `Mat4::truncate()` (4x4 -> 3x3) but wrong for `Mat3::truncate()` (3x3 -> 2x2), confirming a
copy-paste origin.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
grep -n "pub fn truncate" -A1 module/math/ndarray_cg/src/d2/mat3x3/general.rs
```
**Expected** (fixed): the preceding doc line reads "Converts this matrix into the 2x2 matrix,
dropping the last row and column."
**Actual** (pre-fix): the preceding doc line reads "Convertes this matrix into the 3x3 matrix"
(both a shape mismatch and a spelling error).

## Root Cause

`Mat3::truncate()`'s doc comment was copy-pasted from `Mat4::truncate()` (`d2/mat4x4/general.rs`,
where the identical text "Convertes this matrix into the 3x3 matrix" is correct, since that method
really does convert a 4x4 matrix into a 3x3 one) without updating the shape it names for this
type's own 3x3 -> 2x2 conversion.

## Why Not Caught

`test_truncate_row_major`/`test_truncate_column_major` (`tests/inc/mat3x3_test/general_test.rs`)
already assert the correct 2x2 runtime output, but no test read the doc comment's own text -- a doc
string carries zero compiler enforcement, so a behaviorally-correct function can carry an
arbitrarily wrong description indefinitely with every runtime test still green.

## Fix Applied (2026-08-18)

**`module/math/ndarray_cg/src/d2/mat3x3/general.rs`:** reworded `Mat3::truncate()`'s doc comment
from "Convertes this matrix into the 3x3 matrix" to "Converts this matrix into the 2x2 matrix,
dropping the last row and column." Added a 3-field `Fix(BUG-287)`/`Root cause`/`Pitfall` source
comment directly above the doc comment. No behavioral change.

**New regression test** (`module/math/ndarray_cg/tests/inc/mat3x3_test/general_test.rs`):
`truncate_doc_matches_2x2_output` -- reads the crate's own source file via `include_str!`, locates
the doc comment line immediately preceding `pub fn truncate`, and asserts it mentions "2x2" and
does not mention "3x3." Since this is a pure doc-string defect with no behavioral component, no
runtime assertion on `truncate()`'s output could ever distinguish pre-fix from post-fix -- the test
instead directly asserts on the doc text itself, the only artifact that actually changed.

## Verification

`longrun`-detached, from repo root. Revert-and-rerun proof used a scratchpad copy of the fixed
`general.rs` plus `git show HEAD:<path>` to temporarily restore pristine content -- never
`git stash`.

- **Pre-fix (RED):** `cargo test -p ndarray_cg --test tests -- truncate_doc_matches_2x2_output`
  against the temporarily-restored pristine source: `0 passed; 1 failed` -- the new test's first
  assertion (`doc_line.contains("2x2")`) failed against the pristine "3x3" text, confirming the bug
  before any fix existed.
- **Post-fix (GREEN):** same targeted command, plus the 4 pre-existing `test_truncate_*` tests
  (mat3x3 and mat4x4, both descriptors): `5 passed; 0 failed` -- confirms the fix and that
  `Mat4::truncate()`'s own (already-correct) doc was left untouched.
- **Full scoped confirmation:** `cargo nextest run -p ndarray_cg --all-features` and
  `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings`, both clean (see
  `task/bug/readme.md`'s Closed Bugs row for this bug's final combined pass count, recorded once
  after all bugs from this same investigation pass were fixed).

## Generalized Version

Any doc comment copy-pasted from a sibling method of the same name on a differently-shaped type
(here, `Mat3::truncate()` from `Mat4::truncate()`) needs its shape-specific nouns re-derived from
the actual signature, not merely inherited from the source it was copied from -- text that reads as
plausible prose gives no signal that it was never re-checked against the new context. The crate's
own `vec4/general.rs::truncate()` (truncating a `Vec4` to a `Vec3`) was checked as a comparison
point during this same investigation and found correctly worded, showing this defect class is not
systemic across the crate's other `truncate()` methods -- `Mat3`'s was the sole instance.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found during a systematic bug-hunting pass across `ndarray_cg`'s matrix/vector/quaternion modules. Root cause: `Mat3::truncate()`'s doc comment copy-pasted from the correctly-worded `Mat4::truncate()` sibling without updating the claimed output shape from 3x3 to 2x2. Fixed by rewording the doc comment; no behavioral change (runtime output was already correct). Verified via a new doc-text regression test (`include_str!`-based, since no runtime assertion can distinguish this doc-only defect), confirmed failing against a temporarily-restored pristine source (scratchpad copy + `git show HEAD:<path>`, no `git stash`) then passing post-fix, plus the 4 pre-existing sibling `truncate` tests. `task/readme.md`'s `highest_id` stood at 286 at filing time, confirmed via a fresh on-disk scan across all `task/bug/` lifecycle subdirectories immediately before filing. |

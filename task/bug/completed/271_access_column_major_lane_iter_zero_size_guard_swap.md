# BUG-271: `access_column_major`'s `lane_iter`/`lane_iter_mut` degenerate-size guards test the wrong dimension per branch, panicking instead of returning empty for asymmetric zero-size matrices

- **Severity:** Low (unreachable through any of the crate's own shipped matrix types --
  `Mat2`/`Mat3`/`Mat4` are always square and never zero-sized -- reachable only via a deliberate
  asymmetric zero-dimension instantiation of the public generic `Mat<ROWS,COLS,E,Descriptor>` type)
- **state:** Completed
- **Affects:** `Mat<ROWS,COLS,E,DescriptorOrderColumnMajor>::lane_iter`/`lane_iter_mut`
  (`IndexingRef`/`IndexingMut` impls) for any `ROWS != COLS` with exactly one of them `0`
- **Component:** `module/math/ndarray_cg` (`src/d2/mat/access_column_major.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/ (self)
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`lane_iter`'s row branch (`varying_dim == 0`, indexed by `lane < ROWS`) guarded its
degenerate/empty-iterator case with `if COLS == 0` instead of `if ROWS == 0`; its column branch
(`varying_dim == 1`, indexed by `lane < COLS`) guarded with `if ROWS == 0` instead of
`if COLS == 0` -- the two conditions were swapped relative to the dimension each branch's own
`else`-arm assertion actually bounds. `lane_iter_mut` (`IndexingMut`) duplicates the identical
branching logic and carried the same swap. For a matrix where `ROWS != COLS` and exactly one of
them is `0` (e.g. `Mat<0,3,..>` or `Mat<3,0,..>`), the guard took the `else` branch instead of
returning an empty iterator, hitting `assert!( lane < ROWS )` / `assert!( lane < COLS, .. )` with
`lane == 0` against a zero bound and panicking -- unlike the row-major sibling
(`access_row_major.rs`), which guards each branch on its own correctly-matching dimension and
returns gracefully empty for the identical inputs.

## Impact

**Who is affected:** any caller constructing an asymmetric zero-size `Mat<ROWS,COLS,E,
DescriptorOrderColumnMajor>` (`ROWS != COLS`, one of them `0`) through the public generic `Mat`
type and calling `lane_iter`/`lane_iter_mut` on it. The crate's own shipped aliases (`Mat2`,
`Mat3`, `Mat4`, and their typed variants) are always square and never zero-sized, so no internal
call site is affected.

**What breaks:** instead of gracefully returning an empty iterator (the correct behavior for
iterating a lane that has zero elements, and the behavior `access_row_major.rs` already produces
for the same inputs), the column-major implementation panics with either
`"assertion failed: lane < ROWS"` (row branch) or `"lane:0 | COLS:0"` (column branch), even for
the always-valid `lane == 0` case on a genuinely zero-length dimension.

**Entity Scope:** `None` -- source-level control-flow defect, not entity directory instances.

## How Discovered

During this session's review of `module/math/ndarray_cg/src/d2/mat/access_column_major.rs`,
required cross-consistency checking against its row-major sibling
(`access_row_major.rs`) and the shared `access_common.rs`/`access_mirror.rs` layer surfaced that
`access_row_major.rs`'s row/column branches each guard on their own matching dimension
(`ROWS`/`COLS` respectively), while `access_column_major.rs`'s two branches guard on the *other*
branch's dimension instead -- confirmed by directly comparing both files' `lane_iter` bodies
side by side.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p ndarray_cg --all-features asymmetric_zero_size
```
**Expected** (fixed): 4 passed (2 `lane_iter` + 2 `lane_iter_mut`, one row-major/one
column-major instantiation each).
**Actual** (pre-fix, confirmed via a temporary direct-source-edit revert of both guard
conditions in `access_column_major.rs`, real run, `--no-fail-fast`): 2 passed (row-major,
unaffected) / 2 failed (column-major):
```
thread '...test_lane_iter_asymmetric_zero_size_column_major' panicked at
  .../access_column_major.rs:52:11:
assertion failed: lane < ROWS
thread '...test_lane_iter_mut_asymmetric_zero_size_column_major' panicked at
  .../access_column_major.rs:200:11:
assertion failed: lane < ROWS
```
A second isolated revert (row-branch guard restored, column-branch guard alone left reverted)
independently reproduced the column branch's own distinct panic:
```
thread '...' panicked at .../access_column_major.rs:73:11:
lane:0 | COLS:0
```

## Root Cause

`access_column_major.rs`'s `lane_iter` (pre-fix), abbreviated:
```rust
match varying_dim
{
  0 => // Iterate over a row -- indexed by `lane < ROWS`
  {
    let ( skip, step, take ) = if COLS == 0        // WRONG dimension -- should be ROWS == 0
    { ( 0, 1, 0 ) }
    else { assert!( lane < ROWS ); ( lane, ROWS, COLS ) };
    // ...
  },
  1 => // Iterate over a column -- indexed by `lane < COLS`
  {
    let ( skip, take ) = if ROWS == 0               // WRONG dimension -- should be COLS == 0
    { ( 0, 0 ) }
    else { assert!( lane < COLS, "lane:{lane} | COLS:{COLS}" ); ( lane * ROWS, ROWS ) };
    // ...
  },
  // ...
}
```
Each branch's `else`-arm assertion bounds `lane` against one specific dimension (`ROWS` for the
row branch, `COLS` for the column branch), but the preceding degenerate-size guard checked the
*other* dimension instead -- a straightforward row/column guard swap. `lane_iter_mut`
(`IndexingMut`) duplicates this exact branching shape and carried the identical swap in both
branches.

## Why Not Caught

The crate's existing `lane_test.rs` coverage for the degenerate/zero-size case
(`test_valid_row_iteration_generic`/`test_valid_column_iteration_generic`) only exercises the
*symmetric* `0x0` matrix, where `ROWS == COLS == 0` makes the buggy condition (`COLS == 0`) and
the correct one (`ROWS == 0`) evaluate identically -- masking the swap completely. No existing
test constructed an *asymmetric* zero-size matrix (`0xN` or `Nx0` with `N > 0`), the only shape
that can distinguish a correct per-branch guard from one where both branches' conditions were
swapped with each other.

## Fix Applied (2026-08-17)

**`src/d2/mat/access_column_major.rs`:** in both `lane_iter` (`IndexingRef`) and `lane_iter_mut`
(`IndexingMut`), swapped the row branch's guard from `if COLS == 0` to `if ROWS == 0`, and the
column branch's guard from `if ROWS == 0` to `if COLS == 0` -- matching each branch to the
dimension its own `else`-arm assertion actually bounds, and now consistent with
`access_row_major.rs`'s already-correct per-branch guards.

**`tests/inc/d2_test/access_test/indexing_test/lane_test.rs`** (new tests):
`test_lane_iter_asymmetric_zero_size_generic` and `test_lane_iter_mut_asymmetric_zero_size_generic`
(each instantiated for both `DescriptorOrderRowMajor`/`DescriptorOrderColumnMajor`) construct
`Mat<0,3,..>` and `Mat<3,0,..>` and assert `lane_iter( 0, 0 )`/`lane_iter( 1, 0 )` (and their
`_mut` counterparts) yield zero elements instead of panicking.

## Verification

`longrun`-detached, from the crate directory:
- `cargo test --all-features --no-fail-fast asymmetric_zero_size` -- pre-fix (temporary
  direct-source-edit revert of all 4 guard conditions, real run): 2 passed (row-major) / 2
  failed (column-major), panicking exactly as described above. A follow-up isolated revert
  (only the column-branch guard reverted) independently reproduced that branch's own distinct
  `"lane:0 | COLS:0"` panic message. Post-fix (all reverts restored): 4 passed, 0 failed.
- `cargo test -p ndarray_cg --all-features` (full scoped suite) and
  `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings`: recorded together
  with this session's other review findings in this crate; see `## History`.

## Generalized Version

**Broken assumption:** a degenerate-size guard pair (`if DIM_A == 0 {..} else { assert!( lane <
DIM_A ) }`) placed above a matching `else`-arm assertion is itself correctly wired to that
assertion's own dimension, just because the two sit next to each other in the same branch. A
guard and its assertion must be checked against the *same* named dimension explicitly -- testing
only the symmetric `ROWS == COLS == 0` case can never distinguish a correct per-branch guard from
one where two sibling branches' conditions were swapped with each other, since both conditions
degenerate to the same truth value when `ROWS == COLS`.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's review of `module/math/ndarray_cg`'s `d2/mat/` access-layer files, required to cross-check `access_column_major.rs`/`access_row_major.rs`/`access_mirror.rs` for consistency. Root cause: `access_column_major.rs`'s `lane_iter`/`lane_iter_mut` row and column branches each guarded on the *other* branch's dimension instead of their own, invisible under the crate's own always-square `Mat2`/`Mat3`/`Mat4` usage and under the existing test suite's symmetric-only `0x0` degenerate coverage. Fixed by swapping both guard conditions in both functions to match each branch's own assertion dimension. Verified via 4 new native unit tests (2 confirmed fail pre-fix via a full revert-and-rerun -- real panics, exact messages captured for both the row-branch and, via a follow-up isolated revert, the column-branch -- and pass post-fix) plus the crate's full scoped suite and clean clippy. The immutable `IndexingRef`/mutable `IndexingMut` core-indexing logic (`m[i][j]` element access via `scalar_offset`) was independently cross-checked via a concrete non-square numeric trace and found fully consistent between row-major and column-major -- no classic transpose bug found; this guard-swap in the degenerate-size path was the sole defect identified. Filed as BUG-271 after a fresh on-disk scan (both `task/` and `task/bug/` namespaces) found 270 as the highest existing bug ID and 254 as the highest existing task ID. |

# Fix ndarray_cg debug-only dimension checks (silently unchecked in release)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** 2026-08-10
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/math/ndarray_cg
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

`ndarray_cg` has dimension/bounds checks gated to debug builds only (e.g. `debug_assert!`) where the
consequence of a mismatch in release is silent wrong-data output rather than a loud failure, violating
the workspace's "loud failures, never silent" testing principle (P2 — remaining logic bugs, Fix-in-place).
**Carried forward from the audit triage plan — exact file/line is not re-verified in this filing pass;
re-confirm against current `module/math/ndarray_cg/src/` before touching**, and decide case-by-case
whether each site should become a real runtime check (`Result`/panic) or is genuinely
performance-critical-enough to justify staying debug-only with an explicit doc comment explaining why.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, Fix-in-place bucket.

- **[2026-08-10]** `IMPLEMENTED` — Re-confirmed every site against current source via
  `grep -rn debug_assert src/` before touching anything: 18 raw matches across 6 files, resolving to 14
  distinct functions (11 converted to unconditional real checks, 2 left debug-only as already-safe, 1
  removed entirely). TDD process: wrote failing/characterizing tests first, empirically proved the RED
  (silent-failure) state via `cargo nextest run -p ndarray_cg --all-features --release --no-fail-fast`
  (257 tests: 233 passed, 24 failed — every failure landed exactly on a hypothesized-dangerous site, zero
  failures on the two already-safe sites), then applied the minimal fix per site, then re-verified GREEN in
  both profiles.

  **Converted to unconditional real checks (11 functions, 15 check-site matches):**
  - `src/d2/arithmetics/add.rs:16` — removed `#[ cfg( debug_assertions ) ]` gating `add()`'s
    dimension-compatibility check.
  - `src/d2/arithmetics/mul.rs:19` — removed `#[ cfg( debug_assertions ) ]` gating `mul()`'s
    dimension-compatibility check.
  - `src/d2/arithmetics/mul.rs:59` — removed `#[ cfg( debug_assertions ) ]` gating `mul_mat_vec()`'s
    dimension-compatibility check.
  - `src/d2/mat/access_common.rs:83` — `from_row_major`: `debug_assert_eq!` → `assert_eq!` (size check).
  - `src/d2/mat/access_common.rs:91` — `from_column_major`: `debug_assert_eq!` → `assert_eq!` (size check).
  - `src/d2/mat/access_row_major.rs:29,52` — `lane_iter`: `debug_assert!` → `assert!` (row-bound and
    column-bound checks).
  - `src/d2/mat/access_row_major.rs:164,176` — `lane_iter_mut`: `debug_assert!` → `assert!` (row-bound and
    column-bound checks).
  - `src/d2/mat/access_row_major.rs:308` — `with_column_major`: `debug_assert_eq!` → `assert_eq!`
    (**CRITICAL**: this size check was the sole guard in front of an
    `unsafe { ptr.add( col * ROWS + row ) }` read; unchecked in release this was undefined behavior, not
    just wrong data).
  - `src/d2/mat/access_column_major.rs:31,54` — `lane_iter`: `debug_assert!` → `assert!` (row-bound and
    column-bound checks).
  - `src/d2/mat/access_column_major.rs:166,190` — `lane_iter_mut`: `debug_assert!` → `assert!` (row-bound
    and column-bound checks).
  - `src/d2/mat/access_column_major.rs:317` — `with_row_major`: `debug_assert_eq!` → `assert_eq!`
    (**CRITICAL**: mirror of the row-major case above, guarding
    `unsafe { ptr.add( row * COLS + col ) }`).

  **Left debug-only, no change (2 functions, 2 matches — already safe in every build profile):**
  - `src/d2/mat/access_row_major.rs:296` and `access_column_major.rs:310` — both `raw_set`'s
    `debug_assert_eq!( scalars.len(), ROWS*COLS, .. )`. The very next line,
    `self.raw_slice_mut().copy_from_slice( &scalars )`, already panics unconditionally (every build
    profile) on a length mismatch via `[T]::copy_from_slice`'s own always-on check, so the debug_assert is
    genuinely redundant rather than a release-mode gap. New tests in `raw_slice_test.rs` confirm this
    empirically: the same-major combinations that hit this path show zero release-mode regressions,
    unlike the two CRITICAL cross-major sites above.

  **Removed entirely (1 function, 1 match — a debug-only false positive, not a release-mode gap):**
  - `src/quaternion/from.rs:17` — `impl From<&[E]> for Quat<E>`'s
    `debug_assert!( value.len() > 4, .. )`. The condition used `> 4` instead of the correct `>= 4`, so a
    valid, correctly-sized 4-element slice failed the assertion in every *debug* build — the opposite
    direction from the other 13 fixed/unchanged sites (this one over-fired in debug rather than
    under-firing in release). Also fully redundant: the next line, `value.try_into().unwrap()`, already
    panics unconditionally on `len() != 4` in every profile.

  **Tests added (29 new functions across 5 files, all in `tests/` per the workspace's public-API
  test-placement rule):**
  - `tests/inc/d2_test/access_test/indexing_test/lane_test.rs` — `test_out_of_bounds_column_lane_index_generic`,
    `test_lane_iter_mut_out_of_bounds_row_generic`, `test_lane_iter_mut_out_of_bounds_column_generic`
    (+ `_row_major`/`_column_major` wrappers each; 9 fns).
  - `tests/inc/d2_test/arithmetic_test/mul_test.rs` — replaced a pre-existing dead commented-out block
    with `test_multiply_incompatible_dimensions_generic` and
    `test_multiply_vec_incompatible_dimensions_generic` (+ wrappers each; 6 fns).
  - `tests/inc/d2_test/raw_slice_test.rs` — `test_set_column_major_size_mismatch_generic`,
    `test_set_row_major_size_mismatch_generic` (+ wrappers each; 6 fns) — directly exercise the two
    CRITICAL unsafe-guarded sites (and their already-safe same-major counterparts).
  - `tests/inc/d2_test/fns_test.rs` — `test_from_row_major_size_mismatch_generic`,
    `test_from_column_major_size_mismatch_generic` (+ wrappers each; 6 fns).
  - `tests/inc/quat_test/general.rs` — `test_quat_from_slice_valid` (RED under normal debug execution
    before the fix — the sole debug-only-false-positive case), `test_quat_from_slice_wrong_length` (2 fns).

  **Verification:**
  - RED (pre-fix): `cargo nextest run -p ndarray_cg --all-features --release --no-fail-fast` → 257 run,
    233 passed, **24 failed**, matching exactly the CRITICAL/converted sites' new and pre-existing tests
    (including 4 pre-existing `lane_indexed_test.rs` tests not modified by this task, which transitively
    exercise the same `lane_iter` bound checks); zero failures on the 2 already-safe `raw_set` sites.
    `test_quat_from_slice_valid` was separately RED under the normal debug profile.
  - GREEN (post-fix): `cargo nextest run -p ndarray_cg --all-features --no-fail-fast` → 257 run,
    **257 passed**, 0 failed (debug profile). `cargo nextest run -p ndarray_cg --all-features --release
    --no-fail-fast` → 257 run, **257 passed**, 0 failed (release profile).
  - Final mandated gate: `will .test l::3` (nextest --all-features + doc tests --all-features + clippy
    --all-targets --all-features -D warnings) → **4/4 commands passed, 0 failed**, exit 0.

- **[2026-08-10]** `VERIFIED` — Independent follow-up Miri sweep (`cargo +nightly miri test -p ndarray_cg
  --all-features`), closing the gap noted below that this task's own Verification Record (B5) covered
  release-profile nextest and clippy but not Miri, despite 2 CRITICAL unsafe-guarded sites among its 14.
  Result: 249 passed, 8 failed, exit 101. **Zero UB found in any of the 14 touched sites** — both
  CRITICAL `with_column_major`/`with_row_major` sites and the `raw_set`/`raw_slice_mut` tests
  (`test_raw_set_column_major`, `test_raw_slice_mut_row_major`, `test_set_row_major_size_mismatch_*`,
  etc., and this task's own 29 new tests including `test_quat_from_slice_valid`) all pass clean under
  Miri. The 8 failures are unrelated pre-existing tests outside this task's scope
  (`mat2x2_test::transformation_test::test_rot`, `mat2x2h_test::transformation_test::test_rot`/
  `test_rot_around_point`, `quat_test::arithmetic::test_from_angle_x/y/z`, `test_from_euler_xyz`,
  `quat_test::general::test_slerp`) — all `assert_abs_diff_eq!`/`assert_eq!` mismatches at the 6th-7th
  decimal place (e.g. `0.7071066` vs `0.707107`) in trig-heavy (`sin`/`cos`/slerp) computations, matching
  Miri's well-documented software-vs-hardware floating-point transcendental-function variance rather than
  a memory-safety defect — confirmed both native `cargo nextest` runs (debug and release) pass all 257
  tests clean, so these 8 assertions only fail under Miri's interpreter specifically.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | 2 of 14 sites were genuine release-mode UB (unsafe-block size guard), not just silent wrong-data — higher-severity than the task's own filing anticipated | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | All touched fns are public trait methods — tests correctly placed in `tests/`, not in-source | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| B1 | Rulebook Compliance | — | 🟢 | — | — |
| B2 | Test-First Requirement | — | 🟢 | — | — |
| B3 | Evidence of Failure | — | 🟢 | — | — |
| B4 | Proper Fix Only | — | 🟢 | 11 sites: `debug_assert`→`assert` (unconditional); 2 already-safe sites correctly left untouched; 1 buggy redundant site (`> 4` vs `>= 4`) correctly removed rather than "fixed in place" | — |
| B5 | Fix Verification | — | 🟢 | Independently re-ran myself: `longrun`-launched package-scoped `will .test l::3` → exit 0, 4/4; direct `cargo nextest -p ndarray_cg --all-features --release` → 257/257 passed (the release-profile claim this task hinges on); direct `cargo clippy` → clean | — |
| B6 | Knowledge Preservation | — | 🟢 | 3-field `Fix(TASK-014)`/`Root cause`/`Pitfall` comments spot-checked directly on both CRITICAL unsafe sites and the `Quat::from` removal — all present, accurate, technically precise | — |
| B7 | Code Cleanliness | — | 🟢 | 6 additional dirty files in this crate (`vector/*`, `tests/inc/mod.rs`, etc.) confirmed pre-existing from session start (present in the very first `git status` snapshot at conversation start, predating any work this session) — not introduced by this task | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 15 dimensions clean on both passes, zero Blocking Findings. Verification independently re-executed (`longrun`, direct `cargo nextest --release`, direct `cargo clippy`, direct diff reads of the 2 CRITICAL unsafe-guarded sites and the `Quat::from` removal) rather than solely trusted from the implementing subagent's own prose, per this session's Stale Evidence Trust discipline.

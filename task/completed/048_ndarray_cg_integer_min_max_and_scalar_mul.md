# Consume relaxed min/max bound in ndarray_cg's Vector; add missing integer scalar×vector Mul impls

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-09
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/math/ndarray_cg
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-09
- **blocked_by:** 044

## Goal

Second half of the integer-vector-math gap identified from root `todo.md`'s "i32 and u32 Vectors"
claim (first half: `044`, `mdmath_core`). Two independent gaps live in this crate:

(1) `Vector::min()`/`Vector::max()` (`src/vector/arithmetics.rs:78,85`) sit in an
`impl< E : MatEl + NdFloat, LEN > Vector< E, LEN >` block (lines 61-96) alongside `normalize`/`mag`/
`distance`, which genuinely need `NdFloat` (they call `sqrt` internally via `mag`). `min`/`max`
themselves just delegate to `mdmath_core::vector::{min, max}` (doc comments at lines 76-77, 83-84
self-admit "Currently float-only"). Once `044` relaxes those free functions' bound, this crate's
wrapper methods must move to a matching relaxed bound or they keep failing to compile for integer
`E` even though their backing implementation no longer requires `NdFloat`. **This task is
`blocked_by: 044`** for this half of its work.

(2) The commutative scalar×vector `Mul` (`E * Vector< E, LEN >`) is implemented concretely only for
`f32`/`f64` (`src/vector/operator/mul.rs:69,80`), while the already-generic `Vector< E, LEN > * E`
form (same file, line 39, bound only by `MatNum`) and its backing `mul_scalar` free function
(`mdmath_core/src/vector/arithmetics.rs:457`, bound only by `Scalar` — documented "Satisfied by all
integer primitives and floats") already fully support integers. This half is unrelated to (1),
needs no upstream change, and is not blocked by anything — it is 4 missing, purely mechanical `impl`
blocks that could be implemented independently and first if this task is picked up before `044`
lands.

Observable: `I32x3::new(3,1,2).min(I32x3::new(1,5,0))` compiles and returns `(1,1,0)`;
`2_i32 * I32x3::new(1,2,3)` compiles and returns `(2,4,6)` — both fail to compile today (E0599 for
the first, since `min`/`max` don't exist for `i32`; E0277/no matching impl for the second). Testable
via new cases added to the existing macro-parameterized `tests/inc/integer_test/arithmetic_test.rs`
(i32/i64/u32/u64 covered in one macro invocation, matching its established pattern).

**Related Tasks:** `blocked_by` `044` (`task/verified/044_mdmath_core_min_max_integer_bound.md`) for
the min/max half only — see Work Procedure step 1.

## In Scope

- `module/math/ndarray_cg/src/vector/arithmetics.rs`: split the
  `impl< E : MatEl + NdFloat, LEN > Vector< E, LEN >` block (lines 61-96) into two blocks — keep
  `normalize`/`mag`/`distance` on `E : MatEl + NdFloat`; move `min`/`max` (lines 78, 85) onto the
  relaxed bound `044` establishes in `mdmath_core`; update their doc comments to drop "Currently
  float-only"
- `module/math/ndarray_cg/src/vector/operator/mul.rs`: add
  `impl< const LEN : usize > Mul< Vector< {i32,i64,u32,u64}, LEN > > for {i32,i64,u32,u64}`
  (4 new blocks after line 88), each calling `mul_scalar( &rhs, self )`, mirroring the existing
  `f32`/`f64` blocks (lines 69-88) including their overflow doc-comment
- `module/math/ndarray_cg/tests/inc/integer_test/arithmetic_test.rs`: add a
  `vector_min_max_generic< E >()` test function and a commutative-multiplication test function, both
  registered inside the `integer_arithmetic_tests!` macro body (lines 348-413) so `i32`/`i64`/`u32`/
  `u64` all get coverage automatically from one macro invocation

## Out of Scope

- `044` itself (the `mdmath_core` free-function bound relaxation) — this task is `blocked_by` it,
  not a duplicate of it
- `normalize()`/`mag()`/`distance()` — correctly stay `NdFloat`-bound (genuine `sqrt` need); not
  touched
- Matrix (`Mat< .. >`) scalar multiplication or min/max — `todo.md`'s claim was scoped to "Vectors";
  matrices are a separate, unaudited surface
- `usize`/`isize` or integer widths beyond `i32`/`i64`/`u32`/`u64` — no named vector type aliases
  exist for other widths today
- `Animatable` trait coverage for these integer types in the `animation` crate — tracked separately
  as task `045`

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed before its implementing change landed
-   Minimum code to satisfy Test Matrix — no features beyond requirements
-   `verb/test_only` (package-scoped to `ndarray_cg`) passes with zero failures and zero warnings
-   No duplication introduced; public items keep `///` doc comments accurate to new behavior
-   All Rust code uses 2-space indentation, no `cargo fmt`

## Work Procedure

1. Confirm `044`'s relaxed bound has landed in `mdmath_core` (this task is `blocked_by: 044` for
   this step only — step 3's Mul-impl work has no such dependency and may be done first).
2. In `ndarray_cg/src/vector/arithmetics.rs`, split lines 61-96 into two `impl` blocks:
   `impl< E : MatEl + NdFloat, LEN > Vector< E, LEN >` keeping only `normalize`/`mag`/`distance`;
   a new `impl< E : MatEl + <044's relaxed bound>, LEN > Vector< E, LEN >` holding `min`/`max`.
   Update their doc comments to remove "Currently float-only" wording.
3. In `ndarray_cg/src/vector/operator/mul.rs`, add the 4 missing commutative impls (`i32`/`i64`/
   `u32`/`u64`) after line 88, each calling `mul_scalar( &rhs, self )`, matching the `f32`/`f64`
   pattern (lines 69-88) exactly including the overflow doc-comment.
4. In `ndarray_cg/tests/inc/integer_test/arithmetic_test.rs`, add `vector_min_max_generic< E >()`
   (asserting componentwise min/max, mirroring `vector_dot_generic`'s style at line 99) and a
   commutative-multiplication test near `vector_scalar_mul_div_generic` (line 24) asserting
   `E::from( 3 ) * v == v * E::from( 3 )`; register both inside `integer_arithmetic_tests!`
   (lines 348-413).
5. Run `verb/test_only` scoped to `ndarray_cg` (§ Long-Run Execution : Breadth Selection — package-
   scoped, not full workspace) to confirm new tests pass and existing float-path tests are
   unaffected.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|---|
| T01 | `I32x3::new(3,1,2).min(I32x3::new(1,5,0))` | `Vector::min`, `i32` | Compiles; returns `(1,1,0)` |
| T02 | `I32x3::new(3,1,2).max(I32x3::new(1,5,0))` | `Vector::max`, `i32` | Compiles; returns `(3,5,2)` |
| T03 | Same as T01/T02 for `i64`, `u32`, `u64` | `Vector::min`/`max` | Compiles; correct componentwise result each type |
| T04 | `2_i32 * I32x3::new(1,2,3)` | commutative `Mul<Vector<i32,3>> for i32` | Compiles; returns `(2,4,6)` |
| T05 | Same as T04 for `i64`, `u32`, `u64` | commutative `Mul` | Compiles; correct componentwise result each type |
| T06 | `F32x3::new(1.0,5.0,2.0).min(F32x3::new(3.0,1.0,0.0))` / `2.0_f32 * F32x3::new(1.0,2.0,3.0)` | existing float paths | Unchanged behavior — regression check |

## Acceptance Criteria

- `Vector::min()`/`Vector::max()` compile and produce correct componentwise results for `i32`,
  `i64`, `u32`, `u64`
- `E * Vector<E,LEN>` compiles for `i32`/`i64`/`u32`/`u64`, matching `Vector<E,LEN> * E`'s existing
  result
- Existing `f32`/`f64` behavior for all touched operations is unchanged
- Every Test Matrix row has a corresponding passing test
- `verb/test_only` scoped to `ndarray_cg` passes with zero failures and zero new warnings

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass re-ran `grep -n` against live source for every file:line citation (`mul.rs:69,80`; `arithmetics.rs:9,34,61`) — all confirmed unchanged and accurate | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | `044` referenced only as a `blocked_by` dependency, not a deliverable path of this task — all actual deliverables resolve inside `ndarray_cg` | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | Grounded against `module/math/ndarray_cg/readme.md`'s actual text ("comprehensive matrix and linear algebra library... for computer graphics applications") — this task only widens existing operations' applicability, doesn't add a new responsibility | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 8 dimensions 🟢 on both passes, no fixes needed.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-09]** `FILED` — Filed as the `ndarray_cg`-side half of the integer-vector-math gap from
  root `todo.md`'s "i32 and u32 Vectors" claim, investigated per user request ("are those legit
  tasks/bugs? investigate and if so file them"). Split from `044` (`mdmath_core`) per
  `tsk.rulebook.md`'s crate-scoped task convention — `blocked_by: 044` for the min/max half; the
  scalar×vector `Mul` half is independent and unblocked.
- **[2026-08-09]** `VERIFY_PASS` — Readiness Verification Gate (Tier 2 Dual-Role Self-Check) run: all
  8 dimensions PASS on both passes, no defects found. State → 🎯 Verified; file moved to
  `task/verified/`.
- **[2026-08-09]** `RESOLVED` — `Vector::min()`/`Vector::max()` (`src/vector/arithmetics.rs`) moved
  off the `NdFloat`-bound impl block into a new `impl< E : MatNum + PartialOrd, LEN >` block,
  consuming `044`'s relaxed `mdmath_core::vector::{min,max}` free functions; doc comments updated to
  remove "Currently float-only" wording. `operator/mul.rs` gained 4 new commutative
  `impl Mul<Vector<{i32,i64,u32,u64},LEN>> for {i32,i64,u32,u64}` blocks (mirroring the existing
  `f32`/`f64` pattern, including the overflow doc-comment), consuming the already-generic
  `mul_scalar` free function. New `vector_min_max_generic< E >()` and
  `vector_scalar_mul_commutative_generic< E >()` test functions added to
  `tests/inc/integer_test/arithmetic_test.rs`, registered inside `integer_arithmetic_tests!` for
  i32/i64/u32/u64 (T01-T06 all covered). TDD confirmed: both fixes temporarily reverted together
  reproduced 8 genuine compile errors (4× E0599/E0308 for missing integer `min`/`max`, 4× E0277 for
  missing commutative `Mul`) via `cargo nextest run -p ndarray_cg --all-features`; fixes restored →
  `237 tests run: 237 passed, 0 skipped`, including all 8 new test instances (i32/i64/u32/u64 × 2).
  `cargo clippy -p ndarray_cg --all-targets --all-features -- -D warnings` clean. Same-session,
  self-administered (filer = fixer = verifier) — Tier 2 Dual-Role Self-Check per
  governance/maav.rulebook.md's default, not an independent PROC16-style acceptance pass. State →
  ✅ Completed.

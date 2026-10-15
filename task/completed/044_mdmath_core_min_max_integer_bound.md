# Relax mdmath_core's vector min/max free functions from NdFloat to an integer-inclusive bound

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
- **unit:** lib/yrd_gamedev/cgtools/module/math/mdmath_core
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-09
- **blocked_by:** null

## Goal

Root `todo.md`'s "i32 and u32 Vectors" claim ("Math operations need to be implemented for `i32`,
`i64`, `u32`, `u64`") is true for one narrow, concrete gap in this crate — not the broad claim as
stated (`dot`/`mag2`/`distance_squared` already work generically for all integer types, confirmed
by `ndarray_cg`'s existing `tests/inc/integer_test/arithmetic_test.rs`). `mdmath_core`'s
`vector::arithmetics::{min_mut, min, max_mut, max}` free functions
(`src/vector/arithmetics.rs:544-597`) are bound `E : NdFloat` and call `(*r).min(*a)`/`(*r).max(*a)`,
which resolve via that bound to `num_traits::Float::min`/`max`. `NdFloat` is re-exported verbatim
from the external `ndarray` crate (`src/nd.rs:14`: `exposed use ::ndarray::{ LinalgScalar, NdFloat };`)
and requires `Float`, so no integer type can ever satisfy it — yet element-wise min/max is pure
ordering comparison, not floating-point arithmetic; nothing about it needs `sqrt` or any other
float-specific capability the way `mag`/`normalized`/`distance` genuinely do. This crate's own
`float.rs:6-9` already documents exactly the kind of bound this should use instead: a `Scalar` trait
"supporting field-agnostic arithmetic... without requiring float-specific operations like `sqrt`...
Satisfied by all integer primitives and floats" — though whether `Scalar`, `PartialOrd`, or another
bound is the correct replacement is an implementation decision (see Work Procedure), not fixed here.

Fix confined entirely to this crate: relax the bound, rewrite the two comparisons so they no longer
depend on a `Float`- or `Ord`-only method, and add integer-input test coverage. Observable:
`mdmath_core::vector::min(&[1i32,5,2], &[3,1,4])`-shaped calls compile and return the componentwise
minimum for `i32`/`i64`/`u32`/`u64` arrays — today this fails with E0277 (`i32` does not implement
`NdFloat`).

**Related Tasks:** `ndarray_cg`'s `Vector::min()`/`Vector::max()` wrapper methods call directly into
this crate's `min`/`max` free functions and cannot compile against a relaxed integer type until this
task lands — see `048` (`blocked_by` this task). `048` also covers an unrelated second gap (missing
integer commutative scalar×vector `Mul` impls) that lives entirely in `ndarray_cg` and does not
depend on this task.

## In Scope

- `module/math/mdmath_core/src/vector/arithmetics.rs`: relax `min_mut`, `min`, `max_mut`, `max`
  (lines 544-597) from `E : NdFloat` to a bound satisfied by `f32`/`f64`/`i32`/`i64`/`u32`/`u64`
  alike; rewrite their bodies' `(*r).min(*a)` / `(*r).max(*a)` calls to explicit comparison logic
  that does not rely on a `Float`- or `Ord`-only method
- Doc comments on the 4 functions, updated to describe the new (wider) applicability and to state
  the chosen float NaN tie-break behavior (see Work Procedure step 2)
- New or extended tests in `module/math/mdmath_core/tests/` exercising `min`/`max`/`min_mut`/`max_mut`
  against integer-array inputs, following this crate's own existing test conventions

## Out of Scope

- Anything in `ndarray_cg` (the `Vector::min()`/`Vector::max()` wrapper methods that call these free
  functions, and the unrelated missing commutative scalar×vector `Mul` impls) — tracked as `048`,
  which is `blocked_by` this task for its min/max half
- `dot`/`mag2`/`mag`/`normalized`/`normalized_to`/`distance`-family free functions in this same file
  — already correctly scoped (generic-numeric or correctly float-only per genuine `sqrt` need,
  confirmed by prior investigation) — not touched
- Redefining `NdFloat` itself — it is external, re-exported verbatim from the `ndarray` crate; not
  this crate's to change

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
-   `verb/test_only` (package-scoped to `mdmath_core`) passes with zero failures and zero warnings
-   No duplication introduced; public items keep `///` doc comments accurate to new behavior
-   All Rust code uses 2-space indentation, no `cargo fmt`

## Work Procedure

1. Change `min_mut`/`min`/`max_mut`/`max`'s trait bound (`src/vector/arithmetics.rs:544-597`) from
   `E : NdFloat` to `E : PartialOrd` (or the narrowest bound found to compile cleanly for both float
   and all 4 integer types — an implementation-time decision, not fixed in advance here).
2. Rewrite `min_mut`'s body (`*r = ( *r ).min( *a );`) to explicit comparison
   (e.g. `*r = if *a < *r { *a } else { *r };`), and `max_mut`'s body (`*r = ( *r ).max( *a );`)
   likewise (e.g. `*r = if *a > *r { *a } else { *r };`) — avoids depending on `Float::min`/`max` or
   `Ord::min`/`max`, either of which would re-narrow the bound back to one type family. Document the
   chosen float NaN tie-break behavior (which operand wins when either is NaN) in each function's doc
   comment, since this changes from `Float::min`/`max`'s IEEE-754 semantics to whatever the explicit
   comparison yields.
3. Update the 4 functions' doc comments to remove float-only wording and state the new bound.
4. Add test cases for `min`/`max`/`min_mut`/`max_mut` in this crate's `tests/` directory. No test
   file in `mdmath_core/tests/` currently exercises these 4 functions at all (confirmed by search —
   `tests/inc/vector_test/float_test.rs` covers only `all_true`/`any_true`/`is_nan`; there is no
   existing float-path coverage to extend, let alone integer). Create a new test module (e.g.
   `tests/inc/vector_test/min_max_test.rs`, registered in `tests/inc/vector_test.rs`'s `mod` list
   alongside its `float_test`/`array_test`/`slice_test` siblings) covering both the float path
   (regression) and the new integer paths.
5. Run `verb/test_only` scoped to `mdmath_core` (§ Long-Run Execution : Breadth Selection — package-
   scoped, not full workspace) to confirm new tests pass and existing float-path tests are unaffected.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|---|
| T01 | `min(&[3i32,1,2], &[1i32,5,0])` | `min` free fn, `i32` | Compiles; returns `[1,1,0]` |
| T02 | `max(&[3i32,1,2], &[1i32,5,0])` | `max` free fn, `i32` | Compiles; returns `[3,5,2]` |
| T03 | Same as T01/T02 for `i64`, `u32`, `u64` | `min`/`max` | Compiles; correct componentwise result each type |
| T04 | `min(&[3.0f32,1.0,2.0], &[1.0f32,5.0,0.0])` | existing float path | Unchanged: `[1.0,1.0,0.0]` — regression check |
| T05 | `min`/`max` with a `NaN` component (`f32`) | float NaN tie-break | Matches whichever behavior step 2 documents (not IEEE `Float::min` unless explicitly reproduced) |

## Acceptance Criteria

- `min`/`min_mut`/`max`/`max_mut` compile and produce correct componentwise results for `i32`,
  `i64`, `u32`, `u64` array/vector inputs
- Existing `f32`/`f64` behavior for these 4 functions is unchanged except for the documented NaN
  tie-break wording
- `NdFloat` is no longer named in any of the 4 functions' bounds
- Every Test Matrix row has a corresponding passing test
- `verb/test_only` scoped to `mdmath_core` passes with zero failures and zero new warnings

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass independently confirmed `[E; N]` implements `VectorIter`/`VectorIterMut` (`src/vector/array.rs:40,50`), so the Goal's Observable example call shape is real, not fabricated | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | 🟡 | 🟢 | Confirming pass named `tests/inc/vector_test/float_test.rs` as the "closest existing home" for min/max tests; adversarial pass greped the whole `mdmath_core/tests/` tree and found that file only covers `all_true`/`any_true`/`is_nan` — no min/max coverage exists anywhere in this crate today | Rewrote Work Procedure step 4 to state no existing coverage exists and to create a new `tests/inc/vector_test/min_max_test.rs` module instead |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | Grounded against `module/math/mdmath_core/readme.md`'s actual text ("essential vector operations and geometric primitives") rather than asserted — this task only widens an existing operation's bound, doesn't add a new responsibility | — |
| **Total** | | 🔴 | 🟢 | 1 fixed | 1/1 |

**Aggregate verdict:** PASS — all 8 dimensions 🟢 on both passes after one Fix-and-Recheck round.

## Verification

### Checklist

- [x] C1 — Are `min_mut`/`min`/`max_mut`/`max` no longer bound by `NdFloat`? `grep -n "NdFloat" module/math/mdmath_core/src/vector/arithmetics.rs` → 9 hits, none inside these 4 functions (lines 555-620); those functions instead show `E : Scalar + PartialOrd` (at lines 559, 577, 596, 614).
- [x] C2 — Do the 4 functions use explicit comparisons rather than `Float::min`/`max` or `Ord::min`/`max`? Direct read of `arithmetics.rs` around `min_mut`/`max_mut` → `*r = if *a < *r { *a } else { *r };` and `*r = if *a > *r { *a } else { *r };` — no `.min(`/`.max(` method call present anywhere in the 4 bodies.
- [x] C3 — Do `min`/`max`/`min_mut`/`max_mut` compile and produce correct componentwise results for `i32`/`i64`/`u32`/`u64`? `module/math/mdmath_core/tests/inc/vector_test/min_max_test.rs` (7 test fns: `integer_i32`, `integer_i64`, `integer_u32`, `integer_u64`, `float_regression`, `float_nan_tie_break`, `mut_variants`) — all 7 pass in this session's fresh run (see I1).
- [x] C4 — Is existing `f32`/`f64` behavior unchanged except for the documented NaN tie-break wording? `float_regression` asserts the unchanged `[1.0,1.0,0.0]`/`[3.0,5.0,2.0]` results; `float_nan_tie_break` independently pins the newly-documented tie-break (accumulator `r` wins whenever either operand is NaN) — both pass.
- [x] C5 — Is the new test module actually wired into the suite? `module/math/mdmath_core/tests/inc/vector_test.rs` → `mod min_max_test;` present; confirmed NOT present in the pre-fix parent commit (`git show 9b71cf39^:module/math/mdmath_core/tests/inc/vector_test.rs` has no such line).

### Measurements

- [x] M1 — `NdFloat`-bound occurrences across `min_mut`/`min`/`max_mut`/`max`: `0` (was: `4`, one per function — `git show 9b71cf39^:module/math/mdmath_core/src/vector/arithmetics.rs` lines 548/563/577/592 each show `E : NdFloat,` directly inside these 4 functions).
- [x] M2 — Test functions exercising `min`/`max`/`min_mut`/`max_mut`: `7` in `min_max_test.rs` (was: `0` — `git show 9b71cf39^:module/math/mdmath_core/tests/inc/vector_test.rs` confirms no `min_max_test` module existed at all pre-fix).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p mdmath_core --all-features` (via `longrun`) → exit 0, 89 tests run: 89 passed, 0 skipped, including 7/7 `min_max_test::*` (log `-0014_longrun.log`).
- [x] I2 — Lints clean: `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings` (via `longrun`) → exit 0, zero warnings (log `-0018_longrun.log`).

### Anti-faking checks

- [x] AF1 — Guards against a future change re-narrowing the bound back to `NdFloat`/`Float`/`Ord`: re-run C1's grep — the 4 functions' `where` clauses must still show `Scalar + PartialOrd`, never `NdFloat`, `Float`, or a bare `Ord` bound.
- [x] AF2 — Guards against the integer coverage being silently deleted while the bound stays relaxed (untested API widening): `grep -c "^fn " module/math/mdmath_core/tests/inc/vector_test/min_max_test.rs` must stay ≥ 7, and `mod min_max_test;` must remain registered in `vector_test.rs`.

## History

*(append-only — newest entry last; never edit or remove past entries)*

- **[2026-08-09]** `FILED` — Filed from root `todo.md`'s "i32 and u32 Vectors" claim, investigated per
  user request ("are those legit tasks/bugs? investigate and if so file them"). Split from the
  originally-scoped single "generalize integer vector math (ndarray_cg)" item after investigation
  traced the actual restriction to this crate's `min`/`max` free functions (`NdFloat`, sourced from
  the external `ndarray` crate) — `ndarray_cg`'s `Vector::min/max` merely delegate here, so the two
  crates' halves of the fix are split per `tsk.rulebook.md`'s crate-scoped task convention (see `048`).
- **[2026-08-09]** `VERIFY_PASS` — Readiness Verification Gate (Tier 2 Dual-Role Self-Check) run: all
  8 dimensions PASS. D4's adversarial pass caught a factually wrong "closest existing test" claim
  (fixed in place — see Verification Record); all other dimensions clean on first pass. State →
  🎯 Verified; file moved to `task/verified/`.
- **[2026-08-09]** `RESOLVED` — `min_mut`/`min`/`max_mut`/`max` (`src/vector/arithmetics.rs:544-613`)
  relaxed from `E : NdFloat` to `E : Scalar + PartialOrd`; bodies rewritten to explicit
  `if *a < *r {*a} else {*r}` / `if *a > *r {*a} else {*r}` comparisons (documented NaN tie-break:
  the accumulator `r` wins on any unordered comparison, so NaN in the public `min`/`max` wrappers'
  first argument propagates through and NaN in the second argument is ignored). New
  `tests/inc/vector_test/min_max_test.rs` (registered in `tests/inc/vector_test.rs`) covers all 5
  Test Matrix rows (T01-T05) plus the `_mut` variants. TDD confirmed: bound temporarily reverted to
  `NdFloat` reproduced 10 `E0277` errors on the new integer tests
  (`cargo nextest run -p mdmath_core --all-features`), bound restored → `73 tests run: 73 passed, 0
  skipped`; `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings` clean (one
  `clippy::float_cmp` false-positive suppressed via `#![ allow(...) ]`, matching this crate's own
  existing `tests/inc/arithmetics.rs` convention). Same-session, self-administered (filer = fixer =
  verifier) — Tier 2 Dual-Role Self-Check per governance/maav.rulebook.md's default, not an
  independent PROC16-style acceptance pass. State → ✅ Completed.

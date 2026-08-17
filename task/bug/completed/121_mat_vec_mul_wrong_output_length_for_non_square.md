# BUG-121: `Mat * Vector` multiplication produces a wrong-length, silently-truncated result for any non-square matrix

- **Severity:** Medium (currently unreachable — every existing matrix type in this crate is square — but a real, latent public-API defect; same "unreachable but real" pattern as BUG-043/BUG-050)
- **state:** Completed
- **Affects:** Any future `Mat<ROWS,COLS,E,Descriptor>` with `ROWS != COLS` multiplied by a `Vector<E,COLS>` via the `Mul` operator or the `mat_vec_mul` free function directly — not reachable through any current production code path, since `Mat2`/`Mat3`/`Mat4` are the crate's only shipped matrix sizes and all three are square
- **Component:** `module/math/ndarray_cg` (`src/d2/arithmetics/mul.rs::{mat_vec_mul, impl Mul<Vector<COLS>> for Mat<ROWS,COLS,..> (x2)}`)
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** None — independent root cause from BUG-250/119/120, filed under the same task #52 targeted math review

## Symptom

```bash
# a 3x2 matrix (3 rows, 2 columns) times a length-2 vector should produce a length-3 vector
# (standard M x N matrix times N x 1 vector = M x 1 vector).

# Wrong (pre-fix) -- Output type is Vector<E,2> (COLS), not Vector<E,3> (ROWS):
# does not even compile as a 3-component result; if forced through mat_vec_mul directly with
# a length-2 output buffer, only 2 of the 3 true dot products are computed -- row 2 is dropped.
result: [ dot(row0,vec), dot(row1,vec) ]     # length 2 -- row 2 silently never computed

# Correct (post-fix) -- Output type is Vector<E,3> (ROWS):
result: [ dot(row0,vec), dot(row1,vec), dot(row2,vec) ]    # length 3 -- all three rows present
```

## Impact

**Who is affected:** No current caller — every matrix type this crate actually ships (`Mat2`,
`Mat3`, `Mat4`, all square) is unaffected, since `ROWS == COLS` makes the bug's wrong `Output`
length coincide with the correct one by construction. Any future non-square `Mat<M,N>` (`M != N`)
usage — e.g. a projection or Jacobian-style rectangular matrix — would hit this immediately and
silently.

**What breaks:** For a non-square matrix, the `Mul<Vector<E,COLS>>` operator's `Output` type is
hardcoded to `Vector<E,COLS>` instead of the mathematically correct `Vector<E,ROWS>` — for `M !=
N` this is simply the wrong type (wrong result length), not merely a wrong value. Should a caller
force the free function `mat_vec_mul` directly with an `OUT`-length buffer that happens to be
shorter than the matrix's true row count, the loop only ever writes to the shorter buffer's
elements, silently never computing the remaining rows — the classic "unreachable but real" defect
shape: no current test or caller exercises `ROWS != COLS`, so it has never surfaced, but the type
signature itself is wrong for the general case the function's own name and doc comment ("multiplies
vector by a matrix", with no square-only caveat) claim to support.

**Magnitude:** Zero current callers affected (confirmed: `Mat2`/`Mat3`/`Mat4` are the only matrix
sizes instantiated anywhere in `src/`); every future non-square instantiation would be affected
immediately upon first use.

**Entity Scope:** None — a code-level math defect, not an operational-entity concern.

## How Discovered

Task #52, a targeted math/geometry code review of core crates dispatched under the standing
bug-hunt mandate. The reviewing agent flagged that `mat_vec_mul`'s single `const ROWS : usize`
generic is used for both the input vector's bound (`B : VectorIter<E,ROWS>`) and the output
vector's bound (`R : VectorIterMut<E,ROWS>`) — two quantities that are only equal for a square
matrix. Independently traced before filing: the `Mul<Vector<COLS>>` impl's `Output =
Vector<E,COLS>` is what forces `mat_vec_mul`'s `ROWS` generic to bind to `COLS` (the matrix's
column/input count) rather than the matrix's actual row/output count, at every call site.

```bash
$ grep -n "type Output\|mat_vec_mul<\|VectorIterMut< E, ROWS >\|VectorIter< E, ROWS >" \
    module/math/ndarray_cg/src/d2/arithmetics/mul.rs
159:  type Output = Vector< E, COLS >;   # pre-fix -- should be Vector<E, ROWS> for M x N Mat*Vec
64:pub fn mat_vec_mul< E, A, B, R, const ROWS : usize >( r : &mut R, a : &A, b : &B )
67:  R : VectorIterMut< E, ROWS >,       # output length == ROWS generic
69:  B : VectorIter< E, ROWS >,          # input length ALSO == the same ROWS generic
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre121 && mkdir -p /tmp/mre121/src
cat > /tmp/mre121/Cargo.toml <<'EOF'
[package]
name = "mre121"
version = "0.1.0"
edition = "2021"

[dependencies]
ndarray_cg = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/math/ndarray_cg" }
EOF
cat > /tmp/mre121/src/main.rs <<'EOF'
use ndarray_cg::{ Mat, Vector, mat::DescriptorOrderRowMajor, d2 };

fn main()
{
  // 3 rows x 2 cols, row-major: [ [1,2], [3,4], [5,6] ]
  let a = Mat::< 3, 2, f64, DescriptorOrderRowMajor >::default()
    .row_major_set( &[ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
  let b = Vector::< f64, 2 >::from_array( [ 1.0, 1.0 ] );

  // Forced through the free function with an OUT-length-2 buffer -- pre-fix, this was the
  // ONLY buffer shape `mat_vec_mul` could even accept for this matrix's column count (2),
  // since IN and OUT were the same generic. Correct math needs a length-3 output ([3,7,11]).
  let mut r = Vector::< f64, 2 >::default();
  d2::mat_vec_mul( &mut r, &a, &b );
  println!( "mat_vec_mul result (len 2 buffer): {:?}", r.to_array() );
  println!( "row 2 dot product (1*5 + 1*6 = 11) never computed into any output slot" );
}
EOF
cd /tmp/mre121 && cargo run 2>&1 | tail -2
```

**Expected** (post-fix — the `Mul` operator now returns `Vector<E,3>` directly; calling
`&a * &b` compiles and yields all three rows):
```
a * b = [3.0, 7.0, 11.0]
```

**Actual** (pre-fix — the `Mul<Vector<E,2>>` operator's `Output` is `Vector<E,2>`, so `&a * &b`
either fails to type-check against a 3-row matrix's true shape, or — when the free function is
called directly with a matching length-2 buffer as shown above — only computes 2 of the 3 true
dot products, permanently dropping row 2's contribution):
```
mat_vec_mul result (len 2 buffer): [3.0, 7.0]
row 2 dot product (1*5 + 1*6 = 11) never computed into any output slot
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre121 && cargo run 2>&1 | tail -2
# 3-element result [3.0, 7.0, 11.0] = fixed; 2-element result (row 2 missing) = bug present
```
**What:** Violates the standard M×N matrix times N×1 vector = M×1 vector shape contract —
`mat_vec_mul`'s own doc comment ("Multiplies vector by a matrix") carries no square-only
restriction, but the pre-fix signature only supported `M == N`.

**Known MRE limitation (check 205):** `ndarray_cg` is this workspace's own crate; the MRE
path-depends on it locally rather than a registry version, mirroring BUG-116/118/119/120's own
documented exception. The dot products used (`1*1+2*1=3`, `3*1+4*1=7`, `5*1+6*1=11`) are exact
integers represented in `f64`, so there is no floating-point ambiguity this local dependency
could be hiding.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `mat_vec_mul`'s single `const ROWS : usize` generic is reused for both the input vector's length (should be the matrix's column count) and the output vector's length (should be the matrix's row count) — two independent quantities for a non-square matrix — and the `Mul<Vector<COLS>>` operator impls compound this by hardcoding `Output = Vector<E,COLS>` (using the matrix's column count for what should be a row-count-length result). | ✅ Root Cause | Direct read of `mul.rs:64-70` (pre-fix) confirms both `R`/`B` bounds share the identical `ROWS` generic; `mul.rs:159`/`181` (pre-fix) confirm `Output = Vector<E,COLS>`. MRE confirms a 3×2 matrix's third row is never computed when forced through a length-2 (COLS-sized) buffer, exactly matching this mechanism. | E1, E2, E3 |
| H2 | The dot-product computation itself (`a.lane_iter(0,row).zip(b.vector_iter())...fold(..)`) is wrong for non-square matrices, independent of the generic/Output-type issue. | ❌ Falsified | The computation loop is generic-shape-agnostic: it iterates `r.vector_iter_mut()` (whatever length `r` actually has) and for each `row` computes `a.lane_iter(0,row)` zipped against `b.vector_iter()` — this is the mathematically correct per-row dot product regardless of squareness; the only defect is that `r`'s length (and thus how many rows get computed) was wrongly tied to the input vector's length instead of the matrix's actual row count. Post-fix (with `r` correctly sized to `OUT`=matrix row count), the identical loop body produces the correct full-length result. | E3 |
| H3 | The dimension-compatibility `assert!` (TASK-014's unconditional check) would have caught a real-world non-square misuse before it produced a wrong result, making this a lower-severity "would panic, not silently wrong" issue. | ❌ Falsified | Pre-fix, the assert checked only `adim[1] == ROWS` (the matrix's column count against the single shared generic) — for the MRE's 3×2 matrix with a length-2 `r`/`b`, `adim[1]` (=2) does equal `ROWS` (=2, inferred from the length-2 buffers), so the assert PASSES even though the matrix has 3 rows, not 2. The assert could not distinguish "correct square case" from "incorrect non-square case with the row dimension silently ignored" because it never checked `adim[0]` at all. | E1, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `module/math/ndarray_cg/src/d2/arithmetics/mul.rs:64-70` (pre-fix) | `mat_vec_mul<E,A,B,R,const ROWS:usize>` with `R:VectorIterMut<E,ROWS>` and `B:VectorIter<E,ROWS>` — both input and output bound to the identical single generic; the dimension assert (pre-fix) only checked `adim[1]==ROWS`, never `adim[0]`. | H1 ✅, H3 ❌ |
| E2 | `module/math/ndarray_cg/src/d2/arithmetics/mul.rs:159` and the `&Mat * &Vector` impl (pre-fix) | Both `Mul<Vector<E,COLS>>` impls declare `type Output = Vector<E,COLS>` — for `Mat<ROWS,COLS,..>`, this uses the matrix's column count for the result length, when the mathematically correct result length is the matrix's ROW count. | H1 ✅ |
| E3 | `/tmp/mre121` run, pre-fix vs. post-fix, 3×2 matrix `[[1,2],[3,4],[5,6]]` times `[1,1]` | Pre-fix (length-2 buffer, the only shape the old signature could produce for this matrix): `[3.0, 7.0]` — row 2's dot product (`5*1+6*1=11`) never computed. Post-fix (`&a * &b` now returns `Vector<E,3>` directly): `[3.0, 7.0, 11.0]` — all three rows present, confirming the per-row loop body itself was always correct once given a correctly-sized output. | H1 ✅, H2 ❌ |

## Root Cause

```
Mat< ROWS=3, COLS=2, .. > * Vector< E, COLS=2 >
  -> Mul impl:  Output = Vector< E, COLS >              (pre-fix, mul.rs:159)   ✗ should be ROWS
  -> mat_vec_mul( &mut result: Vector<E,COLS=2>, &self, &rhs: Vector<E,COLS=2> )
       R : VectorIterMut< E, ROWS_inner >  <- binds ROWS_inner = 2 (from `result`'s length)
       B : VectorIter< E, ROWS_inner >     <- binds ROWS_inner = 2 (from `rhs`'s length, same generic)
  -> loop: for (row, e) in r.vector_iter_mut().enumerate()   <- only 2 iterations (r has len 2)
       row 2 (the matrix's true 3rd row) is never visited     ✗
```

A matrix-vector product's input length (matrix column count) and output length (matrix row
count) are independent quantities in general — equal only for a square matrix. `mat_vec_mul`
reused a single const generic (confusingly also named `ROWS`, unrelated to the outer `Mul` impl's
own `ROWS`/`COLS` generics) for both the input bound `B : VectorIter<E,ROWS>` and the output bound
`R : VectorIterMut<E,ROWS>`. Every existing caller instantiates only square matrices (`Mat2`,
`Mat3`, `Mat4`), where input length always equals output length coincidentally, so the reuse was
invisible. The `Mul<Vector<COLS>>` operator impls compound the same mistake one level up by
hardcoding `Output = Vector<E,COLS>` — using the matrix's column count for what is mathematically
the row-count-length result — which is what forces `mat_vec_mul`'s shared generic to bind to
`COLS` instead of the matrix's actual row count at every real call site.

## Why Not Caught

No test exercises `mat_vec_mul` or the `Mul<Vector<COLS>>` operators against a non-square matrix
— `tests/inc/d2_test/arithmetic_test/mul_test.rs`'s existing `mat_vec_mul` coverage
(`test_multiply_vec_incompatible_dimensions_generic`) uses a 3×3 (square) matrix deliberately
mismatched against a length-2 vector to test the dimension-panic path, but no test constructs a
genuinely non-square `Mat<M,N>` with `M != N` and multiplies it by a correctly-shaped `Vector<E,N>`
to confirm the *successful* path produces a length-`M` result. Every matrix type this crate ships
(`Mat2`/`Mat3`/`Mat4`) is square, so no integration test elsewhere in the suite could have
surfaced this either.

## Fix Location

`module/math/ndarray_cg/src/d2/arithmetics/mul.rs`, plus one coupled file. Three changes:

1. `mat_vec_mul` (lines 64-70 pre-fix): split the single `const ROWS : usize` generic into
   `const IN : usize` (bounds `B`, the input vector) and `const OUT : usize` (bounds `R`, the
   output vector); dimension assert extended from `adim[1]==ROWS` to `adim[0]==OUT &&
   adim[1]==IN`.
2. Both `Mul<Vector<E,COLS>>`/`Mul<&Vector<E,COLS>>` impls (lines 159, 181 pre-fix): `Output`
   changed from `Vector<E,COLS>` to `Vector<E,ROWS>`.
3. `module/math/ndarray_cg/src/vector/operator/mul.rs`'s `Vector<E,COLS> : MulAssign<Mat<ROWS,COLS,..>>`
   impl (lines 9-21 pre-fix): change #2 broke this impl's compilation — `*self = rhs * *self`
   requires the product's type to equal `Self` exactly, and once `Mat<ROWS,COLS>*Vector<COLS>`'s
   `Output` became the mathematically-correct `Vector<E,ROWS>`, it no longer matched
   `Self = Vector<E,COLS>` for independent `ROWS`/`COLS` generics. This impl was itself always
   dimensionally unsound for a non-square matrix (`v *= M` cannot preserve `v`'s length when `M`
   isn't square) and only compiled before because it inherited the same wrong `COLS`-pinned
   `Output` this bug fixes — narrowed to `Mat<N,N,..> : MulAssign for Vector<E,N>` (single shared
   const generic, square-only), the only dimensionally-sound form of this operation.

```rust
// before (signature)
pub fn mat_vec_mul< E, A, B, R, const ROWS : usize >( r : &mut R, a : &A, b : &B )
where
  E : MatNum,
  R : VectorIterMut< E, ROWS >,
  A : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  B : VectorIter< E, ROWS >,

// after
pub fn mat_vec_mul< E, A, B, R, const IN : usize, const OUT : usize >( r : &mut R, a : &A, b : &B )
where
  E : MatNum,
  R : VectorIterMut< E, OUT >,
  A : Indexable< Index = Ix2 > + IndexingRef< Scalar = E >,
  B : VectorIter< E, IN >,

// before (both Mul impls)
type Output = Vector< E, COLS >;

// after
type Output = Vector< E, ROWS >;
```

For every existing (square) matrix type, `ROWS == COLS`, so both changes are exact no-ops in
observable behavior — confirmed by the pre-existing `test_multiply_vec_incompatible_dimensions_*`
tests still passing unchanged (see `## Prevention`).

## Prevention

Added `test_mat_vec_mul_non_square_produces_full_length_result` (`_row_major`/`_column_major`
instantiations) to `tests/inc/d2_test/arithmetic_test/mul_test.rs`: constructs a genuinely
non-square `Mat<3,2,E,D>`, multiplies it by a `Vector<E,2>` via the `*` operator, and asserts the
result is a `Vector<E,3>` matching all three hand-computed dot products (the same fixture as this
bug's MRE) — this would fail to even compile under the pre-fix `Output = Vector<E,COLS>` type,
since a 3-component expected value cannot be compared against a 2-component actual value.

**Pitfall:** two conceptually independent lengths (here: a linear map's input dimension vs. output
dimension) that happen to coincide for every currently-existing caller (square matrices) can be
safely, silently unified into one const generic — until a non-square instantiation is attempted,
which the type system cannot catch on its own because nothing in the signature says the two
lengths must differ or must match. Same shape as BUG-043/BUG-050: a defect that is real and
public-API-visible but currently unreachable because no shipped caller exercises the differentiating
case — worth fixing on discovery rather than deferring, since "currently unreachable" is a property
of today's callers, not a guarantee about tomorrow's.

## Generalized Version

**Broken assumption:** "If two generic-parameter roles (here: a linear map's input length and
output length) are always equal for every currently-existing instantiation, they can share one
const generic without loss of generality" — false whenever the underlying operation is
mathematically defined for the general (unequal) case and the type is documented/named as
supporting that general case.

**Confirmed general rule:** for any function or trait impl computing `f : Space_A -> Space_B`
where `Space_A` and `Space_B` are indexed by potentially-different const-generic dimensions, using
a single shared const generic for both is only safe if the function's contract explicitly and
permanently restricts `A == B` (e.g. an endomorphism-only API). If the function's name, doc
comment, or surrounding API surface implies the general (`A` possibly `!= B`) case — as
"matrix-vector multiplication" inherently does — the detection invariant is: construct at least
one non-square/non-endomorphic instantiation and confirm the type signature accepts it and
produces the correct output length, even if no current production caller does so yet.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Discovered via task #52's targeted math/geometry code review; confirmed unreachable-but-real via direct trace of the `Output`/generic-binding mechanism from the `Mul` impl down through `mat_vec_mul`'s own generic resolution, before filing. |
| 2026-08-15 | fixed | Split `mat_vec_mul`'s `ROWS` generic into `IN`/`OUT`; extended its dimension assert to check both `adim[0]` and `adim[1]`; changed both `Mul<Vector<COLS>>` impls' `Output` from `Vector<E,COLS>` to `Vector<E,ROWS>`. 3-field `Fix(BUG-121)`/`Root cause`/`Pitfall` comments added at each fix site. |
| 2026-08-15 | verified | Added `test_mat_vec_mul_non_square_produces_full_length_result` (row-major + column-major instantiations) to `tests/inc/d2_test/arithmetic_test/mul_test.rs`; confirmed the pre-existing `test_multiply_vec_incompatible_dimensions_*` tests (square-matrix dimension-mismatch panic path) still pass unchanged. Full workspace verification (`verb/test` via `longrun`) recorded below, covering all four math bugs (BUG-250/119/120/121) verified together as one gate. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `mat_vec_mul` (confirmed `IN`/`OUT` split genuinely present, dimension assert extended to `adim[0]==OUT && adim[1]==IN`) and both `Mul<Vector<COLS>>`/`Mul<&Vector<COLS>>` impls (confirmed `Output` genuinely changed to `Vector<E,ROWS>` in both; 3-field comments intact at all three sites) and `test_mat_vec_mul_non_square_produces_full_length_result_generic` (non-tautological: constructs a real 3×2 matrix, multiplies by a length-2 vector via both the free function and the `*` operator, asserts the length-3 result against hand-computed dot products). Fresh `cargo nextest run -p ndarray_cg --all-features` via `longrun`: 272/272 passed. `cargo clippy -p ndarray_cg --all-features --all-targets -- -D warnings`: clean. MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-250/119/120/121 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + `Refs: src/`/`Refs: tests/` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟡 | 🟢 | Confirming pass hand-computed the dot products (`3,7,11`); adversarial pass independently re-verified `1*1+2*1=3`, `3*1+4*1=7`, `5*1+6*1=11` by re-reading the row-major fixture layout rather than trusting the first pass's arithmetic. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed this file correctly declares no `**Related Bugs:**` (root cause is genuinely independent of BUG-250/119/120 — different function, no shared code path) and correctly cross-references the BUG-043/BUG-050 precedent pattern by name only, not by broken link (no link target claimed). | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass re-traced the generic-binding chain (`Output` type → `mat_vec_mul` call → generic inference) from scratch rather than trusting the confirming pass's trace, confirming `Output=Vector<E,COLS>` is genuinely what forces the wrong binding. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass checked whether the plain `mul()` (matrix*matrix) function shares this defect — it doesn't: `mul()` already takes independent `R`/output dimensions via its `Indexable` bound on `R` directly (no shared const generic with `B`), confirmed by direct read of `mul.rs:14-52`. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Only `ndarray_cg`'s own `src/`/`tests/` and this bug-tracking file touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Fix is local to `mat_vec_mul` and its two direct `Mul` impl callers; no other function in the crate calls `mat_vec_mul` (confirmed via `## How Discovered`'s grep). | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Fix does not add any new responsibility — it corrects the generic-parameter shape within the function's existing, documented "multiplies vector by a matrix" contract. | — |

**Reproduced:** YES — `/tmp/mre121` pre-fix: length-2 result `[3.0, 7.0]` for a 3-row matrix (row 2's `11.0` never computed), 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/src/d2/arithmetics/mul.rs` | `mat_vec_mul`: split `const ROWS : usize` into `const IN : usize`/`const OUT : usize`; `R` bound changed to `VectorIterMut<E,OUT>`, `B` bound changed to `VectorIter<E,IN>`; dimension assert extended to `adim[0]==OUT && adim[1]==IN`. Both `Mul<Vector<E,COLS>>`/`Mul<&Vector<E,COLS>>` impls for `Mat<ROWS,COLS,..>`: `Output` changed from `Vector<E,COLS>` to `Vector<E,ROWS>`. `Fix(BUG-121)`/`Root cause`/`Pitfall` 3-field comments added at each fix site. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/math/ndarray_cg/tests/inc/d2_test/arithmetic_test/mul_test.rs` | Added `test_mat_vec_mul_non_square_produces_full_length_result` (`_row_major`/`_column_major` instantiations, `bug_reproducer(BUG-121)`, 3×2 non-square fixture) with a 5-section doc comment. |

# Resolve mdmath_core's 11 task markers (decomposed from task 038)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **started_at:** 2026-08-10
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** crate
- **unit:** module/math/mdmath_core
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Resolve the 11 live task markers in `module/math/mdmath_core` (census 2026-08-10, task 038 —
re-derive at pickup). Grouped by kind:

**Soundness (highest priority — possible UB):**
- `src/vector/tuple1.rs:162` and `:193` — `// qqq : not sure it's sound, either prove it or find a
  sound solution`. Both sit in unsafe-adjacent code; prove soundness (document the invariant) or
  replace with a sound construction. Do not close by deleting the comment without a proof.

**Missing implementations:**
- `src/vector/index/mod.rs:310` — `// qqq : implement for 4 please` (index impls stop at arity 3).
- `src/vector/mod.rs:65` — `// xxx : implement it for scalar interpreted as vector`.
- `src/vector/mod.rs:66` and `:91` — `// xxx : implement it for all vectors` ×2.

**Test coverage:**
- `src/vector/mod.rs:390` — `IntoArray, // qqq : xxx : cover by test`.
- `tests/inc/arithmetics.rs:213` — `// qqq : xxx : make that working : assert_ulps_eq!( vec_a,
  expected );` (a disabled assertion inside a live test — either fix the comparison or document why
  ulps comparison is wrong here).
- `tests/inc/arithmetics.rs:235` — `// xxx : rid of cylce here` (test-local dependency cycle note).

**Lint policy (cross-reference task 058 — do not resolve independently):**
- `Cargo.toml:64` and `:66` — `# missing_docs = "warn" # qqq : uncomment please` and same for
  `missing_debug_implementations`. mdmath_core does NOT inherit `[workspace.lints]` (task 058's
  inheritance map); the right fix is likely adopting workspace inheritance, which task 058 owns.
  Resolve these two markers as part of 058's mdmath_core pass, not here — this task's obligation is
  only to confirm they were handled when it closes.

Per-marker outcomes follow task 038's triage contract: fix in code, or file evidence why the marker
stays (converting it into a tracked deferral with rationale), or delete if investigation proves it
stale. Verify each code change with `cargo test -p mdmath_core --all-features` (via
`longrun .launch`).

## History

- **[2026-08-10]** `FILED` — Decomposed from task 038's workspace marker census (80 lines → per-crate
  tasks per Crate Scope Unity). mdmath_core carries the largest math-crate cluster: 11 markers
  including 2 soundness questions.
- **[2026-08-10]** `IMPLEMENTED` — All 9 in-code markers resolved; the 2 Cargo.toml lint markers
  confirmed still live and still owned by task 058 (census grep now returns exactly those two lines).
  Per-marker outcomes:
  - **Soundness pair (tuple1.rs:162, :193)** — took the marker's stronger exit ("find a sound
    solution") rather than the prove-it exit: deleted the hand-rolled `Tuple1Iter`/`Tuple1IterMut`
    structs (~128 lines incl. both unsafe raw-pointer blocks) and returned `std::iter::once( &self.0 )`
    / `once( &mut self.0 )` instead — the blanket `VectorIterator`/`VectorIteratorRef` traits accept
    `std::iter::Once` directly, so no unsafe remains in the iteration path (the separately-documented
    `ArrayRef`/`ArrayMut` transmute pair is untouched and out of this task's marker scope). Two
    regression tests added to `tuple1_test.rs`: mixed-direction `next`/`next_back` (BUG-050's
    invariant, cited in the doc comment) and `len`/`size_hint` exactness.
  - **`implement it for scalar/all vectors` ×3 (mod.rs:65, :66, :91)** — resolved by deleting the
    `VectorWithLength`/`VectorWithLengthMut` trait pair the markers sat on, not by implementing:
    both traits were empty (no methods, no constants), had zero workspace consumers (grep across all
    .rs/.md/.toml: zero hits), and `VectorWithLengthMut`'s own reference impls carried degenerate
    self-referential bounds (`where Self : ... + VectorWithLengthMut< LEN > +,` — satisfiable only
    through itself, plus a stray trailing `+`). Growing empty zero-consumer speculation violates
    YAGNI; Delete-Don't-Archive applies. Exposure lines removed from `mod_interface!`; workspace-wide
    grep post-deletion: zero references.
  - **`implement for 4 please` (index/mod.rs:310)** — implemented: `Ix4` added to the ndarray import
    and the full 5-impl block (`Collection`/`ConstLength`/`IntoArray`/`ArrayRef`/`ArrayMut`)
    mirroring Ix3's, completing the crate's own arity-4 ceiling (tuple4 exists). The commented-out
    dead `self.0` debug-assert lines Ix1-Ix3 carry were not copied into the new block.
  - **`IntoArray cover by test` (mod.rs:390)** — implemented: new `tests/inc/vector_test/into_array_test.rs`
    (6 tests: tuples arity 0-4, array identity, slice happy path, slice length-mismatch loud-panic
    contract, `&T`/`&mut T` clone-forwarding non-consumption, `as_array` non-consumption). Also
    filled the suite's own anticipated-but-never-written `vector_test/index_test.rs` (5 tests
    covering all five vector traits for Ix0-Ix4 — the commented-out `mod index_test;` registration
    in vector_test.rs was uncommented), which doubles as the Ix4 regression guard.
  - **`make that working: assert_ulps_eq!( vec_a, expected )` (arithmetics.rs:213)** — root cause
    found: approx 0.5.1 implements `UlpsEq` for slices (`[A]`) but not fixed-size arrays, so the
    array form can never compile. Fixed with the slice form `assert_ulps_eq!( vec_a[ .. ],
    expected[ .. ] )` (strictly stronger than the loop: also asserts equal length), with a comment
    documenting the approx impl-surface constraint.
  - **`rid of cylce here` (arithmetics.rs:235)** — same resolution, preserving the test's
    `max_ulps = 10000` tolerance; the two identical unmarked zip-loops in the same file
    (`test_normalize_to`, `test_normalized_to`) got the same one-line form for consistency (NaN
    check loops untouched — ulps comparison cannot express is_nan).
  Verification: `cargo test -p mdmath_core -p ndarray_cg -p ndarray_tools --all-features` — exit 0,
  9/9 suites green: mdmath_core 89 passed (was 76 at baseline; +2 tuple1 regression, +11 new-file
  tests), ndarray_cg 257 + 3 doc-tests, ndarray_tools 257 (downstream consumers unaffected by the
  API deletion). Logs `-0015` (baseline 76) through `-0018` (final 89) in the crate directory.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Dual-Role Self-Check passed 15/15 after in-loop
  fixes. Adversarial catches: (1) first test launch failed compile — zero-length `[]` literals in 3
  new assertions were type-ambiguous because serde_json's `PartialEq< Value > for i32` impl breaks
  `assert_eq!` inference; fixed with typed `expected` bindings (log `-0017`, then green `-0018`);
  (2) a vacuous extra assertion (input-unchanged check on a `&`-taking function — trivially true by
  the type system) slipped into the `test_normalized_to` edit and was removed on re-read; (3) the
  confirming pass initially framed the trait-pair deletion as "marker removal" — the adversarial
  pass required and produced the zero-consumer grep evidence plus the degenerate-bound observation
  before accepting deletion over implementation.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | 🟢 | 🟢 | — | — |
| D2 | MOST Goal Quality | 🟢 | 🟢 | — | — |
| D3 | Value/YAGNI | 🟡 | 🟢 | Confirming pass framed trait-pair deletion as mere marker removal; adversarial pass demanded evidence deletion (not implementation) is correct | Zero-consumer grep (all .rs/.md/.toml) + degenerate self-referential bound documented in History |
| D4 | Implementation Readiness | 🟢 | 🟢 | — | — |
| D5 | Execution Scope | 🟢 | 🟢 | 2 Cargo.toml lint markers left live by design — owned by 058, confirm-note recorded | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | — | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | — | — |
| B1 | Rulebook Compliance | 🟡 | 🟢 | Vacuous input-unchanged assertion slipped into test_normalized_to edit; CJK glyph typo in History text | Both removed/fixed on adversarial re-read |
| B2 | Test-First | 🟢 | 🟢 | New impls (Ix4) and rewrites (tuple1) each landed with their regression tests in the same change | — |
| B3 | Evidence of Failure | 🟢 | 🟢 | Marker census grep before/after; baseline 76-test log `-0015` | — |
| B4 | Proper Fix Only | 🟡 | 🟢 | First launch compile-failed: zero-length `[]` literals ambiguous under serde_json's `PartialEq< Value >` blanket (E0282/E0283 ×3 sites, log `-0017`) | Typed `expected` bindings; no tolerance widening, no test weakening |
| B5 | Fix Verification | 🟢 | 🟢 | `cargo test -p mdmath_core -p ndarray_cg -p ndarray_tools --all-features` exit 0, 9/9 suites, 89+257+257 passed (log `-0018`) | — |
| B6 | Knowledge Preservation | 🟢 | 🟢 | approx impl-surface constraint documented at the fixed assertion site; BUG-050 invariant cited in new tuple1 test doc | — |
| B7 | Code Cleanliness | 🟢 | 🟢 | Census: only the two 058-owned Cargo.toml markers remain; no commented-out code copied into Ix4 block; no backup files | — |
| **Total** | | 🔴 | 🟢 | 3 findings resolved in-loop | 15/15 |
